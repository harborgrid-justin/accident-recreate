//! Index builder for bulk indexing operations

use crate::config::SearchConfig;
use crate::error::{SearchError, SearchResult};
use crate::index::schema::IndexSchema;
use rayon::prelude::*;
use std::path::Path;
use tantivy::{Index, IndexWriter};
use tracing::{info, warn};

pub struct IndexBuilder {
    index: Index,
    schema: IndexSchema,
    config: SearchConfig,
}

impl IndexBuilder {
    /// Create a new index builder
    pub fn new(index_path: &Path, config: SearchConfig) -> SearchResult<Self> {
        let schema = IndexSchema::default();

        std::fs::create_dir_all(index_path)?;

        let index = if index_path.join("meta.json").exists() {
            warn!("Opening existing index at {:?}", index_path);
            Index::open_in_dir(index_path)?
        } else {
            info!("Creating new index at {:?}", index_path);
            Index::create_in_dir(index_path, schema.tantivy_schema.clone())?
        };

        Ok(Self {
            index,
            schema,
            config,
        })
    }

    /// Build index from documents
    pub async fn build<T: serde::Serialize + Send + Sync>(
        &self,
        documents: Vec<(String, T)>,
    ) -> SearchResult<IndexBuildStats> {
        let start = std::time::Instant::now();

        info!("Building index with {} documents", documents.len());

        let mut writer = self.index.writer(self.config.writer_heap_size)?;

        // Process documents in parallel batches
        let batch_size = 1000;
        let mut total_indexed = 0;
        let mut total_errors = 0;

        for chunk in documents.chunks(batch_size) {
            let docs: Vec<_> = chunk
                .par_iter()
                .filter_map(|(id, data)| {
                    match self.schema.create_document(id, data) {
                        Ok(doc) => Some(doc),
                        Err(e) => {
                            warn!("Failed to create document {}: {}", id, e);
                            None
                        }
                    }
                })
                .collect();

            for doc in docs {
                writer.add_document(doc)?;
                total_indexed += 1;
            }

            total_errors += chunk.len() - (total_indexed % batch_size);

            if total_indexed % 10000 == 0 {
                info!("Indexed {} documents...", total_indexed);
            }
        }

        info!("Committing index...");
        writer.commit()?;

        let stats = IndexBuildStats {
            total_documents: documents.len(),
            indexed_documents: total_indexed,
            failed_documents: total_errors,
            duration_secs: start.elapsed().as_secs_f64(),
        };

        info!("Index build completed: {:?}", stats);

        Ok(stats)
    }

    /// Rebuild entire index (delete and recreate)
    pub async fn rebuild<T: serde::Serialize + Send + Sync>(
        &self,
        documents: Vec<(String, T)>,
    ) -> SearchResult<IndexBuildStats> {
        info!("Rebuilding index...");

        let mut writer = self.index.writer(self.config.writer_heap_size)?;
        writer.delete_all_documents()?;
        writer.commit()?;

        self.build(documents).await
    }

    /// Optimize index (merge segments)
    pub async fn optimize(&self) -> SearchResult<()> {
        info!("Optimizing index...");

        let mut writer = self.index.writer(self.config.writer_heap_size)?;

        // Merge segments
        let segment_ids = writer
            .index()
            .searchable_segment_ids()?;

        if segment_ids.len() > 1 {
            info!("Merging {} segments", segment_ids.len());
            writer.merge(&segment_ids).wait()?;
        }

        writer.commit()?;

        info!("Index optimization completed");
        Ok(())
    }

    /// Get index statistics
    pub fn stats(&self) -> SearchResult<IndexStats> {
        let searcher = self.index.reader()?.searcher();

        Ok(IndexStats {
            num_documents: searcher.num_docs(),
            num_segments: searcher.segment_readers().len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct IndexBuildStats {
    pub total_documents: usize,
    pub indexed_documents: usize,
    pub failed_documents: usize,
    pub duration_secs: f64,
}

impl IndexBuildStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_documents == 0 {
            0.0
        } else {
            self.indexed_documents as f64 / self.total_documents as f64
        }
    }

    pub fn throughput(&self) -> f64 {
        if self.duration_secs == 0.0 {
            0.0
        } else {
            self.indexed_documents as f64 / self.duration_secs
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub num_documents: u64,
    pub num_segments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_index_builder() {
        let temp_dir = TempDir::new().unwrap();
        let config = SearchConfig::default();

        let builder = IndexBuilder::new(temp_dir.path(), config).unwrap();

        let documents = vec![
            ("doc1".to_string(), json!({"title": "Test 1", "content": "Content 1"})),
            ("doc2".to_string(), json!({"title": "Test 2", "content": "Content 2"})),
        ];

        let stats = builder.build(documents).await.unwrap();
        assert_eq!(stats.indexed_documents, 2);
        assert!(stats.success_rate() > 0.99);
    }
}
