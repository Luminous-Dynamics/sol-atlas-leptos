use jsonschema::validator_for;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::process::ExitCode;

const EXPECTED_SCHEMA_VERSION: &str = "0.1.0";
const EXPECTED_BENCHMARK_ID: &str = "SA-WORKFORCE-CHAIN-CONFORMANCE-001";
const REQUIRED_STAGES: [&str; 6] = [
    "source_profile",
    "active_evidence",
    "diagnosis",
    "intervention",
    "outcome",
    "robustness",
];
const FIXTURE_DEFAULT: &str =
    "docs/research/fixtures/workforce_chain_conformance_v0_1.json";
const SCHEMA_DEFAULT: &str =
    "docs/research/schemas/workforce_chain_manifest_v0_1.schema.json";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Corpus {
    fixture_notice: String,
    schema_version: String,
    benchmark_id: String,
    synthetic_only: bool,
    claim_ceiling: String,
    cases: Vec<Case>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Case {
    id: String,
    description: String,
    stages: Vec<Stage>,
    links: Vec<Link>,
    requested_claim_type: String,
    requested_claim_strength: String,
    used_query_ids: Vec<String>,
    permitted_query_ids: Vec<String>,
    authorization_status: String,
    service_verification_status: String,
    payment_evidence_status: String,
    causal_design_status: String,
    counterfactual_basis: String,
    preregistration_status: String,
    expected_verdict: String,
    expected_failure_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Stage {
    stage_id: String,
    stage_type: String,
    artifact_path: String,
    pinned_artifact_sha256: String,
    observed_artifact_sha256: String,
    schema_path: String,
    pinned_schema_sha256: String,
    observed_schema_sha256: String,
    scope: Scope,
    evidence_class: String,
    claim_ceiling: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Scope {
    case_scope_id: String,
    task_scope_id: String,
    geography: String,
    period_start: String,
    period_end: String,
    decision_cutoff: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Link {
    from_stage_id: String,
    to_stage_id: String,
    required: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct CaseReceipt {
    case_id: String,
    expected_verdict: String,
    observed_verdict: String,
    expected_failure_codes: Vec<String>,
    failure_codes: Vec<String>,
    effective_claim_ceiling: String,
    passed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct Receipt {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    fixture_sha256: String,
    manifest_schema_sha256: String,
    case_count: usize,
    cases_passed: usize,
    cases_failed: usize,
    cases: Vec<CaseReceipt>,
    limitations: Vec<&'static str>,
    claim_ceiling: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn strength_rank(value: &str) -> Option<u8> {
    match value {
        "synthetic_contract_only" => Some(0),
        "descriptive_association_only" => Some(1),
        "verified_operational_outcome" => Some(2),
        "causal_impact" => Some(3),
        _ => None,
    }
}

fn strength_name(rank: u8) -> &'static str {
    match rank {
        0 => "synthetic_contract_only",
        1 => "descriptive_association_only",
        2 => "verified_operational_outcome",
        3 => "causal_impact",
        _ => "synthetic_contract_only",
    }
}

fn evidence_cap(value: &str) -> u8 {
    match value {
        "synthetic" => 0,
        "authorized_observation" => 1,
        "independently_verified_observation" => 3,
        _ => 0,
    }
}

fn stage_order(stage_type: &str) -> usize {
    REQUIRED_STAGES
        .iter()
        .position(|expected| *expected == stage_type)
        .unwrap_or(usize::MAX)
}

fn canonicalize(corpus: &mut Corpus) {
    corpus.cases.sort_by(|a, b| a.id.cmp(&b.id));
    for case in &mut corpus.cases {
        case.stages.sort_by(|a, b| {
            (stage_order(&a.stage_type), a.stage_id.as_str())
                .cmp(&(stage_order(&b.stage_type), b.stage_id.as_str()))
        });
        case.links.sort_by(|a, b| {
            (&a.from_stage_id, &a.to_stage_id).cmp(&(&b.from_stage_id, &b.to_stage_id))
        });
        case.used_query_ids.sort();
        case.permitted_query_ids.sort();
        case.expected_failure_codes.sort();
    }
}

fn validate_schema(schema: &Value, manifest: &Value) -> Result<(), String> {
    let validator =
        validator_for(schema).map_err(|error| format!("schema cannot be compiled: {error}"))?;
    let errors: Vec<String> = validator
        .iter_errors(manifest)
        .map(|error| error.to_string())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "JSON Schema validation failed:\n{}",
            errors.join("\n")
        ))
    }
}

fn validate_contract(corpus: &Corpus) -> Result<(), String> {
    let mut errors = Vec::new();
    if corpus.schema_version != EXPECTED_SCHEMA_VERSION {
        errors.push(format!(
            "schema_version must be {EXPECTED_SCHEMA_VERSION}, got {}",
            corpus.schema_version
        ));
    }
    if corpus.benchmark_id != EXPECTED_BENCHMARK_ID {
        errors.push(format!(
            "benchmark_id must be {EXPECTED_BENCHMARK_ID}, got {}",
            corpus.benchmark_id
        ));
    }
    if !corpus.synthetic_only {
        errors.push("this verifier accepts only synthetic chain fixtures".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES AND DIGESTS ARE SYNTHETIC")
    {
        errors.push("fixture notice must mark values and digests synthetic".to_string());
    }
    if corpus.claim_ceiling != "synthetic_contract_only" {
        errors.push("synthetic manifest claim ceiling must be synthetic_contract_only".to_string());
    }
    if corpus.cases.is_empty() {
        errors.push("case list cannot be empty".to_string());
    }

    let mut case_ids = BTreeSet::new();
    for case in &corpus.cases {
        if !case_ids.insert(case.id.clone()) {
            errors.push(format!("duplicate case ID {}", case.id));
        }
        let mut stage_ids = BTreeSet::new();
        for stage in &case.stages {
            if !stage_ids.insert(stage.stage_id.clone()) {
                errors.push(format!("{}: duplicate stage ID {}", case.id, stage.stage_id));
            }
            if stage.artifact_path.trim().is_empty() || stage.schema_path.trim().is_empty() {
                errors.push(format!("{} / {}: artifact and schema paths are required", case.id, stage.stage_id));
            }
            if stage.scope.case_scope_id.trim().is_empty()
                || stage.scope.task_scope_id.trim().is_empty()
                || stage.scope.geography.trim().is_empty()
            {
                errors.push(format!("{} / {}: scope identifiers cannot be empty", case.id, stage.stage_id));
            }
            if stage.scope.period_start > stage.scope.period_end {
                errors.push(format!("{} / {}: scope period is reversed", case.id, stage.stage_id));
            }
        }
        let mut link_pairs = BTreeSet::new();
        for link in &case.links {
            if !link.required {
                errors.push(format!("{}: this v0.1 chain only accepts required links", case.id));
            }
            if !link_pairs.insert((link.from_stage_id.clone(), link.to_stage_id.clone())) {
                errors.push(format!("{}: duplicate link {} -> {}", case.id, link.from_stage_id, link.to_stage_id));
            }
        }
        let used: BTreeSet<&str> = case.used_query_ids.iter().map(String::as_str).collect();
        let permitted: BTreeSet<&str> = case.permitted_query_ids.iter().map(String::as_str).collect();
        if used.len() != case.used_query_ids.len() {
            errors.push(format!("{}: duplicate used query ID", case.id));
        }
        if permitted.len() != case.permitted_query_ids.len() {
            errors.push(format!("{}: duplicate permitted query ID", case.id));
        }
        if case.expected_failure_codes.iter().collect::<BTreeSet<_>>().len()
            != case.expected_failure_codes.len()
        {
            errors.push(format!("{}: duplicate expected failure code", case.id));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn check_case(corpus_synthetic_only: bool, case: &Case) -> (Vec<String>, String) {
    let mut failures = BTreeSet::new();
    let mut stage_ids = BTreeSet::new();
    let mut stage_types: BTreeMap<&str, Vec<&Stage>> = BTreeMap::new();

    for stage in &case.stages {
        if !stage_ids.insert(stage.stage_id.as_str()) {
            failures.insert("duplicate_stage_id".to_string());
        }
        stage_types
            .entry(stage.stage_type.as_str())
            .or_default()
            .push(stage);

        if stage.pinned_artifact_sha256 != stage.observed_artifact_sha256 {
            failures.insert("stale_artifact_digest".to_string());
        }
        if stage.pinned_schema_sha256 != stage.observed_schema_sha256 {
            failures.insert("stale_schema_digest".to_string());
        }
        if corpus_synthetic_only && stage.evidence_class != "synthetic" {
            failures.insert("synthetic_fixture_stage_relabelled".to_string());
        }
    }

    for (stage_type, stages) in &stage_types {
        if stages.len() > 1 {
            failures.insert(format!("duplicate_stage_type_{stage_type}"));
        }
    }
    for required in REQUIRED_STAGES {
        if !stage_types.contains_key(required) {
            failures.insert("missing_required_stage".to_string());
        }
    }

    let mut ordered: Vec<&Stage> = case.stages.iter().collect();
    ordered.sort_by(|a, b| {
        (stage_order(&a.stage_type), a.stage_id.as_str())
            .cmp(&(stage_order(&b.stage_type), b.stage_id.as_str()))
    });

    let links: BTreeSet<(&str, &str)> = case
        .links
        .iter()
        .map(|link| (link.from_stage_id.as_str(), link.to_stage_id.as_str()))
        .collect();

    for link in &case.links {
        if !stage_ids.contains(link.from_stage_id.as_str())
            || !stage_ids.contains(link.to_stage_id.as_str())
        {
            failures.insert("broken_chain_link".to_string());
        }
        let from = case.stages.iter().find(|stage| stage.stage_id == link.from_stage_id);
        let to = case.stages.iter().find(|stage| stage.stage_id == link.to_stage_id);
        if let (Some(from), Some(to)) = (from, to) {
            if stage_order(&to.stage_type) != stage_order(&from.stage_type).saturating_add(1) {
                failures.insert("broken_chain_link".to_string());
            }
        }
    }
    for pair in ordered.windows(2) {
        if stage_order(&pair[1].stage_type) == stage_order(&pair[0].stage_type).saturating_add(1)
            && !links.contains(&(pair[0].stage_id.as_str(), pair[1].stage_id.as_str()))
        {
            failures.insert("broken_chain_link".to_string());
        }
    }

    if let Some(first) = ordered.first() {
        for stage in ordered.iter().skip(1) {
            if stage.scope.case_scope_id != first.scope.case_scope_id
                || stage.scope.task_scope_id != first.scope.task_scope_id
                || stage.scope.geography != first.scope.geography
                || stage.scope.period_start != first.scope.period_start
                || stage.scope.period_end != first.scope.period_end
                || stage.scope.decision_cutoff != first.scope.decision_cutoff
            {
                failures.insert("scope_mismatch".to_string());
            }
        }
    }

    let permitted: BTreeSet<&str> = case.permitted_query_ids.iter().map(String::as_str).collect();
    if case
        .used_query_ids
        .iter()
        .any(|query_id| !permitted.contains(query_id.as_str()))
    {
        failures.insert("unpermitted_evidence_query".to_string());
    }
    if !case.used_query_ids.is_empty() && case.authorization_status != "permitted" {
        failures.insert("authorization_not_positive".to_string());
    }

    if case.requested_claim_type == "service_restoration"
        && case.service_verification_status != "independently_verified"
    {
        failures.insert("service_outcome_unverified".to_string());
    }
    if case.requested_claim_type == "paid_work_outcome"
        && case.payment_evidence_status != "verified_payment_evidence"
    {
        failures.insert("payment_evidence_missing".to_string());
    }
    if case.requested_claim_type == "causal_impact"
        || case.requested_claim_strength == "causal_impact"
    {
        if case.causal_design_status != "registered_credible"
            || case.preregistration_status != "verified"
            || case.counterfactual_basis.trim().is_empty()
        {
            failures.insert("causal_design_missing".to_string());
        }
    }

    let chain_ceiling = ordered
        .iter()
        .map(|stage| {
            let declared = strength_rank(&stage.claim_ceiling).unwrap_or(0);
            let evidence_cap = evidence_cap(&stage.evidence_class);
            declared.min(evidence_cap)
        })
        .min()
        .unwrap_or(0);

    if strength_rank(&case.requested_claim_strength).unwrap_or(u8::MAX) > chain_ceiling {
        failures.insert("claim_exceeds_evidence_ceiling".to_string());
    }

    let codes: Vec<String> = failures.into_iter().collect();
    (codes, strength_name(chain_ceiling).to_string())
}

fn verdict_from_failures(failures: &[String]) -> &'static str {
    if failures.is_empty() {
        "conformant"
    } else {
        "rejected"
    }
}

fn verify_values(
    schema_value: &Value,
    manifest_value: &Value,
    fixture_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<Receipt, String> {
    validate_schema(schema_value, manifest_value)?;
    let mut corpus: Corpus = serde_json::from_value(manifest_value.clone())
        .map_err(|error| format!("manifest decoding failed: {error}"))?;
    validate_contract(&corpus)?;
    canonicalize(&mut corpus);

    let mut receipts = Vec::new();
    for case in &corpus.cases {
        let (failure_codes, effective_claim_ceiling) = check_case(corpus.synthetic_only, case);
        let observed_verdict = verdict_from_failures(&failure_codes).to_string();
        let mut expected_codes = case.expected_failure_codes.clone();
        expected_codes.sort();
        let passed = observed_verdict == case.expected_verdict && failure_codes == expected_codes;
        receipts.push(CaseReceipt {
            case_id: case.id.clone(),
            expected_verdict: case.expected_verdict.clone(),
            observed_verdict,
            expected_failure_codes: expected_codes,
            failure_codes,
            effective_claim_ceiling,
            passed,
        });
    }

    let cases_passed = receipts.iter().filter(|case| case.passed).count();
    let cases_failed = receipts.len().saturating_sub(cases_passed);
    Ok(Receipt {
        status: if cases_failed == 0 {
            "SYNTHETIC_CHAIN_CONFORMANCE_PASS"
        } else {
            "SYNTHETIC_CHAIN_CONFORMANCE_FAIL"
        },
        benchmark_id: corpus.benchmark_id,
        schema_version: corpus.schema_version,
        fixture_sha256: sha256(fixture_bytes),
        manifest_schema_sha256: sha256(schema_bytes),
        case_count: receipts.len(),
        cases_passed,
        cases_failed,
        cases: receipts,
        limitations: vec![
            "All chain stages and digests in this fixture are synthetic contract inputs.",
            "Per-stage observed digest fields simulate adapter reports; this v0.1 slice checks pin/report equality, not actual upstream artifact bytes.",
            "A conformant synthetic chain does not validate real sources, real authorization, real outcomes or causal impact.",
            "The verifier performs no network requests and cannot grant consent or authority.",
        ],
        claim_ceiling: corpus.claim_ceiling,
    })
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let fixture_path = args.next().unwrap_or_else(|| FIXTURE_DEFAULT.to_string());
    let schema_path = args.next().unwrap_or_else(|| SCHEMA_DEFAULT.to_string());
    let fixture_bytes =
        fs::read(&fixture_path).map_err(|error| format!("cannot read {fixture_path}: {error}"))?;
    let schema_bytes =
        fs::read(&schema_path).map_err(|error| format!("cannot read {schema_path}: {error}"))?;
    let manifest_value: Value = serde_json::from_slice(&fixture_bytes)
        .map_err(|error| format!("manifest JSON is invalid: {error}"))?;
    let schema_value: Value = serde_json::from_slice(&schema_bytes)
        .map_err(|error| format!("schema JSON is invalid: {error}"))?;
    let receipt = verify_values(
        &schema_value,
        &manifest_value,
        &fixture_bytes,
        &schema_bytes,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt)
            .map_err(|error| format!("receipt serialization failed: {error}"))?
    );
    if receipt.cases_failed > 0 {
        return Err(format!(
            "{} chain conformance cases did not match their expected verdicts",
            receipt.cases_failed
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../../docs/research/fixtures/workforce_chain_conformance_v0_1.json"
    );
    const SCHEMA: &str = include_str!(
        "../../../docs/research/schemas/workforce_chain_manifest_v0_1.schema.json"
    );

    fn loaded() -> (Value, Value) {
        (
            serde_json::from_str(FIXTURE).expect("fixture JSON parses"),
            serde_json::from_str(SCHEMA).expect("schema JSON parses"),
        )
    }

    fn report(fixture: &Value, schema: &Value) -> Receipt {
        verify_values(
            schema,
            fixture,
            FIXTURE.as_bytes(),
            SCHEMA.as_bytes(),
        )
        .expect("synthetic chain manifest validates")
    }

    fn find_case<'a>(receipt: &'a Receipt, id: &str) -> &'a CaseReceipt {
        receipt
            .cases
            .iter()
            .find(|case| case.case_id == id)
            .expect("case ID exists")
    }

    #[test]
    fn all_frozen_positive_and_negative_controls_match_expected_codes() {
        let (fixture, schema) = loaded();
        let receipt = report(&fixture, &schema);
        assert_eq!(receipt.status, "SYNTHETIC_CHAIN_CONFORMANCE_PASS");
        assert_eq!(receipt.case_count, 12);
        assert_eq!(receipt.cases_passed, 12);
        assert_eq!(receipt.cases_failed, 0);
    }

    #[test]
    fn stale_artifact_and_schema_digests_are_rejected() {
        let (fixture, schema) = loaded();
        assert_eq!(find_case(&report(&fixture, &schema), "WC-002").failure_codes, vec![
            "stale_artifact_digest"
        ]);
        assert_eq!(find_case(&report(&fixture, &schema), "WC-012").failure_codes, vec![
            "stale_schema_digest"
        ]);
    }

    #[test]
    fn scope_mismatch_breaks_the_chain() {
        let (fixture, schema) = loaded();
        for id in ["WC-003", "WC-010"] {
            assert_eq!(
                find_case(&report(&fixture, &schema), id).failure_codes,
                vec!["scope_mismatch"]
            );
        }
    }

    #[test]
    fn prohibited_queries_and_nonpositive_authorization_are_rejected() {
        let (fixture, schema) = loaded();
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-004").failure_codes,
            vec!["unpermitted_evidence_query"]
        );
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-009").failure_codes,
            vec!["authorization_not_positive"]
        );
    }

    #[test]
    fn synthetic_stage_relabeling_and_causal_promotion_are_rejected() {
        let (fixture, schema) = loaded();
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-005").failure_codes,
            vec![
                "causal_design_missing",
                "claim_exceeds_evidence_ceiling"
            ]
        );
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-011").failure_codes,
            vec![
                "claim_exceeds_evidence_ceiling",
                "synthetic_fixture_stage_relabelled"
            ]
        );
    }

    #[test]
    fn service_and_paid_work_claims_need_their_own_evidence_gates() {
        let (fixture, schema) = loaded();
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-006").failure_codes,
            vec!["payment_evidence_missing"]
        );
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-007").failure_codes,
            vec!["service_outcome_unverified"]
        );
    }

    #[test]
    fn missing_stage_and_broken_links_are_not_auto_repaired() {
        let (fixture, schema) = loaded();
        assert_eq!(
            find_case(&report(&fixture, &schema), "WC-008").failure_codes,
            vec!["broken_chain_link", "missing_required_stage"]
        );
    }

    #[test]
    fn input_order_permutation_preserves_semantic_receipt() {
        let (fixture, schema) = loaded();
        let original = report(&fixture, &schema);
        let mut permuted = fixture.clone();
        permuted["cases"]
            .as_array_mut()
            .expect("cases array")
            .reverse();
        for case in permuted["cases"].as_array_mut().expect("cases array") {
            case["stages"]
                .as_array_mut()
                .expect("stages array")
                .reverse();
            case["links"]
                .as_array_mut()
                .expect("links array")
                .reverse();
            case["used_query_ids"]
                .as_array_mut()
                .expect("used-query array")
                .reverse();
        }
        let reordered = report(&permuted, &schema);
        assert_eq!(original.cases, reordered.cases);
        assert_eq!(original.cases_passed, reordered.cases_passed);
        assert_eq!(original.cases_failed, reordered.cases_failed);
    }

    #[test]
    fn manifest_cannot_claim_to_be_observed_data() {
        let (mut fixture, schema) = loaded();
        fixture["synthetic_only"] = Value::Bool(false);
        assert!(verify_values(
            &schema,
            &fixture,
            FIXTURE.as_bytes(),
            SCHEMA.as_bytes()
        )
        .is_err());
    }
}
