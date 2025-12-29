//! Search index management

pub mod builder;
pub mod schema;
pub mod writer;

use crate::config::SearchConfig;
use crate::error::{SearchError, SearchResult};
use crate::query::{Query, SearchFilters};
use crate::ranking::SearchResults;
use schema::IndexSchema;
use std::path::Path;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};
use writer::BatchWriter;

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    writer: Option<BatchWriter>,
    schema: IndexSchema,
    config: SearchConfig,
}

impl SearchIndex {
    /// Create a new search index
    pub async fn new(config: &SearchConfig) -> SearchResult<Self> {
        let schema = IndexSchema::default();
        let index = Self::create_or_open_index(&config.index_path, &schema)?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .map_err(|e| SearchError::IndexError(format!("Failed to create reader: {}", e)))?;

        let writer = BatchWriter::new(index.clone(), config.clone())?;

        Ok(Self {
            index,
            reader,
            writer: Some(writer),
            schema,
            config: config.clone(),
        })
    }

    fn create_or_open_index(path: &Path, schema: &IndexSchema) -> SearchResult<Index> {
        std::fs::create_dir_all(path)?;

        if path.join("meta.json").exists() {
            Index::open_in_dir(path)
                .map_err(|e| SearchError::IndexError(format!("Failed to open index: {}", e)))
        } else {
            Index::create_in_dir(path, schema.tantivy_schema.clone())
                .map_err(|e| SearchError::IndexError(format!("Failed to create index: {}", e)))
        }
    }

    /// Add a document to the index
    pub async fn add_document<T: serde::Serialize>(
        &mut self,
        id: &str,
        document: T,
    ) -> SearchResult<()> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| SearchError::IndexError("Writer not available".to_string()))?;

        let doc = self.schema.create_document(id, document)?;
        writer.add_document(doc).await
    }

    /// Add multiple documents in batch
    pub async fn add_batch<T: serde::Serialize>(
        &mut self,
        documents: Vec<(String, T)>,
    ) -> SearchResult<()> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| SearchError::IndexError("Writer not available".to_string()))?;

        let docs: Result<Vec<_>, _> = documents
            .into_iter()
            .map(|(id, doc)| self.schema.create_document(&id, doc))
            .collect();

        writer.add_batch(docs?).await
    }

    /// Search the index
    pub async fn search(
        &self,
        query: &Query,
        filters: Option<SearchFilters>,
    ) -> SearchResult<SearchResults> {
        use crate::query::QueryExecutor;

        let searcher = self.reader.searcher();
        let executor = QueryExecutor::new(searcher, &self.schema, &self.config);

        executor.execute(query, filters).await
    }

    /// Get auto-complete suggestions
    pub async fn suggest(
        &self,
        prefix: &str,
        limit: usize,
    ) -> SearchResult<Vec<String>> {
        use crate::suggestions::SuggestionEngine;

        let searcher = self.reader.searcher();
        let engine = SuggestionEngine::new(searcher, &self.schema);

        engine.suggest(prefix, limit).await
    }

    /// Get facet counts
    pub async fn facets(&self, field: &str) -> SearchResult<Vec<(String, u64)>> {
        use crate::ranking::facets::FacetCollector;

        let searcher = self.reader.searcher();
        let collector = FacetCollector::new(&self.schema, field)?;

        collector.collect(&searcher).await
    }

    /// Delete a document
    pub async fn delete_document(&mut self, id: &str) -> SearchResult<()> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| SearchError::IndexError("Writer not available".to_string()))?;

        writer.delete_document(id).await
    }

    /// Update a document
    pub async fn update_document<T: serde::Serialize>(
        &mut self,
        id: &str,
        document: T,
    ) -> SearchResult<()> {
        self.delete_document(id).await?;
        self.add_document(id, document).await
    }

    /// Commit pending changes
    pub async fn commit(&mut self) -> SearchResult<()> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| SearchError::IndexError("Writer not available".to_string()))?;

        writer.commit().await
    }

    /// Get document count
    pub async fn document_count(&self) -> SearchResult<u64> {
        let searcher = self.reader.searcher();
        Ok(searcher.num_docs())
    }

    /// Get index size in bytes
    pub async fn size_bytes(&self) -> SearchResult<u64> {
        let mut total_size = 0u64;

        for entry in std::fs::read_dir(&self.config.index_path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_index_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SearchConfig::default();
        config.index_path = temp_dir.path().to_path_buf();

        let index = SearchIndex::new(&config).await.unwrap();
        assert!(index.document_count().await.unwrap() == 0);
    }
}
