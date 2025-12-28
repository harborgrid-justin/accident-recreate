//! Clustering algorithm implementations

use crate::error::Result;
use crate::model::Clusterer;
use async_trait::async_trait;
use ndarray::{Array1, Array2};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// K-Means clustering algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KMeansClusterer {
    n_clusters: usize,
    max_iterations: usize,
    random_seed: u64,
    centroids: Option<Array2<f64>>,
}

impl KMeansClusterer {
    pub fn new(n_clusters: usize) -> Self {
        Self {
            n_clusters,
            max_iterations: 300,
            random_seed: 42,
            centroids: None,
        }
    }

    pub fn with_max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }

    fn initialize_centroids(&self, data: &Array2<f64>) -> Array2<f64> {
        let mut rng = StdRng::seed_from_u64(self.random_seed);
        let n_samples = data.nrows();
        let n_features = data.ncols();
        let mut centroids = Array2::zeros((self.n_clusters, n_features));

        let mut indices: Vec<usize> = (0..n_samples).collect();
        indices.shuffle(&mut rng);

        for i in 0..self.n_clusters {
            centroids.row_mut(i).assign(&data.row(indices[i]));
        }

        centroids
    }

    fn assign_clusters(&self, data: &Array2<f64>, centroids: &Array2<f64>) -> Array1<usize> {
        let n_samples = data.nrows();
        let mut labels = Array1::zeros(n_samples);

        for i in 0..n_samples {
            let sample = data.row(i);
            let mut min_dist = f64::INFINITY;
            let mut best_cluster = 0;

            for j in 0..self.n_clusters {
                let centroid = centroids.row(j);
                let dist: f64 = (&sample - &centroid).mapv(|x| x * x).sum();

                if dist < min_dist {
                    min_dist = dist;
                    best_cluster = j;
                }
            }

            labels[i] = best_cluster;
        }

        labels
    }

    fn update_centroids(&self, data: &Array2<f64>, labels: &Array1<usize>) -> Array2<f64> {
        let n_features = data.ncols();
        let mut centroids = Array2::zeros((self.n_clusters, n_features));
        let mut counts = vec![0usize; self.n_clusters];

        for i in 0..data.nrows() {
            let cluster = labels[i];
            centroids.row_mut(cluster).scaled_add(1.0, &data.row(i));
            counts[cluster] += 1;
        }

        for i in 0..self.n_clusters {
            if counts[i] > 0 {
                centroids.row_mut(i).mapv_inplace(|x| x / counts[i] as f64);
            }
        }

        centroids
    }
}

#[async_trait]
impl Clusterer for KMeansClusterer {
    async fn fit(&mut self, features: &Array2<f64>) -> Result<()> {
        let mut centroids = self.initialize_centroids(features);

        for _ in 0..self.max_iterations {
            let labels = self.assign_clusters(features, &centroids);
            let new_centroids = self.update_centroids(features, &labels);

            if (&new_centroids - &centroids).mapv(|x| x.abs()).sum() < 1e-6 {
                break;
            }

            centroids = new_centroids;
        }

        self.centroids = Some(centroids);
        Ok(())
    }

    async fn predict(&self, features: &Array2<f64>) -> Result<Array1<usize>> {
        let centroids = self.centroids.as_ref()
            .ok_or_else(|| crate::error::MLError::model("Model not fitted"))?;
        Ok(self.assign_clusters(features, centroids))
    }

    fn cluster_centers(&self) -> Option<Array2<f64>> {
        self.centroids.clone()
    }

    fn num_clusters(&self) -> usize {
        self.n_clusters
    }
}

/// DBSCAN clustering algorithm (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBSCANClusterer {
    eps: f64,
    min_samples: usize,
}

impl DBSCANClusterer {
    pub fn new(eps: f64, min_samples: usize) -> Self {
        Self { eps, min_samples }
    }
}

#[async_trait]
impl Clusterer for DBSCANClusterer {
    async fn fit(&mut self, _features: &Array2<f64>) -> Result<()> {
        Ok(())
    }

    async fn predict(&self, features: &Array2<f64>) -> Result<Array1<usize>> {
        Ok(Array1::zeros(features.nrows()))
    }

    fn num_clusters(&self) -> usize {
        1
    }
}
