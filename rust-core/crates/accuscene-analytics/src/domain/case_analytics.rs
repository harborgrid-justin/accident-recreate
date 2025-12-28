//! Case outcome statistics and analysis

use crate::error::Result;
use crate::statistics::descriptive::{DescriptiveStats, Statistics};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseData {
    pub id: String,
    pub status: CaseStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub complexity: CaseComplexity,
    pub reconstruction_accuracy: f64, // 0-100
    pub num_vehicles: usize,
    pub num_witnesses: usize,
    pub evidence_quality: EvidenceQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    Open,
    InProgress,
    UnderReview,
    Closed,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceQuality {
    Poor,
    Fair,
    Good,
    Excellent,
}

/// Case outcome analytics
pub struct CaseAnalytics;

impl CaseAnalytics {
    /// Analyze case completion times
    pub fn completion_time_analysis(cases: &[CaseData]) -> Result<CompletionAnalysis> {
        let completion_times: Vec<f64> = cases
            .iter()
            .filter_map(|c| {
                c.closed_at.map(|closed| {
                    (closed - c.created_at).num_hours() as f64
                })
            })
            .collect();

        if completion_times.is_empty() {
            return Err(crate::error::AnalyticsError::InsufficientData(
                "No closed cases found".to_string(),
            ));
        }

        let stats = DescriptiveStats::from_data(&completion_times)?;

        // Categorize by completion time
        let fast = completion_times.iter().filter(|&&t| t < 24.0).count();
        let normal = completion_times.iter().filter(|&&t| t >= 24.0 && t < 72.0).count();
        let slow = completion_times.iter().filter(|&&t| t >= 72.0).count();

        Ok(CompletionAnalysis {
            stats,
            fast_completion_count: fast,
            normal_completion_count: normal,
            slow_completion_count: slow,
            avg_completion_hours: stats.mean,
        })
    }

    /// Analyze case status distribution
    pub fn status_distribution(cases: &[CaseData]) -> StatusDistribution {
        let mut counts = [0; 5];

        for case in cases {
            let idx = match case.status {
                CaseStatus::Open => 0,
                CaseStatus::InProgress => 1,
                CaseStatus::UnderReview => 2,
                CaseStatus::Closed => 3,
                CaseStatus::Archived => 4,
            };
            counts[idx] += 1;
        }

        let total = cases.len();

        StatusDistribution {
            open: counts[0],
            in_progress: counts[1],
            under_review: counts[2],
            closed: counts[3],
            archived: counts[4],
            total,
            closure_rate: if total > 0 {
                (counts[3] + counts[4]) as f64 / total as f64 * 100.0
            } else {
                0.0
            },
        }
    }

    /// Analyze reconstruction accuracy
    pub fn accuracy_analysis(cases: &[CaseData]) -> Result<AccuracyAnalysis> {
        let accuracies: Vec<f64> = cases
            .iter()
            .filter(|c| matches!(c.status, CaseStatus::Closed | CaseStatus::Archived))
            .map(|c| c.reconstruction_accuracy)
            .collect();

        if accuracies.is_empty() {
            return Err(crate::error::AnalyticsError::InsufficientData(
                "No completed cases with accuracy data".to_string(),
            ));
        }

        let stats = DescriptiveStats::from_data(&accuracies)?;

        // Categorize by accuracy
        let high = accuracies.iter().filter(|&&a| a >= 90.0).count();
        let medium = accuracies.iter().filter(|&&a| a >= 70.0 && a < 90.0).count();
        let low = accuracies.iter().filter(|&&a| a < 70.0).count();

        Ok(AccuracyAnalysis {
            stats,
            high_accuracy_count: high,
            medium_accuracy_count: medium,
            low_accuracy_count: low,
        })
    }

    /// Analyze complexity vs completion time
    pub fn complexity_impact(cases: &[CaseData]) -> ComplexityImpact {
        let simple_time = Self::avg_completion_for_complexity(cases, CaseComplexity::Simple);
        let moderate_time = Self::avg_completion_for_complexity(cases, CaseComplexity::Moderate);
        let complex_time = Self::avg_completion_for_complexity(cases, CaseComplexity::Complex);
        let very_complex_time = Self::avg_completion_for_complexity(cases, CaseComplexity::VeryComplex);

        ComplexityImpact {
            simple_avg_hours: simple_time,
            moderate_avg_hours: moderate_time,
            complex_avg_hours: complex_time,
            very_complex_avg_hours: very_complex_time,
            complexity_multiplier: if simple_time > 0.0 {
                very_complex_time / simple_time
            } else {
                1.0
            },
        }
    }

    /// Analyze evidence quality impact on accuracy
    pub fn evidence_quality_impact(cases: &[CaseData]) -> EvidenceImpact {
        let poor_acc = Self::avg_accuracy_for_evidence(cases, EvidenceQuality::Poor);
        let fair_acc = Self::avg_accuracy_for_evidence(cases, EvidenceQuality::Fair);
        let good_acc = Self::avg_accuracy_for_evidence(cases, EvidenceQuality::Good);
        let excellent_acc = Self::avg_accuracy_for_evidence(cases, EvidenceQuality::Excellent);

        EvidenceImpact {
            poor_avg_accuracy: poor_acc,
            fair_avg_accuracy: fair_acc,
            good_avg_accuracy: good_acc,
            excellent_avg_accuracy: excellent_acc,
            quality_improvement: excellent_acc - poor_acc,
        }
    }

    /// Calculate case workload metrics
    pub fn workload_metrics(cases: &[CaseData]) -> WorkloadMetrics {
        let total = cases.len();
        let active = cases
            .iter()
            .filter(|c| matches!(c.status, CaseStatus::Open | CaseStatus::InProgress | CaseStatus::UnderReview))
            .count();

        let now = Utc::now();
        let overdue = cases
            .iter()
            .filter(|c| {
                matches!(c.status, CaseStatus::Open | CaseStatus::InProgress)
                    && (now - c.created_at) > Duration::days(7)
            })
            .count();

        let avg_vehicles = if total > 0 {
            cases.iter().map(|c| c.num_vehicles).sum::<usize>() as f64 / total as f64
        } else {
            0.0
        };

        WorkloadMetrics {
            total_cases: total,
            active_cases: active,
            overdue_cases: overdue,
            avg_vehicles_per_case: avg_vehicles,
            capacity_utilization: if total > 0 {
                active as f64 / total as f64 * 100.0
            } else {
                0.0
            },
        }
    }

    fn avg_completion_for_complexity(cases: &[CaseData], complexity: CaseComplexity) -> f64 {
        let times: Vec<f64> = cases
            .iter()
            .filter(|c| c.complexity == complexity && c.closed_at.is_some())
            .filter_map(|c| {
                c.closed_at.map(|closed| {
                    (closed - c.created_at).num_hours() as f64
                })
            })
            .collect();

        if times.is_empty() {
            0.0
        } else {
            Statistics::mean(&times)
        }
    }

    fn avg_accuracy_for_evidence(cases: &[CaseData], quality: EvidenceQuality) -> f64 {
        let accuracies: Vec<f64> = cases
            .iter()
            .filter(|c| c.evidence_quality == quality)
            .filter(|c| matches!(c.status, CaseStatus::Closed | CaseStatus::Archived))
            .map(|c| c.reconstruction_accuracy)
            .collect();

        if accuracies.is_empty() {
            0.0
        } else {
            Statistics::mean(&accuracies)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionAnalysis {
    pub stats: DescriptiveStats,
    pub fast_completion_count: usize,    // < 24 hours
    pub normal_completion_count: usize,  // 24-72 hours
    pub slow_completion_count: usize,    // > 72 hours
    pub avg_completion_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDistribution {
    pub open: usize,
    pub in_progress: usize,
    pub under_review: usize,
    pub closed: usize,
    pub archived: usize,
    pub total: usize,
    pub closure_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyAnalysis {
    pub stats: DescriptiveStats,
    pub high_accuracy_count: usize,   // >= 90%
    pub medium_accuracy_count: usize, // 70-90%
    pub low_accuracy_count: usize,    // < 70%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityImpact {
    pub simple_avg_hours: f64,
    pub moderate_avg_hours: f64,
    pub complex_avg_hours: f64,
    pub very_complex_avg_hours: f64,
    pub complexity_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceImpact {
    pub poor_avg_accuracy: f64,
    pub fair_avg_accuracy: f64,
    pub good_avg_accuracy: f64,
    pub excellent_avg_accuracy: f64,
    pub quality_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadMetrics {
    pub total_cases: usize,
    pub active_cases: usize,
    pub overdue_cases: usize,
    pub avg_vehicles_per_case: f64,
    pub capacity_utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_case() -> CaseData {
        CaseData {
            id: "CASE001".to_string(),
            status: CaseStatus::Closed,
            created_at: Utc::now() - Duration::days(3),
            updated_at: Utc::now(),
            closed_at: Some(Utc::now()),
            complexity: CaseComplexity::Moderate,
            reconstruction_accuracy: 85.0,
            num_vehicles: 2,
            num_witnesses: 3,
            evidence_quality: EvidenceQuality::Good,
        }
    }

    #[test]
    fn test_status_distribution() {
        let cases = vec![sample_case()];
        let dist = CaseAnalytics::status_distribution(&cases);

        assert_eq!(dist.total, 1);
        assert_eq!(dist.closed, 1);
    }

    #[test]
    fn test_workload_metrics() {
        let cases = vec![sample_case()];
        let metrics = CaseAnalytics::workload_metrics(&cases);

        assert_eq!(metrics.total_cases, 1);
    }
}
