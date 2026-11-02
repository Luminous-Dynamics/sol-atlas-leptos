//! Independent audit of declared non-anticipativity across scenario schedules.
//!
//! The evaluator checks that scenarios carrying the same caller-declared
//! information-set digest have identical dispatch actions at that interval.
//! It also checks that those scenarios carry the same source-observation
//! prefix through that interval. It cannot establish that the labels faithfully
//! represent what an operator/controller could actually observe.

use super::{
    audit_error, current_binary_sha256, parse_unique_json, recompute_objectives, sha256,
    validate_request, validate_schedule, AuditError, AuditFinding, DispatchSchedule,
    IntervalDispatch, SourceRequest, MAX_INTERVALS, MAX_REQUEST_BYTES, MAX_SCHEDULE_BYTES,
    SCHEDULE_SCHEMA_VERSION, SOURCE_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const NONANTICIPATIVITY_AUDITOR_ID: &str = "energy-dispatch-nonanticipativity-auditor";
pub const NONANTICIPATIVITY_RECEIPT_SCHEMA_V1: &str =
    "energy-dispatch-nonanticipativity-receipt-v0.1";
pub const MAX_SCENARIOS: usize = 256;

#[derive(Debug, Clone, Copy)]
pub struct NonAnticipativityScenarioBytes<'a> {
    pub scenario_id: &'a str,
    pub request_bytes: &'a [u8],
    pub schedule_bytes: &'a [u8],
    /// One SHA-256 information-set label per interval. Equal labels assert
    /// that the corresponding scenario histories are indistinguishable to
    /// the decision-maker at that decision time.
    pub information_set_sha256_by_interval: &'a [String],
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
    pub information_set_sha256_by_interval: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonAnticipativityFindingV1 {
    pub predicate: String,
    pub interval_index: Option<usize>,
    pub information_set_sha256: Option<String>,
    pub observation_prefix_sha256: Option<String>,
    pub scenario_ids: Vec<String>,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonAnticipativityReceiptV1 {
    pub receipt_schema: String,
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

struct ParsedScenario {
    scenario_id: String,
    request_sha256: String,
    schedule_sha256: String,
    request: SourceRequest,
    schedule: DispatchSchedule,
    information_set_sha256_by_interval: Vec<String>,
    domain_valid: bool,
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
    observation_prefix_sha256: Option<String>,
    scenario_ids: Vec<String>,
    passed: bool,
    detail: impl Into<String>,
) {
    findings.push(NonAnticipativityFindingV1 {
        predicate: predicate.to_owned(),
        interval_index,
        information_set_sha256,
        observation_prefix_sha256,
        scenario_ids,
        passed,
        detail: detail.into(),
    });
}

/// Evaluate a set of source-bound complete-horizon schedules against declared
/// information-set labels.
///
/// For every interval, scenarios that share an information-set digest must:
/// 1. have identical source-observation prefixes through that interval; and
/// 2. take identical dispatch actions at that interval.
///
/// Distinct information-set labels may take different actions. A missing
/// shared information set is rejected rather than treated as vacuous success.
/// The evaluator does not prove that caller-supplied information-set labels
/// correspond to authentic, complete or decision-time-available observations.
pub fn evaluate_nonanticipativity_bytes(
    scenarios: &[NonAnticipativityScenarioBytes<'_>],
) -> Result<NonAnticipativityReceiptV1, AuditError> {
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
            || source.information_set_sha256_by_interval.len() != request.intervals.len()
            || schedule.interval_dispatches.len() != request.intervals.len()
        {
            return Err(audit_error(
                "each scenario must supply one information-set digest and one schedule step per source interval",
            ));
        }
        if source
            .information_set_sha256_by_interval
            .iter()
            .any(|digest| !super::is_sha256_hex(digest))
        {
            return Err(audit_error(
                "information-set labels must be 64 lowercase hexadecimal characters",
            ));
        }

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
        let domain_valid = domain_findings.iter().all(|finding| finding.passed);

        parsed.push(ParsedScenario {
            scenario_id: source.scenario_id.to_owned(),
            request_sha256,
            schedule_sha256,
            request,
            schedule,
            information_set_sha256_by_interval: source
                .information_set_sha256_by_interval
                .to_vec(),
            domain_valid,
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
            "scenario requests must share the same schema, asset configuration, energy quantum and interval chronology",
        ));
    }

    let interval_count = reference.request.intervals.len();
    let scenario_bindings: Vec<_> = parsed
        .iter()
        .map(|scenario| NonAnticipativityScenarioBindingV1 {
            scenario_id: scenario.scenario_id.clone(),
            request_sha256: scenario.request_sha256.clone(),
            schedule_sha256: scenario.schedule_sha256.clone(),
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
            "the scenario request and complete schedule must satisfy the standalone domain predicates",
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
        let observation_prefix = &first.request.intervals[..=interval_index];
        let observation_prefix_bytes = serde_json::to_vec(observation_prefix)
            .map_err(|error| audit_error(format!("serialize observation prefix: {error}")))?;
        let observation_prefix_sha256 = sha256(&observation_prefix_bytes);
        let observation_prefix_equal = members.iter().skip(1).all(|member| {
            parsed[*member].request.intervals[..=interval_index] == *observation_prefix
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
            Some(observation_prefix_sha256),
            scenario_ids.clone(),
            observation_prefix_equal,
            if observation_prefix_equal {
                "scenarios sharing this information set have identical declared exogenous observation prefixes"
            } else {
                "scenarios share an information-set label but their declared observation prefixes differ"
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
            "no interval contains a shared information set; non-anticipativity would otherwise pass vacuously",
        );
    }

    let disposition = if findings.iter().all(|finding| finding.passed) {
        NonAnticipativityDispositionV1::Accepted
    } else {
        NonAnticipativityDispositionV1::Rejected
    };
    let receipt_schema = NONANTICIPATIVITY_RECEIPT_SCHEMA_V1;
    let auditor_id = NONANTICIPATIVITY_AUDITOR_ID;
    let auditor_version = env!("CARGO_PKG_VERSION");
    let auditor_binary_sha256 = current_binary_sha256()?;
    let claim_ceiling = concat!(
        "equality of exact dispatch action fields for scenarios sharing caller-declared ",
        "information-set SHA-256 labels, plus equality of their declared source-observation ",
        "prefixes; does not authenticate information-set labels or prove observation ",
        "availability, forecast calibration, stochastic optimality, service-floor compliance, ",
        "physical safety or actuation authority"
    );
    let subject = NonAnticipativityReceiptSubjectV1 {
        receipt_schema,
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
        .map_err(|error| audit_error(format!("serialize non-anticipativity receipt subject: {error}")))?;

    Ok(NonAnticipativityReceiptV1 {
        receipt_schema: receipt_schema.to_owned(),
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

    fn digest(byte: u8) -> String {
        format!("{byte:02x}{}", "00".repeat(31))
    }

    fn request_bytes(case_id: &str, first_pv: u16) -> Vec<u8> {
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
                    "essential_demand_quanta": 1,
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
        let first_soc = u16::from(charge_in_first_interval);
        let terminal_soc = first_soc - u16::from(discharge_in_second_interval);
        let first_charge = u16::from(charge_in_first_interval);
        let first_pv_used = first_charge;
        let first_pv_curtailed = 1 - first_pv_used;
        let second_discharge = u16::from(discharge_in_second_interval);
        let second_served = second_discharge;
        let second_unserved = 1 - second_served;

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

    fn scenario<'a>(
        scenario_id: &'a str,
        request: &'a [u8],
        schedule: &'a [u8],
        labels: &'a [String],
    ) -> NonAnticipativityScenarioBytes<'a> {
        NonAnticipativityScenarioBytes {
            scenario_id,
            request_bytes: request,
            schedule_bytes: schedule,
            information_set_sha256_by_interval: labels,
        }
    }

    #[test]
    fn shared_pre_branch_actions_must_be_identical() {
        let request_a = request_bytes("scenario-a", 1);
        let request_b = request_bytes("scenario-b", 1);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, false, false);
        let labels = [digest(1), digest(2)];

        let receipt = evaluate_nonanticipativity_bytes(&[
            scenario("scenario-a", &request_a, &schedule_a, &labels),
            scenario("scenario-b", &request_b, &schedule_b, &labels),
        ])
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "nonanticipative-dispatch-action"
                && finding.interval_index == Some(0)
                && !finding.passed
        }));
    }

    #[test]
    fn actions_may_diverge_after_information_set_splits() {
        let request_a = request_bytes("scenario-a", 1);
        let request_b = request_bytes("scenario-b", 1);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let labels_a = [digest(1), digest(2)];
        let labels_b = [digest(1), digest(3)];

        let receipt = evaluate_nonanticipativity_bytes(&[
            scenario("scenario-a", &request_a, &schedule_a, &labels_a),
            scenario("scenario-b", &request_b, &schedule_b, &labels_b),
        ])
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Accepted);
        assert_eq!(receipt.shared_information_set_count, 1);
        assert_eq!(receipt.checked_action_comparison_count, 1);
        assert!(receipt.findings.iter().all(|finding| finding.passed));
    }

    #[test]
    fn same_label_with_different_declared_observation_prefix_is_rejected() {
        let request_a = request_bytes("scenario-a", 1);
        let request_b = request_bytes("scenario-b", 0);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, false, false);
        let labels = [digest(1), digest(2)];

        let receipt = evaluate_nonanticipativity_bytes(&[
            scenario("scenario-a", &request_a, &schedule_a, &labels),
            scenario("scenario-b", &request_b, &schedule_b, &labels),
        ])
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "shared-information-set-observation-prefix"
                && finding.interval_index == Some(0)
                && !finding.passed
        }));
    }

    #[test]
    fn no_shared_information_set_is_not_a_vacuous_pass() {
        let request_a = request_bytes("scenario-a", 1);
        let request_b = request_bytes("scenario-b", 1);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let labels_a = [digest(1), digest(2)];
        let labels_b = [digest(3), digest(4)];

        let receipt = evaluate_nonanticipativity_bytes(&[
            scenario("scenario-a", &request_a, &schedule_a, &labels_a),
            scenario("scenario-b", &request_b, &schedule_b, &labels_b),
        ])
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Rejected);
        assert!(receipt.findings.iter().any(|finding| {
            finding.predicate == "shared-information-set-coverage" && !finding.passed
        }));
    }

    #[test]
    fn exact_receipt_digest_binds_scenario_schedule_and_information_labels() {
        let request_a = request_bytes("scenario-a", 1);
        let request_b = request_bytes("scenario-b", 1);
        let schedule_a = schedule_bytes(&request_a, true, true);
        let schedule_b = schedule_bytes(&request_b, true, false);
        let labels_a = [digest(1), digest(2)];
        let labels_b = [digest(1), digest(3)];

        let receipt = evaluate_nonanticipativity_bytes(&[
            scenario("scenario-a", &request_a, &schedule_a, &labels_a),
            scenario("scenario-b", &request_b, &schedule_b, &labels_b),
        ])
        .unwrap();

        assert_eq!(receipt.disposition, NonAnticipativityDispositionV1::Accepted);
        assert!(super::super::is_sha256_hex(&receipt.auditor_binary_sha256));
        assert!(super::super::is_sha256_hex(&receipt.receipt_sha256));
        assert_eq!(receipt.scenario_count, 2);
        assert_eq!(receipt.interval_count, 2);
    }
}
