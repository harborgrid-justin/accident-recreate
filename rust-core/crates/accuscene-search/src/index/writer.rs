//! Index writer with batching and auto-commit

use crate::config::SearchConfig;
use crate::error::{SearchError, SearchResult};
use crate::index::schema::IndexSchema;
use std::sync::Arc;
use tantivy::{Document, Index, IndexWriter, Term};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

pub struct BatchWriter {
    writer: Arc<Mutex<IndexWriter>>,
    schema: Arc<IndexSchema>,
    config: SearchConfig,
    pending_count: Arc<Mutex<usize>>,
}

impl BatchWriter {
    pub fn new(index: Index, config: SearchConfig) -> SearchResult<Self> {
        let writer = index.writer(config.writer_heap_size)?;
        let schema = Arc::new(IndexSchema::default());

        let writer = Arc::new(Mutex::new(writer));
        let pending_count = Arc::new(Mutex::new(0));

        // Start auto-commit task
        let writer_clone = writer.clone();
        let pending_clone = pending_count.clone();
        let interval_secs = config.commit_interval_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            loop {
                ticker.tick().await;

                let count = {
                    let count = pending_clone.lock().await;
                    *count
                };

                if count > 0 {
                    info!("Auto-committing {} pending documents", count);
                    if let Ok(mut w) = writer_clone.try_lock() {
                        if let Err(e) = w.commit() {
                            warn!("Auto-commit failed: {}", e);
                        } else {
                            let mut pending = pending_clone.lock().await;
                            *pending = 0;
                        }
                    }
                }
            }
        });

        Ok(Self {
            writer,
            schema,
            config,
            pending_count,
        })
    }

    /// Add a single document
    pub async fn add_document(&self, doc: Document) -> SearchResult<()> {
        let mut writer = self.writer.lock().await;
        writer.add_document(doc)?;

        let mut count = self.pending_count.lock().await;
        *count += 1;

        Ok(())
    }

    /// Add multiple documents in batch
    pub async fn add_batch(&self, documents: Vec<Document>) -> SearchResult<()> {
        let mut writer = self.writer.lock().await;

        for doc in &documents {
            writer.add_document(doc.clone())?;
        }

        let mut count = self.pending_count.lock().await;
        *count += documents.len();

        debug!("Added batch of {} documents", documents.len());

        Ok(())
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, id: &str) -> SearchResult<()> {
        let mut writer = self.writer.lock().await;

        let id_field = self.schema.id;
        let term = Term::from_field_text(id_field, id);
        writer.delete_term(term);

        let mut count = self.pending_count.lock().await;
        *count += 1;

        Ok(())
    }

    /// Delete documents matching a term
    pub async fn delete_term(&self, field: &str, value: &str) -> SearchResult<()> {
        let mut writer = self.writer.lock().await;

        let field = self.schema.get_field(field)?;
        let term = Term::from_field_text(field, value);
        writer.delete_term(term);

        Ok(())
    }

    /// Manually commit pending changes
    pub async fn commit(&self) -> SearchResult<()> {
        let mut writer = self.writer.lock().await;

        let count = {
            let count = self.pending_count.lock().await;
            *count
        };

        if count > 0 {
            info!("Committing {} pending documents", count);
            writer.commit()?;

            let mut pending = self.pending_count.lock().await;
            *pending = 0;
        }

        Ok(())
    }

    /// Get pending document count
    pub async fn pending_count(&self) -> usize {
        let count = self.pending_count.lock().await;
        *count
    }

    /// Rollback uncommitted changes
    pub async fn rollback(&self) -> SearchResult<()> {
        let mut writer = self.writer.lock().await;
        writer.rollback()?;

        let mut count = self.pending_count.lock().await;
        *count = 0;

        Ok(())
    }
}

impl Drop for BatchWriter {
    fn drop(&mut self) {
        // Try to commit on drop (best effort)
        if let Ok(mut writer) = self.writer.try_lock() {
            let _ = writer.commit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::schema::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_batch_writer() {
        let temp_dir = TempDir::new().unwrap();
        let schema = IndexSchema::default();

        let index = Index::create_in_dir(
            temp_dir.path(),
            schema.tantivy_schema.clone(),
        )
        .unwrap();

        let config = SearchConfig::default();
        let writer = BatchWriter::new(index, config).unwrap();

        let doc = schema.create_document("test1", serde_json::json!({
            "title": "Test Document"
        })).unwrap();

        writer.add_document(doc).await.unwrap();
        assert_eq!(writer.pending_count().await, 1);

        writer.commit().await.unwrap();
        assert_eq!(writer.pending_count().await, 0);
    }
}
