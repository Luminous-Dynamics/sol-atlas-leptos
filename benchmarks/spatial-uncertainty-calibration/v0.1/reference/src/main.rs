use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{env, fs, process};

const EPSILON: f64 = 1e-10;
const LOG_EPSILON: f64 = 1e-15;

#[derive(Clone, Debug)]
struct ProbabilityPrediction {
    prediction_id: String,
    site_id: String,
    sensor_group: String,
    probability: f64,
    outcome: bool,
}

#[derive(Clone, Debug)]
struct MetrologyObservation {
    observation_id: String,
    site_id: String,
    sensor_group: String,
    reference_value: f64,
    estimate: f64,
    lower: f64,
    upper: f64,
}

#[derive(Clone, Debug)]
struct Dataset {
    training_sites: BTreeSet<String>,
    evaluation_sites: BTreeSet<String>,
    predictions: Vec<ProbabilityPrediction>,
    measurements: Vec<MetrologyObservation>,
    calibration_bins: usize,
    interval_level: f64,
    selective_thresholds: Vec<f64>,
}

fn fail(message: impl AsRef<str>) -> ! {
    panic!("INVALID: {}", message.as_ref());
}

fn required<'a>(value: &'a Value, key: &str, path: &str) -> &'a Value {
    value
        .get(key)
        .unwrap_or_else(|| fail(format!("{path}.{key} is required")))
}

fn as_array<'a>(value: &'a Value, path: &str) -> &'a [Value] {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_else(|| fail(format!("{path} must be an array")))
}

fn as_string(value: &Value, path: &str) -> String {
    value
        .as_str()
        .unwrap_or_else(|| fail(format!("{path} must be a string")))
        .to_owned()
}

fn as_number(value: &Value, path: &str) -> f64 {
    let number = value
        .as_f64()
        .unwrap_or_else(|| fail(format!("{path} must be numeric")));
    if !number.is_finite() {
        fail(format!("{path} must be finite"));
    }
    number
}

fn read_site_set(value: &Value, path: &str) -> BTreeSet<String> {
    let mut sites = BTreeSet::new();
    for (index, item) in as_array(value, path).iter().enumerate() {
        let site = as_string(item, &format!("{path}[{index}]"));
        if !sites.insert(site.clone()) {
            fail(format!("duplicate site ID in {path}: {site}"));
        }
    }
    sites
}

fn parse_dataset(case: &Value) -> Dataset {
    if as_string(required(case, "fixtureStatus", "case"), "case.fixtureStatus") != "synthetic" {
        fail("reference evaluator only accepts synthetic fixtures");
    }

    let training_sites = read_site_set(
        required(case, "trainingSiteIds", "case"),
        "case.trainingSiteIds",
    );
    let evaluation_sites = read_site_set(
        required(case, "evaluationSiteIds", "case"),
        "case.evaluationSiteIds",
    );
    if training_sites.is_empty() || evaluation_sites.is_empty() {
        fail("training and evaluation site inventories must both be non-empty");
    }
    if !training_sites.is_disjoint(&evaluation_sites) {
        fail("training/evaluation site leakage: site IDs must be disjoint");
    }

    let calibration_bins = required(case, "calibrationBins", "case")
        .as_u64()
        .unwrap_or_else(|| fail("calibrationBins must be a positive integer")) as usize;
    if !(2..=50).contains(&calibration_bins) {
        fail("calibrationBins must be between 2 and 50");
    }

    let interval_level = as_number(
        required(case, "nominalIntervalLevel", "case"),
        "case.nominalIntervalLevel",
    );
    if !(0.0..1.0).contains(&interval_level) {
        fail("nominalIntervalLevel must be within (0, 1)");
    }

    let mut selective_thresholds = Vec::new();
    for (index, item) in as_array(
        required(case, "selectiveConfidenceThresholds", "case"),
        "case.selectiveConfidenceThresholds",
    )
    .iter()
    .enumerate()
    {
        let threshold = as_number(item, &format!("selectiveConfidenceThresholds[{index}]"));
        if !(0.5..=1.0).contains(&threshold) {
            fail("selective confidence thresholds must be within [0.5, 1]");
        }
        selective_thresholds.push(threshold);
    }
    selective_thresholds.sort_by(f64::total_cmp);
    selective_thresholds.dedup_by(|a, b| (*a - *b).abs() <= EPSILON);

    let mut predictions = Vec::new();
    let mut prediction_ids = BTreeSet::new();
    for (index, item) in as_array(
        required(case, "probabilityPredictions", "case"),
        "case.probabilityPredictions",
    )
    .iter()
    .enumerate()
    {
        let path = format!("probabilityPredictions[{index}]");
        let prediction_id = as_string(
            required(item, "predictionId", &path),
            &format!("{path}.predictionId"),
        );
        if !prediction_ids.insert(prediction_id.clone()) {
            fail(format!("duplicate prediction ID: {prediction_id}"));
        }
        let site_id = as_string(required(item, "siteId", &path), &format!("{path}.siteId"));
        if !evaluation_sites.contains(&site_id) {
            fail(format!("prediction {prediction_id} uses non-evaluation site {site_id}"));
        }
        let sensor_group = as_string(
            required(item, "sensorGroup", &path),
            &format!("{path}.sensorGroup"),
        );
        let probability = as_number(
            required(item, "probability", &path),
            &format!("{path}.probability"),
        );
        if !(0.0..=1.0).contains(&probability) {
            fail(format!("prediction {prediction_id} probability must be within [0, 1]"));
        }
        let outcome = required(item, "outcome", &path)
            .as_bool()
            .unwrap_or_else(|| fail(format!("{path}.outcome must be boolean")));
        predictions.push(ProbabilityPrediction {
            prediction_id,
            site_id,
            sensor_group,
            probability,
            outcome,
        });
    }
    if predictions.is_empty() {
        fail("at least one probability prediction is required");
    }

    let mut measurements = Vec::new();
    let mut observation_ids = BTreeSet::new();
    for (index, item) in as_array(
        required(case, "metrologyObservations", "case"),
        "case.metrologyObservations",
    )
    .iter()
    .enumerate()
    {
        let path = format!("metrologyObservations[{index}]");
        let observation_id = as_string(
            required(item, "observationId", &path),
            &format!("{path}.observationId"),
        );
        if !observation_ids.insert(observation_id.clone()) {
            fail(format!("duplicate metrology observation ID: {observation_id}"));
        }
        let site_id = as_string(required(item, "siteId", &path), &format!("{path}.siteId"));
        if !evaluation_sites.contains(&site_id) {
            fail(format!("metrology observation {observation_id} uses non-evaluation site {site_id}"));
        }
        let sensor_group = as_string(
            required(item, "sensorGroup", &path),
            &format!("{path}.sensorGroup"),
        );
        let reference_value = as_number(
            required(item, "referenceValue", &path),
            &format!("{path}.referenceValue"),
        );
        let estimate = as_number(
            required(item, "estimate", &path),
            &format!("{path}.estimate"),
        );
        let lower = as_number(required(item, "lower95", &path), &format!("{path}.lower95"));
        let upper = as_number(required(item, "upper95", &path), &format!("{path}.upper95"));
        if lower > upper {
            fail(format!("metrology observation {observation_id} has lower95 greater than upper95"));
        }
        measurements.push(MetrologyObservation {
            observation_id,
            site_id,
            sensor_group,
            reference_value,
            estimate,
            lower,
            upper,
        });
    }
    if measurements.is_empty() {
        fail("at least one metrology observation is required");
    }

    predictions.sort_by(|a, b| a.prediction_id.cmp(&b.prediction_id));
    measurements.sort_by(|a, b| a.observation_id.cmp(&b.observation_id));
    Ok_or_fail(Dataset {
        training_sites,
        evaluation_sites,
        predictions,
        measurements,
        calibration_bins,
        interval_level,
        selective_thresholds,
    })
}

fn Ok_or_fail(dataset: Dataset) -> Dataset {
    dataset
}

fn calibration_metrics(dataset: &Dataset) -> Value {
    let count = dataset.predictions.len() as f64;
    let brier_score = dataset
        .predictions
        .iter()
        .map(|prediction| {
            let observed = if prediction.outcome { 1.0 } else { 0.0 };
            (prediction.probability - observed).powi(2)
        })
        .sum::<f64>()
        / count;

    let log_loss = dataset
        .predictions
        .iter()
        .map(|prediction| {
            let p = prediction.probability.clamp(LOG_EPSILON, 1.0 - LOG_EPSILON);
            if prediction.outcome {
                -p.ln()
            } else {
                -(1.0 - p).ln()
            }
        })
        .sum::<f64>()
        / count;

    let mut bins: Vec<Vec<&ProbabilityPrediction>> =
        (0..dataset.calibration_bins).map(|_| Vec::new()).collect();
    for prediction in &dataset.predictions {
        let index = ((prediction.probability * dataset.calibration_bins as f64).floor() as usize)
            .min(dataset.calibration_bins - 1);
        bins[index].push(prediction);
    }

    let mut expected_calibration_error = 0.0;
    let calibration_bins: Vec<Value> = bins
        .iter()
        .enumerate()
        .map(|(index, bucket)| {
            let lower = index as f64 / dataset.calibration_bins as f64;
            let upper = (index + 1) as f64 / dataset.calibration_bins as f64;
            if bucket.is_empty() {
                return json!({
                    "lowerInclusive": lower,
                    "upperExclusiveExceptLast": upper,
                    "count": 0,
                    "meanPredictedProbability": null,
                    "observedFrequency": null,
                    "absoluteCalibrationGap": null
                });
            }
            let mean_probability = bucket
                .iter()
                .map(|prediction| prediction.probability)
                .sum::<f64>()
                / bucket.len() as f64;
            let observed_frequency = bucket
                .iter()
                .filter(|prediction| prediction.outcome)
                .count() as f64
                / bucket.len() as f64;
            let gap = (mean_probability - observed_frequency).abs();
            expected_calibration_error += bucket.len() as f64 / count * gap;
            json!({
                "lowerInclusive": lower,
                "upperExclusiveExceptLast": upper,
                "count": bucket.len(),
                "meanPredictedProbability": mean_probability,
                "observedFrequency": observed_frequency,
                "absoluteCalibrationGap": gap
            })
        })
        .collect();

    let selective_risk: Vec<Value> = dataset
        .selective_thresholds
        .iter()
        .map(|threshold| {
            let accepted: Vec<&ProbabilityPrediction> = dataset
                .predictions
                .iter()
                .filter(|prediction| {
                    prediction.probability.max(1.0 - prediction.probability) >= *threshold
                })
                .collect();
            let errors = accepted
                .iter()
                .filter(|prediction| {
                    (prediction.probability >= 0.5) != prediction.outcome
                })
                .count();
            let accepted_count = accepted.len();
            json!({
                "confidenceThreshold": threshold,
                "acceptedCount": accepted_count,
                "selectiveCoverage": accepted_count as f64 / count,
                "selectiveRisk": if accepted_count == 0 { Value::Null } else { json!(errors as f64 / accepted_count as f64) }
            })
        })
        .collect();

    json!({
        "sampleCount": dataset.predictions.len(),
        "brierScore": brier_score,
        "logLoss": log_loss,
        "expectedCalibrationError": expected_calibration_error,
        "binCount": dataset.calibration_bins,
        "reliabilityBins": calibration_bins,
        "selectiveRisk": selective_risk
    })
}

fn group_coverages<F>(measurements: &[MetrologyObservation], key: F) -> Value
where
    F: Fn(&MetrologyObservation) -> &str,
{
    let mut groups: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for observation in measurements {
        let entry = groups.entry(key(observation).to_owned()).or_insert((0, 0));
        entry.0 += 1;
        if observation.lower <= observation.reference_value
            && observation.reference_value <= observation.upper
        {
            entry.1 += 1;
        }
    }
    let rows: Vec<Value> = groups
        .iter()
        .map(|(group, (count, covered))| {
            json!({
                "group": group,
                "sampleCount": count,
                "coveredCount": covered,
                "empiricalCoverage": *covered as f64 / *count as f64
            })
        })
        .collect();
    json!(rows)
}

fn worst_coverage(groups: &Value) -> f64 {
    groups
        .as_array()
        .unwrap_or_else(|| fail("group coverage result must be an array"))
        .iter()
        .map(|row| {
            as_number(
                required(row, "empiricalCoverage", "groupCoverage"),
                "groupCoverage.empiricalCoverage",
            )
        })
        .fold(f64::INFINITY, f64::min)
}

fn metrology_metrics(dataset: &Dataset) -> Value {
    let count = dataset.measurements.len() as f64;
    let covered = dataset
        .measurements
        .iter()
        .filter(|observation| {
            observation.lower <= observation.reference_value
                && observation.reference_value <= observation.upper
        })
        .count();
    let mean_absolute_error = dataset
        .measurements
        .iter()
        .map(|observation| (observation.estimate - observation.reference_value).abs())
        .sum::<f64>()
        / count;
    let mean_interval_width = dataset
        .measurements
        .iter()
        .map(|observation| observation.upper - observation.lower)
        .sum::<f64>()
        / count;
    let sensor_coverage = group_coverages(&dataset.measurements, |item| &item.sensor_group);
    let site_coverage = group_coverages(&dataset.measurements, |item| &item.site_id);

    json!({
        "sampleCount": dataset.measurements.len(),
        "nominalIntervalLevel": dataset.interval_level,
        "empiricalIntervalCoverage": covered as f64 / count,
        "meanIntervalWidth": mean_interval_width,
        "meanAbsoluteError": mean_absolute_error,
        "bySensorGroup": sensor_coverage,
        "bySite": site_coverage,
        "worstSensorGroupCoverage": worst_coverage(&sensor_coverage),
        "worstSiteCoverage": worst_coverage(&site_coverage)
    })
}

fn calculate(case: &Value) -> Value {
    let dataset = parse_dataset(case);
    json!({
        "benchmarkId": "sol-atlas-spatial-uncertainty-calibration",
        "version": "0.1.0",
        "fixtureStatus": "synthetic",
        "split": {
            "trainingSiteCount": dataset.training_sites.len(),
            "evaluationSiteCount": dataset.evaluation_sites.len(),
            "siteOverlap": 0
        },
        "probabilisticCalibration": calibration_metrics(&dataset),
        "spatialMetrology": metrology_metrics(&dataset),
        "qualificationBoundary": {
            "fieldEvidence": false,
            "holdoutPolicy": "split by entire site/campaign; never split acquisitions from the same site between training and evaluation",
            "requiredReporting": [
                "proper scoring rules",
                "reliability diagram and calibration error",
                "selective risk versus coverage",
                "nominal interval coverage and width",
                "worst sensor-group and worst-site coverage"
            ]
        }
    })
}

fn read_json(path: &str) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|error| fail(format!("cannot read {path}: {error}")));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| fail(format!("invalid JSON in {path}: {error}")))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("fixtures/calibration-case.json");
    if args.len() > 2 {
        eprintln!("usage: sol-atlas-spatial-calibration [calibration-case.json]");
        process::exit(64);
    }
    let result = calculate(&read_json(path));
    println!(
        "{}",
        serde_json::to_string_pretty(&result).expect("result serializes")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../../fixtures/calibration-case.json")).unwrap()
    }

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() <= 1e-8, "expected {right}, got {left}");
    }

    #[test]
    fn computes_proper_scores_ece_and_spatial_interval_coverage() {
        let result = calculate(&fixture());
        close(
            result["probabilisticCalibration"]["brierScore"].as_f64().unwrap(),
            0.21501666666666666,
        );
        close(
            result["probabilisticCalibration"]["logLoss"].as_f64().unwrap(),
            0.9844225560537794,
        );
        close(
            result["probabilisticCalibration"]["expectedCalibrationError"]
                .as_f64()
                .unwrap(),
            0.24833333333333332,
        );
        close(
            result["spatialMetrology"]["empiricalIntervalCoverage"]
                .as_f64()
                .unwrap(),
            4.0 / 6.0,
        );
        close(
            result["spatialMetrology"]["meanIntervalWidth"]
                .as_f64()
                .unwrap(),
            0.566666666666667,
        );
        close(
            result["spatialMetrology"]["meanAbsoluteError"]
                .as_f64()
                .unwrap(),
            0.3166666666666666,
        );
        close(
            result["spatialMetrology"]["worstSensorGroupCoverage"]
                .as_f64()
                .unwrap(),
            1.0 / 3.0,
        );
        close(
            result["spatialMetrology"]["worstSiteCoverage"]
                .as_f64()
                .unwrap(),
            0.5,
        );
        close(
            result["probabilisticCalibration"]["selectiveRisk"][2]["selectiveRisk"]
                .as_f64()
                .unwrap(),
            1.0 / 3.0,
        );
    }

    #[test]
    fn rejects_training_evaluation_site_leakage() {
        let mut case = fixture();
        case["trainingSiteIds"].as_array_mut().unwrap().push(json!("site-alpha"));
        let _ = calculate(&case);
    }

    #[test]
    fn rejects_observation_from_unregistered_evaluation_site() {
        let mut case = fixture();
        case["metrologyObservations"][0]["siteId"] = json!("unregistered-site");
        let _ = calculate(&case);
    }

    #[test]
    fn rejects_probability_outside_unit_interval() {
        let mut case = fixture();
        case["probabilityPredictions"][0]["probability"] = json!(1.01);
        let _ = calculate(&case);
    }

    #[test]
    fn rejects_inverted_metrology_interval() {
        let mut case = fixture();
        case["metrologyObservations"][0]["lower95"] = json!(11.0);
        case["metrologyObservations"][0]["upper95"] = json!(10.0);
        let _ = calculate(&case);
    }

    #[test]
    fn input_order_does_not_change_metric_results() {
        let mut case = fixture();
        case["probabilityPredictions"].as_array_mut().unwrap().reverse();
        case["metrologyObservations"].as_array_mut().unwrap().reverse();
        assert_eq!(calculate(&fixture()), calculate(&case));
    }
}
