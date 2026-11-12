use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const SCHEMA_VERSION: &str = "0.1.0";
const BENCHMARK_ID: &str = "GMD-PROTECTION-ENVELOPE-001";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    schema_version: String,
    benchmark_id: String,
    fixture_notice: String,
    synthetic_only: bool,
    claim_ceiling: String,
    parameter_domain: Vec<Axis>,
    required_services: Vec<RequiredService>,
    cells: Vec<Cell>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Axis {
    axis_id: String,
    unit: String,
    lower: f64,
    upper: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AxisBound {
    axis_id: String,
    unit: String,
    lower: f64,
    upper: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredService {
    service_id: String,
    unit: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    cell_id: String,
    bounds: Vec<AxisBound>,
    coverage: CoverageEvidence,
    required_capabilities: EvidenceCapabilities,
    available_capabilities: EvidenceCapabilities,
    outcomes: Option<OutcomeEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageEvidence {
    coverage_kind: CoverageKind,
    covers_entire_cell: bool,
    uncertainty_set_covered: bool,
    proof_method: Option<String>,
    model_revision: Option<String>,
    source_artifact: Option<ArtifactRef>,
    model_artifact: Option<ArtifactRef>,
    proof_artifact: Option<ArtifactRef>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CoverageKind {
    PointSamplesOnly,
    IntervalEnclosure,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactRef {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceCapabilities {
    waveform_resolved: bool,
    independent_validation: bool,
    physical_relay_observed: bool,
    physical_power_interface_observed: bool,
    field_exercise_observed: bool,
}

impl EvidenceCapabilities {
    fn missing_from(&self, available: &Self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.waveform_resolved && !available.waveform_resolved {
            missing.push("waveform_resolved");
        }
        if self.independent_validation && !available.independent_validation {
            missing.push("independent_validation");
        }
        if self.physical_relay_observed && !available.physical_relay_observed {
            missing.push("physical_relay_observed");
        }
        if self.physical_power_interface_observed && !available.physical_power_interface_observed {
            missing.push("physical_power_interface_observed");
        }
        if self.field_exercise_observed && !available.field_exercise_observed {
            missing.push("field_exercise_observed");
        }
        missing
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomeEvidence {
    scope: OutcomeScope,
    false_trip_count_upper_bound: Option<u64>,
    false_trip_denominator: Option<u64>,
    failure_to_trip_count_upper_bound: Option<u64>,
    failure_to_trip_denominator: Option<u64>,
    minimum_coordination_margin_pu: Option<f64>,
    required_coordination_margin_pu: Option<f64>,
    service_floor_margins: Vec<ServiceFloorMargin>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OutcomeScope {
    PointObserved,
    WholeCellBound,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceFloorMargin {
    service_id: String,
    unit: String,
    minimum_margin: f64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Classification {
    Certified,
    Excluded,
    Unknown,
}

#[derive(Debug, Serialize)]
struct RateBound {
    numerator: u64,
    denominator: u64,
}

#[derive(Debug, Serialize)]
struct CellResult {
    cell_id: String,
    classification: Classification,
    reasons: Vec<String>,
    false_trip_rate_upper_bound: Option<RateBound>,
    failure_to_trip_rate_upper_bound: Option<RateBound>,
    coordination_margin_surplus_pu: Option<f64>,
    service_floor_margins: Vec<ServiceFloorMarginReceipt>,
}

#[derive(Debug, Serialize)]
struct ServiceFloorMarginReceipt {
    service_id: String,
    unit: String,
    minimum_margin: f64,
}

#[derive(Debug, Serialize)]
struct Report {
    status: &'static str,
    benchmark_id: String,
    schema_version: String,
    input_sha256: String,
    synthetic_only: bool,
    claim_ceiling: String,
    certified_cell_count: usize,
    excluded_cell_count: usize,
    unknown_cell_count: usize,
    cells: Vec<CellResult>,
    coverage_note: &'static str,
    limitations: Vec<&'static str>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_count_pair(
    label: &str,
    numerator: Option<u64>,
    denominator: Option<u64>,
) -> Result<(), String> {
    match (numerator, denominator) {
        (None, None) => Ok(()),
        (Some(n), Some(d)) if d > 0 && n <= d => Ok(()),
        (Some(_), Some(0)) => Err(format!("{label}: denominator must be greater than zero")),
        (Some(n), Some(d)) if n > d => {
            Err(format!("{label}: numerator {n} exceeds denominator {d}"))
        }
        _ => Err(format!(
            "{label}: numerator and denominator must be supplied together"
        )),
    }
}

fn validate_corpus(corpus: &Corpus) -> Result<(), String> {
    if corpus.schema_version != SCHEMA_VERSION {
        return Err(format!("schema_version must be {SCHEMA_VERSION}"));
    }
    if corpus.benchmark_id != BENCHMARK_ID {
        return Err(format!("benchmark_id must be {BENCHMARK_ID}"));
    }
    if !corpus.synthetic_only {
        return Err("v0.1 verifier accepts synthetic contract fixtures only".to_string());
    }
    if !corpus
        .fixture_notice
        .to_ascii_uppercase()
        .contains("ALL VALUES ARE SYNTHETIC")
    {
        return Err("fixture_notice must explicitly say ALL VALUES ARE SYNTHETIC".to_string());
    }
    if corpus.claim_ceiling != "E2" {
        return Err("synthetic verifier claim_ceiling must be E2".to_string());
    }
    if corpus.parameter_domain.is_empty() || corpus.cells.is_empty() {
        return Err("parameter_domain and cells must be non-empty".to_string());
    }
    if corpus.required_services.is_empty() {
        return Err("required_services must be non-empty".to_string());
    }

    let mut axis_ids = BTreeSet::new();
    for axis in &corpus.parameter_domain {
        if axis.axis_id.trim().is_empty() || axis.unit.trim().is_empty() {
            return Err("axis_id and unit must be non-empty".to_string());
        }
        if !axis_ids.insert(axis.axis_id.as_str()) {
            return Err(format!("duplicate domain axis {}", axis.axis_id));
        }
        if !axis.lower.is_finite() || !axis.upper.is_finite() || axis.lower >= axis.upper {
            return Err(format!("{}: invalid finite domain interval", axis.axis_id));
        }
    }

    let mut service_ids = BTreeSet::new();
    for service in &corpus.required_services {
        if service.service_id.trim().is_empty()
            || service.unit.trim().is_empty()
            || !service_ids.insert(service.service_id.as_str())
        {
            return Err(format!(
                "empty or duplicate required service ID/unit: {}",
                service.service_id
            ));
        }
    }

    let mut cell_ids = BTreeSet::new();
    for cell in &corpus.cells {
        if cell.cell_id.trim().is_empty() || !cell_ids.insert(cell.cell_id.as_str()) {
            return Err(format!("empty or duplicate cell ID: {}", cell.cell_id));
        }
        if cell.bounds.len() != corpus.parameter_domain.len() {
            return Err(format!("{}: cell/domain dimension mismatch", cell.cell_id));
        }
        let mut bound_ids = BTreeSet::new();
        for bound in &cell.bounds {
            if !bound_ids.insert(bound.axis_id.as_str()) {
                return Err(format!(
                    "{}: duplicate cell bound for {}",
                    cell.cell_id, bound.axis_id
                ));
            }
            let axis = corpus
                .parameter_domain
                .iter()
                .find(|axis| axis.axis_id == bound.axis_id)
                .ok_or_else(|| format!("{}: unknown axis {}", cell.cell_id, bound.axis_id))?;
            if bound.unit != axis.unit {
                return Err(format!(
                    "{}: unit mismatch for axis {}: expected {}, got {}",
                    cell.cell_id, bound.axis_id, axis.unit, bound.unit
                ));
            }
            if !bound.lower.is_finite()
                || !bound.upper.is_finite()
                || bound.lower >= bound.upper
                || bound.lower < axis.lower
                || bound.upper > axis.upper
            {
                return Err(format!(
                    "{}: out-of-domain or invalid bound for {}",
                    cell.cell_id, bound.axis_id
                ));
            }
        }
        if cell.coverage.coverage_kind == CoverageKind::PointSamplesOnly
            && cell.coverage.covers_entire_cell
        {
            return Err(format!(
                "{}: point_samples_only cannot cover an entire cell",
                cell.cell_id
            ));
        }
        if let Some(outcomes) = &cell.outcomes {
            validate_count_pair(
                &format!("{} false-trip count", cell.cell_id),
                outcomes.false_trip_count_upper_bound,
                outcomes.false_trip_denominator,
            )?;
            validate_count_pair(
                &format!("{} failure-to-trip count", cell.cell_id),
                outcomes.failure_to_trip_count_upper_bound,
                outcomes.failure_to_trip_denominator,
            )?;
            for value in [
                outcomes.minimum_coordination_margin_pu,
                outcomes.required_coordination_margin_pu,
            ]
            .into_iter()
            .flatten()
            {
                if !value.is_finite() {
                    return Err(format!("{}: non-finite coordination margin", cell.cell_id));
                }
            }
            let mut local_services = BTreeSet::new();
            for margin in &outcomes.service_floor_margins {
                let required_service = corpus
                    .required_services
                    .iter()
                    .find(|service| service.service_id == margin.service_id)
                    .ok_or_else(|| {
                        format!(
                            "{}: unexpected service-floor margin {}",
                            cell.cell_id, margin.service_id
                        )
                    })?;
                if margin.service_id.trim().is_empty()
                    || margin.unit.trim().is_empty()
                    || margin.unit != required_service.unit
                    || !margin.minimum_margin.is_finite()
                    || !local_services.insert(margin.service_id.as_str())
                {
                    return Err(format!(
                        "{}: invalid, unit-mismatched or duplicate service-floor margin {}",
                        cell.cell_id, margin.service_id
                    ));
                }
            }
        }
        for artifact in [
            cell.coverage.source_artifact.as_ref(),
            cell.coverage.model_artifact.as_ref(),
            cell.coverage.proof_artifact.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if !is_sha256(&artifact.sha256) {
                return Err(format!(
                    "{}: malformed SHA-256 for {}",
                    cell.cell_id, artifact.path
                ));
            }
            let relative = Path::new(&artifact.path);
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|part| !matches!(part, Component::Normal(_)))
            {
                return Err(format!(
                    "{}: artifact path must be a safe relative path",
                    cell.cell_id
                ));
            }
        }
    }
    Ok(())
}

fn verify_artifact(reference: Option<&ArtifactRef>, root: &Path) -> Result<bool, String> {
    let Some(reference) = reference else {
        return Ok(false);
    };
    let path = root.join(&reference.path);
    let bytes = fs::read(&path)
        .map_err(|error| format!("cannot read artifact {}: {error}", path.display()))?;
    let actual = sha256(&bytes);
    if actual != reference.sha256 {
        return Err(format!(
            "artifact digest mismatch for {}: expected {}, got {}",
            reference.path, reference.sha256, actual
        ));
    }
    Ok(true)
}

fn verify_corpus(
    corpus: &Corpus,
    input_bytes: &[u8],
    artifact_root: &Path,
) -> Result<Report, String> {
    validate_corpus(corpus)?;

    let mut results = Vec::with_capacity(corpus.cells.len());
    for cell in &corpus.cells {
        let source_ok = verify_artifact(cell.coverage.source_artifact.as_ref(), artifact_root)?;
        let model_ok = verify_artifact(cell.coverage.model_artifact.as_ref(), artifact_root)?;
        let proof_ok = verify_artifact(cell.coverage.proof_artifact.as_ref(), artifact_root)?;
        results.push(assess_cell(
            cell,
            &corpus.required_services,
            source_ok,
            model_ok,
            proof_ok,
        ));
    }

    let certified = results
        .iter()
        .filter(|item| item.classification == Classification::Certified)
        .count();
    let excluded = results
        .iter()
        .filter(|item| item.classification == Classification::Excluded)
        .count();
    let unknown = results
        .iter()
        .filter(|item| item.classification == Classification::Unknown)
        .count();

    Ok(Report {
        status: "valid_synthetic_contract",
        benchmark_id: corpus.benchmark_id.clone(),
        schema_version: corpus.schema_version.clone(),
        input_sha256: sha256(input_bytes),
        synthetic_only: true,
        claim_ceiling: corpus.claim_ceiling.clone(),
        certified_cell_count: certified,
        excluded_cell_count: excluded,
        unknown_cell_count: unknown,
        cells: results,
        coverage_note: "Cell counts are not volume-weighted envelope coverage. A partition and an explicit measure are required before reporting a percentage.",
        limitations: vec![
            "E2 synthetic logic only; no physical protection or operational-safety qualification.",
            "The verifier checks declared whole-cell coverage metadata and digests, not the mathematical truth of a certificate producer.",
            "Point samples can falsify a cell when they show a qualified counterexample, but passing point samples cannot certify a cell.",
            "Evidence capabilities are independent facets, not a single total ordering of RMS, EMT, HIL, laboratory and field evidence.",
            "Owner-defined margins and service floors remain inputs; this tool does not invent universal thresholds.",
        ],
    })
}

fn assess_cell(
    cell: &Cell,
    required_services: &[RequiredService],
    source_ok: bool,
    model_ok: bool,
    proof_ok: bool,
) -> CellResult {
    let mut reasons = Vec::new();
    let mut false_rate = None;
    let mut failure_rate = None;
    let mut coordination_surplus = None;
    let mut service_receipts = Vec::new();

    let Some(outcomes) = &cell.outcomes else {
        return unknown(cell, vec!["outcome evidence is absent".to_string()]);
    };

    if let (Some(numerator), Some(denominator)) = (
        outcomes.false_trip_count_upper_bound,
        outcomes.false_trip_denominator,
    ) {
        false_rate = Some(RateBound {
            numerator,
            denominator,
        });
    }
    if let (Some(numerator), Some(denominator)) = (
        outcomes.failure_to_trip_count_upper_bound,
        outcomes.failure_to_trip_denominator,
    ) {
        failure_rate = Some(RateBound {
            numerator,
            denominator,
        });
    }
    if let (Some(minimum), Some(required)) = (
        outcomes.minimum_coordination_margin_pu,
        outcomes.required_coordination_margin_pu,
    ) {
        coordination_surplus = Some(minimum - required);
    }
    for margin in &outcomes.service_floor_margins {
        service_receipts.push(ServiceFloorMarginReceipt {
            service_id: margin.service_id.clone(),
            unit: margin.unit.clone(),
            minimum_margin: margin.minimum_margin,
        });
    }

    let missing_capabilities = cell
        .required_capabilities
        .missing_from(&cell.available_capabilities);
    if !missing_capabilities.is_empty() {
        reasons.push(format!(
            "required evidence capabilities unavailable: {}",
            missing_capabilities.join(", ")
        ));
        return CellResult {
            cell_id: cell.cell_id.clone(),
            classification: Classification::Unknown,
            reasons,
            false_trip_rate_upper_bound: false_rate,
            failure_to_trip_rate_upper_bound: failure_rate,
            coordination_margin_surplus_pu: coordination_surplus,
            service_floor_margins: service_receipts,
        };
    }
    if !source_ok || !model_ok {
        if !source_ok {
            reasons.push("source artifact is absent or not digest-verified".to_string());
        }
        if !model_ok {
            reasons.push("model artifact is absent or not digest-verified".to_string());
        }
        return CellResult {
            cell_id: cell.cell_id.clone(),
            classification: Classification::Unknown,
            reasons,
            false_trip_rate_upper_bound: false_rate,
            failure_to_trip_rate_upper_bound: failure_rate,
            coordination_margin_surplus_pu: coordination_surplus,
            service_floor_margins: service_receipts,
        };
    }

    let mut violations = Vec::new();
    if outcomes.false_trip_count_upper_bound.unwrap_or(0) > 0 {
        violations.push("false-trip count is non-zero");
    }
    if outcomes.failure_to_trip_count_upper_bound.unwrap_or(0) > 0 {
        violations.push("failure-to-trip count is non-zero");
    }
    if let (Some(minimum), Some(required)) = (
        outcomes.minimum_coordination_margin_pu,
        outcomes.required_coordination_margin_pu,
    ) && minimum < required
    {
        violations.push("minimum coordination margin is below the owner requirement");
    }
    for margin in &outcomes.service_floor_margins {
        if margin.minimum_margin < 0.0 {
            violations.push("a declared service-floor margin is negative");
            break;
        }
    }
    if !violations.is_empty() {
        reasons.extend(violations.into_iter().map(str::to_string));
        return CellResult {
            cell_id: cell.cell_id.clone(),
            classification: Classification::Excluded,
            reasons,
            false_trip_rate_upper_bound: false_rate,
            failure_to_trip_rate_upper_bound: failure_rate,
            coordination_margin_surplus_pu: coordination_surplus,
            service_floor_margins: service_receipts,
        };
    }

    let whole_cell_evidence = cell.coverage.covers_entire_cell
        && cell.coverage.uncertainty_set_covered
        && cell.coverage.coverage_kind == CoverageKind::IntervalEnclosure
        && outcomes.scope == OutcomeScope::WholeCellBound
        && proof_ok
        && cell
            .coverage
            .proof_method
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        && cell
            .coverage
            .model_revision
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());

    if !whole_cell_evidence {
        reasons.push(
            "no complete digest-bound evidence covering the entire cell and uncertainty set"
                .to_string(),
        );
        return CellResult {
            cell_id: cell.cell_id.clone(),
            classification: Classification::Unknown,
            reasons,
            false_trip_rate_upper_bound: false_rate,
            failure_to_trip_rate_upper_bound: failure_rate,
            coordination_margin_surplus_pu: coordination_surplus,
            service_floor_margins: service_receipts,
        };
    }

    let counts_complete = outcomes.false_trip_count_upper_bound.is_some()
        && outcomes.false_trip_denominator.is_some()
        && outcomes.failure_to_trip_count_upper_bound.is_some()
        && outcomes.failure_to_trip_denominator.is_some();
    let coordination_complete = outcomes.minimum_coordination_margin_pu.is_some()
        && outcomes.required_coordination_margin_pu.is_some();
    let services_complete = required_services.iter().all(|required| {
        outcomes
            .service_floor_margins
            .iter()
            .any(|margin| margin.service_id == required.service_id && margin.unit == required.unit)
    });

    if !counts_complete || !coordination_complete || !services_complete {
        if !counts_complete {
            reasons.push(
                "false-trip or failure-to-trip bounds/denominators are incomplete".to_string(),
            );
        }
        if !coordination_complete {
            reasons.push("coordination-margin bounds or owner requirement are missing".to_string());
        }
        if !services_complete {
            reasons.push("one or more required service-floor margins are missing".to_string());
        }
        return CellResult {
            cell_id: cell.cell_id.clone(),
            classification: Classification::Unknown,
            reasons,
            false_trip_rate_upper_bound: false_rate,
            failure_to_trip_rate_upper_bound: failure_rate,
            coordination_margin_surplus_pu: coordination_surplus,
            service_floor_margins: service_receipts,
        };
    }

    reasons.push("whole-cell bounds pass the declared protection and service constraints at the declared evidence capability".to_string());
    CellResult {
        cell_id: cell.cell_id.clone(),
        classification: Classification::Certified,
        reasons,
        false_trip_rate_upper_bound: false_rate,
        failure_to_trip_rate_upper_bound: failure_rate,
        coordination_margin_surplus_pu: coordination_surplus,
        service_floor_margins: service_receipts,
    }
}

fn unknown(cell: &Cell, reasons: Vec<String>) -> CellResult {
    CellResult {
        cell_id: cell.cell_id.clone(),
        classification: Classification::Unknown,
        reasons,
        false_trip_rate_upper_bound: None,
        failure_to_trip_rate_upper_bound: None,
        coordination_margin_surplus_pu: None,
        service_floor_margins: Vec::new(),
    }
}

fn run(path: &Path) -> Result<Report, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read corpus {}: {error}", path.display()))?;
    let corpus: Corpus = serde_json::from_slice(&bytes).map_err(|error| {
        format!("corpus JSON does not match the strict typed contract: {error}")
    })?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    verify_corpus(&corpus, &bytes, root)
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(path) = args.next() else {
        eprintln!(
            "usage: {} <synthetic-corpus.json>",
            PathBuf::from(program).display()
        );
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("accepts exactly one corpus path");
        return ExitCode::from(2);
    }
    match run(Path::new(&path)).and_then(|report| {
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("cannot encode report: {error}"))
    }) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("GMD envelope verification failed: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/research/gmd-ibr-protection/fixtures/gmd_protection_envelope_corpus_v0_1.json")
    }

    fn fixture() -> (Corpus, Vec<u8>, PathBuf) {
        let path = fixture_path();
        let bytes = fs::read(&path).expect("fixture must be readable");
        let corpus: Corpus = serde_json::from_slice(&bytes).expect("fixture must parse");
        let root = path.parent().expect("fixture parent");
        (corpus, bytes, root.to_path_buf())
    }

    #[test]
    fn frozen_fixture_has_one_certified_three_excluded_and_two_unknown_cells() {
        let (corpus, bytes, root) = fixture();
        let report = verify_corpus(&corpus, &bytes, &root).expect("valid synthetic corpus");
        assert_eq!(report.certified_cell_count, 1);
        assert_eq!(report.excluded_cell_count, 3);
        assert_eq!(report.unknown_cell_count, 2);
    }

    #[test]
    fn passing_point_samples_never_certify_a_cell() {
        let (corpus, bytes, root) = fixture();
        let report = verify_corpus(&corpus, &bytes, &root).expect("valid synthetic corpus");
        let cell = report
            .cells
            .iter()
            .find(|cell| cell.cell_id == "cell-003-passing-points-only")
            .unwrap();
        assert_eq!(cell.classification, Classification::Unknown);
    }

    #[test]
    fn physical_relay_requirement_is_not_satisfied_by_software_waveforms() {
        let (corpus, bytes, root) = fixture();
        let report = verify_corpus(&corpus, &bytes, &root).expect("valid synthetic corpus");
        let cell = report
            .cells
            .iter()
            .find(|cell| cell.cell_id == "cell-004-relay-capability-missing")
            .unwrap();
        assert_eq!(cell.classification, Classification::Unknown);
    }

    #[test]
    fn service_floor_violation_excludes_even_when_protection_metrics_pass() {
        let (corpus, bytes, root) = fixture();
        let report = verify_corpus(&corpus, &bytes, &root).expect("valid synthetic corpus");
        let cell = report
            .cells
            .iter()
            .find(|cell| cell.cell_id == "cell-005-service-floor-violation")
            .unwrap();
        assert_eq!(cell.classification, Classification::Excluded);
        assert!(
            cell.reasons
                .iter()
                .any(|reason| reason.contains("service-floor"))
        );
    }

    #[test]
    fn digest_mismatch_fails_the_entire_verification() {
        let (mut corpus, bytes, root) = fixture();
        corpus.cells[0]
            .coverage
            .proof_artifact
            .as_mut()
            .unwrap()
            .sha256 = "0".repeat(64);
        let error = verify_corpus(&corpus, &bytes, &root).unwrap_err();
        assert!(error.contains("artifact digest mismatch"));
    }

    #[test]
    fn out_of_domain_cell_fails_closed() {
        let (mut corpus, bytes, root) = fixture();
        corpus.cells[0].bounds[0].upper = 31.0;
        let error = verify_corpus(&corpus, &bytes, &root).unwrap_err();
        assert!(error.contains("out-of-domain"));
    }

    #[test]
    fn denominator_zero_is_rejected() {
        let (mut corpus, _bytes, _root) = fixture();
        let outcomes = corpus.cells[0].outcomes.as_mut().unwrap();
        outcomes.false_trip_denominator = Some(0);
        let error = validate_corpus(&corpus).unwrap_err();
        assert!(error.contains("denominator"));
    }

    #[test]
    fn axis_unit_mismatch_is_rejected() {
        let (mut corpus, _bytes, _root) = fixture();
        corpus.cells[0].bounds[0].unit = "mA".to_string();
        let error = validate_corpus(&corpus).unwrap_err();
        assert!(error.contains("unit mismatch"));
    }

    #[test]
    fn service_floor_unit_mismatch_is_rejected() {
        let (mut corpus, _bytes, _root) = fixture();
        corpus.cells[0]
            .outcomes
            .as_mut()
            .unwrap()
            .service_floor_margins[0]
            .unit = "litres_per_second".to_string();
        let error = validate_corpus(&corpus).unwrap_err();
        assert!(error.contains("unit-mismatched"));
    }

    #[test]
    fn duplicate_axis_bound_is_rejected() {
        let (mut corpus, _bytes, _root) = fixture();
        corpus.cells[0].bounds[1].axis_id = corpus.cells[0].bounds[0].axis_id.clone();
        let error = validate_corpus(&corpus).unwrap_err();
        assert!(error.contains("duplicate cell bound"));
    }

    #[test]
    fn failure_to_trip_counterexample_excludes_the_cell() {
        let (corpus, bytes, root) = fixture();
        let report = verify_corpus(&corpus, &bytes, &root).expect("valid synthetic corpus");
        let cell = report
            .cells
            .iter()
            .find(|cell| cell.cell_id == "cell-006-failure-to-trip-counterexample")
            .unwrap();
        assert_eq!(cell.classification, Classification::Excluded);
    }
}
