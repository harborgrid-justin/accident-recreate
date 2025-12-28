//! Train/test splitting utilities

use crate::error::Result;
use crate::training::dataset::{Dataset, DatasetSplit};
use rand::prelude::*;

/// Train-test splitter
pub struct TrainTestSplit {
    /// Test size (0.0 to 1.0)
    test_size: f64,

    /// Random seed
    random_seed: u64,

    /// Shuffle before splitting
    shuffle: bool,
}

impl TrainTestSplit {
    /// Create a new train-test splitter
    pub fn new(test_size: f64) -> Self {
        Self {
            test_size,
            random_seed: 42,
            shuffle: true,
        }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = seed;
        self
    }

    /// Set shuffle
    pub fn with_shuffle(mut self, shuffle: bool) -> Self {
        self.shuffle = shuffle;
        self
    }

    /// Split the dataset
    pub fn split(&self, mut dataset: Dataset) -> Result<(Dataset, Dataset)> {
        if self.shuffle {
            dataset.shuffle(self.random_seed);
        }

        let n_samples = dataset.num_samples();
        let n_test = (n_samples as f64 * self.test_size).round() as usize;
        let n_train = n_samples - n_test;

        let train_indices: Vec<usize> = (0..n_train).collect();
        let test_indices: Vec<usize> = (n_train..n_samples).collect();

        let train = dataset.subset(&train_indices)?;
        let test = dataset.subset(&test_indices)?;

        Ok((train, test))
    }
}

/// Validation splitter (train/val/test)
pub struct ValidationSplit {
    /// Test size (0.0 to 1.0)
    test_size: f64,

    /// Validation size (0.0 to 1.0)
    val_size: f64,

    /// Random seed
    random_seed: u64,

    /// Shuffle before splitting
    shuffle: bool,
}

impl ValidationSplit {
    /// Create a new validation splitter
    pub fn new(val_size: f64, test_size: f64) -> Self {
        Self {
            test_size,
            val_size,
            random_seed: 42,
            shuffle: true,
        }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.random_seed = seed;
        self
    }

    /// Set shuffle
    pub fn with_shuffle(mut self, shuffle: bool) -> Self {
        self.shuffle = shuffle;
        self
    }

    /// Split the dataset into train/val/test
    pub fn split(&self, mut dataset: Dataset) -> Result<DatasetSplit> {
        if self.shuffle {
            dataset.shuffle(self.random_seed);
        }

        let n_samples = dataset.num_samples();
        let n_test = (n_samples as f64 * self.test_size).round() as usize;
        let n_val = (n_samples as f64 * self.val_size).round() as usize;
        let n_train = n_samples - n_test - n_val;

        let train_indices: Vec<usize> = (0..n_train).collect();
        let val_indices: Vec<usize> = (n_train..n_train + n_val).collect();
        let test_indices: Vec<usize> = (n_train + n_val..n_samples).collect();

        let train = dataset.subset(&train_indices)?;
        let val = dataset.subset(&val_indices)?;
        let test = dataset.subset(&test_indices)?;

        Ok(DatasetSplit {
            train,
            val: Some(val),
            test,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr1, arr2};

    #[test]
    fn test_train_test_split() -> Result<()> {
        let features = arr2(&[[1.0], [2.0], [3.0], [4.0], [5.0]]);
        let targets = arr1(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let dataset = Dataset::new(features, targets)?;

        let splitter = TrainTestSplit::new(0.2);
        let (train, test) = splitter.split(dataset)?;

        assert_eq!(train.num_samples() + test.num_samples(), 5);
        assert_eq!(test.num_samples(), 1);

        Ok(())
    }
}
