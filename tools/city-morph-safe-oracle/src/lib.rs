//! Synthetic-first evidence gate for neighbourhood safety design comparisons.
//!
//! This crate deliberately does not predict crime, determine that a place is safe,
//! calculate fire/structural performance, or certify legal compliance. It only
//! classifies whether a bounded synthetic review packet is blocked, unknown, or
//! eligible to be passed to qualified professionals for review.

pub const CORPUS_REVISION: &str = "city-morph-safe-v0.1";
pub const PROFILE_REVISION: &str = "neighbourhood-intervention-review-v0.1";
pub const EXPECTED_CASE_COUNT: usize = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceState {
    Verified,
    Missing,
    Stale,
    Unsupported,
    ScopeMismatch,
    Restricted,
    Conflicting,
}

impl EvidenceState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::Unsupported => "unsupported",
            Self::ScopeMismatch => "scope_mismatch",
            Self::Restricted => "restricted",
            Self::Conflicting => "conflicting",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateState {
    Satisfied,
    Failed,
    Unknown,
}

impl GateState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrityState {
    Valid,
    InvalidDenominator,
    ScopeMismatch,
    CandidateDigestMismatch,
    ProfileRevisionMismatch,
    CorpusRevisionMismatch,
    DuplicateIdentifier,
}

impl IntegrityState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::InvalidDenominator => "invalid_denominator",
            Self::ScopeMismatch => "scope_mismatch",
            Self::CandidateDigestMismatch => "candidate_digest_mismatch",
            Self::ProfileRevisionMismatch => "profile_revision_mismatch",
            Self::CorpusRevisionMismatch => "corpus_revision_mismatch",
            Self::DuplicateIdentifier => "duplicate_identifier",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestedOutput {
    ProfessionalReviewPacket,
    AutomatedSafetyClassification,
    CrimeReductionCertification,
    FireSafetyCertification,
    CodeComplianceCertification,
}

impl RequestedOutput {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProfessionalReviewPacket => "professional_review_packet",
            Self::AutomatedSafetyClassification => "automated_safety_classification",
            Self::CrimeReductionCertification => "crime_reduction_certification",
            Self::FireSafetyCertification => "fire_safety_certification",
            Self::CodeComplianceCertification => "code_compliance_certification",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Blocked,
    Unknown,
    EligibleForProfessionalReview,
}

impl Disposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::Unknown => "unknown",
            Self::EligibleForProfessionalReview => "eligible_for_professional_review",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceSet {
    pub recorded_crime: EvidenceState,
    pub victimisation_survey: EvidenceState,
    pub route_audit: EvidenceState,
    pub site_context: EvidenceState,
    pub adjacent_area_impact: EvidenceState,
    pub maintenance_baseline: EvidenceState,
}

impl EvidenceSet {
    pub const fn all_verified() -> Self {
        Self {
            recorded_crime: EvidenceState::Verified,
            victimisation_survey: EvidenceState::Verified,
            route_audit: EvidenceState::Verified,
            site_context: EvidenceState::Verified,
            adjacent_area_impact: EvidenceState::Verified,
            maintenance_baseline: EvidenceState::Verified,
        }
    }

    pub const fn as_array(self) -> [EvidenceState; 6] {
        [
            self.recorded_crime,
            self.victimisation_survey,
            self.route_audit,
            self.site_context,
            self.adjacent_area_impact,
            self.maintenance_baseline,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardGates {
    pub fire_egress: GateState,
    pub responder_access: GateState,
    pub accessible_journey: GateState,
    pub public_access: GateState,
    pub privacy_and_consent: GateState,
    pub emergency_manual_fallback: GateState,
    pub no_unmitigated_spillover: GateState,
}

impl HardGates {
    pub const fn all_satisfied() -> Self {
        Self {
            fire_egress: GateState::Satisfied,
            responder_access: GateState::Satisfied,
            accessible_journey: GateState::Satisfied,
            public_access: GateState::Satisfied,
            privacy_and_consent: GateState::Satisfied,
            emergency_manual_fallback: GateState::Satisfied,
            no_unmitigated_spillover: GateState::Satisfied,
        }
    }

    pub const fn as_array(self) -> [GateState; 7] {
        [
            self.fire_egress,
            self.responder_access,
            self.accessible_journey,
            self.public_access,
            self.privacy_and_consent,
            self.emergency_manual_fallback,
            self.no_unmitigated_spillover,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub id: &'static str,
    pub title: &'static str,
    pub evidence: EvidenceSet,
    pub gates: HardGates,
    pub integrity: IntegrityState,
    pub requested_output: RequestedOutput,
    /// Fixture oracle expectation; evaluate() never reads this field.
    pub expected: Disposition,
}

const fn baseline(id: &'static str, title: &'static str) -> Scenario {
    Scenario {
        id,
        title,
        evidence: EvidenceSet::all_verified(),
        gates: HardGates::all_satisfied(),
        integrity: IntegrityState::Valid,
        requested_output: RequestedOutput::ProfessionalReviewPacket,
        expected: Disposition::EligibleForProfessionalReview,
    }
}

/// Frozen v0.1 synthetic cases. These are test vectors, not measurements of
/// Johannesburg, South Africa, or any real neighbourhood.
pub const SCENARIOS: [Scenario; EXPECTED_CASE_COUNT] = [
    baseline("SAFE-001", "complete synthetic review packet"),
    Scenario {
        id: "SAFE-002",
        title: "new barrier blocks fire egress",
        gates: HardGates {
            fire_egress: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-002", "new barrier blocks fire egress")
    },
    Scenario {
        id: "SAFE-003",
        title: "access control prevents responder access",
        gates: HardGates {
            responder_access: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-003", "access control prevents responder access")
    },
    Scenario {
        id: "SAFE-004",
        title: "design severs accessible journey",
        gates: HardGates {
            accessible_journey: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-004", "design severs accessible journey")
    },
    Scenario {
        id: "SAFE-005",
        title: "intervention removes lawful public access",
        gates: HardGates {
            public_access: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-005", "intervention removes lawful public access")
    },
    Scenario {
        id: "SAFE-006",
        title: "privacy or consent requirement failed",
        gates: HardGates {
            privacy_and_consent: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-006", "privacy or consent requirement failed")
    },
    Scenario {
        id: "SAFE-007",
        title: "emergency manual fallback unavailable",
        gates: HardGates {
            emergency_manual_fallback: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-007", "emergency manual fallback unavailable")
    },
    Scenario {
        id: "SAFE-008",
        title: "unmitigated risk displacement to adjacent area",
        gates: HardGates {
            no_unmitigated_spillover: GateState::Failed,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-008", "unmitigated risk displacement to adjacent area")
    },
    Scenario {
        id: "SAFE-009",
        title: "fire egress review unresolved",
        gates: HardGates {
            fire_egress: GateState::Unknown,
            ..HardGates::all_satisfied()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-009", "fire egress review unresolved")
    },
    Scenario {
        id: "SAFE-010",
        title: "recorded crime source missing",
        evidence: EvidenceSet {
            recorded_crime: EvidenceState::Missing,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-010", "recorded crime source missing")
    },
    Scenario {
        id: "SAFE-011",
        title: "victimisation survey missing",
        evidence: EvidenceSet {
            victimisation_survey: EvidenceState::Missing,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-011", "victimisation survey missing")
    },
    Scenario {
        id: "SAFE-012",
        title: "route audit stale",
        evidence: EvidenceSet {
            route_audit: EvidenceState::Stale,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-012", "route audit stale")
    },
    Scenario {
        id: "SAFE-013",
        title: "site context unsupported",
        evidence: EvidenceSet {
            site_context: EvidenceState::Unsupported,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-013", "site context unsupported")
    },
    Scenario {
        id: "SAFE-014",
        title: "adjacent-area impact evidence missing",
        evidence: EvidenceSet {
            adjacent_area_impact: EvidenceState::Missing,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-014", "adjacent-area impact evidence missing")
    },
    Scenario {
        id: "SAFE-015",
        title: "maintenance baseline stale",
        evidence: EvidenceSet {
            maintenance_baseline: EvidenceState::Stale,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-015", "maintenance baseline stale")
    },
    Scenario {
        id: "SAFE-016",
        title: "restricted personal data used as evidence",
        evidence: EvidenceSet {
            victimisation_survey: EvidenceState::Restricted,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Blocked,
        ..baseline("SAFE-016", "restricted personal data used as evidence")
    },
    Scenario {
        id: "SAFE-017",
        title: "route evidence has mismatched spatial scope",
        evidence: EvidenceSet {
            route_audit: EvidenceState::ScopeMismatch,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-017", "route evidence has mismatched spatial scope")
    },
    Scenario {
        id: "SAFE-018",
        title: "crime count supplied without a valid denominator",
        integrity: IntegrityState::InvalidDenominator,
        expected: Disposition::Blocked,
        ..baseline("SAFE-018", "crime count supplied without a valid denominator")
    },
    Scenario {
        id: "SAFE-019",
        title: "candidate geography does not match assessment scope",
        integrity: IntegrityState::ScopeMismatch,
        expected: Disposition::Unknown,
        ..baseline("SAFE-019", "candidate geography does not match assessment scope")
    },
    Scenario {
        id: "SAFE-020",
        title: "candidate revision digest mismatch",
        integrity: IntegrityState::CandidateDigestMismatch,
        expected: Disposition::Unknown,
        ..baseline("SAFE-020", "candidate revision digest mismatch")
    },
    Scenario {
        id: "SAFE-021",
        title: "safety profile revision mismatch",
        integrity: IntegrityState::ProfileRevisionMismatch,
        expected: Disposition::Unknown,
        ..baseline("SAFE-021", "safety profile revision mismatch")
    },
    Scenario {
        id: "SAFE-022",
        title: "benchmark corpus revision mismatch",
        integrity: IntegrityState::CorpusRevisionMismatch,
        expected: Disposition::Unknown,
        ..baseline("SAFE-022", "benchmark corpus revision mismatch")
    },
    Scenario {
        id: "SAFE-023",
        title: "duplicate scenario identifier",
        integrity: IntegrityState::DuplicateIdentifier,
        expected: Disposition::Blocked,
        ..baseline("SAFE-023", "duplicate scenario identifier")
    },
    Scenario {
        id: "SAFE-024",
        title: "camera detections requested as crime-reduction proof",
        requested_output: RequestedOutput::CrimeReductionCertification,
        expected: Disposition::Blocked,
        ..baseline("SAFE-024", "camera detections requested as crime-reduction proof")
    },
    Scenario {
        id: "SAFE-025",
        title: "rendering-only request for automated safety classification",
        requested_output: RequestedOutput::AutomatedSafetyClassification,
        expected: Disposition::Blocked,
        ..baseline("SAFE-025", "rendering-only request for automated safety classification")
    },
    Scenario {
        id: "SAFE-026",
        title: "model-confidence-only request for safety classification",
        requested_output: RequestedOutput::AutomatedSafetyClassification,
        expected: Disposition::Blocked,
        ..baseline("SAFE-026", "model-confidence-only request for safety classification")
    },
    Scenario {
        id: "SAFE-027",
        title: "automatic fire-safety certification requested",
        requested_output: RequestedOutput::FireSafetyCertification,
        expected: Disposition::Blocked,
        ..baseline("SAFE-027", "automatic fire-safety certification requested")
    },
    Scenario {
        id: "SAFE-028",
        title: "automatic code-compliance certification requested",
        requested_output: RequestedOutput::CodeComplianceCertification,
        expected: Disposition::Blocked,
        ..baseline("SAFE-028", "automatic code-compliance certification requested")
    },
    Scenario {
        id: "SAFE-029",
        title: "contradictory source evidence",
        evidence: EvidenceSet {
            recorded_crime: EvidenceState::Conflicting,
            ..EvidenceSet::all_verified()
        },
        expected: Disposition::Unknown,
        ..baseline("SAFE-029", "contradictory source evidence")
    },
    baseline("SAFE-030", "second fully evidenced design alternative"),
];

/// Pure, non-probabilistic evaluation of a synthetic evidence packet.
///
/// This function is intentionally incapable of producing a real-world safety,
/// crime-reduction, fire-safety, or code-compliance finding.
pub fn evaluate(case: &Scenario) -> Disposition {
    if case.requested_output != RequestedOutput::ProfessionalReviewPacket {
        return Disposition::Blocked;
    }

    if matches!(
        case.integrity,
        IntegrityState::InvalidDenominator | IntegrityState::DuplicateIdentifier
    ) {
        return Disposition::Blocked;
    }

    if case.gates.as_array().contains(&GateState::Failed) {
        return Disposition::Blocked;
    }

    if case.evidence.as_array().contains(&EvidenceState::Restricted) {
        return Disposition::Blocked;
    }

    if case.integrity != IntegrityState::Valid {
        return Disposition::Unknown;
    }

    if case.gates.as_array().contains(&GateState::Unknown) {
        return Disposition::Unknown;
    }

    if case
        .evidence
        .as_array()
        .iter()
        .any(|state| *state != EvidenceState::Verified)
    {
        return Disposition::Unknown;
    }

    Disposition::EligibleForProfessionalReview
}

/// Stable, non-cryptographic FNV-1a fixture identifier. It detects accidental
/// corpus drift; it is NOT a cryptographic digest or a provenance signature.
pub fn fixture_fingerprint() -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    fn feed(hash: &mut u64, bytes: &[u8]) {
        const PRIME: u64 = 0x00000100000001b3;
        for byte in bytes {
            *hash ^= u64::from(*byte);
            *hash = (*hash).wrapping_mul(PRIME);
        }
        *hash ^= 0xff;
        *hash = (*hash).wrapping_mul(PRIME);
    }

    let mut hash = OFFSET;
    feed(&mut hash, CORPUS_REVISION.as_bytes());
    feed(&mut hash, PROFILE_REVISION.as_bytes());

    for case in SCENARIOS {
        feed(&mut hash, case.id.as_bytes());
        feed(&mut hash, case.title.as_bytes());
        for state in case.evidence.as_array() {
            feed(&mut hash, state.as_str().as_bytes());
        }
        for gate in case.gates.as_array() {
            feed(&mut hash, gate.as_str().as_bytes());
        }
        feed(&mut hash, case.integrity.as_str().as_bytes());
        feed(&mut hash, case.requested_output.as_str().as_bytes());
        feed(&mut hash, case.expected.as_str().as_bytes());
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn frozen_suite_has_exactly_thirty_unique_ids() {
        assert_eq!(SCENARIOS.len(), EXPECTED_CASE_COUNT);
        let ids: HashSet<&str> = SCENARIOS.iter().map(|case| case.id).collect();
        assert_eq!(ids.len(), EXPECTED_CASE_COUNT);
    }

    #[test]
    fn all_frozen_expected_dispositions_match_the_pure_oracle() {
        for case in SCENARIOS {
            assert_eq!(
                evaluate(&case),
                case.expected,
                "{}: {}",
                case.id,
                case.title
            );
        }
    }

    #[test]
    fn failed_hard_gate_dominates_every_favourable_input() {
        let mut case = SCENARIOS[0];
        case.gates.fire_egress = GateState::Failed;
        assert_eq!(evaluate(&case), Disposition::Blocked);
    }

    #[test]
    fn unknown_gate_never_becomes_eligible() {
        let mut case = SCENARIOS[0];
        case.gates.responder_access = GateState::Unknown;
        assert_eq!(evaluate(&case), Disposition::Unknown);
    }

    #[test]
    fn missing_or_stale_evidence_never_becomes_eligible() {
        for state in [EvidenceState::Missing, EvidenceState::Stale] {
            let mut case = SCENARIOS[0];
            case.evidence.recorded_crime = state;
            assert_eq!(evaluate(&case), Disposition::Unknown);
        }
    }

    #[test]
    fn restricted_evidence_is_blocked() {
        let mut case = SCENARIOS[0];
        case.evidence.victimisation_survey = EvidenceState::Restricted;
        assert_eq!(evaluate(&case), Disposition::Blocked);
    }

    #[test]
    fn adjacent_area_evidence_is_non_optional_for_this_profile() {
        let mut case = SCENARIOS[0];
        case.evidence.adjacent_area_impact = EvidenceState::Missing;
        assert_eq!(evaluate(&case), Disposition::Unknown);
    }

    #[test]
    fn invalid_denominators_and_duplicate_ids_are_blocked() {
        for integrity in [
            IntegrityState::InvalidDenominator,
            IntegrityState::DuplicateIdentifier,
        ] {
            let mut case = SCENARIOS[0];
            case.integrity = integrity;
            assert_eq!(evaluate(&case), Disposition::Blocked);
        }
    }

    #[test]
    fn scope_and_revision_mismatches_are_unknown() {
        for integrity in [
            IntegrityState::ScopeMismatch,
            IntegrityState::CandidateDigestMismatch,
            IntegrityState::ProfileRevisionMismatch,
            IntegrityState::CorpusRevisionMismatch,
        ] {
            let mut case = SCENARIOS[0];
            case.integrity = integrity;
            assert_eq!(evaluate(&case), Disposition::Unknown);
        }
    }

    #[test]
    fn forbidden_automatic_certification_requests_are_blocked() {
        for request in [
            RequestedOutput::AutomatedSafetyClassification,
            RequestedOutput::CrimeReductionCertification,
            RequestedOutput::FireSafetyCertification,
            RequestedOutput::CodeComplianceCertification,
        ] {
            let mut case = SCENARIOS[0];
            case.requested_output = request;
            assert_eq!(evaluate(&case), Disposition::Blocked);
        }
    }

    #[test]
    fn identical_frozen_inputs_produce_identical_dispositions_and_fingerprint() {
        let first: Vec<Disposition> = SCENARIOS.iter().map(evaluate).collect();
        let second: Vec<Disposition> = SCENARIOS.iter().map(evaluate).collect();
        assert_eq!(first, second);
        assert_eq!(fixture_fingerprint(), fixture_fingerprint());
    }

    #[test]
    fn stale_to_missing_does_not_improve_a_disposition() {
        let mut stale = SCENARIOS[0];
        stale.evidence.route_audit = EvidenceState::Stale;
        let mut missing = stale;
        missing.evidence.route_audit = EvidenceState::Missing;
        assert_eq!(evaluate(&stale), Disposition::Unknown);
        assert_eq!(evaluate(&missing), Disposition::Unknown);
    }

    #[test]
    fn output_vocabulary_has_no_real_world_safety_certification() {
        for case in SCENARIOS {
            let output = evaluate(&case).as_str();
            assert!(
                [
                    "blocked",
                    "unknown",
                    "eligible_for_professional_review"
                ]
                .contains(&output)
            );
        }
    }
}
