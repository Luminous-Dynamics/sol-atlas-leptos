//! Independent audit of non-anticipativity across scenario schedules.
//!
//! Information-set identities are derived from exact source-observation
//! prefixes, decision timestamps, and the exact bytes of time-bounded forecast
//! bundles. Callers cannot submit an arbitrary information-set hash. Forecast
//! authenticity and calibration still require an independent provenance layer.

use super::{
    audit_error, current_binary_sha256, parse_unique_json, recompute_objectives, sha256,
    validate_request, validate_schedule, AuditError, DispatchSchedule,
    IntervalDispatch, SourceRequest, MAX_INTERVALS, MAX_REQUEST_BYTES, MAX_SCHEDULE_BYTES,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const NONANTICIPATIVITY_AUDITOR_ID: &str = "energy-dispatch-nonanticipativity-auditor";
pub const NONANTICIPATIVITY_RECEIPT_SCHEMA_V3: &str =
    "energy-dispatch-nonanticipativity-receipt-v0.3";
pub const MAX_SCENARIOS: usize = 256;

/// Information-set metadata reconstructed from exact observation and forecast artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NonAnticipativityInformationSetV1 {
    pub observed_at_unix_seconds: i64,
    pub observation_bundle_sha256: String,
    pub forecast_as_of_unix_seconds: i64,
    pub forecast_bundle_sha256: String,
}

/// Exact decision-time evidence supplied to the auditor. All digests are
/// recomputed from these bytes; no caller-selected information-set hash exists.
#[derive(Debug, Clone, Copy)]
pub struct NonAnticipativityInformationSetBytes<'a> {
    pub observed_at_unix_seconds: i64,
    pub observation_bundle_bytes: &'a [u8],
    pub forecast_as_of_unix_seconds: i64,
    pub forecast_bundle_bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub struct NonAnticipativityScenarioBytes<'a> {
    pub scenario_id: &'a str,
    pub request_bytes: &'a [u8],
    pub schedule_bytes: &'a [u8],
    /// One exact observation bundle and one exact forecast bundle per
    /// decision interval. The evaluator derives a cumulative history from
    /// these artifacts; it never treats the full realized request trajectory
    /// as information available to the policy.
    pub information_sets_by_interval: &'a [NonAnticipativityInformationSetBytes<'a>],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NonAnticipativityDispositionV1 {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonAnticipativityScenarioBindingV1 {
    pub scenario_id: String,
    pub request_sha256: String,
    pub schedule_sha256: String,
    pub information_sets_by_interval: Vec<NonAnticipativityInformationSetV1>,
    pub information_set_sha256_by_interval: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonAnticipativityFindingV1 {
    pub predicate: String,
    pub interval_index: Option<usize>,
    pub information_set_sha256: Option<String>,
    pub observation_history_sha256: Option<String>,
    pub scenario_ids: Vec<String>,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonAnticipativityReceiptV1 {
    pub receipt_schema: String,
    pub scenario_set_id: String,
    pub auditor_id: String,
    pub auditor_version: String,
    pub auditor_binary_sha256: String,
    pub scenario_count: usize,
    pub interval_count: usize,
    pub scenario_bindings: Vec<NonAnticipativityScenarioBindingV1>,
    pub shared_information_set_count: usize,
    pub checked_action_comparison_count: usize,
    pub disposition: NonAnticipativityDispositionV1,
    pub findings: Vec<NonAnticipativityFindingV1>,
    pub receipt_sha256: String,
    pub claim_ceiling: String,
}

#[derive(Serialize)]
struct NonAnticipativityReceiptSubjectV1<'a> {
    receipt_schema: &'a str,
    scenario_set_id: &'a str,
    auditor_id: &'a str,
    auditor_version: &'a str,
    auditor_binary_sha256: &'a str,
    scenario_count: usize,
    interval_count: usize,
    scenario_bindings: &'a [NonAnticipativityScenarioBindingV1],
    shared_information_set_count: usize,
    checked_action_comparison_count: usize,
    disposition: NonAnticipativityDispositionV1,
    findings: &'a [NonAnticipativityFindingV1],
    claim_ceiling: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DispatchActionV1 {
    grid_import_quanta: u16,
    charge_input_quanta: u16,
    discharge_withdrawal_quanta: u16,
    soc_after_quanta: u16,
    essential_served_quanta: u16,
    essential_unserved_quanta: u16,
    flexible_served_quanta: u16,
    flexible_unserved_quanta: u16,
    pv_used_quanta: u16,
    pv_curtailed_quanta: u16,
}

impl From<&IntervalDispatch> for DispatchActionV1 {
    fn from(step: &IntervalDispatch) -> Self {
        Self {
            grid_import_quanta: step.grid_import_quanta,
            charge_input_quanta: step.charge_input_quanta,
            discharge_withdrawal_quanta: step.discharge_withdrawal_quanta,
            soc_after_quanta: step.soc_after_quanta,
            essential_served_quanta: step.essential_served_quanta,
            essential_unserved_quanta: step.essential_unserved_quanta,
            flexible_served_quanta: step.flexible_served_quanta,
            flexible_unserved_quanta: step.flexible_unserved_quanta,
            pv_used_quanta: step.pv_used_quanta,
            pv_curtailed_quanta: step.pv_curtailed_quanta,
        }
    }
}

#[derive(Serialize)]
struct ObservationHistorySubjectV1<'a> {
    schema_version: &'a str,
    scenario_set_id: &'a str,
    interval_index: usize,
    previous_history_sha256: &'a str,
    observed_at_unix_seconds: i64,
    observation_bundle_sha256: &'a str,
}

#[derive(Serialize)]
struct InformationSetSubjectV1<'a> {
    schema_version: &'a str,
    scenario_set_id: &'a str,
    decision_time_unix_seconds: i64,
    observation_history_sha256: &'a str,
    observed_at_unix_seconds: i64,
    observation_bundle_sha256: &'a str,
    forecast_as_of_unix_seconds: i64,
    forecast_bundle_sha256: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservationBundleV1 {
    schema_version: String,
    scenario_set_id: String,
    bundle_id: String,
    observed_at_unix_seconds: i64,
    observation_payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForecastBundleV1 {
    schema_version: String,
    scenario_set_id: String,
    bundle_id: String,
    issued_at_unix_seconds: i64,
    valid_from_unix_seconds: i64,
    valid_through_unix_seconds: i64,
    forecast_payload: serde_json::Value,
}

const INFORMATION_SET_SCHEMA_V1: &str = "energy-dispatch-information-set-v0.1";
const OBSERVATION_BUNDLE_SCHEMA_V1: &str = "energy-dispatch-observation-bundle-v0.1";
const FORECAST_BUNDLE_SCHEMA_V1: &str = "energy-dispatch-forecast-bundle-v0.1";
pub const MAX_OBSERVATION_BUNDLE_BYTES: usize = 1_048_576;
pub const MAX_FORECAST_BUNDLE_BYTES: usize = 1_048_576;

fn valid_bundle_id(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
}

fn derive_information_set_sha256(
    scenario_set_id: &str,
    request: &SourceRequest,
    interval_index: usize,
    previous_history_sha256: &str,
    information_set: &NonAnticipativityInformationSetBytes<'_>,
) -> Result<(NonAnticipativityInformationSetV1, String, String), AuditError> {
    if information_set.observation_bundle_bytes.len() > MAX_OBSERVATION_BUNDLE_BYTES {
        return Err(audit_error("observation bundle exceeds the 1 MiB byte limit"));
    }
    if information_set.forecast_bundle_bytes.len() > MAX_FORECAST_BUNDLE_BYTES {
        return Err(audit_error("forecast bundle exceeds the 1 MiB byte limit"));
    }

    let interval = request
        .intervals
        .get(interval_index)
        .ok_or_else(|| audit_error("information-set interval index is outside the request"))?;
    let decision_time = interval.start_unix_seconds;

    let observation_value = parse_unique_json(information_set.observation_bundle_bytes)?;
    let observation: ObservationBundleV1 = serde_json::from_value(observation_value)
        .map_err(|error| audit_error(format!("observation bundle schema: {error}")))?;
    if observation.schema_version != OBSERVATION_BUNDLE_SCHEMA_V1
        || observation.scenario_set_id != scenario_set_id
        || !valid_bundle_id(&observation.bundle_id)
        || observation.observed_at_unix_seconds != information_set.observed_at_unix_seconds
        || observation.observed_at_unix_seconds > decision_time
        || !observation
            .observation_payload
            .as_object()
            .is_some_and(|payload| !payload.is_empty())
    {
        return Err(audit_error(
            "observation bundle identity, observation time or payload is invalid for this decision",
        ));
    }

    let forecast_value = parse_unique_json(information_set.forecast_bundle_bytes)?;
    let forecast: ForecastBundleV1 = serde_json::from_value(forecast_value)
        .map_err(|error| audit_error(format!("forecast bundle schema: {error}")))?;
    if forecast.schema_version != FORECAST_BUNDLE_SCHEMA_V1
        || forecast.scenario_set_id != scenario_set_id
        || !valid_bundle_id(&forecast.bundle_id)
        || forecast.issued_at_unix_seconds != information_set.forecast_as_of_unix_seconds
        || forecast.issued_at_unix_seconds > decision_time
        || forecast.valid_from_unix_seconds > decision_time
        || forecast.valid_through_unix_seconds < decision_time
        || forecast.valid_through_unix_seconds < forecast.valid_from_unix_seconds
        || !forecast
            .forecast_payload
            .as_object()
            .is_some_and(|payload| !payload.is_empty())
    {
        return Err(audit_error(
            "forecast bundle identity, issue time, validity window or payload is invalid for this decision",
        ));
    }

    let observation_bundle_sha256 = sha256(information_set.observation_bundle_bytes);
    let forecast_bundle_sha256 = sha256(information_set.forecast_bundle_bytes);
    let history_subject = ObservationHistorySubjectV1 {
        schema_version: "energy-dispatch-observation-history-v0.1",
        scenario_set_id,
        interval_index,
        previous_history_sha256,
        observed_at_unix_seconds: information_set.observed_at_unix_seconds,
        observation_bundle_sha256: &observation_bundle_sha256,
    };
    let history_bytes = serde_json::to_vec(&history_subject)
        .map_err(|error| audit_error(format!("serialize observation history: {error}")))?;
    let observation_history_sha256 = sha256(&history_bytes);
    let information_subject = InformationSetSubjectV1 {
        schema_version: INFORMATION_SET_SCHEMA_V1,
        scenario_set_id,
        decision_time_unix_seconds: decision_time,
        observation_history_sha256: &observation_history_sha256,
        observed_at_unix_seconds: information_set.observed_at_unix_seconds,
        observation_bundle_sha256: &observation_bundle_sha256,
        forecast_as_of_unix_seconds: information_set.forecast_as_of_unix_seconds,
        forecast_bundle_sha256: &forecast_bundle_sha256,
    };
    let information_bytes = serde_json::to_vec(&information_subject)
        .map_err(|error| audit_error(format!("serialize information-set subject: {error}")))?;
    Ok((
        NonAnticipativityInformationSetV1 {
            observed_at_unix_seconds: information_set.observed_at_unix_seconds,
            observation_bundle_sha256,
            forecast_as_of_unix_seconds: information_set.forecast_as_of_unix_seconds,
            forecast_bundle_sha256,
        },
        observation_history_sha256,
        sha256(&information_bytes),
    ))
}

struct ParsedScenario {
    scenario_id: String,
    request_sha256: String,
    schedule_sha256: String,
    request: SourceRequest,
    schedule: DispatchSchedule,
    information_sets_by_interval: Vec<NonAnticipativityInformationSetV1>,
    observation_history_sha256_by_interval: Vec<String>,
    information_set_sha256_by_interval: Vec<String>,
    domain_valid: bool,
    domain_failure_detail: String,
}

fn safe_scenario_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || "-_.:".contains(ch))
}

fn add_finding(
    findings: &mut Vec<NonAnticipativityFindingV1>,
    predicate: &str,
    interval_index: Option<usize>,
    information_set_sha256: Option<String>,
    observation_history_sha256: Option<String>,
    scenario_ids: Vec<String>,
    passed: bool,
    detail: impl Into<String>,
) {
    findings.push(NonAnticipativityFindingV1 {
        predicate: predicate.to_owned(),
        interval_index,
        information_set_sha256,
        observation_history_sha256,
        scenario_ids,
        passed,
        detail: detail.into(),
    });
}

/// Evaluate complete-horizon scenario schedules using auditor-derived information sets.
///
/// Each interval's information-set identity is derived from the exact source
/// observation prefix, decision timestamp, forecast vintage, scenario-set ID,
/// and SHA-256 of the exact strict-parsed forecast bundle bytes. The bundle's
/// issue time and validity window must cover the decision time.
///
/// For every interval, scenarios with equal derived identities must take the
/// same dispatch action. Distinct identities may diverge after new observations
/// or a new forecast artifact becomes available. A missing shared information
/// set is rejected rather than treated as vacuous success.
///
/// The evaluator binds forecast bytes but does not authenticate their producer,
/// prove their availability to an operator, or establish forecast calibration.
pub fn evaluate_nonanticipativity_bytes(
    scenario_set_id: &str,
    scenarios: &[NonAnticipativityScenarioBytes<'_>],
) -> Result<NonAnticipativityReceiptV1, AuditError> {
    if !safe_scenario_id(scenario_set_id) {
        return Err(audit_error("scenario_set_id must be a safe non-empty identifier"));
    }
    if !(2..=MAX_SCENARIOS).contains(&scenarios.len()) {
        return Err(audit_error(format!(
            "scenario count must be between 2 and {MAX_SCENARIOS}"
        )));
    }

    let mut parsed = Vec::with_capacity(scenarios.len());
    let mut seen_ids = std::collections::BTreeSet::new();
    for source in scenarios {
        if !safe_scenario_id(source.scenario_id) || !seen_ids.insert(source.scenario_id) {
            return Err(audit_error(
                "scenario IDs must be safe, non-empty and unique",
            ));
        }
        if source.request_bytes.len() > MAX_REQUEST_BYTES
            || source.schedule_bytes.len() > MAX_SCHEDULE_BYTES
        {
            return Err(audit_error("non-anticipativity input exceeds declared byte limit"));
        }

        let request_value = parse_unique_json(source.request_bytes)?;
        let request: SourceRequest = serde_json::from_value(request_value)
            .map_err(|error| audit_error(format!("scenario request schema: {error}")))?;
        let schedule_value = parse_unique_json(source.schedule_bytes)?;
        let schedule: DispatchSchedule = serde_json::from_value(schedule_value)
            .map_err(|error| audit_error(format!("scenario schedule schema: {error}")))?;

        if request.case_id != source.scenario_id {
            return Err(audit_error(
                "scenario ID must equal the source request case_id",
            ));
        }
        if request.intervals.is_empty()
            || request.intervals.len() > MAX_INTERVALS
            || source.information_sets_by_interval.len() != request.intervals.len()
            || schedule.interval_dispatches.len() != request.intervals.len()
        {
            return Err(audit_error(
                "each scenario must supply one information-set digest and one schedule step \
                 per source interval",
            ));
        }
        let mut information_set_material = Vec::with_capacity(
            source.information_sets_by_interval.len(),
        );
        let mut previous_history_sha256 =
            sha256(b"energy-dispatch-observation-history-root-v0.1");
        for (interval_index, information_set) in
            source.information_sets_by_interval.iter().enumerate()
        {
            let material = derive_information_set_sha256(
                scenario_set_id,
                &request,
                interval_index,
                &previous_history_sha256,
                information_set,
            )?;
            previous_history_sha256 = material.1.clone();
            information_set_material.push(material);
        }
        let information_sets_by_interval = information_set_material
            .iter()
            .map(|(information_set, _, _)| information_set.clone())
            .collect::<Vec<_>>();
        let observation_history_sha256_by_interval = information_set_material
            .iter()
            .map(|(_, observation_digest, _)| observation_digest.clone())
            .collect::<Vec<_>>();
        let information_set_sha256_by_interval = information_set_material
            .into_iter()
            .map(|(_, _, information_digest)| information_digest)
            .collect::<Vec<_>>();

        let request_sha256 = sha256(source.request_bytes);
        let schedule_sha256 = sha256(source.schedule_bytes);
        let mut domain_findings = Vec::new();
        validate_request(&request, &mut domain_findings);
        validate_schedule(
            &request,
            &schedule,
            &request_sha256,
            &mut domain_findings,
        );
        // Objective recomputation is part of admission: an overflow or missing
        // complete trajectory is not silently tolerated by this verifier.
        let _ = recompute_objectives(&request, &schedule)?;
        let failed_domain_findings: Vec<_> = domain_findings
            .iter()
            .filter(|finding| !finding.passed)
            .map(|finding| format!("{}: {}", finding.predicate, finding.detail))
            .collect();
        let domain_valid = failed_domain_findings.is_empty();
        let domain_failure_detail = if domain_valid {
            "all standalone request/schedule domain predicates passed".to_owned()
        } else {
            failed_domain_findings.join("; ")
        };

        parsed.push(ParsedScenario {
            scenario_id: source.scenario_id.to_owned(),
            request_sha256,
            schedule_sha256,
            request,
            schedule,
            information_sets_by_interval,
            observation_history_sha256_by_interval,
            information_set_sha256_by_interval,
            domain_valid,
            domain_failure_detail,
        });
    }

    let reference = &parsed[0];
    let common_model = parsed.iter().all(|scenario| {
        scenario.request.schema_version == reference.request.schema_version
            && scenario.request.synthetic == reference.request.synthetic
            && scenario.request.evidence_class == reference.request.evidence_class
            && scenario.request.max_candidate_expansions
                == reference.request.max_candidate_expansions
            && scenario.request.energy_quantum_kwh.to_bits()
                == reference.request.energy_quantum_kwh.to_bits()
            && scenario.request.battery == reference.request.battery
            && scenario.request.intervals.len() == reference.request.intervals.len()
            && scenario
                .request
                .intervals
                .iter()
                .zip(&reference.request.intervals)
                .all(|(left, right)| {
                    left.interval_id == right.interval_id
                        && left.start_unix_seconds == right.start_unix_seconds
                        && left.duration_seconds == right.duration_seconds
                })
    });
    if !common_model {
        return Err(audit_error(
            "scenario requests must share the same schema, asset configuration, energy \
             quantum and interval chronology",
        ));
    }

    let interval_count = reference.request.intervals.len();
    let scenario_bindings: Vec<_> = parsed
        .iter()
        .map(|scenario| NonAnticipativityScenarioBindingV1 {
            scenario_id: scenario.scenario_id.clone(),
            request_sha256: scenario.request_sha256.clone(),
            schedule_sha256: scenario.schedule_sha256.clone(),
            information_sets_by_interval: scenario.information_sets_by_interval.clone(),
            information_set_sha256_by_interval: scenario
                .information_set_sha256_by_interval
                .clone(),
        })
        .collect();

    let mut findings = Vec::new();
    for scenario in &parsed {
        add_finding(
            &mut findings,
            "scenario-schedule-domain-validity",
            None,
            None,
            None,
            vec![scenario.scenario_id.clone()],
            scenario.domain_valid,
            scenario.domain_failure_detail.clone(),
        );
    }

    let root_information_set_sha256 =
        parsed[0].information_set_sha256_by_interval[0].clone();
    let root_shared_by_all = parsed.iter().all(|scenario| {
        scenario.information_set_sha256_by_interval[0] == root_information_set_sha256
    });
    add_finding(
        &mut findings,
        "root-information-set-coverage",
        Some(0),
        Some(root_information_set_sha256),
        None,
        parsed.iter().map(|scenario| scenario.scenario_id.clone()).collect(),
        root_shared_by_all,
        if root_shared_by_all {
            "all scenarios share one initial information set before any declared branch"
        } else {
            "all scenarios in one scenario tree must share the same initial information set"
        },
    );

    // A scenario tree represents one decision process. If two scenarios
    // have identical source-observation histories at the same decision time,
    // they must not be split by assigning them different forecast artifacts.
    // Forecast updates may change the information set, but the same update
    // must be presented to every scenario with the same observed history.
    let mut observation_groups = BTreeMap::<(usize, String), Vec<usize>>::new();
    for (scenario_index, scenario) in parsed.iter().enumerate() {
        for (interval_index, observation_history_sha256) in
            scenario.observation_history_sha256_by_interval.iter().enumerate()
        {
            observation_groups
                .entry((interval_index, observation_history_sha256.clone()))
                .or_default()
                .push(scenario_index);
        }
    }

    for ((interval_index, observation_history_sha256), members) in observation_groups {
        if members.len() < 2 {
            continue;
        }
        let first = &parsed[members[0]];
        let first_information_set = &first.information_sets_by_interval[interval_index];
        let forecast_consistent = members.iter().skip(1).all(|member| {
            &parsed[*member].information_sets_by_interval[interval_index]
                == first_information_set
        });
        let mut scenario_ids: Vec<_> = members
            .iter()
            .map(|index| parsed[*index].scenario_id.clone())
            .collect();
        scenario_ids.sort();
        add_finding(
            &mut findings,
            "shared-observation-forecast-bundle-consistency",
            Some(interval_index),
            Some(first.information_set_sha256_by_interval[interval_index].clone()),
            Some(observation_history_sha256),
            scenario_ids,
            forecast_consistent,
            if forecast_consistent {
                "scenarios with identical observed history use the same forecast vintage \
                 and exact forecast bundle"
            } else {
                "scenarios with identical observed history cite different forecast \
                 vintages or exact forecast bundles"
            },
        );
    }

    let mut groups = BTreeMap::<(usize, String), Vec<usize>>::new();
    for (scenario_index, scenario) in parsed.iter().enumerate() {
        for (interval_index, information_set_sha256) in
            scenario.information_set_sha256_by_interval.iter().enumerate()
        {
            groups
                .entry((interval_index, information_set_sha256.clone()))
                .or_default()
                .push(scenario_index);
        }
    }

    let mut shared_information_set_count = 0usize;
    let mut checked_action_comparison_count = 0usize;
    for ((interval_index, information_set_sha256), members) in groups {
        if members.len() < 2 {
            continue;
        }
        shared_information_set_count += 1;
        checked_action_comparison_count = checked_action_comparison_count
            .checked_add(members.len() - 1)
            .ok_or_else(|| audit_error("action comparison count overflow"))?;

        let first = &parsed[members[0]];
        let observation_history = &first.request.intervals[..=interval_index];
        let observation_history_bytes = serde_json::to_vec(observation_history)
            .map_err(|error| audit_error(format!("serialize observation prefix: {error}")))?;
        let observation_history_sha256 = sha256(&observation_history_bytes);
        let observation_history_equal = members.iter().skip(1).all(|member| {
            &parsed[*member].request.intervals[..=interval_index] == observation_history
        });
        let first_action =
            DispatchActionV1::from(&first.schedule.interval_dispatches[interval_index]);
        let actions_equal = members.iter().skip(1).all(|member| {
            DispatchActionV1::from(
                &parsed[*member].schedule.interval_dispatches[interval_index],
            ) == first_action
        });
        let mut scenario_ids: Vec<_> = members
            .iter()
            .map(|index| parsed[*index].scenario_id.clone())
            .collect();
        scenario_ids.sort();

        add_finding(
            &mut findings,
            "shared-information-set-observation-prefix",
            Some(interval_index),
            Some(information_set_sha256.clone()),
            Some(observation_history_sha256),
            scenario_ids.clone(),
            observation_history_equal,
            if observation_history_equal {
                "scenarios sharing this information set have identical declared \
                 exogenous observation prefixes"
            } else {
                "scenarios share an information-set label but their declared \
                 observation prefixes differ"
            },
        );
        add_finding(
            &mut findings,
            "nonanticipative-dispatch-action",
            Some(interval_index),
            Some(information_set_sha256),
            None,
            scenario_ids,
            actions_equal,
            if actions_equal {
                "all scenarios with this information set choose the same dispatch action"
            } else {
                "scenarios with the same information set choose different dispatch actions"
            },
        );
    }

    if shared_information_set_count == 0 {
        add_finding(
            &mut findings,
            "shared-information-set-coverage",
            None,
            None,
            None,
            parsed.iter().map(|scenario| scenario.scenario_id.clone()).collect(),
            false,
            "no interval contains a shared information set; non-anticipativity would \
             otherwise pass vacuously",
        );
    }

    let disposition = if findings.iter().all(|finding| finding.passed) {
        NonAnticipativityDispositionV1::Accepted
    } else {
        NonAnticipativityDispositionV1::Rejected
    };
    let receipt_schema = NONANTICIPATIVITY_RECEIPT_SCHEMA_V3;
    let auditor_id = NONANTICIPATIVITY_AUDITOR_ID;
    let auditor_version = env!("CARGO_PKG_VERSION");
    let auditor_binary_sha256 = current_binary_sha256()?;
    let claim_ceiling = concat!(
        "equality of exact dispatch action fields for scenarios sharing information-set ",
        "SHA-256 identities derived by this auditor from exact source-observation prefixes ",
        "and exact time-bounded forecast bundle bytes; forecast bytes are content-bound but ",
        "their producer authenticity, actual operator availability and calibration are not ",
        "verified; no stochastic optimality, service-floor, physical safety or actuation claim"
    );
    let subject = NonAnticipativityReceiptSubjectV1 {
        receipt_schema,
        scenario_set_id,
        auditor_id,
        auditor_version,
        auditor_binary_sha256: &auditor_binary_sha256,
        scenario_count: parsed.len(),
        interval_count,
        scenario_bindings: &scenario_bindings,
        shared_information_set_count,
        checked_action_comparison_count,
        disposition,
        findings: &findings,
        claim_ceiling,
    };
    let subject_bytes = serde_json::to_vec(&subject)
        .map_err(|error| {
            audit_error(format!(
                "serialize non-anticipativity receipt subject: {error}"
            ))
        })?;

    Ok(NonAnticipativityReceiptV1 {
        receipt_schema: receipt_schema.to_owned(),
        scenario_set_id: scenario_set_id.to_owned(),
        auditor_id: auditor_id.to_owned(),
        auditor_version: auditor_version.to_owned(),
        auditor_binary_sha256,
        scenario_count: parsed.len(),
        interval_count,
        scenario_bindings,
        shared_information_set_count,
        checked_action_comparison_count,
        disposition,
        findings,
        receipt_sha256: sha256(&subject_bytes),
        claim_ceiling: claim_ceiling.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENARIO_SET: &str = "nonanticipativity-test-set";
    const OBSERVED_ROOT_AT: i64 = 1_799_999_999;
    const OBSERVED_BRANCH_AT: i64 = 1_800_003_600;
    const FORECAST_AS_OF: i64 = 1_799_996_400;

    fn observation_bundle_bytes(
        scenario_set_id: &str,
        bundle_id: &str,
        observed_at_unix_seconds: i64,
        observation_payload: serde_json::Value,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": "energy-dispatch-observation-bundle-v0.1",
            "scenario_set_id": scenario_set_id,
            "bundle_id": bundle_id,
            "observed_at_unix_seconds": observed_at_unix_seconds,
            "observation_payload": observation_payload
        }))
        .unwrap()
    }

    fn forecast_bundle_bytes(
        scenario_set_id: &str,
        bundle_id: &str,
        first_demand: u16,
        second_demand: u16,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": "energy-dispatch-forecast-bundle-v0.1",
            "scenario_set_id": scenario_set_id,
            "bundle_id": bundle_id,
            "issued_at_unix_seconds": FORECAST_AS_OF,
            "valid_from_unix_seconds": FORECAST_AS_OF,
            "valid_through_unix_seconds": 1_800_007_200_i64,
            "forecast_payload": {
                "scenario_probabilities": [
                    {
                        "scenario_id": "scenario-a",
                        "weight_numerator": 1,
                        "weight_denominator": 2
                    },
                    {
                        "scenario_id": "scenario-b",
                        "weight_numerator": 1,
                        "weight_denominator": 2
                    }
                ],
                "expected_essential_demand_quanta": [first_demand, second_demand]
            }
        }))
        .unwrap()
    }

    fn request_bytes(case_id: &str, first_pv: u16, second_demand: u16) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": "energy-dispatch-v0.1",
            "case_id": case_id,
            "synthetic": true,
            "evidence_class": "analyst_assumption",
            "evidence_ref": null,
            "evidence_as_of": null,
            "energy_quantum_kwh": 1.0,
            "max_candidate_expansions": 1000,
            "battery": {
                "capacity_quanta": 1,
                "initial_soc_quanta": 0,
                "charge_input_limit_quanta": 1,
                "discharge_withdrawal_limit_quanta": 1,
                "charge_efficiency_numerator": 1,
                "charge_efficiency_denominator": 1,
                "discharge_efficiency_numerator": 1,
                "discharge_efficiency_denominator": 1,
                "terminal_reserve_quanta": 0
            },
            "intervals": [
                {
                    "interval_id": "charge-from-pv",
                    "start_unix_seconds": 1_800_000_000_i64,
                    "duration_seconds": 3600_u64,
                    "grid_outage": false,
                    "pv_available_quanta": first_pv,
                    "grid_import_limit_quanta": 0,
                    "essential_demand_quanta": 0,
                    "flexible_demand_quanta": 0,
                    "grid_cost_units_per_quantum": 3,
                    "grid_carbon_grams_per_quantum": 7
                },
                {
                    "interval_id": "serve-during-outage",
                    "start_unix_seconds": 1_800_003_600_i64,
                    "duration_seconds": 3600_u64,
                    "grid_outage": true,
                    "pv_available_quanta": 0,
                    "grid_import_limit_quanta": 0,
                    "essential_demand_quanta": second_demand,
                    "flexible_demand_quanta": 0,
                    "grid_cost_units_per_quantum": 3,
                    "grid_carbon_grams_per_quantum": 7
                }
            ]
        }))
        .unwrap()
    }

    fn schedule_bytes(
        request: &[u8],
        charge_in_first_interval: bool,
        discharge_in_second_interval: bool,
    ) -> Vec<u8> {
        let request_value: serde_json::Value = serde_json::from_slice(request).unwrap();
        let case_id = request_value["case_id"].as_str().unwrap();
        let first_soc = if charge_in_first_interval { 1_u16 } else { 0_u16 };
        let second_discharge = if discharge_in_second_interval { 1_u16 } else { 0_u16 };
        let terminal_soc = first_soc - second_discharge;
        let first_charge = if charge_in_first_interval { 1_u16 } else { 0_u16 };
        let first_pv_used = first_charge;
        let first_pv_available = u16::try_from(
            request_value["intervals"][0]["pv_available_quanta"]
                .as_u64()
                .unwrap(),
        )
        .unwrap();
        let first_pv_curtailed = first_pv_available - first_pv_used;
        let second_served = second_discharge;
        let second_demand = u16::try_from(
            request_value["intervals"][1]["essential_demand_quanta"]
                .as_u64()
                .unwrap(),
        )
        .unwrap();
        let second_unserved = second_demand - second_served;

        serde_json::to_vec(&serde_json::json!({
            "schema_version": "energy-dispatch-schedule-v0.1",
            "case_id": case_id,
            "request_sha256": sha256(request),
            "interval_dispatches": [
                {
                    "interval_id": "charge-from-pv",
                    "grid_import_quanta": 0,
                    "charge_input_quanta": first_charge,
                    "discharge_withdrawal_quanta": 0,
                    "soc_after_quanta": first_soc,
                    "essential_served_quanta": 0,
                    "essential_unserved_quanta": 0,
                    "flexible_served_quanta": 0,
                    "flexible_unserved_quanta": 0,
                    "pv_used_quanta": first_pv_used,
                    "pv_curtailed_quanta": first_pv_curtailed
                },
                {
                    "interval_id": "serve-during-outage",
                    "grid_import_quanta": 0,
                    "charge_input_quanta": 0,
                    "discharge_withdrawal_quanta": second_discharge,
                    "soc_after_quanta": terminal_soc,
                    "essential_served_quanta": second_served,
                    "essential_unserved_quanta": second_unserved,
                    "flexible_served_quanta": 0,
                    "flexible_unserved_quanta": 0,
                    "pv_used_quanta": 0,
                    "pv_curtailed_quanta": 0
                }
            ],
            "terminal_soc_quanta": terminal_soc
        }))
        .unwrap()
    }

    fn information_sets<'a>(
        observations: [&'a [u8]; 2],
        forecasts: [&'a [u8]; 2],
    ) -> Vec<NonAnticipativityInformationSetBytes<'a>> {
        [
            NonAnticipativityInformationSetBytes {
                observed_at_unix_seconds: OBSERVED_ROOT_AT,
                observation_bundle_bytes: observations[0],
                forecast_as_of_unix_seconds: FORECAST_AS_OF,
                forecast_bundle_bytes: forecasts[0],
            },
            NonAnticipativityInformationSetBytes {
                observed_at_unix_seconds: OBSERVED_BRANCH_AT,
                observation_bundle_bytes: observations[1],
                forecast_as_of_unix_seconds: FORECAST_AS_OF,
                forecast_bundle_bytes: forecasts[1],
            },
        ]
        .to_vec()
    }

    fn root_observation() -> Vec<u8> {
        observation_bundle_bytes(
            SCENARIO_SET,
            "root-observation-v1",
            OBSERVED_ROOT_AT,
            serde_json::json!({
                "grid_connected": true,
                "pv_available_quanta": 1,
                "battery_soc_quanta": 0
            }),
        )
    }

    fn branch_observation(scenario: &str, essential_demand: u16) -> Vec<u8> {
        observation_bundle_bytes(
            SCENARIO_SET,
            &format!("{scenario}-branch-observation-v1"),
            OBSERVED_BRANCH_AT,
            serde_json::json!({
                "grid_connected": false,
                "essential_demand_quanta": essential_demand
            }),
        )
    }

    fn scenario<'a>(
        scenario_id: &'a str,
        request: &'a [u8],
        schedule: &'a [u8],
        information_sets: &'a [NonAnticipativityInformationSetBytes<'a>],
    ) -> NonAnticipativityScenarioBytes<'a> {
        NonAnticipativityScenarioBytes {
            scenario_id,
            request_bytes: request,
            schedule_bytes: schedule,
            information_sets_by_interval: information_sets,
        }
    }

    #[test]
    fn shared_pre_branch_actions_must_be_identical() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, false, false);
        let root = root_observation();
        let branch_a = branch_observation("scenario-a", 1);
        let branch_b = branch_observation("scenario-b", 0);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-root-v1", 1, 0);
        let info_a = information_sets([&root, &branch_a], [&forecast, &forecast]);
        let info_b = information_sets([&root, &branch_b], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "nonanticipative-dispatch-action"
                && finding.interval_index == Some(0)
                && !finding.passed
        }));
    }

    #[test]
    fn actions_may_diverge_after_observed_history_splits() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let root = root_observation();
        let branch_a = branch_observation("scenario-a", 1);
        let branch_b = branch_observation("scenario-b", 0);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-root-v1", 1, 0);
        let info_a = information_sets([&root, &branch_a], [&forecast, &forecast]);
        let info_b = information_sets([&root, &branch_b], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Accepted);
        assert_eq!(receipt.shared_information_set_count, 1);
        assert_eq!(receipt.checked_action_comparison_count, 1);
        assert!(receipt.findings.iter().all(|finding| finding.passed));
        assert_eq!(
            receipt.scenario_bindings[0].information_set_sha256_by_interval[0],
            receipt.scenario_bindings[1].information_set_sha256_by_interval[0]
        );
        assert_ne!(
            receipt.scenario_bindings[0].information_set_sha256_by_interval[1],
            receipt.scenario_bindings[1].information_set_sha256_by_interval[1]
        );
    }

    #[test]
    fn future_request_trajectory_does_not_leak_into_earlier_information_sets() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let root = root_observation();
        let branch_a = branch_observation("scenario-a", 1);
        let branch_b = branch_observation("scenario-b", 0);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-root-v1", 1, 0);
        let info_a = information_sets([&root, &branch_a], [&forecast, &forecast]);
        let info_b = information_sets([&root, &branch_b], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(
            receipt.scenario_bindings[0].information_set_sha256_by_interval[0],
            receipt.scenario_bindings[1].information_set_sha256_by_interval[0]
        );
        assert_ne!(
            receipt.scenario_bindings[0].request_sha256,
            receipt.scenario_bindings[1].request_sha256
        );
    }

    #[test]
    fn information_set_digests_bind_exact_observation_and_forecast_bytes() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let root = root_observation();
        let branch_a = branch_observation("scenario-a", 1);
        let branch_b = branch_observation("scenario-b", 0);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-root-v1", 1, 0);
        let info_a = information_sets([&root, &branch_a], [&forecast, &forecast]);
        let info_b = information_sets([&root, &branch_b], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        let binding_a = &receipt.scenario_bindings[0];
        let binding_b = &receipt.scenario_bindings[1];
        assert_eq!(
            binding_a.information_sets_by_interval[0].observation_bundle_sha256,
            sha256(&root)
        );
        assert_eq!(
            binding_a.information_sets_by_interval[0].forecast_bundle_sha256,
            sha256(&forecast)
        );
        assert_eq!(
            binding_a.information_set_sha256_by_interval[0],
            binding_b.information_set_sha256_by_interval[0]
        );
        assert_ne!(
            binding_a.information_set_sha256_by_interval[1],
            binding_b.information_set_sha256_by_interval[1]
        );
    }

    #[test]
    fn identical_observation_histories_cannot_split_on_different_forecast_artifacts() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 1);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, false, false);
        let root = root_observation();
        let branch = branch_observation("shared", 1);
        let forecast_a = forecast_bundle_bytes(SCENARIO_SET, "forecast-a", 1, 1);
        let forecast_b = forecast_bundle_bytes(SCENARIO_SET, "forecast-b", 1, 1);
        let info_a = information_sets([&root, &branch], [&forecast_a, &forecast_a]);
        let info_b = information_sets([&root, &branch], [&forecast_b, &forecast_b]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "shared-observation-forecast-bundle-consistency"
                && finding.interval_index == Some(0)
                && !finding.passed
        }));
    }

    #[test]
    fn future_observation_and_future_forecast_artifacts_are_rejected() {
        let request = request_bytes("scenario-a", 1, 1);
        let schedule = schedule_bytes(&request, true, true);
        let future_observation = observation_bundle_bytes(
            SCENARIO_SET,
            "future-observation",
            1_800_000_001,
            serde_json::json!({"pv_available_quanta": 1}),
        );
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-v1", 1, 0);
        let future_forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-v2", 1, 0);
        let mut future_value: serde_json::Value =
            serde_json::from_slice(&future_forecast).unwrap();
        future_value["issued_at_unix_seconds"] = serde_json::json!(1_800_000_001_i64);
        let future_forecast = serde_json::to_vec(&future_value).unwrap();

        let information = [
            NonAnticipativityInformationSetBytes {
                observed_at_unix_seconds: 1_800_000_001,
                observation_bundle_bytes: &future_observation,
                forecast_as_of_unix_seconds: FORECAST_AS_OF,
                forecast_bundle_bytes: &forecast,
            },
            NonAnticipativityInformationSetBytes {
                observed_at_unix_seconds: OBSERVED_BRANCH_AT,
                observation_bundle_bytes: &future_observation,
                forecast_as_of_unix_seconds: 1_800_000_001,
                forecast_bundle_bytes: &future_forecast,
            },
        ];
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let root = root_observation();
        let branch = branch_observation("scenario-b", 0);
        let valid_information = information_sets([&root, &branch], [&forecast, &forecast]);

        assert!(
            evaluate_nonanticipativity_bytes(
                SCENARIO_SET,
                &[
                    scenario("scenario-a", &request, &schedule, &information),
                    scenario("scenario-b", &request_b, &schedule_b, &valid_information),
                ],
            )
            .is_err()
        );
    }

    #[test]
    fn observation_and_forecast_bundles_for_another_scenario_set_are_rejected() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let wrong_observation = observation_bundle_bytes(
            "another-scenario-set",
            "wrong-observation",
            OBSERVED_ROOT_AT,
            serde_json::json!({"pv_available_quanta": 1}),
        );
        let wrong_forecast =
            forecast_bundle_bytes("another-scenario-set", "wrong-forecast", 1, 0);
        let information = [
            NonAnticipativityInformationSetBytes {
                observed_at_unix_seconds: OBSERVED_ROOT_AT,
                observation_bundle_bytes: &wrong_observation,
                forecast_as_of_unix_seconds: FORECAST_AS_OF,
                forecast_bundle_bytes: &wrong_forecast,
            },
            NonAnticipativityInformationSetBytes {
                observed_at_unix_seconds: OBSERVED_BRANCH_AT,
                observation_bundle_bytes: &wrong_observation,
                forecast_as_of_unix_seconds: FORECAST_AS_OF,
                forecast_bundle_bytes: &wrong_forecast,
            },
        ];

        assert!(
            evaluate_nonanticipativity_bytes(
                SCENARIO_SET,
                &[
                    scenario("scenario-a", &request_a, &schedule_a, &information),
                    scenario("scenario-b", &request_b, &schedule_b, &information),
                ],
            )
            .is_err()
        );
    }

    #[test]
    fn scenarios_must_share_one_root_observation_history_before_branching() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 0, 1);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, false, false);
        let root_a = root_observation();
        let root_b = observation_bundle_bytes(
            SCENARIO_SET,
            "different-root-observation",
            OBSERVED_ROOT_AT,
            serde_json::json!({
                "grid_connected": true,
                "pv_available_quanta": 0,
                "battery_soc_quanta": 0
            }),
        );
        let branch = branch_observation("branch", 1);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-v1", 1, 0);
        let info_a = information_sets([&root_a, &branch], [&forecast, &forecast]);
        let info_b = information_sets([&root_b, &branch], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "root-information-set-coverage" && !finding.passed
        }));
    }

    #[test]
    fn no_shared_information_set_is_not_a_vacuous_pass() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 0, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, false, false);
        let root_a = root_observation();
        let root_b = observation_bundle_bytes(
            SCENARIO_SET,
            "different-root",
            OBSERVED_ROOT_AT,
            serde_json::json!({"grid_connected": false}),
        );
        let branch_a = branch_observation("scenario-a", 1);
        let branch_b = branch_observation("scenario-b", 0);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-v1", 1, 0);
        let info_a = information_sets([&root_a, &branch_a], [&forecast, &forecast]);
        let info_b = information_sets([&root_b, &branch_b], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "root-information-set-coverage" && !finding.passed
        }));
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "shared-information-set-coverage" && !finding.passed
        }));
    }

    #[test]
    fn exact_receipt_digest_binds_request_schedule_observation_forecast_and_history() {
        let request_a = request_bytes("scenario-a", 1, 1);
        let request_b = request_bytes("scenario-b", 1, 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let root = root_observation();
        let branch_a = branch_observation("scenario-a", 1);
        let branch_b = branch_observation("scenario-b", 0);
        let forecast = forecast_bundle_bytes(SCENARIO_SET, "forecast-v1", 1, 0);
        let info_a = information_sets([&root, &branch_a], [&forecast, &forecast]);
        let info_b = information_sets([&root, &branch_b], [&forecast, &forecast]);

        let receipt = evaluate_nonanticipativity_bytes(
            SCENARIO_SET,
            &[
                scenario("scenario-a", &request_a, &schedule_a, &info_a),
                scenario("scenario-b", &request_b, &schedule_b, &info_b),
            ],
        )
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Accepted);
        assert!(super::super::is_sha256_hex(&receipt.auditor_binary_sha256));
        assert!(super::super::is_sha256_hex(&receipt.receipt_sha256));
        assert_eq!(receipt.scenario_set_id, SCENARIO_SET);
        assert_eq!(receipt.scenario_count, 2);
        assert_eq!(receipt.interval_count, 2);
        assert_eq!(
            receipt.scenario_bindings[0].information_sets_by_interval[0]
                .observation_bundle_sha256,
            sha256(&root)
        );
        assert_eq!(
            receipt.scenario_bindings[0].information_sets_by_interval[0]
                .forecast_bundle_sha256,
            sha256(&forecast)
        );
    }
}
