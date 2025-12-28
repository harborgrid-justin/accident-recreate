//! ML algorithm implementations

pub mod classification;
pub mod clustering;
pub mod ensemble;
pub mod regression;

pub use classification::{DecisionTreeClassifier, LogisticRegression, SVMClassifier};
pub use clustering::{DBSCANClusterer, KMeansClusterer};
pub use ensemble::{GradientBoostingRegressor, RandomForestRegressor};
pub use regression::{LassoRegression, LinearRegression, RidgeRegression};
