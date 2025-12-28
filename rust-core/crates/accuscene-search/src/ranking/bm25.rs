//! BM25 ranking algorithm

use crate::config::BM25Config;

/// BM25 (Best Matching 25) ranking calculator
pub struct BM25Ranker {
    k1: f32,
    b: f32,
}

impl BM25Ranker {
    pub fn new(config: &BM25Config) -> Self {
        Self {
            k1: config.k1,
            b: config.b,
        }
    }

    /// Calculate BM25 score
    ///
    /// # Arguments
    /// * `tf` - Term frequency in document
    /// * `df` - Document frequency (number of documents containing term)
    /// * `dl` - Document length
    /// * `avgdl` - Average document length
    /// * `n` - Total number of documents
    pub fn score(
        &self,
        tf: f32,
        df: f32,
        dl: f32,
        avgdl: f32,
        n: f32,
    ) -> f32 {
        let idf = self.idf(df, n);
        let norm = self.normalize(tf, dl, avgdl);
        idf * norm
    }

    /// Calculate Inverse Document Frequency
    fn idf(&self, df: f32, n: f32) -> f32 {
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Normalize term frequency with length normalization
    fn normalize(&self, tf: f32, dl: f32, avgdl: f32) -> f32 {
        let numerator = tf * (self.k1 + 1.0);
        let denominator = tf + self.k1 * (1.0 - self.b + self.b * (dl / avgdl));
        numerator / denominator
    }

    /// Calculate score for multiple terms
    pub fn score_terms(
        &self,
        term_freqs: &[(f32, f32)], // (tf, df) pairs
        dl: f32,
        avgdl: f32,
        n: f32,
    ) -> f32 {
        term_freqs
            .iter()
            .map(|(tf, df)| self.score(*tf, *df, dl, avgdl, n))
            .sum()
    }
}

impl Default for BM25Ranker {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
        }
    }
}

/// BM25+ variant (handles long documents better)
pub struct BM25PlusRanker {
    k1: f32,
    b: f32,
    delta: f32,
}

impl BM25PlusRanker {
    pub fn new(config: &BM25Config) -> Self {
        Self {
            k1: config.k1,
            b: config.b,
            delta: 1.0,
        }
    }

    pub fn score(
        &self,
        tf: f32,
        df: f32,
        dl: f32,
        avgdl: f32,
        n: f32,
    ) -> f32 {
        let idf = self.idf(df, n);
        let norm = self.normalize(tf, dl, avgdl);
        idf * (norm + self.delta)
    }

    fn idf(&self, df: f32, n: f32) -> f32 {
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    fn normalize(&self, tf: f32, dl: f32, avgdl: f32) -> f32 {
        let numerator = tf * (self.k1 + 1.0);
        let denominator = tf + self.k1 * (1.0 - self.b + self.b * (dl / avgdl));
        numerator / denominator
    }
}

/// BM25F variant (for multi-field documents)
pub struct BM25FRanker {
    k1: f32,
    field_weights: std::collections::HashMap<String, f32>,
}

impl BM25FRanker {
    pub fn new(k1: f32, field_weights: std::collections::HashMap<String, f32>) -> Self {
        Self { k1, field_weights }
    }

    /// Calculate weighted term frequency across fields
    pub fn weighted_tf(&self, field_tfs: &std::collections::HashMap<String, f32>) -> f32 {
        field_tfs
            .iter()
            .map(|(field, tf)| {
                let weight = self.field_weights.get(field).unwrap_or(&1.0);
                tf * weight
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_score() {
        let config = BM25Config::default();
        let ranker = BM25Ranker::new(&config);

        let score = ranker.score(
            3.0,  // term frequency
            10.0, // document frequency
            100.0, // document length
            50.0, // average document length
            1000.0, // total documents
        );

        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25_idf() {
        let ranker = BM25Ranker::default();
        let idf = ranker.idf(10.0, 1000.0);
        assert!(idf > 0.0);

        // Rare terms should have higher IDF
        let idf_rare = ranker.idf(1.0, 1000.0);
        let idf_common = ranker.idf(100.0, 1000.0);
        assert!(idf_rare > idf_common);
    }

    #[test]
    fn test_bm25_normalize() {
        let ranker = BM25Ranker::default();

        // Higher TF should give higher score
        let norm1 = ranker.normalize(1.0, 100.0, 50.0);
        let norm2 = ranker.normalize(5.0, 100.0, 50.0);
        assert!(norm2 > norm1);

        // Length normalization
        let norm_short = ranker.normalize(3.0, 50.0, 100.0);
        let norm_long = ranker.normalize(3.0, 200.0, 100.0);
        assert!(norm_short > norm_long);
    }

    #[test]
    fn test_bm25_plus() {
        let config = BM25Config::default();
        let ranker = BM25PlusRanker::new(&config);

        let score = ranker.score(3.0, 10.0, 100.0, 50.0, 1000.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25f_weighted_tf() {
        let mut weights = std::collections::HashMap::new();
        weights.insert("title".to_string(), 2.0);
        weights.insert("content".to_string(), 1.0);

        let ranker = BM25FRanker::new(1.2, weights);

        let mut field_tfs = std::collections::HashMap::new();
        field_tfs.insert("title".to_string(), 1.0);
        field_tfs.insert("content".to_string(), 3.0);

        let weighted = ranker.weighted_tf(&field_tfs);
        assert_eq!(weighted, 5.0); // (1.0 * 2.0) + (3.0 * 1.0)
    }
}
