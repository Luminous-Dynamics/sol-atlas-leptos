// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed bridge from qualified binary detector performance to likelihoods
//! consumable by the existing discrete EVSI kernel.
//!
//! This module does not implement EVSI, calibrate POD/PFA, or authenticate
//! qualification receipts. It prevents a route false-alarm rate from silently
//! becoming a per-opportunity probability and keeps synthetic point estimates
//! visibly below the physical qualification claim ceiling.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LikelihoodEvidenceStatus {
    SyntheticAnalyticOnly,
    ResearchPointEstimate,
    IndependentlyQualifiedForDeclaredDomain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FalseAlarmEvidence {
    /// P(detector call | no target) for one precisely defined search
    /// opportunity. The opportunity definition and support must be pinned.
    PerOpportunity {
        probability: f64,
        opportunity_definition_id: String,
        qualification_receipt_id: String,
        support_id: String,
        status: LikelihoodEvidenceStatus,
    },
    /// Convert a route-level event rate into a search-window probability only
    /// when a separately qualified conditional homogeneous Poisson model is
    /// recorded for the exact sensor/configuration/domain/support.
    HomogeneousPoissonRouteRate {
        false_alarms_per_km: f64,
        searched_route_km: f64,
        model_id: String,
        qualification_receipt_id: String,
        support_id: String,
        sensor_configuration_id: String,
        domain_id: String,
        process_model_status: PoissonProcessQualification,
        evidence_status: LikelihoodEvidenceStatus,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoissonProcessQualification {
    NotQualified,
    IndependentlyQualifiedForDeclaredDomain,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectionLikelihoodEvidence {
    pub evidence_id: String,
    pub envelope_id: String,
    pub target_class: String,
    pub support_id: String,
    pub sensor_configuration_id: String,
    pub domain_id: String,
    /// Point POD for the declared target/environment/configuration stratum.
    pub probability_of_detection: f64,
    pub pod_calibration_receipt_id: String,
    pub pod_status: LikelihoodEvidenceStatus,
    pub false_alarm_evidence: FalseAlarmEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryDetectionLikelihoods {
    pub evidence_id: String,
    pub envelope_id: String,
    pub target_class: String,
    pub support_id: String,
    pub sensor_configuration_id: String,
    pub domain_id: String,
    pub probability_of_detection: f64,
    pub probability_of_no_detection_given_target: f64,
    pub probability_of_false_alarm: f64,
    pub probability_of_no_alarm_given_absence: f64,
    pub evidence_status: LikelihoodEvidenceStatus,
    pub false_alarm_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionLikelihoodError {
    EmptyIdentifier(&'static str),
    InvalidProbability(&'static str),
    InvalidRouteRate,
    InvalidSearchLength,
    MissingPoissonModelIdentity,
    MissingPerOpportunityIdentity,
    FalseAlarmSupportMismatch,
    SensorConfigurationMismatch,
    DomainMismatch,
    PoissonProcessNotQualified,
    LikelihoodComplementMismatch(&'static str),
}

fn valid_probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn merge_evidence_status(
    pod: LikelihoodEvidenceStatus,
    false_alarm: LikelihoodEvidenceStatus,
) -> LikelihoodEvidenceStatus {
    use LikelihoodEvidenceStatus::*;
    match (pod, false_alarm) {
        (IndependentlyQualifiedForDeclaredDomain, IndependentlyQualifiedForDeclaredDomain) => {
            IndependentlyQualifiedForDeclaredDomain
        }
        (SyntheticAnalyticOnly, _) | (_, SyntheticAnalyticOnly) => SyntheticAnalyticOnly,
        _ => ResearchPointEstimate,
    }
}

impl DetectionLikelihoodEvidence {
    pub fn to_binary_likelihoods(
        &self,
    ) -> Result<BinaryDetectionLikelihoods, DetectionLikelihoodError> {
        for (field, value) in [
            ("evidence_id", self.evidence_id.as_str()),
            ("envelope_id", self.envelope_id.as_str()),
            ("target_class", self.target_class.as_str()),
            ("support_id", self.support_id.as_str()),
            (
                "sensor_configuration_id",
                self.sensor_configuration_id.as_str(),
            ),
            ("domain_id", self.domain_id.as_str()),
            (
                "pod_calibration_receipt_id",
                self.pod_calibration_receipt_id.as_str(),
            ),
        ] {
            if !non_empty(value) {
                return Err(DetectionLikelihoodError::EmptyIdentifier(field));
            }
        }
        if !valid_probability(self.probability_of_detection) {
            return Err(DetectionLikelihoodError::InvalidProbability(
                "probability_of_detection",
            ));
        }

        let (pfa, model_status, false_alarm_model) = match &self.false_alarm_evidence {
            FalseAlarmEvidence::PerOpportunity {
                probability,
                opportunity_definition_id,
                qualification_receipt_id,
                support_id,
                status,
            } => {
                if !valid_probability(*probability) {
                    return Err(DetectionLikelihoodError::InvalidProbability(
                        "false_alarm_probability",
                    ));
                }
                if !non_empty(opportunity_definition_id) || !non_empty(qualification_receipt_id) {
                    return Err(DetectionLikelihoodError::MissingPerOpportunityIdentity);
                }
                if support_id != &self.support_id {
                    return Err(DetectionLikelihoodError::FalseAlarmSupportMismatch);
                }
                (*probability, *status, "per_opportunity".to_string())
            }
            FalseAlarmEvidence::HomogeneousPoissonRouteRate {
                false_alarms_per_km,
                searched_route_km,
                model_id,
                qualification_receipt_id,
                support_id,
                sensor_configuration_id,
                domain_id,
                process_model_status,
                evidence_status,
            } => {
                if !false_alarms_per_km.is_finite() || *false_alarms_per_km < 0.0 {
                    return Err(DetectionLikelihoodError::InvalidRouteRate);
                }
                if !searched_route_km.is_finite() || *searched_route_km <= 0.0 {
                    return Err(DetectionLikelihoodError::InvalidSearchLength);
                }
                if !non_empty(model_id) || !non_empty(qualification_receipt_id) {
                    return Err(DetectionLikelihoodError::MissingPoissonModelIdentity);
                }
                if support_id != &self.support_id {
                    return Err(DetectionLikelihoodError::FalseAlarmSupportMismatch);
                }
                if sensor_configuration_id != &self.sensor_configuration_id {
                    return Err(DetectionLikelihoodError::SensorConfigurationMismatch);
                }
                if domain_id != &self.domain_id {
                    return Err(DetectionLikelihoodError::DomainMismatch);
                }
                if *process_model_status
                    != PoissonProcessQualification::IndependentlyQualifiedForDeclaredDomain
                {
                    return Err(DetectionLikelihoodError::PoissonProcessNotQualified);
                }

                let probability = 1.0 - (-*false_alarms_per_km * *searched_route_km).exp();
                (
                    probability,
                    *evidence_status,
                    "qualified_homogeneous_poisson_route_rate".to_string(),
                )
            }
        };

        Ok(BinaryDetectionLikelihoods {
            evidence_id: self.evidence_id.clone(),
            envelope_id: self.envelope_id.clone(),
            target_class: self.target_class.clone(),
            support_id: self.support_id.clone(),
            sensor_configuration_id: self.sensor_configuration_id.clone(),
            domain_id: self.domain_id.clone(),
            probability_of_detection: self.probability_of_detection,
            probability_of_no_detection_given_target: 1.0 - self.probability_of_detection,
            probability_of_false_alarm: pfa,
            probability_of_no_alarm_given_absence: 1.0 - pfa,
            evidence_status: merge_evidence_status(self.pod_status, model_status),
            false_alarm_model,
        })
    }
}

impl BinaryDetectionLikelihoods {
    fn validate_internal_probabilities(&self) -> Result<(), DetectionLikelihoodError> {
        for (field, value) in [
            ("probability_of_detection", self.probability_of_detection),
            (
                "probability_of_no_detection_given_target",
                self.probability_of_no_detection_given_target,
            ),
            ("probability_of_false_alarm", self.probability_of_false_alarm),
            (
                "probability_of_no_alarm_given_absence",
                self.probability_of_no_alarm_given_absence,
            ),
        ] {
            if !valid_probability(value) {
                return Err(DetectionLikelihoodError::InvalidProbability(field));
            }
        }

        if (self.probability_of_no_detection_given_target
            - (1.0 - self.probability_of_detection))
            .abs()
            > 1e-12
        {
            return Err(DetectionLikelihoodError::LikelihoodComplementMismatch(
                "probability_of_no_detection_given_target",
            ));
        }
        if (self.probability_of_no_alarm_given_absence
            - (1.0 - self.probability_of_false_alarm))
            .abs()
            > 1e-12
        {
            return Err(DetectionLikelihoodError::LikelihoodComplementMismatch(
                "probability_of_no_alarm_given_absence",
            ));
        }

        Ok(())
    }

    /// Convert to the map shape consumed by the existing discrete EVSI kernel:
    /// outcome -> latent state -> P(outcome | state).
    pub fn as_evsi_outcome_likelihoods(&self) -> BTreeMap<String, BTreeMap<String, f64>> {
        BTreeMap::from([
            (
                "hit".to_string(),
                BTreeMap::from([
                    ("target_present".to_string(), self.probability_of_detection),
                    ("target_absent".to_string(), self.probability_of_false_alarm),
                ]),
            ),
            (
                "no_hit".to_string(),
                BTreeMap::from([
                    (
                        "target_present".to_string(),
                        self.probability_of_no_detection_given_target,
                    ),
                    (
                        "target_absent".to_string(),
                        self.probability_of_no_alarm_given_absence,
                    ),
                ]),
            ),
        ])
    }

    /// Positive predictive value under an explicitly supplied target
    /// prevalence. The caller owns prevalence evidence; this function does not
    /// supply a default or infer it from sensor accuracy.
    pub fn posterior_target_probability_given_hit(
        &self,
        target_prevalence: f64,
    ) -> Result<f64, DetectionLikelihoodError> {
        self.validate_internal_probabilities()?;
        if !valid_probability(target_prevalence) {
            return Err(DetectionLikelihoodError::InvalidProbability(
                "target_prevalence",
            ));
        }
        let hit_probability = self.probability_of_detection * target_prevalence
            + self.probability_of_false_alarm * (1.0 - target_prevalence);
        if hit_probability <= f64::EPSILON {
            return Err(DetectionLikelihoodError::InvalidProbability(
                "zero_probability_hit",
            ));
        }
        Ok(self.probability_of_detection * target_prevalence / hit_probability)
    }

    /// Posterior target probability after a non-detection under an explicitly
    /// declared prevalence and one qualified search opportunity. A high
    /// posterior is not an absence claim; a low posterior is not a survey
    /// certification. The caller retains the target and opportunity scope.
    pub fn posterior_target_probability_given_no_hit(
        &self,
        target_prevalence: f64,
    ) -> Result<f64, DetectionLikelihoodError> {
        self.validate_internal_probabilities()?;
        if !valid_probability(target_prevalence) {
            return Err(DetectionLikelihoodError::InvalidProbability(
                "target_prevalence",
            ));
        }
        let no_hit_probability = self.probability_of_no_detection_given_target
            * target_prevalence
            + self.probability_of_no_alarm_given_absence * (1.0 - target_prevalence);
        if no_hit_probability <= f64::EPSILON {
            return Err(DetectionLikelihoodError::InvalidProbability(
                "zero_probability_no_hit",
            ));
        }
        Ok(self.probability_of_no_detection_given_target * target_prevalence
            / no_hit_probability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(
        pod: f64,
        pfa: f64,
        status: LikelihoodEvidenceStatus,
    ) -> DetectionLikelihoodEvidence {
        DetectionLikelihoodEvidence {
            evidence_id: "detector-evidence-001".to_string(),
            envelope_id: "soe-envelope-001".to_string(),
            target_class: "buried_utility".to_string(),
            support_id: "route-window-001".to_string(),
            sensor_configuration_id: "gpr-400mhz-configuration".to_string(),
            domain_id: "dry-sand-domain".to_string(),
            probability_of_detection: pod,
            pod_calibration_receipt_id: "pod-calibration-receipt-001".to_string(),
            pod_status: status,
            false_alarm_evidence: FalseAlarmEvidence::PerOpportunity {
                probability: pfa,
                opportunity_definition_id: "one-100m-route-search".to_string(),
                qualification_receipt_id: "false-alarm-receipt-001".to_string(),
                support_id: "route-window-001".to_string(),
                status,
            },
        }
    }

    #[test]
    fn binary_likelihoods_produce_complete_evsi_outcome_table() {
        let model = evidence(
            0.95,
            0.02,
            LikelihoodEvidenceStatus::IndependentlyQualifiedForDeclaredDomain,
        );
        let result = model.to_binary_likelihoods().unwrap();
        let table = result.as_evsi_outcome_likelihoods();
        assert_eq!(table["hit"]["target_present"], 0.95);
        assert!((table["no_hit"]["target_present"] - 0.05).abs() < 1e-12);
        assert_eq!(table["hit"]["target_absent"], 0.02);
        assert_eq!(table["no_hit"]["target_absent"], 0.98);
        for state in ["target_present", "target_absent"] {
            let total: f64 = table.values().map(|outcome| outcome[state]).sum();
            assert!((total - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn no_hit_posterior_uses_explicit_prevalence_and_both_likelihoods() {
        let model = evidence(
            0.95,
            0.02,
            LikelihoodEvidenceStatus::IndependentlyQualifiedForDeclaredDomain,
        )
        .to_binary_likelihoods()
        .unwrap();
        let posterior = model
            .posterior_target_probability_given_no_hit(0.5)
            .unwrap();
        assert!((posterior - (0.05 * 0.5) / (0.05 * 0.5 + 0.98 * 0.5)).abs() < 1e-12);
    }

    #[test]
    fn no_hit_posterior_does_not_default_target_prevalence() {
        let model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate)
            .to_binary_likelihoods()
            .unwrap();
        let low_prevalence = model
            .posterior_target_probability_given_no_hit(0.01)
            .unwrap();
        let high_prevalence = model
            .posterior_target_probability_given_no_hit(0.5)
            .unwrap();
        assert!(low_prevalence < high_prevalence);
    }

    #[test]
    fn no_hit_posterior_rejects_invalid_deserialized_likelihood_value() {
        let mut model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate)
            .to_binary_likelihoods()
            .unwrap();
        model.probability_of_no_detection_given_target = 1.5;
        assert_eq!(
            model.posterior_target_probability_given_no_hit(0.5),
            Err(DetectionLikelihoodError::InvalidProbability(
                "probability_of_no_detection_given_target"
            ))
        );
    }

    #[test]
    fn no_hit_posterior_rejects_inconsistent_complement_pair() {
        let mut model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate)
            .to_binary_likelihoods()
            .unwrap();
        model.probability_of_no_detection_given_target = 0.10;
        assert_eq!(
            model.posterior_target_probability_given_no_hit(0.5),
            Err(DetectionLikelihoodError::LikelihoodComplementMismatch(
                "probability_of_no_detection_given_target"
            ))
        );
    }

    #[test]
    fn no_hit_posterior_rejects_invalid_prevalence() {
        let model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate)
            .to_binary_likelihoods()
            .unwrap();
        assert_eq!(
            model.posterior_target_probability_given_no_hit(f64::NAN),
            Err(DetectionLikelihoodError::InvalidProbability("target_prevalence"))
        );
    }

    #[test]
    fn impossible_no_hit_is_rejected_instead_of_silently_defaulted() {
        let mut model = evidence(1.0, 1.0, LikelihoodEvidenceStatus::ResearchPointEstimate)
            .to_binary_likelihoods()
            .unwrap();
        model.probability_of_no_detection_given_target = 0.0;
        model.probability_of_no_alarm_given_absence = 0.0;
        assert_eq!(
            model.posterior_target_probability_given_no_hit(0.5),
            Err(DetectionLikelihoodError::InvalidProbability("zero_probability_no_hit"))
        );
    }

    #[test]
    fn synthetic_or_point_estimates_do_not_become_qualified_evsi_evidence() {
        let synthetic = evidence(0.95, 0.02, LikelihoodEvidenceStatus::SyntheticAnalyticOnly);
        let result = synthetic.to_binary_likelihoods().unwrap();
        assert_eq!(
            result.evidence_status,
            LikelihoodEvidenceStatus::SyntheticAnalyticOnly
        );

        let point = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate);
        let result = point.to_binary_likelihoods().unwrap();
        assert_eq!(
            result.evidence_status,
            LikelihoodEvidenceStatus::ResearchPointEstimate
        );
    }

    #[test]
    fn false_alarm_per_kilometre_is_not_implicitly_a_per_opportunity_probability() {
        let mut model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate);
        model.false_alarm_evidence = FalseAlarmEvidence::HomogeneousPoissonRouteRate {
            false_alarms_per_km: 0.5,
            searched_route_km: 1.0,
            model_id: "unqualified-poisson-model".to_string(),
            qualification_receipt_id: "rate-receipt-001".to_string(),
            support_id: "route-window-001".to_string(),
            sensor_configuration_id: "gpr-400mhz-configuration".to_string(),
            domain_id: "dry-sand-domain".to_string(),
            process_model_status: PoissonProcessQualification::NotQualified,
            evidence_status: LikelihoodEvidenceStatus::ResearchPointEstimate,
        };
        assert_eq!(
            model.to_binary_likelihoods(),
            Err(DetectionLikelihoodError::PoissonProcessNotQualified)
        );
    }

    #[test]
    fn qualified_poisson_conversion_depends_on_exact_search_length() {
        let mut model = evidence(
            0.95,
            0.02,
            LikelihoodEvidenceStatus::IndependentlyQualifiedForDeclaredDomain,
        );
        model.false_alarm_evidence = FalseAlarmEvidence::HomogeneousPoissonRouteRate {
            false_alarms_per_km: 0.5,
            searched_route_km: 1.0,
            model_id: "poisson-model-v1".to_string(),
            qualification_receipt_id: "rate-receipt-001".to_string(),
            support_id: "route-window-001".to_string(),
            sensor_configuration_id: "gpr-400mhz-configuration".to_string(),
            domain_id: "dry-sand-domain".to_string(),
            process_model_status:
                PoissonProcessQualification::IndependentlyQualifiedForDeclaredDomain,
            evidence_status: LikelihoodEvidenceStatus::IndependentlyQualifiedForDeclaredDomain,
        };
        let one_km = model.to_binary_likelihoods().unwrap();
        assert!((one_km.probability_of_false_alarm - (1.0 - (-0.5_f64).exp())).abs() < 1e-12);

        if let FalseAlarmEvidence::HomogeneousPoissonRouteRate {
            searched_route_km, ..
        } = &mut model.false_alarm_evidence
        {
            *searched_route_km = 2.0;
        }
        let two_km = model.to_binary_likelihoods().unwrap();
        assert!(two_km.probability_of_false_alarm > one_km.probability_of_false_alarm);
    }

    #[test]
    fn target_prevalence_changes_positive_predictive_value() {
        let model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate);
        let result = model.to_binary_likelihoods().unwrap();
        let rare = result.posterior_target_probability_given_hit(0.01).unwrap();
        let common = result.posterior_target_probability_given_hit(0.5).unwrap();
        assert!(common > rare);
        assert!((rare - (0.95 * 0.01 / (0.95 * 0.01 + 0.02 * 0.99))).abs() < 1e-12);
    }

    #[test]
    fn false_alarm_support_configuration_and_domain_must_match_exact_envelope() {
        let mut model = evidence(0.95, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate);
        if let FalseAlarmEvidence::PerOpportunity { support_id, .. } =
            &mut model.false_alarm_evidence
        {
            *support_id = "different-search-support".to_string();
        }
        assert_eq!(
            model.to_binary_likelihoods(),
            Err(DetectionLikelihoodError::FalseAlarmSupportMismatch)
        );
    }

    #[test]
    fn invalid_probabilities_are_rejected() {
        let model = evidence(1.1, 0.02, LikelihoodEvidenceStatus::ResearchPointEstimate);
        assert_eq!(
            model.to_binary_likelihoods(),
            Err(DetectionLikelihoodError::InvalidProbability(
                "probability_of_detection"
            ))
        );
    }
}
