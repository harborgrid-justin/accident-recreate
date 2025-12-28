//! Benchmarks for AccuScene Analytics Engine

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use accuscene_analytics::*;
use chrono::Utc;

fn metrics_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");

    // Counter benchmarks
    group.bench_function("counter_inc", |b| {
        let counter = Counter::new("test");
        b.iter(|| {
            counter.inc();
        });
    });

    // Gauge benchmarks
    group.bench_function("gauge_set", |b| {
        let gauge = Gauge::new("test");
        b.iter(|| {
            gauge.set(black_box(42.0));
        });
    });

    // Histogram benchmarks
    group.bench_function("histogram_observe", |b| {
        let histogram = Histogram::new_linear("test", 0.0, 10.0, 100);
        b.iter(|| {
            histogram.observe(black_box(5.5));
        });
    });

    // TimeSeries benchmarks
    group.bench_function("timeseries_add", |b| {
        let ts = TimeSeries::new("test", 1000, 3600);
        b.iter(|| {
            ts.add(black_box(42.0));
        });
    });

    group.finish();
}

fn aggregation_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregation");

    // Temporal aggregation
    group.bench_function("temporal_aggregation", |b| {
        use aggregation::temporal::{TemporalAggregator, TemporalResolution};

        let agg = TemporalAggregator::new(TemporalResolution::Hour, AggregationOp::Sum);
        b.iter(|| {
            agg.add(black_box(Utc::now()), black_box(100.0));
        });
    });

    // Spatial aggregation
    group.bench_function("spatial_aggregation", |b| {
        use aggregation::spatial::{SpatialAggregator, SpatialGrid, SpatialPoint};

        let origin = SpatialPoint::new(0.0, 0.0);
        let grid = SpatialGrid::new(origin, 1000.0);
        let agg = SpatialAggregator::new(grid, AggregationOp::Sum);

        b.iter(|| {
            let point = SpatialPoint::new(black_box(0.01), black_box(0.01));
            agg.add(point, black_box(100.0));
        });
    });

    // Dimensional aggregation
    group.bench_function("dimensional_aggregation", |b| {
        let agg = DimensionalAggregator::new(AggregationOp::Sum);

        b.iter(|| {
            let dims = vec![
                Dimension::string("region", "west"),
                Dimension::int("severity", black_box(5)),
            ];
            agg.add(dims, black_box(100.0));
        });
    });

    group.finish();
}

fn statistics_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    let data: Vec<f64> = (0..1000).map(|i| i as f64).collect();

    // Descriptive statistics
    group.bench_function("descriptive_stats", |b| {
        b.iter(|| {
            DescriptiveStats::from_data(black_box(&data)).unwrap()
        });
    });

    // Linear regression
    group.bench_function("linear_regression", |b| {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..100).map(|i| i as f64 * 2.0 + 1.0).collect();

        b.iter(|| {
            LinearRegression::fit(black_box(&x), black_box(&y)).unwrap()
        });
    });

    // Correlation
    group.bench_function("pearson_correlation", |b| {
        let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..100).map(|i| i as f64 * 2.0).collect();

        b.iter(|| {
            CorrelationAnalyzer::pearson(black_box(&x), black_box(&y)).unwrap()
        });
    });

    group.finish();
}

fn anomaly_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("anomaly_detection");

    let mut data: Vec<f64> = (0..1000).map(|i| (i as f64 / 10.0).sin() * 100.0).collect();
    data.push(1000.0); // Add anomaly

    // Z-score detector
    group.bench_function("zscore_detector", |b| {
        let detector = ZScoreDetector::new(3.0);
        b.iter(|| {
            detector.detect(black_box(&data)).unwrap()
        });
    });

    // IQR detector
    group.bench_function("iqr_detector", |b| {
        let detector = IQRDetector::new(1.5);
        b.iter(|| {
            detector.detect(black_box(&data)).unwrap()
        });
    });

    group.finish();
}

fn forecasting_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("forecasting");

    let data: Vec<f64> = (0..100).map(|i| (i as f64 / 10.0).sin() * 100.0).collect();

    // Moving average forecaster
    group.bench_function("moving_average_forecast", |b| {
        let forecaster = MovingAverageForecaster::new(10);
        b.iter(|| {
            forecaster.forecast(black_box(&data), black_box(10)).unwrap()
        });
    });

    // Exponential smoothing
    group.bench_function("exponential_smoothing_forecast", |b| {
        let forecaster = ExponentialSmoothingForecaster::new(0.3).unwrap();
        b.iter(|| {
            forecaster.forecast(black_box(&data), black_box(10)).unwrap()
        });
    });

    group.finish();
}

fn windowing_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("windowing");

    // Sliding window
    group.bench_function("sliding_window", |b| {
        let window = SlidingWindow::<i32>::new(100);
        b.iter(|| {
            window.add(black_box(42));
        });
    });

    // Tumbling window
    group.bench_function("tumbling_window", |b| {
        let window = TumblingWindow::<i32>::new(100);
        b.iter(|| {
            window.add(black_box(42));
        });
    });

    group.finish();
}

fn storage_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage");

    // Analytics storage
    group.bench_function("storage_put_get", |b| {
        let storage = AnalyticsStorage::new(3600);
        b.iter(|| {
            storage.put_json("test", &42).unwrap();
            let _: i32 = storage.get_json("test").unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    metrics_benchmarks,
    aggregation_benchmarks,
    statistics_benchmarks,
    anomaly_benchmarks,
    forecasting_benchmarks,
    windowing_benchmarks,
    storage_benchmarks
);

criterion_main!(benches);
