use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    process::ExitCode,
};

const EPSILON: f64 = 1e-9;

#[derive(Clone)]
struct Observation {
    x: f64,
    y: f64,
    graph_community: String,
    admin_area: String,
    region: String,
    campaign: String,
}

fn required<'a>(value: &'a Value, key: &str, path: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("{path}.{key} is required"))
}

fn as_string(value: &Value, path: &str) -> Result<String, String> {
    value
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{path} must be a non-empty string"))
}

fn as_number(value: &Value, path: &str) -> Result<f64, String> {
    let number = value
        .as_f64()
        .ok_or_else(|| format!("{path} must be numeric"))?;
    if !number.is_finite() {
        return Err(format!("{path} must be finite"));
    }
    Ok(number)
}

fn validate_digest(value: &Value, path: &str) -> Result<String, String> {
    let digest = as_string(value, path)?;
    let hex = digest.strip_prefix("sha256:").ok_or_else(|| {
        format!("{path} must use sha256:<64 lowercase hexadecimal characters>")
    })?;
    if hex.len() != 64 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(format!("{path} must use sha256:<64 hexadecimal characters>"));
    }
    Ok(digest)
}

fn parse_observations(value: &Value) -> Result<BTreeMap<String, Observation>, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| "case.observations must be an array".to_string())?;
    if rows.is_empty() {
        return Err("case.observations must not be empty".into());
    }
    let mut observations = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let path = format!("case.observations[{index}]");
        let id = as_string(
            required(row, "observationId", &path)?,
            &format!("{path}.observationId"),
        )?;
        let x = as_number(required(row, "x", &path)?, &format!("{path}.x"))?;
        let y = as_number(required(row, "y", &path)?, &format!("{path}.y"))?;
        let observation = Observation {
            x,
            y,
            graph_community: as_string(
                required(row, "graphCommunityId", &path)?,
                &format!("{path}.graphCommunityId"),
            )?,
            admin_area: as_string(
                required(row, "adminAreaId", &path)?,
                &format!("{path}.adminAreaId"),
            )?,
            region: as_string(
                required(row, "regionId", &path)?,
                &format!("{path}.regionId"),
            )?,
            campaign: as_string(
                required(row, "campaignId", &path)?,
                &format!("{path}.campaignId"),
            )?,
        };
        if observations.insert(id.clone(), observation).is_some() {
            return Err(format!("duplicate observationId {id}"));
        }
    }
    Ok(observations)
}

fn parse_id_set(value: &Value, path: &str) -> Result<Vec<String>, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| format!("{path} must be an array"))?;
    if rows.is_empty() {
        return Err(format!("{path} must not be empty"));
    }
    let mut ids = Vec::new();
    let mut unique = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let id = as_string(row, &format!("{path}[{index}]"))?;
        if !unique.insert(id.clone()) {
            return Err(format!("{path} contains duplicate identifier {id}"));
        }
        ids.push(id);
    }
    ids.sort();
    Ok(ids)
}

fn overlap_dimension(
    train: &[String],
    evaluation: &[String],
    observations: &BTreeMap<String, Observation>,
    dimension: fn(&Observation) -> &String,
) -> Vec<String> {
    let train_values: BTreeSet<String> = train
        .iter()
        .map(|id| dimension(&observations[id]).clone())
        .collect();
    let evaluation_values: BTreeSet<String> = evaluation
        .iter()
        .map(|id| dimension(&observations[id]).clone())
        .collect();
    train_values.intersection(&evaluation_values).cloned().collect()
}

fn evaluate_case(
    case: &Value,
    all_cases: &BTreeMap<String, &Value>,
) -> Result<Value, String> {
    let case_id = as_string(required(case, "caseId", "case")?, "case.caseId")?;
    let regime = as_string(required(case, "regime", "case")?, "case.regime")?;
    let allowed_regimes = [
        "T0_RANDOM_DIAGNOSTIC",
        "T1_MATCHED_SIZE_RANDOM_CONTROL",
        "T2_COORDINATE_BUFFERED",
        "T3_GRAPH_COMMUNITY_HOLDOUT",
        "T4_ADMINISTRATIVE_HOLDOUT",
        "T5_EXTERNAL_REGION_CAMPAIGN",
    ];
    if !allowed_regimes.contains(&regime.as_str()) {
        return Err(format!("case {case_id} uses unsupported regime {regime}"));
    }

    let dataset_digest = validate_digest(
        required(case, "datasetDigest", "case")?,
        "case.datasetDigest",
    )?;
    let observations = parse_observations(required(case, "observations", "case")?)?;
    let train = parse_id_set(
        required(case, "trainingObservationIds", "case")?,
        "case.trainingObservationIds",
    )?;
    let evaluation = parse_id_set(
        required(case, "evaluationObservationIds", "case")?,
        "case.evaluationObservationIds",
    )?;

    let training_set: BTreeSet<&String> = train.iter().collect();
    let evaluation_set: BTreeSet<&String> = evaluation.iter().collect();
    let identity_overlap: Vec<String> = training_set
        .intersection(&evaluation_set)
        .map(|id| (*id).clone())
        .collect();
    if !identity_overlap.is_empty() {
        return Err(format!(
            "case {case_id} has train/evaluation observation leakage: {identity_overlap:?}"
        ));
    }
    for id in train.iter().chain(evaluation.iter()) {
        if !observations.contains_key(id) {
            return Err(format!("case {case_id} references unknown observation {id}"));
        }
    }

    let mut matched_training_count = Value::Null;
    if regime == "T1_MATCHED_SIZE_RANDOM_CONTROL" {
        let reference_id = as_string(
            required(case, "matchedToCaseId", "case")?,
            "case.matchedToCaseId",
        )?;
        if reference_id == case_id {
            return Err(format!("case {case_id} cannot use itself as its matched-size reference"));
        }
        let reference = all_cases
            .get(&reference_id)
            .ok_or_else(|| format!("case {case_id} references unknown matched case {reference_id}"))?;
        let reference_regime = as_string(
            required(reference, "regime", "matched case")?,
            "matched case.regime",
        )?;
        if ![
            "T2_COORDINATE_BUFFERED",
            "T3_GRAPH_COMMUNITY_HOLDOUT",
            "T4_ADMINISTRATIVE_HOLDOUT",
            "T5_EXTERNAL_REGION_CAMPAIGN",
        ]
        .contains(&reference_regime.as_str())
        {
            return Err(format!(
                "case {case_id} matched-size reference must be a spatial or external holdout, not {reference_regime}"
            ));
        }
        let reference_train = parse_id_set(
            required(reference, "trainingObservationIds", "matched case")?,
            "matched case.trainingObservationIds",
        )?;
        let reference_digest = validate_digest(
            required(reference, "datasetDigest", "matched case")?,
            "matched case.datasetDigest",
        )?;
        if reference_digest != dataset_digest {
            return Err(format!("case {case_id} matched control uses a different dataset digest"));
        }
        if train.len() != reference_train.len() {
            return Err(format!(
                "case {case_id} training size {} does not match {} training observations in {}",
                train.len(),
                reference_train.len(),
                reference_id
            ));
        }
        matched_training_count = json!(reference_train.len());
    }

    let mut minimum_distance = Value::Null;
    if regime == "T2_COORDINATE_BUFFERED" {
        let crs = as_string(
            required(case, "coordinateReferenceSystem", "case")?,
            "case.coordinateReferenceSystem",
        )?;
        let units = as_string(
            required(case, "coordinateUnits", "case")?,
            "case.coordinateUnits",
        )?;
        if units != "metre" {
            return Err(format!("case {case_id} coordinateUnits must be metre"));
        }
        let threshold = as_number(
            required(case, "minimumSeparationMeters", "case")?,
            "case.minimumSeparationMeters",
        )?;
        if threshold <= 0.0 {
            return Err(format!("case {case_id} minimumSeparationMeters must be positive"));
        }
        if crs.trim().is_empty() {
            return Err(format!("case {case_id} coordinateReferenceSystem must be declared"));
        }
        let mut observed_minimum = f64::INFINITY;
        for train_id in &train {
            for evaluation_id in &evaluation {
                let left = &observations[train_id];
                let right = &observations[evaluation_id];
                let distance = (left.x - right.x).hypot(left.y - right.y);
                observed_minimum = observed_minimum.min(distance);
            }
        }
        if observed_minimum + EPSILON < threshold {
            return Err(format!(
                "case {case_id} minimum train/evaluation distance {observed_minimum} m is below required {threshold} m"
            ));
        }
        minimum_distance = json!({
            "observedMeters": observed_minimum,
            "requiredMeters": threshold,
            "coordinateReferenceSystem": crs,
            "coordinateUnits": units
        });
    }

    let mut disjoint_dimension_report = BTreeMap::<String, Value>::new();
    let checks: Vec<(&str, fn(&Observation) -> &String)> = match regime.as_str() {
        "T3_GRAPH_COMMUNITY_HOLDOUT" => vec![("graphCommunityId", |row| &row.graph_community)],
        "T4_ADMINISTRATIVE_HOLDOUT" => vec![("adminAreaId", |row| &row.admin_area)],
        "T5_EXTERNAL_REGION_CAMPAIGN" => vec![
            ("regionId", |row| &row.region),
            ("campaignId", |row| &row.campaign),
        ],
        _ => Vec::new(),
    };
    for (name, accessor) in checks {
        let overlaps = overlap_dimension(&train, &evaluation, &observations, accessor);
        if !overlaps.is_empty() {
            return Err(format!(
                "case {case_id} violates {name} holdout; overlapping units: {overlaps:?}"
            ));
        }
        disjoint_dimension_report.insert(name.to_owned(), json!("DISJOINT"));
    }

    let claim_ceiling = match regime.as_str() {
        "T0_RANDOM_DIAGNOSTIC" | "T1_MATCHED_SIZE_RANDOM_CONTROL" => {
            "WITHIN_MIXTURE_INTERPOLATION_DIAGNOSTIC_ONLY"
        }
        "T2_COORDINATE_BUFFERED" | "T3_GRAPH_COMMUNITY_HOLDOUT" => {
            "WITHIN_REGION_SPATIAL_SEPARATION_ONLY"
        }
        "T4_ADMINISTRATIVE_HOLDOUT" => "ADMINISTRATIVE_UNIT_TRANSFER_ONLY",
        "T5_EXTERNAL_REGION_CAMPAIGN" => "EXTERNAL_REGION_AND_CAMPAIGN_TRANSFER_CHALLENGE",
        _ => unreachable!(),
    };

    Ok(json!({
        "caseId": case_id,
        "regime": regime,
        "claimCeiling": claim_ceiling,
        "datasetDigest": dataset_digest,
        "trainingObservationCount": train.len(),
        "evaluationObservationCount": evaluation.len(),
        "identityOverlapCount": 0,
        "matchedTrainingCount": matched_training_count,
        "coordinateSeparation": minimum_distance,
        "disjointDimensions": disjoint_dimension_report,
        "metricScoringPermitted": true
    }))
}

fn evaluate_document(document: &Value) -> Result<Value, String> {
    if as_string(required(document, "benchmarkId", "document")?, "document.benchmarkId")?
        != "sol-atlas-tiered-spatial-validation-contracts"
    {
        return Err("unsupported benchmarkId".into());
    }
    let schema_version = as_string(
        required(document, "schemaVersion", "document")?,
        "document.schemaVersion",
    )?;
    if schema_version != "0.1.0" {
        return Err(format!("unsupported schemaVersion {schema_version}"));
    }
    validate_digest(
        required(document, "protocolDigest", "document")?,
        "document.protocolDigest",
    )?;
    if required(document, "sealedBeforeScoring", "document")?.as_bool() != Some(true) {
        return Err("sealedBeforeScoring must be true before metrics can be scored".into());
    }

    let cases = required(document, "cases", "document")?
        .as_array()
        .ok_or_else(|| "document.cases must be an array".to_string())?;
    if cases.is_empty() {
        return Err("document.cases must not be empty".into());
    }
    let mut case_map = BTreeMap::new();
    for case in cases {
        let id = as_string(required(case, "caseId", "case")?, "case.caseId")?;
        if case_map.insert(id.clone(), case).is_some() {
            return Err(format!("duplicate caseId {id}"));
        }
    }

    let mut results = Vec::new();
    for case in case_map.values() {
        results.push(evaluate_case(case, &case_map)?);
    }
    results.sort_by(|left, right| left["caseId"].as_str().cmp(&right["caseId"].as_str()));
    let observed_regimes: BTreeSet<String> = results
        .iter()
        .filter_map(|row| row["regime"].as_str().map(str::to_owned))
        .collect();

    Ok(json!({
        "benchmarkId": "sol-atlas-tiered-spatial-validation-contracts",
        "version": "0.1.0",
        "fixtureStatus": "synthetic",
        "protocolSealed": true,
        "observedRegimes": observed_regimes,
        "claimCeiling": "SPLIT_INTEGRITY_ONLY_NO_MODEL_PERFORMANCE_CLAIM",
        "caseCount": results.len(),
        "results": results
    }))
}

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/validation-regimes-v0.1.json".to_string());
    let input = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {path}: {error}"))?;
    let document: Value = serde_json::from_str(&input)
        .map_err(|error| format!("invalid JSON in {path}: {error}"))?;
    let report = evaluate_document(&document)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("cannot serialize report: {error}"))?
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tiered-spatial-validation: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../fixtures/validation-regimes-v0.1.json");

    fn fixture() -> Value {
        serde_json::from_str(FIXTURE).expect("fixture must parse")
    }

    fn case<'a>(report: &'a Value, id: &str) -> &'a Value {
        report["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["caseId"] == id)
            .expect("case result exists")
    }

    #[test]
    fn validates_six_distinct_regime_claim_ceilings() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        assert_eq!(report["caseCount"], 6);
        assert_eq!(report["observedRegimes"].as_array().unwrap().len(), 6);
        assert_eq!(
            case(&report, "t0-random")["claimCeiling"],
            "WITHIN_MIXTURE_INTERPOLATION_DIAGNOSTIC_ONLY"
        );
        assert_eq!(
            case(&report, "t5-external")["claimCeiling"],
            "EXTERNAL_REGION_AND_CAMPAIGN_TRANSFER_CHALLENGE"
        );
    }

    #[test]
    fn matched_size_control_is_checked_against_spatial_split() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        assert_eq!(case(&report, "t1-matched-random")["matchedTrainingCount"], 4);
    }

    #[test]
    fn coordinate_buffer_reports_actual_minimum_distance() {
        let report = evaluate_document(&fixture()).expect("fixture evaluates");
        let separation = &case(&report, "t2-buffered")["coordinateSeparation"];
        assert_eq!(separation["observedMeters"], 890.0);
        assert_eq!(separation["requiredMeters"], 500.0);
    }

    #[test]
    fn rejects_coordinate_buffer_violation() {
        let mut document = fixture();
        document["cases"][2]["evaluationObservationIds"] = json!(["o5", "o6"]);
        let result = evaluate_document(&document);
        assert!(result.unwrap_err().contains("below required"));
    }

    #[test]
    fn rejects_graph_community_leakage() {
        let mut document = fixture();
        document["cases"][3]["observations"][2]["graphCommunityId"] = json!("g1");
        let result = evaluate_document(&document);
        assert!(result.unwrap_err().contains("graphCommunityId"));
    }

    #[test]
    fn rejects_administrative_and_external_region_campaign_leakage() {
        let mut admin = fixture();
        admin["cases"][4]["observations"][4]["adminAreaId"] = json!("a1");
        assert!(evaluate_document(&admin).unwrap_err().contains("adminAreaId"));

        let mut external = fixture();
        external["cases"][5]["observations"][4]["regionId"] = json!("r1");
        assert!(evaluate_document(&external).unwrap_err().contains("regionId"));
    }

    #[test]
    fn rejects_matched_control_that_targets_a_random_diagnostic() {
        let mut document = fixture();
        document["cases"][1]["matchedToCaseId"] = json!("t0-random");
        assert!(evaluate_document(&document).unwrap_err().contains("must be a spatial or external holdout"));
    }

    #[test]
    fn rejects_matched_control_with_different_training_size() {
        let mut document = fixture();
        document["cases"][1]["trainingObservationIds"] = json!(["o1", "o3", "o5"]);
        assert!(evaluate_document(&document).unwrap_err().contains("does not match"));
    }

    #[test]
    fn rejects_identity_leakage_and_unsealed_protocol() {
        let mut overlap = fixture();
        overlap["cases"][0]["evaluationObservationIds"] = json!(["o1", "o4", "o6", "o8"]);
        assert!(evaluate_document(&overlap).unwrap_err().contains("observation leakage"));

        let mut unsealed = fixture();
        unsealed["sealedBeforeScoring"] = json!(false);
        assert!(evaluate_document(&unsealed).unwrap_err().contains("sealedBeforeScoring"));
    }

    #[test]
    fn output_is_invariant_to_case_and_split_array_order() {
        let baseline = evaluate_document(&fixture()).expect("baseline evaluates");
        let mut reversed = fixture();
        reversed["cases"].as_array_mut().unwrap().reverse();
        for case in reversed["cases"].as_array_mut().unwrap() {
            case["trainingObservationIds"].as_array_mut().unwrap().reverse();
            case["evaluationObservationIds"].as_array_mut().unwrap().reverse();
        }
        assert_eq!(baseline, evaluate_document(&reversed).expect("reversed fixture evaluates"));
    }
}
