// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Port-City Foundry's first renderer-independent alternative contract.
//!
//! This module describes candidate portfolios, not physical truth. It deliberately
//! references existing Sol-Atlas evidence records by opaque identity and digest rather
//! than defining a second provenance, site-eligibility, GIS, economics, or solver model.
//!
//! A valid manifest is only a well-formed concept description. It is not proof of
//! site suitability, navigation safety, environmental acceptability, project finance,
//! permitting, or engineering feasibility.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Current wire-schema version for port study manifests.
pub const PORT_STUDY_SCHEMA_VERSION: u16 = 1;

/// The alternatives that a port-city study can compare.
///
/// NoBuildOperational is the required counterfactual baseline. It can represent
/// no capital intervention, demand management, scheduling changes, or operational
/// improvements that do not require a new constructed footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlternativeFamily {
    NoBuildOperational,
    ExistingPortUpgrade,
    BrownfieldExpansion,
    OffshoreTerminal,
    BarrierIslandNetwork,
    InlandBasinCanal,
    InlandWaterwayPort,
    DryPortRail,
    DistributedNetwork,
}

impl AlternativeFamily {
    /// All schema-supported families in stable order.
    pub const ALL: [Self; 9] = [
        Self::NoBuildOperational,
        Self::ExistingPortUpgrade,
        Self::BrownfieldExpansion,
        Self::OffshoreTerminal,
        Self::BarrierIslandNetwork,
        Self::InlandBasinCanal,
        Self::InlandWaterwayPort,
        Self::DryPortRail,
        Self::DistributedNetwork,
    ];

    /// Whether the alternative represents a newly configured physical footprint.
    ///
    /// The no-build/operational baseline may omit a design geometry. Every other
    /// family must identify the geometry artifact being evaluated.
    pub const fn requires_design_geometry(self) -> bool {
        !matches!(self, Self::NoBuildOperational)
    }
}

/// Coordinate axis ordering is explicit; it must never be guessed from numeric values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisOrder {
    EastNorth,
    NorthEast,
}

/// Opaque reference to an existing, separately governed geometry artifact.
///
/// This type does not perform coordinate transforms, topology checks, or spatial
/// overlays. Those remain owned by the GIS/provenance pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryReference {
    pub artifact_id: String,
    /// SHA-256 digest of the exact geometry artifact bytes.
    pub sha256: String,
    /// Authority identifier such as an EPSG code or another explicit CRS identifier.
    pub horizontal_crs: String,
    pub axis_order: AxisOrder,
    /// Explicit linear unit for projected geometry (for example, metre).
    pub linear_unit: String,
}

/// Opaque pointer to an existing Sol-Atlas evidence/provenance record.
///
/// The referred record remains authoritative for source, rights, freshness, coverage,
/// validation status, derivation lineage, and claim ceiling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub record_id: String,
    pub revision_id: String,
    /// SHA-256 digest of the exact referenced record/artifact.
    pub sha256: String,
}

/// A bounded design parameter with an explicit physical/economic unit.
///
/// Bounds are intervals, not point estimates. This module checks only their syntax
/// and ordering; it does not interpret the unit or certify that the interval is
/// physically plausible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundedParameter {
    pub parameter_id: String,
    pub lower: f64,
    pub upper: f64,
    pub unit: String,
}

/// Specialist-model domains required to evaluate an alternative.
///
/// Presence in this list is a declaration of required capability, not evidence that
/// a compatible solver exists, ran, converged, or is valid for the context of use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelDomain {
    Hydrodynamics,
    Waves,
    SedimentMorphology,
    Geotechnical,
    Structural,
    Navigation,
    FloodRisk,
    EcologyWaterQuality,
}

/// A single port, harbour, waterway, or inland-logistics alternative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortAlternative {
    pub alternative_id: String,
    pub family: AlternativeFamily,
    pub label: String,
    /// Absent only for the no-build/operational baseline.
    #[serde(default)]
    pub design_geometry: Option<GeometryReference>,
    /// References to existing evidence records; no status is inferred locally.
    #[serde(default)]
    pub evidence: Vec<EvidenceReference>,
    /// Bounded parameters with explicit units; interpretation belongs to domain owners.
    #[serde(default)]
    pub design_parameters: Vec<BoundedParameter>,
    /// Specialist modelling domains needed for a later qualified evaluation.
    #[serde(default)]
    pub required_models: Vec<ModelDomain>,
    /// Human-readable assumptions that must remain visible to downstream reviewers.
    #[serde(default)]
    pub assumptions: Vec<String>,
}

/// A comparison study containing a baseline and one or more alternative portfolios.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortStudyManifest {
    pub schema_version: u16,
    pub study_id: String,
    pub study_region_id: String,
    pub alternatives: Vec<PortAlternative>,
}

impl PortStudyManifest {
    /// Validate structural invariants only. Success means the manifest is well-formed;
    /// it does not mean a candidate is eligible or technically/economically feasible.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut findings = Vec::new();

        if self.schema_version != PORT_STUDY_SCHEMA_VERSION {
            findings.push(format!(
                "unsupported schema_version {}; expected {}",
                self.schema_version, PORT_STUDY_SCHEMA_VERSION
            ));
        }
        require_non_empty("study_id", &self.study_id, &mut findings);
        require_non_empty("study_region_id", &self.study_region_id, &mut findings);

        if self.alternatives.len() < 2 {
            findings.push("study must include a baseline and at least one comparator".into());
        }

        let mut alternative_ids = BTreeSet::new();
        let mut baseline_count = 0usize;

        for (index, alternative) in self.alternatives.iter().enumerate() {
            let prefix = format!("alternatives[{index}]");
            require_non_empty(
                &format!("{prefix}.alternative_id"),
                &alternative.alternative_id,
                &mut findings,
            );
            require_non_empty(
                &format!("{prefix}.label"),
                &alternative.label,
                &mut findings,
            );

            if !alternative.alternative_id.trim().is_empty()
                && !alternative_ids.insert(alternative.alternative_id.as_str())
            {
                findings.push(format!(
                    "{prefix}.alternative_id duplicates {}",
                    alternative.alternative_id
                ));
            }

            if alternative.family == AlternativeFamily::NoBuildOperational {
                baseline_count += 1;
            }

            match (&alternative.design_geometry, alternative.family.requires_design_geometry()) {
                (None, true) => findings.push(format!(
                    "{prefix}.design_geometry is required for {:?}",
                    alternative.family
                )),
                (Some(geometry), _) => {
                    require_non_empty(
                        &format!("{prefix}.design_geometry.artifact_id"),
                        &geometry.artifact_id,
                        &mut findings,
                    );
                    require_digest(
                        &format!("{prefix}.design_geometry.sha256"),
                        &geometry.sha256,
                        &mut findings,
                    );
                    require_non_empty(
                        &format!("{prefix}.design_geometry.horizontal_crs"),
                        &geometry.horizontal_crs,
                        &mut findings,
                    );
                    require_non_empty(
                        &format!("{prefix}.design_geometry.linear_unit"),
                        &geometry.linear_unit,
                        &mut findings,
                    );
                }
                (None, false) => {}
            }

            let mut evidence_ids = BTreeSet::new();
            for (evidence_index, evidence) in alternative.evidence.iter().enumerate() {
                let evidence_prefix = format!("{prefix}.evidence[{evidence_index}]");
                require_non_empty(
                    &format!("{evidence_prefix}.record_id"),
                    &evidence.record_id,
                    &mut findings,
                );
                require_non_empty(
                    &format!("{evidence_prefix}.revision_id"),
                    &evidence.revision_id,
                    &mut findings,
                );
                require_digest(
                    &format!("{evidence_prefix}.sha256"),
                    &evidence.sha256,
                    &mut findings,
                );
                if !evidence.record_id.trim().is_empty()
                    && !evidence_ids.insert(evidence.record_id.as_str())
                {
                    findings.push(format!(
                        "{evidence_prefix}.record_id duplicates {}",
                        evidence.record_id
                    ));
                }
            }

            let mut parameter_ids = BTreeSet::new();
            for (parameter_index, parameter) in
                alternative.design_parameters.iter().enumerate()
            {
                let parameter_prefix = format!("{prefix}.design_parameters[{parameter_index}]");
                require_non_empty(
                    &format!("{parameter_prefix}.parameter_id"),
                    &parameter.parameter_id,
                    &mut findings,
                );
                require_non_empty(
                    &format!("{parameter_prefix}.unit"),
                    &parameter.unit,
                    &mut findings,
                );

                if !parameter.parameter_id.trim().is_empty()
                    && !parameter_ids.insert(parameter.parameter_id.as_str())
                {
                    findings.push(format!(
                        "{parameter_prefix}.parameter_id duplicates {}",
                        parameter.parameter_id
                    ));
                }
                if !parameter.lower.is_finite() || !parameter.upper.is_finite() {
                    findings.push(format!("{parameter_prefix} bounds must be finite"));
                } else if parameter.lower > parameter.upper {
                    findings.push(format!(
                        "{parameter_prefix}.lower must not exceed upper"
                    ));
                }
            }

            let mut model_domains = BTreeSet::new();
            for domain in &alternative.required_models {
                if !model_domains.insert(*domain) {
                    findings.push(format!(
                        "{prefix}.required_models contains duplicate {domain:?}"
                    ));
                }
            }

            for (assumption_index, assumption) in alternative.assumptions.iter().enumerate() {
                if assumption.trim().is_empty() {
                    findings.push(format!(
                        "{prefix}.assumptions[{assumption_index}] must not be empty"
                    ));
                }
            }
        }

        if baseline_count != 1 {
            findings.push(format!(
                "study must contain exactly one NoBuildOperational baseline; found {baseline_count}"
            ));
        }

        if findings.is_empty() {
            Ok(())
        } else {
            findings.sort();
            Err(findings)
        }
    }

    /// Produce deterministic JSON bytes after sorting fields whose order is semantic-set
    /// order rather than decision order. The original manifest is not mutated.
    ///
    /// This is canonical for this versioned Rust/serde schema, not a claim of a
    /// cross-language canonical JSON standard.
    pub fn canonical_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut canonical = self.clone();
        canonical.alternatives.sort_by(|a, b| {
            a.alternative_id
                .cmp(&b.alternative_id)
                .then_with(|| a.family.cmp(&b.family))
        });

        for alternative in &mut canonical.alternatives {
            alternative.evidence.sort_by(|a, b| {
                a.record_id
                    .cmp(&b.record_id)
                    .then(a.revision_id.cmp(&b.revision_id))
            });
            alternative
                .design_parameters
                .sort_by(|a, b| a.parameter_id.cmp(&b.parameter_id));
            alternative.required_models.sort();
            alternative.assumptions.sort();
        }

        serde_json::to_vec(&canonical)
    }
}

fn require_non_empty(field: &str, value: &str, findings: &mut Vec<String>) {
    if value.trim().is_empty() {
        findings.push(format!("{field} must not be empty"));
    }
}

fn require_digest(field: &str, value: &str, findings: &mut Vec<String>) {
    let valid = value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    if !valid {
        findings.push(format!("{field} must be a 64-character hexadecimal SHA-256 digest"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geometry() -> GeometryReference {
        GeometryReference {
            artifact_id: "geometry-001".into(),
            sha256: "a".repeat(64),
            horizontal_crs: "EPSG:32736".into(),
            axis_order: AxisOrder::EastNorth,
            linear_unit: "metre".into(),
        }
    }

    fn alternative(id: &str, family: AlternativeFamily) -> PortAlternative {
        PortAlternative {
            alternative_id: id.into(),
            family,
            label: format!("{family:?}"),
            design_geometry: family.requires_design_geometry().then(geometry),
            evidence: Vec::new(),
            design_parameters: Vec::new(),
            required_models: Vec::new(),
            assumptions: Vec::new(),
        }
    }

    fn valid_manifest() -> PortStudyManifest {
        PortStudyManifest {
            schema_version: PORT_STUDY_SCHEMA_VERSION,
            study_id: "study-001".into(),
            study_region_id: "region-001".into(),
            alternatives: vec![
                alternative("baseline", AlternativeFamily::NoBuildOperational),
                alternative("dry-port", AlternativeFamily::DryPortRail),
            ],
        }
    }

    #[test]
    fn all_nine_alternative_families_are_declared() {
        assert_eq!(AlternativeFamily::ALL.len(), 9);
        assert_eq!(
            AlternativeFamily::ALL
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            9
        );
    }

    #[test]
    fn valid_manifest_requires_and_accepts_explicit_baseline() {
        assert!(valid_manifest().validate().is_ok());
    }

    #[test]
    fn missing_baseline_is_rejected() {
        let mut manifest = valid_manifest();
        manifest
            .alternatives
            .retain(|a| a.family != AlternativeFamily::NoBuildOperational);
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("exactly one NoBuildOperational")));
    }

    #[test]
    fn duplicate_alternative_ids_are_rejected() {
        let mut manifest = valid_manifest();
        manifest.alternatives[1].alternative_id = "baseline".into();
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("alternative_id duplicates")));
    }

    #[test]
    fn invalid_geometry_digest_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.alternatives[1]
            .design_geometry
            .as_mut()
            .unwrap()
            .sha256 = "not-a-digest".into();
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("design_geometry.sha256")));
    }

    #[test]
    fn construction_alternative_requires_geometry() {
        let mut manifest = valid_manifest();
        manifest.alternatives[1].design_geometry = None;
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("design_geometry is required")));
    }

    #[test]
    fn invalid_parameter_intervals_are_rejected() {
        let mut manifest = valid_manifest();
        manifest.alternatives[1].design_parameters.push(BoundedParameter {
            parameter_id: "channel_depth".into(),
            lower: 20.0,
            upper: 10.0,
            unit: "metre".into(),
        });
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("lower must not exceed upper")));
    }

    #[test]
    fn non_finite_parameter_intervals_are_rejected() {
        let mut manifest = valid_manifest();
        manifest.alternatives[1].design_parameters.push(BoundedParameter {
            parameter_id: "channel_depth".into(),
            lower: 0.0,
            upper: f64::INFINITY,
            unit: "metre".into(),
        });
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("bounds must be finite")));
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let mut manifest = valid_manifest();
        manifest.schema_version = PORT_STUDY_SCHEMA_VERSION + 1;
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("unsupported schema_version")));
    }

    #[test]
    fn collection_order_does_not_change_canonical_json() {
        let mut a = valid_manifest();
        a.alternatives[1].design_parameters = vec![
            BoundedParameter {
                parameter_id: "depth".into(),
                lower: 12.0,
                upper: 18.0,
                unit: "metre".into(),
            },
            BoundedParameter {
                parameter_id: "width".into(),
                lower: 100.0,
                upper: 180.0,
                unit: "metre".into(),
            },
        ];
        a.alternatives[1].required_models = vec![ModelDomain::Waves, ModelDomain::Navigation];
        a.alternatives[1].assumptions = vec!["second".into(), "first".into()];

        let mut b = a.clone();
        b.alternatives.reverse();
        b.alternatives[0].design_parameters.reverse();
        b.alternatives[0].required_models.reverse();
        b.alternatives[0].assumptions.reverse();

        assert_eq!(a.canonical_json().unwrap(), b.canonical_json().unwrap());
    }

    #[test]
    fn empty_evidence_is_not_fabricated_into_verified_status() {
        let manifest = valid_manifest();
        assert!(manifest.alternatives[1].evidence.is_empty());
        assert!(manifest.validate().is_ok());
        // Validation confirms only schema structure; evidence eligibility is intentionally
        // delegated to the existing provenance and site-selection owners.
    }
}
