// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance and verification metadata for imported physical-simulation results.
//!
//! This module does not run or validate a physics solver. It makes the claims
//! accompanying a result inspectable and fail-closed: a queued, skipped, stale
//! head, or failed gate never counts as a pass. Physical results should retain
//! their model assumptions, SI quantity labels, immutable source revision,
//! input/output hashes, and conservation evidence when displayed on the atlas.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Version of the serialized physical-run manifest schema supported here.
pub const PHYSICS_RUN_SCHEMA_VERSION: u16 = 1;

/// A named numerical or scientific verification gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationGateKind {
    InputValidation,
    SinglePhysicsReference,
    EnergyConservation,
    GridTimeRefinement,
    ReferenceComparison,
    CouplingStability,
    ExternalReview,
}

/// Workflow outcome as recorded by the producer of a run manifest.
///
/// `Queued`, `Skipped`, `NotRun`, and `Failed` are deliberately distinct
/// from `Passed`. A passing gate counts toward readiness only when it tested
/// the exact immutable source commit that produced the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateOutcome {
    NotRun,
    Queued,
    Skipped,
    Failed,
    Passed,
}

/// Source identity for the solver that produced a simulation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSource {
    /// Human-readable project identifier, for example `symthaea`.
    pub project: String,
    /// Canonical HTTPS repository URL.
    pub repository_url: String,
    /// Full 40-character Git commit SHA used for this run.
    pub commit_sha: String,
    /// Stable model/solver identifier.
    pub model_id: String,
    /// Human-readable model version.
    pub model_version: String,
}

/// Unit declaration for one physical quantity in the run or output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantityUnit {
    /// Quantity name, such as `temperature` or `stored_energy_change`.
    pub quantity: String,
    /// Unit symbol; physical numeric interfaces should prefer SI units.
    pub unit: String,
}

/// Spatial and temporal discretization metadata, where applicable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Discretization {
    /// Total number of spatial cells/elements, if applicable.
    pub cell_count: Option<u64>,
    /// Characteristic spatial spacing in metres, if applicable.
    pub characteristic_length_m: Option<f64>,
    /// Integration/coupling time step in seconds, if applicable.
    pub time_step_s: Option<f64>,
    /// Total simulated duration in seconds, if applicable.
    pub duration_s: Option<f64>,
}

/// Energy-balance terms for the selected control volume, all in joules.
///
/// Sign convention: input and outflow are non-negative magnitudes, while
/// stored-energy change may be positive or negative. The residual is
/// `energy_in_j - energy_out_j - stored_energy_change_j`. The caller must
/// identify all flows across the chosen boundary consistently; this type
/// cannot determine whether a physical model omitted a real energy pathway.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnergyBalance {
    pub energy_in_j: f64,
    pub energy_out_j: f64,
    pub stored_energy_change_j: f64,
    /// Reported signed residual; should match the computed residual.
    pub residual_j: f64,
    /// Accepted absolute residual in joules for this run.
    pub tolerance_j: f64,
}

impl EnergyBalance {
    /// Residual computed from the declared terms and sign convention.
    pub fn computed_residual_j(&self) -> f64 {
        self.energy_in_j - self.energy_out_j - self.stored_energy_change_j
    }

    /// Whether every term is finite and has a physically admissible sign.
    pub fn has_admissible_terms(&self) -> bool {
        self.energy_in_j.is_finite()
            && self.energy_in_j >= 0.0
            && self.energy_out_j.is_finite()
            && self.energy_out_j >= 0.0
            && self.stored_energy_change_j.is_finite()
            && self.residual_j.is_finite()
            && self.tolerance_j.is_finite()
            && self.tolerance_j >= 0.0
            && self.computed_residual_j().is_finite()
    }

    /// Whether the declared residual matches the ledger arithmetic to a
    /// scale-aware floating-point consistency tolerance.
    pub fn residual_is_consistent(&self) -> bool {
        if !self.has_admissible_terms() {
            return false;
        }
        let computed = self.computed_residual_j();
        let scale = self
            .energy_in_j
            .abs()
            .max(self.energy_out_j.abs())
            .max(self.stored_energy_change_j.abs())
            .max(1.0);
        (self.residual_j - computed).abs() <= 1.0e-12 * scale
    }

    /// Whether the declared balance meets its own absolute residual limit.
    ///
    /// This only checks arithmetic against the supplied terms and tolerance;
    /// it is not independent evidence that all physical terms were included.
    pub fn passes_declared_tolerance(&self) -> bool {
        self.residual_is_consistent() && self.residual_j.abs() <= self.tolerance_j
    }
}

/// Outcome of one named verification gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationGate {
    pub gate: VerificationGateKind,
    pub outcome: GateOutcome,
    /// Full source SHA actually tested. A mismatch with `ModelSource.commit_sha`
    /// is stale evidence and never counts toward readiness.
    pub tested_commit_sha: Option<String>,
    /// Stable URL to the result/check log/review evidence.
    pub evidence_url: Option<String>,
}

/// Summary metadata accompanying one simulation result.
///
/// The manifest describes provenance and verification evidence. It does not
/// certify the solver, the input data, or the suitability of the model for a
/// real-world engineering decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsRunManifest {
    pub schema_version: u16,
    pub run_id: String,
    pub source: ModelSource,
    /// SHA-256 of a canonical manifest of model inputs and boundary conditions.
    pub inputs_sha256: String,
    /// SHA-256 hashes of produced output artifacts.
    pub outputs_sha256: Vec<String>,
    /// Explicit model assumptions and scope limitations.
    pub assumptions: Vec<String>,
    /// Unit declarations for quantities exposed by the run.
    pub quantities: Vec<QuantityUnit>,
    /// Optional mesh/time-step metadata.
    pub discretization: Option<Discretization>,
    /// Optional energy ledger for the selected control volume.
    pub energy_balance: Option<EnergyBalance>,
    /// Evidence for individual verification gates.
    pub gates: Vec<VerificationGate>,
    /// Human-readable caveats, warnings, or known model limitations.
    pub notes: Vec<String>,
}

/// Structural validation failures for a physical-run manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    UnsupportedSchemaVersion,
    EmptyRequiredField,
    InvalidRepositoryUrl,
    InvalidCommitSha,
    InvalidSha256,
    MissingAssumptions,
    MissingQuantityUnits,
    DuplicateQuantity,
    InvalidDiscretization,
    InvalidEnergyBalance,
    DuplicateVerificationGate,
    InvalidGateEvidence,
}

impl PhysicsRunManifest {
    /// Validate structural invariants without elevating scientific maturity.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema_version != PHYSICS_RUN_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchemaVersion);
        }
        if self.run_id.trim().is_empty()
            || self.source.project.trim().is_empty()
            || self.source.model_id.trim().is_empty()
            || self.source.model_version.trim().is_empty()
        {
            return Err(ManifestError::EmptyRequiredField);
        }
        if !self.source.repository_url.starts_with("https://") {
            return Err(ManifestError::InvalidRepositoryUrl);
        }
        if !is_commit_sha(&self.source.commit_sha) {
            return Err(ManifestError::InvalidCommitSha);
        }
        if !is_sha256(&self.inputs_sha256)
            || self.outputs_sha256.is_empty()
            || self.outputs_sha256.iter().any(|hash| !is_sha256(hash))
        {
            return Err(ManifestError::InvalidSha256);
        }
        if self.assumptions.is_empty()
            || self.assumptions.iter().any(|assumption| assumption.trim().is_empty())
        {
            return Err(ManifestError::MissingAssumptions);
        }
        if self.quantities.is_empty() {
            return Err(ManifestError::MissingQuantityUnits);
        }
        let mut quantity_names = HashSet::new();
        for quantity in &self.quantities {
            if quantity.quantity.trim().is_empty() || quantity.unit.trim().is_empty() {
                return Err(ManifestError::MissingQuantityUnits);
            }
            if !quantity_names.insert(quantity.quantity.as_str()) {
                return Err(ManifestError::DuplicateQuantity);
            }
        }
        if let Some(grid) = self.discretization {
            if grid.cell_count == Some(0)
                || !optional_positive_finite(grid.characteristic_length_m)
                || !optional_positive_finite(grid.time_step_s)
                || !optional_positive_finite(grid.duration_s)
            {
                return Err(ManifestError::InvalidDiscretization);
            }
        }
        if let Some(balance) = self.energy_balance {
            if !balance.has_admissible_terms() || !balance.residual_is_consistent() {
                return Err(ManifestError::InvalidEnergyBalance);
            }
        }

        let mut seen_gates = HashSet::new();
        for gate in &self.gates {
            if !seen_gates.insert(gate.gate) {
                return Err(ManifestError::DuplicateVerificationGate);
            }
            if let Some(commit) = &gate.tested_commit_sha {
                if !is_commit_sha(commit) {
                    return Err(ManifestError::InvalidGateEvidence);
                }
            }
            if let Some(url) = &gate.evidence_url {
                if !url.starts_with("https://") {
                    return Err(ManifestError::InvalidGateEvidence);
                }
            }
            if gate.outcome == GateOutcome::Passed
                && (gate.tested_commit_sha.is_none() || gate.evidence_url.is_none())
            {
                return Err(ManifestError::InvalidGateEvidence);
            }
        }
        Ok(())
    }

    /// Return true only for a recorded pass tied to this exact source commit.
    pub fn exact_head_gate_passed(&self, kind: VerificationGateKind) -> bool {
        self.gates.iter().any(|gate| {
            gate.gate == kind
                && gate.outcome == GateOutcome::Passed
                && gate.tested_commit_sha.as_deref() == Some(self.source.commit_sha.as_str())
                && gate.evidence_url.as_deref().is_some_and(|url| url.starts_with("https://"))
        })
    }

    /// Conservative evidence maturity derived from the manifest and exact-head gates.
    ///
    /// This is a presentation/triage label, not a safety certification:
    /// `CoupledNumericallyVerified` means the listed gates passed at this head,
    /// not that an independent lab validated the model or that the real system
    /// is safe to operate.
    pub fn readiness(&self) -> EvidenceReadiness {
        if self.validate().is_err() {
            return EvidenceReadiness::Exploratory;
        }
        let hashes_recorded = is_sha256(&self.inputs_sha256)
            && !self.outputs_sha256.is_empty()
            && self.outputs_sha256.iter().all(|hash| is_sha256(hash));
        if !hashes_recorded {
            return EvidenceReadiness::Exploratory;
        }

        // A producer-supplied "passed" gate cannot override an energy ledger
        // whose own declared tolerance is exceeded or whose terms are absent.
        if !self
            .energy_balance
            .is_some_and(|balance| balance.passes_declared_tolerance())
        {
            return EvidenceReadiness::ProvenanceRecorded;
        }

        let core_gates = [
            VerificationGateKind::InputValidation,
            VerificationGateKind::SinglePhysicsReference,
            VerificationGateKind::EnergyConservation,
        ];
        if !core_gates.iter().all(|gate| self.exact_head_gate_passed(*gate)) {
            return EvidenceReadiness::ProvenanceRecorded;
        }

        let refinement_and_reference = [
            VerificationGateKind::GridTimeRefinement,
            VerificationGateKind::ReferenceComparison,
        ];
        if !refinement_and_reference
            .iter()
            .all(|gate| self.exact_head_gate_passed(*gate))
        {
            return EvidenceReadiness::NumericallyVerified;
        }

        if self.exact_head_gate_passed(VerificationGateKind::CouplingStability) {
            EvidenceReadiness::CoupledNumericallyVerified
        } else {
            EvidenceReadiness::ReferenceCompared
        }
    }
}

/// Derived evidence stage for visualization and workflow triage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceReadiness {
    /// Manifest invalid/incomplete or required provenance/hash fields are malformed.
    Exploratory,
    /// Provenance and hash strings are recorded, but matching artifacts have not been replayed or verified here.
    ProvenanceRecorded,
    /// Input, single-physics reference, and energy-conservation gates passed at exact head.
    NumericallyVerified,
    /// Above plus grid/time refinement and reference-comparison gates.
    ReferenceCompared,
    /// Above plus a coupled-stability gate at the same exact head.
    CoupledNumericallyVerified,
}

fn is_commit_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn optional_positive_finite(value: Option<f64>) -> bool {
    value.is_none_or(|number| number.is_finite() && number > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_sha(byte: char) -> String {
        std::iter::repeat_n(byte, 40).collect()
    }

    fn hash(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn passed_gate(gate: VerificationGateKind, commit: &str) -> VerificationGate {
        VerificationGate {
            gate,
            outcome: GateOutcome::Passed,
            tested_commit_sha: Some(commit.to_string()),
            evidence_url: Some("https://github.com/example/project/actions/runs/1".into()),
        }
    }

    fn fixture() -> PhysicsRunManifest {
        let commit = full_sha('a');
        PhysicsRunManifest {
            schema_version: PHYSICS_RUN_SCHEMA_VERSION,
            run_id: "joule-heating-reference-001".into(),
            source: ModelSource {
                project: "symthaea".into(),
                repository_url: "https://github.com/Luminous-Dynamics/symthaea".into(),
                commit_sha: commit.clone(),
                model_id: "lumped-joule-heating".into(),
                model_version: "0.1.0".into(),
            },
            inputs_sha256: hash('b'),
            outputs_sha256: vec![hash('c')],
            assumptions: vec![
                "constant resistance".into(),
                "constant specific heat capacity".into(),
                "adiabatic well-mixed conductor".into(),
            ],
            quantities: vec![
                QuantityUnit { quantity: "temperature".into(), unit: "K".into() },
                QuantityUnit { quantity: "energy".into(), unit: "J".into() },
            ],
            discretization: Some(Discretization {
                cell_count: None,
                characteristic_length_m: None,
                time_step_s: Some(1.0),
                duration_s: Some(10.0),
            }),
            energy_balance: Some(EnergyBalance {
                energy_in_j: 180.0,
                energy_out_j: 0.0,
                stored_energy_change_j: 180.0,
                residual_j: 0.0,
                tolerance_j: 1.0e-9,
            }),
            gates: Vec::new(),
            notes: vec!["Reference case; not a spatial solver.".into()],
        }
    }

    #[test]
    fn valid_fixture_records_provenance_without_claiming_verification() {
        let run = fixture();
        assert_eq!(run.validate(), Ok(()));
        assert_eq!(run.readiness(), EvidenceReadiness::ProvenanceRecorded);
    }

    #[test]
    fn queued_skipped_and_stale_head_gates_do_not_count_as_passes() {
        let commit = full_sha('a');
        let mut run = fixture();
        run.gates = vec![
            VerificationGate {
                gate: VerificationGateKind::InputValidation,
                outcome: GateOutcome::Queued,
                tested_commit_sha: Some(commit.clone()),
                evidence_url: None,
            },
            VerificationGate {
                gate: VerificationGateKind::SinglePhysicsReference,
                outcome: GateOutcome::Skipped,
                tested_commit_sha: Some(commit.clone()),
                evidence_url: None,
            },
            passed_gate(VerificationGateKind::EnergyConservation, &full_sha('d')),
        ];
        assert_eq!(run.validate(), Ok(()));
        assert!(!run.exact_head_gate_passed(VerificationGateKind::InputValidation));
        assert!(!run.exact_head_gate_passed(VerificationGateKind::SinglePhysicsReference));
        assert!(!run.exact_head_gate_passed(VerificationGateKind::EnergyConservation));
        assert_eq!(run.readiness(), EvidenceReadiness::ProvenanceRecorded);
    }

    #[test]
    fn exact_head_gate_sequence_advances_readiness_conservatively() {
        let commit = full_sha('a');
        let mut run = fixture();
        run.gates = vec![
            passed_gate(VerificationGateKind::InputValidation, &commit),
            passed_gate(VerificationGateKind::SinglePhysicsReference, &commit),
            passed_gate(VerificationGateKind::EnergyConservation, &commit),
        ];
        assert_eq!(run.readiness(), EvidenceReadiness::NumericallyVerified);

        run.gates.push(passed_gate(VerificationGateKind::GridTimeRefinement, &commit));
        run.gates.push(passed_gate(VerificationGateKind::ReferenceComparison, &commit));
        assert_eq!(run.readiness(), EvidenceReadiness::ReferenceCompared);

        run.gates.push(passed_gate(VerificationGateKind::CouplingStability, &commit));
        assert_eq!(run.readiness(), EvidenceReadiness::CoupledNumericallyVerified);
    }

    #[test]
    fn energy_gate_cannot_override_a_failed_declared_balance_tolerance() {
        let commit = full_sha('a');
        let mut run = fixture();
        run.energy_balance = Some(EnergyBalance {
            energy_in_j: 181.0,
            energy_out_j: 0.0,
            stored_energy_change_j: 180.0,
            residual_j: 1.0,
            tolerance_j: 0.1,
        });
        run.gates = vec![
            passed_gate(VerificationGateKind::InputValidation, &commit),
            passed_gate(VerificationGateKind::SinglePhysicsReference, &commit),
            passed_gate(VerificationGateKind::EnergyConservation, &commit),
        ];

        assert_eq!(run.validate(), Ok(())); // retain the recorded run and its failure
        assert_eq!(run.readiness(), EvidenceReadiness::ProvenanceRecorded);
        assert!(!run.energy_balance.unwrap().passes_declared_tolerance());
    }

    #[test]
    fn serialized_manifest_round_trips_with_stable_gate_names() {
        let commit = full_sha('a');
        let mut run = fixture();
        run.gates.push(passed_gate(
            VerificationGateKind::SinglePhysicsReference,
            &commit,
        ));

        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("single_physics_reference"));
        let decoded: PhysicsRunManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, run);
        assert_eq!(decoded.validate(), Ok(()));
    }

    #[test]
    fn invalid_manifest_and_ledger_arithmetic_fail_closed() {
        let mut bad_version = fixture();
        bad_version.schema_version = PHYSICS_RUN_SCHEMA_VERSION + 1;
        assert_eq!(bad_version.validate(), Err(ManifestError::UnsupportedSchemaVersion));
        assert_eq!(bad_version.readiness(), EvidenceReadiness::Exploratory);

        let mut bad_sha = fixture();
        bad_sha.source.commit_sha = "deadbeef".into();
        assert_eq!(bad_sha.validate(), Err(ManifestError::InvalidCommitSha));

        let mut bad_ledger = fixture();
        bad_ledger.energy_balance.as_mut().unwrap().residual_j = 10.0;
        assert_eq!(bad_ledger.validate(), Err(ManifestError::InvalidEnergyBalance));
    }

    #[test]
    fn non_finite_energy_and_discretization_are_rejected() {
        let mut bad_energy = fixture();
        bad_energy.energy_balance.as_mut().unwrap().energy_in_j = f64::INFINITY;
        assert_eq!(bad_energy.validate(), Err(ManifestError::InvalidEnergyBalance));

        let mut bad_step = fixture();
        bad_step.discretization.as_mut().unwrap().time_step_s = Some(f64::NAN);
        assert_eq!(bad_step.validate(), Err(ManifestError::InvalidDiscretization));
    }

    #[test]
    fn passed_gate_requires_an_immutable_revision_and_evidence_url() {
        let mut run = fixture();
        run.gates.push(VerificationGate {
            gate: VerificationGateKind::InputValidation,
            outcome: GateOutcome::Passed,
            tested_commit_sha: None,
            evidence_url: None,
        });
        assert_eq!(run.validate(), Err(ManifestError::InvalidGateEvidence));
    }

    #[test]
    fn duplicate_quantity_and_gate_declarations_are_rejected() {
        let mut duplicate_quantity = fixture();
        duplicate_quantity.quantities.push(duplicate_quantity.quantities[0].clone());
        assert_eq!(duplicate_quantity.validate(), Err(ManifestError::DuplicateQuantity));

        let mut duplicate_gate = fixture();
        let pass = passed_gate(VerificationGateKind::InputValidation, &full_sha('a'));
        duplicate_gate.gates = vec![pass.clone(), pass];
        assert_eq!(duplicate_gate.validate(), Err(ManifestError::DuplicateVerificationGate));
    }
}
