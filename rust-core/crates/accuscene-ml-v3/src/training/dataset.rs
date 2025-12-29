//! Dataset management and splitting

use crate::error::{MlError, Result};
use crate::features::FeatureVector;
use ndarray::{Array1, Array2};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ML Dataset
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Features (X)
    pub features: Array2<f64>,

    /// Labels (y)
    pub labels: Array1<f64>,

    /// Feature names
    pub feature_names: Vec<String>,

    /// Sample weights (optional)
    pub sample_weights: Option<Array1<f64>>,
}

/// Dataset split result
#[derive(Debug, Clone)]
pub struct DatasetSplit {
    /// Training set
    pub train: Dataset,

    /// Validation set
    pub validation: Dataset,

    /// Test set (optional)
    pub test: Option<Dataset>,
}

impl Dataset {
    /// Create a new dataset
    pub fn new(features: Array2<f64>, labels: Array1<f64>, feature_names: Vec<String>) -> Result<Self> {
        if features.nrows() != labels.len() {
            return Err(MlError::Dataset(format!(
                "Feature rows ({}) must match label count ({})",
                features.nrows(),
                labels.len()
            )));
        }

        if features.ncols() != feature_names.len() {
            return Err(MlError::Dataset(format!(
                "Feature columns ({}) must match feature names count ({})",
                features.ncols(),
                feature_names.len()
            )));
        }

        Ok(Self {
            features,
            labels,
            feature_names,
            sample_weights: None,
        })
    }

    /// Create from feature vectors and labels
    pub fn from_features(feature_vectors: Vec<FeatureVector>, labels: Vec<f64>) -> Result<Self> {
        if feature_vectors.is_empty() {
            return Err(MlError::Dataset("Empty feature vectors".to_string()));
        }

        if feature_vectors.len() != labels.len() {
            return Err(MlError::Dataset("Features and labels length mismatch".to_string()));
        }

        let n_samples = feature_vectors.len();
        let n_features = feature_vectors[0].values.len();
        let feature_names = feature_vectors[0].names.clone();

        let mut features_vec = Vec::with_capacity(n_samples * n_features);
        for fv in &feature_vectors {
            features_vec.extend_from_slice(&fv.values);
        }

        let features = Array2::from_shape_vec((n_samples, n_features), features_vec)
            .map_err(|e| MlError::Dataset(e.to_string()))?;

        let labels_array = Array1::from_vec(labels);

        Self::new(features, labels_array, feature_names)
    }

    /// Get number of samples
    pub fn len(&self) -> usize {
        self.features.nrows()
    }

    /// Check if dataset is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get number of features
    pub fn n_features(&self) -> usize {
        self.features.ncols()
    }

    /// Split dataset into train/validation
    pub fn split(&self, validation_ratio: f32, random_seed: Option<u64>) -> Result<DatasetSplit> {
        if validation_ratio <= 0.0 || validation_ratio >= 1.0 {
            return Err(MlError::Dataset("Validation ratio must be between 0 and 1".to_string()));
        }

        let n_samples = self.len();
        let n_val = (n_samples as f32 * validation_ratio) as usize;
        let n_train = n_samples - n_val;

        // Create shuffled indices
        let mut indices: Vec<usize> = (0..n_samples).collect();
        if let Some(seed) = random_seed {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            indices.shuffle(&mut rng);
        } else {
            use rand::thread_rng;
            let mut rng = thread_rng();
            indices.shuffle(&mut rng);
        }

        // Split indices
        let train_indices = &indices[..n_train];
        let val_indices = &indices[n_train..];

        // Create train dataset
        let train = self.subset(train_indices)?;
        let validation = self.subset(val_indices)?;

        Ok(DatasetSplit {
            train,
            validation,
            test: None,
        })
    }

    /// Create a subset from indices
    pub fn subset(&self, indices: &[usize]) -> Result<Dataset> {
        let mut features_vec = Vec::with_capacity(indices.len() * self.n_features());
        let mut labels_vec = Vec::with_capacity(indices.len());

        for &idx in indices {
            if idx >= self.len() {
                return Err(MlError::Dataset(format!("Index {} out of bounds", idx)));
            }

            for col in 0..self.n_features() {
                features_vec.push(self.features[[idx, col]]);
            }
            labels_vec.push(self.labels[idx]);
        }

        let features = Array2::from_shape_vec((indices.len(), self.n_features()), features_vec)
            .map_err(|e| MlError::Dataset(e.to_string()))?;

        let labels = Array1::from_vec(labels_vec);

        let mut subset = Dataset::new(features, labels, self.feature_names.clone())?;

        // Copy sample weights if present
        if let Some(weights) = &self.sample_weights {
            let weights_subset: Vec<f64> = indices.iter().map(|&i| weights[i]).collect();
            subset.sample_weights = Some(Array1::from_vec(weights_subset));
        }

        Ok(subset)
    }

    /// Shuffle dataset
    pub fn shuffle(&mut self, random_seed: Option<u64>) {
        let indices: Vec<usize> = (0..self.len()).collect();
        let shuffled = self.subset(&indices).unwrap();
        *self = shuffled;
    }

    /// Save dataset to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = SavedDataset {
            features: self.features.clone().into_raw_vec(),
            labels: self.labels.clone().into_raw_vec(),
            feature_names: self.feature_names.clone(),
            n_samples: self.len(),
            n_features: self.n_features(),
        };

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load dataset from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let data: SavedDataset = serde_json::from_str(&json)?;

        let features = Array2::from_shape_vec((data.n_samples, data.n_features), data.features)
            .map_err(|e| MlError::Dataset(e.to_string()))?;

        let labels = Array1::from_vec(data.labels);

        Self::new(features, labels, data.feature_names)
    }
}

/// Serializable dataset format
#[derive(Debug, Serialize, Deserialize)]
struct SavedDataset {
    features: Vec<f64>,
    labels: Vec<f64>,
    feature_names: Vec<String>,
    n_samples: usize,
    n_features: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset() -> Dataset {
        let features = Array2::from_shape_vec((10, 3), (0..30).map(|x| x as f64).collect()).unwrap();
        let labels = Array1::from_vec((0..10).map(|x| x as f64).collect());
        let feature_names = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        Dataset::new(features, labels, feature_names).unwrap()
    }

    #[test]
    fn test_dataset_creation() {
        let dataset = create_test_dataset();
        assert_eq!(dataset.len(), 10);
        assert_eq!(dataset.n_features(), 3);
    }

    #[test]
    fn test_dataset_split() {
        let dataset = create_test_dataset();
        let split = dataset.split(0.2, Some(42)).unwrap();

        assert_eq!(split.train.len(), 8);
        assert_eq!(split.validation.len(), 2);
    }

    #[test]
    fn test_dataset_subset() {
        let dataset = create_test_dataset();
        let subset = dataset.subset(&[0, 2, 4]).unwrap();

        assert_eq!(subset.len(), 3);
        assert_eq!(subset.labels[0], 0.0);
        assert_eq!(subset.labels[1], 2.0);
        assert_eq!(subset.labels[2], 4.0);
    }
}
