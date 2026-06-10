use serde::{Deserialize, Serialize};

use crate::pillar::RequirementSeverity;
use crate::vendor::ComplianceStatus;

/// Configurable scoring weights for vendor evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub compliant_points: f64,
    pub partial_points: f64,
    pub non_compliant_points: f64,
    pub untested_points: f64,
    pub critical_weight: f64,
    pub high_weight: f64,
    pub medium_weight: f64,
    pub use_severity_weighting: bool,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            compliant_points: 1.0,
            partial_points: 0.5,
            non_compliant_points: 0.0,
            untested_points: 0.0,
            critical_weight: 3.0,
            high_weight: 2.0,
            medium_weight: 1.0,
            use_severity_weighting: true,
        }
    }
}

impl ScoringConfig {
    pub fn status_points(&self, status: ComplianceStatus) -> f64 {
        match status {
            ComplianceStatus::Compliant => self.compliant_points,
            ComplianceStatus::Partial => self.partial_points,
            ComplianceStatus::NonCompliant => self.non_compliant_points,
            ComplianceStatus::Untested => self.untested_points,
        }
    }

    pub fn severity_weight(&self, severity: RequirementSeverity) -> f64 {
        match severity {
            RequirementSeverity::Critical => self.critical_weight,
            RequirementSeverity::High => self.high_weight,
            RequirementSeverity::Medium => self.medium_weight,
        }
    }

    /// Weighted score for a single requirement (0.0 – 1.0 before ×100).
    pub fn requirement_score(
        &self,
        status: ComplianceStatus,
        severity: RequirementSeverity,
    ) -> (f64, f64) {
        let earned = self.status_points(status);
        let max = self.compliant_points;
        if self.use_severity_weighting {
            let w = self.severity_weight(severity);
            (earned * w, max * w)
        } else {
            (earned, max)
        }
    }
}
