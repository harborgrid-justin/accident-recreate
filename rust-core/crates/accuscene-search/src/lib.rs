//! AccuScene Enterprise Search Engine
//!
//! Advanced full-text search with BM25 ranking, faceted search, fuzzy matching,
//! and real-time suggestions.

pub mod config;
pub mod error;
pub mod highlighting;
pub mod index;
pub mod query;
pub mod ranking;
pub mod suggestions;

pub use config::SearchConfig;
pub use error::{SearchError, SearchResult};

use index::SearchIndex;
use query::{Query, QueryBuilder, SearchFilters};
use ranking::SearchResults;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main search engine interface
pub struct SearchEngine {
    index: Arc<RwLock<SearchIndex>>,
    config: SearchConfig,
}

impl SearchEngine {
    /// Create a new search engine with the given configuration
    pub async fn new(config: SearchConfig) -> SearchResult<Self> {
        let index = SearchIndex::new(&config).await?;
        Ok(Self {
            index: Arc::new(RwLock::new(index)),
            config,
        })
    }

    /// Index a document
    pub async fn index_document<T: serde::Serialize>(
        &self,
        id: &str,
        document: T,
    ) -> SearchResult<()> {
        let mut index = self.index.write().await;
        index.add_document(id, document).await
    }

    /// Index multiple documents in batch
    pub async fn index_batch<T: serde::Serialize>(
        &self,
        documents: Vec<(String, T)>,
    ) -> SearchResult<()> {
        let mut index = self.index.write().await;
        index.add_batch(documents).await
    }

    /// Search with query and filters
    pub async fn search(
        &self,
        query: &Query,
        filters: Option<SearchFilters>,
    ) -> SearchResult<SearchResults> {
        let index = self.index.read().await;
        index.search(query, filters).await
    }

    /// Get auto-complete suggestions
    pub async fn suggest(
        &self,
        prefix: &str,
        limit: usize,
    ) -> SearchResult<Vec<String>> {
        let index = self.index.read().await;
        index.suggest(prefix, limit).await
    }

    /// Get facet counts for a field
    pub async fn facets(&self, field: &str) -> SearchResult<Vec<(String, u64)>> {
        let index = self.index.read().await;
        index.facets(field).await
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, id: &str) -> SearchResult<()> {
        let mut index = self.index.write().await;
        index.delete_document(id).await
    }

    /// Update a document
    pub async fn update_document<T: serde::Serialize>(
        &self,
        id: &str,
        document: T,
    ) -> SearchResult<()> {
        let mut index = self.index.write().await;
        index.update_document(id, document).await
    }

    /// Commit pending changes
    pub async fn commit(&self) -> SearchResult<()> {
        let mut index = self.index.write().await;
        index.commit().await
    }

    /// Get search statistics
    pub async fn stats(&self) -> SearchResult<SearchStats> {
        let index = self.index.read().await;
        Ok(SearchStats {
            total_documents: index.document_count().await?,
            index_size_bytes: index.size_bytes().await?,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Search engine statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchStats {
    pub total_documents: u64,
    pub index_size_bytes: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_engine_basic() {
        let config = SearchConfig::default();
        let engine = SearchEngine::new(config).await.unwrap();

        let doc = serde_json::json!({
            "title": "Test Document",
            "content": "This is a test document for searching",
            "category": "test"
        });

        engine.index_document("doc1", doc).await.unwrap();
        engine.commit().await.unwrap();

        let query = QueryBuilder::new()
            .text("test")
            .build();

        let results = engine.search(&query, None).await.unwrap();
        assert!(results.total > 0);
    }
}
