//! Deterministic scoring for the Highveld panel-thread composition benchmark.
//!
//! This crate evaluates candidate *benchmark submissions*. It is not an owner
//! adapter, engineering approval service, manufacturing release system, or
//! robot-authority component. Provisional labels never produce qualification.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const PROFILE_ID: &str = "luminous.highveld.panel-thread-composition.v0.1";
pub const REPORT_SCHEMA: &str = "luminous.highveld.panel-thread-evaluation-report.v0.1";
pub const AUTHORITY_CEILING: &str = "NON_AUTHORIZING_SYNTHETIC_EVALUATION";

const VALID_DISPOSITIONS: &[&str] = &[
    "REVIEW_ONLY",
    "REQUALIFICATION_REQUIRED",
    "BLOCKED_UNKNOWN",
    "BLOCKED_CONFLICT",
    "BLOCKED_AUTHORITY",
];

const VALID_CATEGORIES: &[&str] = &[
    "nominal",
    "identity",
    "model_projection",
    "material_custody",
    "manufacturing_capability",
    "robot_capability",
    "metrology",
    "as_built",
    "change_management",
    "authority",
    "rights",
    "dependence",
    "site_context",
    "repairability",
    "safety",
];

const VALID_CRITICALITIES: &[&str] = &["critical", "major", "informational"];
const VALID_OWNERS: &[&str] = &[
    "sol_atlas",
    "symthaea",
    "symtropy",
    "mycelix",
    "external_standard",
];

const STALE_REASON_FRAGMENTS: &[&str] = &[
    "STALE",
    "REVOKED",
    "NOT_CURRENT",
    "SUPERSEDED",
    "CURRENT_CONTEXT_DIFFERS",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    pub profile_id: String,
    pub version: String,
    pub status: String,
    pub claim_ceiling: String,
    pub owner_refs: Vec<OwnerRef>,
    pub cases: Vec<GoldCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerRef {
    pub owner: String,
    pub repository: String,
    pub references: Vec<String>,
    pub responsibility: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldCase {
    pub case_id: String,
    pub title: String,
    pub category: String,
    pub criticality: String,
    pub preconditions: Vec<String>,
    pub mutation: String,
    pub expected_disposition: String,
    pub expected_reason_codes: Vec<String>,
    pub expected_requalification_domains: Vec<String>,
    pub authority_ceiling: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub profile_id: String,
    pub candidate_id: String,
    pub run_id: String,
    pub case_results: Vec<CandidateCaseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateCaseResult {
    pub case_id: String,
    pub disposition: String,
    pub reason_codes: Vec<String>,
    pub requalification_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvaluationReport {
    pub report_schema: String,
    pub profile_id: String,
    pub candidate_id: String,
    pub run_id: String,
    pub corpus_status: String,
    pub evaluation_status: String,
    pub corpus_sha256: String,
    pub submission_sha256: String,
    pub denominator: usize,
    pub exact_disposition_matches: usize,
    pub disposition_accuracy: f64,
    pub critical_blocked_cases: usize,
    pub critical_false_negatives: usize,
    pub critical_impact_recall: f64,
    pub stale_sensitive_cases: usize,
    pub stale_false_current: usize,
    pub reason_code_precision: f64,
    pub reason_code_recall: f64,
    pub requalification_domain_precision: f64,
    pub requalification_domain_recall: f64,
    pub disposition_confusion_matrix: BTreeMap<String, BTreeMap<String, usize>>,
    pub hard_gates: HardGates,
    pub authority_effects: &'static str,
    pub case_scores: Vec<CaseScore>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HardGates {
    pub zero_critical_false_negatives: bool,
    pub zero_stale_false_current: bool,
    pub complete_case_coverage: bool,
    pub no_authority_effects: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaseScore {
    pub case_id: String,
    pub expected_disposition: String,
    pub actual_disposition: String,
    pub exact_disposition_match: bool,
    pub missing_reason_codes: Vec<String>,
    pub unexpected_reason_codes: Vec<String>,
    pub missing_requalification_domains: Vec<String>,
    pub unexpected_requalification_domains: Vec<String>,
    pub critical_false_negative: bool,
    pub stale_false_current: bool,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

/// Parse, validate and score one complete submission against the supplied corpus.
///
/// All arrays with set semantics are compared as sets, but duplicate values are
/// rejected instead of silently normalized. Every case must be submitted exactly
/// once; omissions and unknown case IDs are hard input errors.
pub fn evaluate_bytes(
    corpus_bytes: &[u8],
    submission_bytes: &[u8],
) -> Result<EvaluationReport, String> {
    let corpus: Corpus = serde_json::from_slice(corpus_bytes)
        .map_err(|error| format!("invalid corpus JSON: {error}"))?;
    let submission: Submission = serde_json::from_slice(submission_bytes)
        .map_err(|error| format!("invalid submission JSON: {error}"))?;

    validate_corpus(&corpus)?;
    validate_submission(&submission, &corpus)?;

    let corpus_sha256 = sha256_hex(corpus_bytes);
    let submission_sha256 = sha256_hex(submission_bytes);
    score(&corpus, &submission, corpus_sha256, submission_sha256)
}

fn validate_corpus(corpus: &Corpus) -> Result<(), String> {
    if corpus.profile_id != PROFILE_ID {
        return Err(format!(
            "unsupported corpus profile_id: {}",
            corpus.profile_id
        ));
    }
    if corpus.version != "0.1.0" {
        return Err(format!("unsupported corpus version: {}", corpus.version));
    }
    if corpus.cases.len() < 20 {
        return Err(format!(
            "corpus must contain at least 20 cases, received {}",
            corpus.cases.len()
        ));
    }
    if corpus.claim_ceiling.chars().count() < 20 {
        return Err("corpus claim_ceiling must contain at least 20 characters".to_owned());
    }
    if corpus.status != "synthetic_specification_unadjudicated" {
        return Err(
            "this evaluator has no trusted adjudication-signature verifier; only              synthetic_specification_unadjudicated corpora are accepted"
                .to_owned(),
        );
    }
    if corpus.owner_refs.is_empty() {
        return Err("corpus must contain at least one owner reference".to_owned());
    }

    for owner_ref in &corpus.owner_refs {
        if !VALID_OWNERS.contains(&owner_ref.owner.as_str()) {
            return Err(format!("unsupported owner reference owner: {}", owner_ref.owner));
        }
        if owner_ref.repository.trim().is_empty() {
            return Err(format!(
                "{} owner reference has an empty repository",
                owner_ref.owner
            ));
        }
        if owner_ref.references.is_empty() {
            return Err(format!(
                "{} owner reference has no source references",
                owner_ref.owner
            ));
        }
        if owner_ref.references.iter().any(|reference| {
            !(reference.starts_with("https://github.com/")
                || reference.starts_with("ISO ")
                || reference.starts_with("NIST "))
        }) {
            return Err(format!(
                "{} owner reference contains an unpinned or unsupported reference",
                owner_ref.owner
            ));
        }
        if owner_ref.responsibility.chars().count() < 10 {
            return Err(format!(
                "{} owner responsibility must contain at least 10 characters",
                owner_ref.owner
            ));
        }
    }

    let mut ids = BTreeSet::new();
    for case in &corpus.cases {
        if !ids.insert(case.case_id.as_str()) {
            return Err(format!("duplicate corpus case_id: {}", case.case_id));
        }
        if !is_case_id(&case.case_id) {
            return Err(format!(
                "{} has invalid case_id; expected HPT- followed by exactly three digits",
                case.case_id
            ));
        }
        if case.title.chars().count() < 5 {
            return Err(format!("{} title is shorter than five characters", case.case_id));
        }
        if !VALID_CATEGORIES.contains(&case.category.as_str()) {
            return Err(format!(
                "{} has unsupported category {}",
                case.case_id, case.category
            ));
        }
        if !VALID_CRITICALITIES.contains(&case.criticality.as_str()) {
            return Err(format!(
                "{} has unsupported criticality {}",
                case.case_id, case.criticality
            ));
        }
        if case.preconditions.iter().any(|value| value.chars().count() < 5) {
            return Err(format!(
                "{} contains a precondition shorter than five characters",
                case.case_id
            ));
        }
        if case.mutation.chars().count() < 5 {
            return Err(format!(
                "{} mutation is shorter than five characters",
                case.case_id
            ));
        }
        if !VALID_DISPOSITIONS.contains(&case.expected_disposition.as_str()) {
            return Err(format!(
                "{} has unsupported expected disposition {}",
                case.case_id, case.expected_disposition
            ));
        }
        if case.authority_ceiling != AUTHORITY_CEILING {
            return Err(format!(
                "{} exceeds or changes the non-authorizing authority ceiling",
                case.case_id
            ));
        }
        let _ = unique_set(
            &case.expected_reason_codes,
            &case.case_id,
            "expected_reason_codes",
        )?;
        let _ = unique_set(
            &case.expected_requalification_domains,
            &case.case_id,
            "expected_requalification_domains",
        )?;
        if case.expected_reason_codes.is_empty() {
            return Err(format!("{} has no expected reason codes", case.case_id));
        }
        if case
            .expected_reason_codes
            .iter()
            .any(|code| !is_reason_code(code))
        {
            return Err(format!(
                "{} has an invalid expected reason code",
                case.case_id
            ));
        }
        if case
            .expected_requalification_domains
            .iter()
            .any(|domain| domain.chars().count() < 2)
        {
            return Err(format!(
                "{} has a requalification domain shorter than two characters",
                case.case_id
            ));
        }
        if case.criticality == "critical" && case.expected_disposition == "REVIEW_ONLY" {
            return Err(format!(
                "{} is critical but has a REVIEW_ONLY provisional label",
                case.case_id
            ));
        }
    }
    Ok(())
}

fn is_case_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes.starts_with(b"HPT-") && bytes[4..].iter().all(u8::is_ascii_digit)
}

fn is_reason_code(value: &str) -> bool {
    let bytes = value.as_bytes();
    (3..=64).contains(&bytes.len())
        && bytes[0].is_ascii_uppercase()
        && bytes
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn validate_submission(submission: &Submission, corpus: &Corpus) -> Result<(), String> {
    if submission.profile_id != PROFILE_ID {
        return Err(format!(
            "unsupported submission profile_id: {}",
            submission.profile_id
        ));
    }
    if submission.candidate_id.trim().is_empty() || submission.run_id.trim().is_empty() {
        return Err("candidate_id and run_id must not be empty".to_owned());
    }
    if submission.case_results.len() != corpus.cases.len() {
        return Err(format!(
            "case coverage mismatch: expected {}, received {}",
            corpus.cases.len(),
            submission.case_results.len()
        ));
    }

    let known: BTreeSet<&str> = corpus
        .cases
        .iter()
        .map(|case| case.case_id.as_str())
        .collect();
    let mut seen = BTreeSet::new();

    for result in &submission.case_results {
        if !seen.insert(result.case_id.as_str()) {
            return Err(format!("duplicate submission case_id: {}", result.case_id));
        }
        if !known.contains(result.case_id.as_str()) {
            return Err(format!("unknown submission case_id: {}", result.case_id));
        }
        if !VALID_DISPOSITIONS.contains(&result.disposition.as_str()) {
            return Err(format!(
                "{} has unsupported disposition {}",
                result.case_id, result.disposition
            ));
        }
        let _ = unique_set(&result.reason_codes, &result.case_id, "reason_codes")?;
        let _ = unique_set(
            &result.requalification_domains,
            &result.case_id,
            "requalification_domains",
        )?;
        if result.reason_codes.is_empty() {
            return Err(format!(
                "{} must return at least one reason code",
                result.case_id
            ));
        }
    }
    if seen.len() != known.len() {
        return Err("submission omitted one or more corpus cases".to_owned());
    }
    Ok(())
}

fn unique_set<'a>(
    values: &'a [String],
    case_id: &str,
    field: &str,
) -> Result<BTreeSet<&'a str>, String> {
    let mut set = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(format!("{case_id}.{field} contains an empty value"));
        }
        if !set.insert(value.as_str()) {
            return Err(format!(
                "{case_id}.{field} contains duplicate value {value}"
            ));
        }
    }
    Ok(set)
}

fn is_stale_sensitive(case: &GoldCase) -> bool {
    case.expected_reason_codes.iter().any(|reason| {
        STALE_REASON_FRAGMENTS
            .iter()
            .any(|fragment| reason.contains(fragment))
    })
}

fn metric(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        round6(numerator as f64 / denominator as f64)
    }
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn score(
    corpus: &Corpus,
    submission: &Submission,
    corpus_sha256: String,
    submission_sha256: String,
) -> Result<EvaluationReport, String> {
    let candidate_by_id: BTreeMap<&str, &CandidateCaseResult> = submission
        .case_results
        .iter()
        .map(|result| (result.case_id.as_str(), result))
        .collect();

    let mut case_scores = Vec::with_capacity(corpus.cases.len());
    let mut confusion: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    let mut exact_matches = 0usize;
    let mut critical_blocked_cases = 0usize;
    let mut critical_false_negatives = 0usize;
    let mut stale_sensitive_cases = 0usize;
    let mut stale_false_current = 0usize;

    let mut reason_tp = 0usize;
    let mut reason_fp = 0usize;
    let mut reason_fn = 0usize;
    let mut domain_tp = 0usize;
    let mut domain_fp = 0usize;
    let mut domain_fn = 0usize;

    for gold in &corpus.cases {
        let candidate = candidate_by_id
            .get(gold.case_id.as_str())
            .ok_or_else(|| format!("missing candidate result for {}", gold.case_id))?;

        let expected_reason = unique_set(
            &gold.expected_reason_codes,
            &gold.case_id,
            "expected_reason_codes",
        )?;
        let actual_reason = unique_set(&candidate.reason_codes, &gold.case_id, "reason_codes")?;
        let expected_domains = unique_set(
            &gold.expected_requalification_domains,
            &gold.case_id,
            "expected_requalification_domains",
        )?;
        let actual_domains = unique_set(
            &candidate.requalification_domains,
            &gold.case_id,
            "requalification_domains",
        )?;

        let missing_reason_codes = difference(&expected_reason, &actual_reason);
        let unexpected_reason_codes = difference(&actual_reason, &expected_reason);
        let missing_domains = difference(&expected_domains, &actual_domains);
        let unexpected_domains = difference(&actual_domains, &expected_domains);

        reason_tp += expected_reason.intersection(&actual_reason).count();
        reason_fp += unexpected_reason_codes.len();
        reason_fn += missing_reason_codes.len();
        domain_tp += expected_domains.intersection(&actual_domains).count();
        domain_fp += unexpected_domains.len();
        domain_fn += missing_domains.len();

        let exact_disposition_match = candidate.disposition == gold.expected_disposition;
        if exact_disposition_match {
            exact_matches += 1;
        }

        *confusion
            .entry(gold.expected_disposition.clone())
            .or_default()
            .entry(candidate.disposition.clone())
            .or_default() += 1;

        let critical_miss = gold.criticality == "critical"
            && safety_floor(&candidate.disposition) < safety_floor(&gold.expected_disposition);
        if gold.criticality == "critical" && gold.expected_disposition != "REVIEW_ONLY" {
            critical_blocked_cases += 1;
        }
        if critical_miss {
            critical_false_negatives += 1;
        }

        let stale_sensitive = is_stale_sensitive(gold);
        let stale_miss = stale_sensitive && candidate.disposition == "REVIEW_ONLY";
        if stale_sensitive {
            stale_sensitive_cases += 1;
        }
        if stale_miss {
            stale_false_current += 1;
        }

        case_scores.push(CaseScore {
            case_id: gold.case_id.clone(),
            expected_disposition: gold.expected_disposition.clone(),
            actual_disposition: candidate.disposition.clone(),
            exact_disposition_match,
            missing_reason_codes,
            unexpected_reason_codes,
            missing_requalification_domains: missing_domains,
            unexpected_requalification_domains: unexpected_domains,
            critical_false_negative: critical_miss,
            stale_false_current: stale_miss,
        });
    }

    let hard_gates = HardGates {
        zero_critical_false_negatives: critical_false_negatives == 0,
        zero_stale_false_current: stale_false_current == 0,
        complete_case_coverage: submission.case_results.len() == corpus.cases.len(),
        no_authority_effects: true,
    };
    let all_hard_gates = hard_gates.zero_critical_false_negatives
        && hard_gates.zero_stale_false_current
        && hard_gates.complete_case_coverage
        && hard_gates.no_authority_effects;

    // This evaluator deliberately has no trusted adjudication-signature
    // verifier. It therefore never emits a pass/fail qualification status.
    // A perfect score on visible provisional labels is only label agreement.
    let evaluation_status = if all_hard_gates {
        "PROVISIONAL_ONLY"
    } else {
        "PROVISIONAL_ONLY_WITH_HARD_GATE_FAILURES"
    };

    Ok(EvaluationReport {
        report_schema: REPORT_SCHEMA.to_owned(),
        profile_id: corpus.profile_id.clone(),
        candidate_id: submission.candidate_id.clone(),
        run_id: submission.run_id.clone(),
        corpus_status: corpus.status.clone(),
        evaluation_status: evaluation_status.to_owned(),
        corpus_sha256,
        submission_sha256,
        denominator: corpus.cases.len(),
        exact_disposition_matches: exact_matches,
        disposition_accuracy: metric(exact_matches, corpus.cases.len()),
        critical_blocked_cases,
        critical_false_negatives,
        critical_impact_recall: metric(
            critical_blocked_cases.saturating_sub(critical_false_negatives),
            critical_blocked_cases,
        ),
        stale_sensitive_cases,
        stale_false_current,
        reason_code_precision: metric(reason_tp, reason_tp + reason_fp),
        reason_code_recall: metric(reason_tp, reason_tp + reason_fn),
        requalification_domain_precision: metric(domain_tp, domain_tp + domain_fp),
        requalification_domain_recall: metric(domain_tp, domain_tp + domain_fn),
        disposition_confusion_matrix: confusion,
        hard_gates,
        authority_effects: "NONE",
        case_scores,
    })
}

fn safety_floor(disposition: &str) -> u8 {
    match disposition {
        "REVIEW_ONLY" => 0,
        "REQUALIFICATION_REQUIRED" => 1,
        "BLOCKED_UNKNOWN" | "BLOCKED_CONFLICT" => 2,
        "BLOCKED_AUTHORITY" => 3,
        _ => 0,
    }
}

fn difference(left: &BTreeSet<&str>, right: &BTreeSet<&str>) -> Vec<String> {
    left.difference(right)
        .map(|value| (*value).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS_JSON: &str = include_str!(
        "../../../docs/research/highveld-keep/HIGHVELD_KEEP_PANEL_THREAD_CORPUS_V0_1.json"
    );

    fn corpus() -> Corpus {
        serde_json::from_str(CORPUS_JSON).expect("checked-in corpus parses")
    }

    fn exact_submission(corpus: &Corpus) -> Submission {
        let results = corpus
            .cases
            .iter()
            .map(|case| CandidateCaseResult {
                case_id: case.case_id.clone(),
                disposition: case.expected_disposition.clone(),
                reason_codes: case.expected_reason_codes.clone(),
                requalification_domains: case.expected_requalification_domains.clone(),
            })
            .collect();
        Submission {
            profile_id: PROFILE_ID.to_owned(),
            candidate_id: "test-exact-match".to_owned(),
            run_id: "unit-test-001".to_owned(),
            case_results: results,
        }
    }

    fn encode<T: Serialize>(value: &T) -> Vec<u8> {
        serde_json::to_vec(value).expect("test structure serializes")
    }

    #[test]
    fn perfect_provisional_submission_never_becomes_qualified() {
        let corpus = corpus();
        let submission = exact_submission(&corpus);
        let report = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap();

        assert_eq!(report.denominator, 24);
        assert_eq!(report.exact_disposition_matches, 24);
        assert_eq!(report.critical_false_negatives, 0);
        assert_eq!(report.stale_false_current, 0);
        assert_eq!(report.evaluation_status, "PROVISIONAL_ONLY");
        assert_eq!(report.authority_effects, "NONE");
    }

    #[test]
    fn critical_abstention_is_counted_as_a_false_negative() {
        let corpus = corpus();
        let mut submission = exact_submission(&corpus);
        let target = submission
            .case_results
            .iter_mut()
            .find(|result| result.case_id == "HPT-003")
            .unwrap();
        target.disposition = "REVIEW_ONLY".to_owned();

        let report = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap();
        assert_eq!(report.critical_false_negatives, 1);
        assert!(!report.hard_gates.zero_critical_false_negatives);
        assert_eq!(
            report.evaluation_status,
            "PROVISIONAL_ONLY_WITH_HARD_GATE_FAILURES"
        );
    }

    #[test]
    fn revoked_certificate_cannot_be_scored_as_current() {
        let corpus = corpus();
        let mut submission = exact_submission(&corpus);
        let target = submission
            .case_results
            .iter_mut()
            .find(|result| result.case_id == "HPT-015")
            .unwrap();
        target.disposition = "REVIEW_ONLY".to_owned();

        let report = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap();
        assert_eq!(report.stale_false_current, 1);
        assert!(!report.hard_gates.zero_stale_false_current);
    }

    #[test]
    fn omitted_case_is_rejected_instead_of_shrinking_denominator() {
        let corpus = corpus();
        let mut submission = exact_submission(&corpus);
        submission.case_results.pop();

        let error = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap_err();
        assert!(error.contains("case coverage mismatch"));
    }

    #[test]
    fn duplicate_reason_code_is_rejected() {
        let corpus = corpus();
        let mut submission = exact_submission(&corpus);
        let duplicate_code = submission.case_results[0].reason_codes[0].clone();
        submission.case_results[0].reason_codes.push(duplicate_code);

        let error = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap_err();
        assert!(error.contains("duplicate value"));
    }

    #[test]
    fn unknown_fields_are_rejected_by_serde() {
        let raw = br#"{
            "profile_id":"luminous.highveld.panel-thread-composition.v0.1",
            "candidate_id":"candidate",
            "run_id":"run",
            "case_results":[],
            "authority":true
        }"#;
        assert!(serde_json::from_slice::<Submission>(raw).is_err());
    }

    #[test]
    fn corpus_cannot_self_assert_independent_adjudication() {
        let mut raw = serde_json::from_str::<serde_json::Value>(CORPUS_JSON).unwrap();
        raw["status"] = serde_json::Value::String("independently_adjudicated".to_owned());
        let tampered = serde_json::to_vec(&raw).unwrap();
        let submission = exact_submission(&corpus());

        let error = evaluate_bytes(&tampered, &encode(&submission)).unwrap_err();
        assert!(error.contains("no trusted adjudication-signature verifier"));
    }

    #[test]
    fn critical_underblocking_is_detected_beyond_review_only() {
        let corpus = corpus();
        let mut submission = exact_submission(&corpus);
        let target = submission
            .case_results
            .iter_mut()
            .find(|result| result.case_id == "HPT-008")
            .unwrap();
        target.disposition = "REQUALIFICATION_REQUIRED".to_owned();

        let report = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap();
        assert_eq!(report.critical_false_negatives, 1);
        assert!(!report.hard_gates.zero_critical_false_negatives);
    }

    #[test]
    fn authority_block_cannot_be_downgraded_to_conflict_block() {
        let corpus = corpus();
        let mut submission = exact_submission(&corpus);
        let target = submission
            .case_results
            .iter_mut()
            .find(|result| result.case_id == "HPT-016")
            .unwrap();
        target.disposition = "BLOCKED_CONFLICT".to_owned();

        let report = evaluate_bytes(CORPUS_JSON.as_bytes(), &encode(&submission)).unwrap();
        assert_eq!(report.critical_false_negatives, 1);
        assert!(!report.hard_gates.zero_critical_false_negatives);
    }

    #[test]
    fn candidate_case_order_does_not_change_scores() {
        let corpus = corpus();
        let submission = exact_submission(&corpus);
        let original_bytes = encode(&submission);
        let original = evaluate_bytes(CORPUS_JSON.as_bytes(), &original_bytes).unwrap();

        let mut reordered = submission.clone();
        reordered.case_results.reverse();
        let reordered_bytes = encode(&reordered);
        let reversed = evaluate_bytes(CORPUS_JSON.as_bytes(), &reordered_bytes).unwrap();

        assert_eq!(
            original.exact_disposition_matches,
            reversed.exact_disposition_matches
        );
        assert_eq!(original.disposition_accuracy, reversed.disposition_accuracy);
        assert_eq!(
            original.critical_impact_recall,
            reversed.critical_impact_recall
        );
        assert_eq!(
            original.disposition_confusion_matrix,
            reversed.disposition_confusion_matrix
        );
        assert_eq!(
            original
                .case_scores
                .iter()
                .map(|score| score.case_id.as_str())
                .collect::<Vec<_>>(),
            reversed
                .case_scores
                .iter()
                .map(|score| score.case_id.as_str())
                .collect::<Vec<_>>()
        );
        assert_ne!(original.submission_sha256, reversed.submission_sha256);
    }

    #[test]
    fn corpus_rejects_invalid_case_id_pattern() {
        let mut raw = serde_json::from_str::<serde_json::Value>(CORPUS_JSON).unwrap();
        raw["cases"][0]["case_id"] = serde_json::Value::String("HPT-X01".to_owned());
        let submission = exact_submission(&corpus());

        let error = evaluate_bytes(&encode(&raw), &encode(&submission)).unwrap_err();
        assert!(error.contains("expected HPT- followed by exactly three digits"));
    }

    #[test]
    fn corpus_rejects_unpinned_owner_reference() {
        let mut raw = serde_json::from_str::<serde_json::Value>(CORPUS_JSON).unwrap();
        raw["owner_refs"][0]["references"][0] =
            serde_json::Value::String("ftp://example.invalid/evidence".to_owned());
        let submission = exact_submission(&corpus());

        let error = evaluate_bytes(&encode(&raw), &encode(&submission)).unwrap_err();
        assert!(error.contains("unpinned or unsupported reference"));
    }

    #[test]
    fn corpus_rejects_malformed_reason_code() {
        let mut raw = serde_json::from_str::<serde_json::Value>(CORPUS_JSON).unwrap();
        raw["cases"][0]["expected_reason_codes"][0] =
            serde_json::Value::String("not a code".to_owned());
        let submission = exact_submission(&corpus());

        let error = evaluate_bytes(&encode(&raw), &encode(&submission)).unwrap_err();
        assert!(error.contains("invalid expected reason code"));
    }

    #[test]
    fn input_hashes_are_sha256_hex() {
        let digest = sha256_hex(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
