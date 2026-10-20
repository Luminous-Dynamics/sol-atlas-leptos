//! Evaluate a compute-energy scenario JSON file with a scheduling baseline.
//!
//! Usage:
//!   cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- <scenario.json> [earliest-ready|lowest-cost|lowest-carbon|pareto]
//!
//! Output is a JSON schedule assessment on stdout. Exit codes:
//!   0 = feasible or capacity-limited assessment completed
//!   2 = protected floors are infeasible
//!   3 = required resource/objective inputs are unknown (indeterminate)
//!   4 = request is outside the evaluator's supported scope
//!   5 = request is invalid
//!   64 = command-line usage error
//!   65 = scenario JSON could not be parsed
//!   66 = scenario file could not be read
//!   70 = assessment could not be serialized

use sol_atlas_core::ai_compute_energy::{
    AssessmentStatus, Request, SchedulingPolicy, compare_scheduling_policies, scenario_receipt,
    schedule_deferrable,
};
use std::env;
use std::fs;
use std::process;

fn parse_policy(value: &str) -> Option<SchedulingPolicy> {
    match value {
        "earliest-ready" => Some(SchedulingPolicy::EarliestReady),
        "lowest-cost" => Some(SchedulingPolicy::LowestCost),
        "lowest-carbon" => Some(SchedulingPolicy::LowestCarbon),
        _ => None,
    }
}

fn policy_name(policy: SchedulingPolicy) -> &'static str {
    match policy {
        SchedulingPolicy::EarliestReady => "earliest-ready",
        SchedulingPolicy::LowestCost => "lowest-cost",
        SchedulingPolicy::LowestCarbon => "lowest-carbon",
    }
}

fn exit_code(status: AssessmentStatus) -> i32 {
    match status {
        AssessmentStatus::Feasible | AssessmentStatus::CapacityLimited => 0,
        AssessmentStatus::Infeasible => 2,
        AssessmentStatus::Indeterminate => 3,
        AssessmentStatus::Unsupported => 4,
        AssessmentStatus::Invalid => 5,
    }
}

fn build_run_output(
    raw_input: &[u8],
    request: &Request,
    mode: &str,
    assessment: serde_json::Value,
) -> serde_json::Value {
    let receipt = scenario_receipt(raw_input, request, mode);
    serde_json::json!({
        "receipt": receipt,
        "assessment": assessment,
    })
}

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!(
            "usage: ai_compute_energy_assess <scenario.json> [earliest-ready|lowest-cost|lowest-carbon|pareto]"
        );
        process::exit(64);
    };

    let (pareto_mode, policy) = match args.next() {
        None => (false, SchedulingPolicy::EarliestReady),
        Some(raw) => match raw.to_str() {
            Some("pareto") => (true, SchedulingPolicy::EarliestReady),
            Some(value) => match parse_policy(value) {
                Some(policy) => (false, policy),
                None => {
                    eprintln!(
                        "policy must be earliest-ready, lowest-cost, lowest-carbon, or pareto"
                    );
                    process::exit(64);
                }
            },
            None => {
                eprintln!("policy argument must be valid UTF-8");
                process::exit(64);
            }
        },
    };
    if args.next().is_some() {
        eprintln!(
            "usage: ai_compute_energy_assess <scenario.json> [earliest-ready|lowest-cost|lowest-carbon|pareto]"
        );
        process::exit(64);
    }

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("could not read scenario file: {error}");
            process::exit(66);
        }
    };

    let request: Request = match serde_json::from_str(&content) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("could not parse compute-energy scenario JSON: {error}");
            process::exit(65);
        }
    };

    let (assessment_value, status_code) = if pareto_mode {
        let result = compare_scheduling_policies(&request);
        (serde_json::to_value(&result), exit_code(result.status))
    } else {
        let result = schedule_deferrable(&request, policy);
        (serde_json::to_value(&result), exit_code(result.status))
    };
    let assessment = match assessment_value {
        Ok(value) => value,
        Err(error) => {
            eprintln!("could not serialize assessment: {error}");
            process::exit(70);
        }
    };

    let mode = if pareto_mode {
        "pareto"
    } else {
        policy_name(policy)
    };
    let output = build_run_output(content.as_bytes(), &request, mode, assessment);

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("could not serialize run receipt: {error}");
            process::exit(70);
        }
    }

    if status_code != 0 {
        process::exit(status_code);
    }
}

#[cfg(test)]
mod tests {
    use super::build_run_output;
    use sol_atlas_core::ai_compute_energy::{Request, scenario_receipt};

    fn fixture_request_and_bytes() -> (&'static str, Request) {
        let fixture = include_str!(
            "../../../schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json"
        );
        let request: Request =
            serde_json::from_str(fixture).expect("committed fixture should deserialize");
        (fixture, request)
    }

    fn policy_assessment(policy: &str) -> serde_json::Value {
        serde_json::json!({
            "status": "feasible",
            "policy": policy,
            "protected_floor_work_units_by_interval": [6, 6, 6, 6],
            "deferrable_work_units_by_interval": [12, 0, 0, 28],
            "scheduled_work_units": 40,
            "deferrable_it_energy_kwh": 20.0,
            "deferrable_facility_energy_kwh": 23.0,
            "modelled_electricity_cost_usd": 1.978,
            "modelled_grid_carbon_gco2": 12305.0,
            "blockers": []
        })
    }

    #[test]
    fn cli_output_binds_exact_input_to_separate_assessment() {
        let (fixture, request) = fixture_request_and_bytes();
        let assessment = serde_json::json!({
            "status": "feasible",
            "scheduled_work_units": 40
        });
        let output = build_run_output(fixture.as_bytes(), &request, "pareto", assessment.clone());

        assert_eq!(output["assessment"], assessment);
        assert_eq!(output["receipt"]["mode"], "pareto");
        assert_eq!(
            output["receipt"]["input_sha256"],
            scenario_receipt(fixture.as_bytes(), &request, "pareto").input_sha256
        );
        assert_eq!(
            output["receipt"]["fixture_id"],
            "synthetic-four-interval-flexible-compute"
        );
        assert_eq!(output["receipt"]["synthetic"], true);
        assert!(output["receipt"]["verification_boundary"]
            .as_str()
            .expect("verification boundary is text")
            .contains("does not validate"));
    }

    #[test]
    fn report_schema_declares_every_receipt_and_frontier_field() {
        let (fixture, request) = fixture_request_and_bytes();
        let assessment = serde_json::json!({
            "status": "feasible",
            "policy_assessments": [
                policy_assessment("lowest-cost"),
                policy_assessment("lowest-carbon")
            ],
            "non_dominated_policies": ["lowest-cost", "lowest-carbon"],
            "blockers": []
        });
        let output = build_run_output(fixture.as_bytes(), &request, "pareto", assessment);
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/ai-compute-energy-run-report-v1.schema.json"
        ))
        .expect("run report schema must be valid JSON");

        assert_eq!(schema["properties"]["receipt"]["$ref"], "#/$defs/receipt");
        assert_eq!(schema["properties"]["assessment"]["oneOf"][1]["$ref"], "#/$defs/frontierAssessment");
        for key in output.as_object().expect("report envelope is object").keys() {
            assert!(schema["properties"].get(key).is_some(), "root field {key} must be schema-declared");
        }
        for key in output["receipt"].as_object().expect("receipt is object").keys() {
            assert!(
                schema["$defs"]["receipt"]["properties"].get(key).is_some(),
                "receipt field {key} must be schema-declared"
            );
        }
        for key in output["assessment"]
            .as_object()
            .expect("frontier assessment is object")
            .keys()
        {
            assert!(
                schema["$defs"]["frontierAssessment"]["properties"]
                    .get(key)
                    .is_some(),
                "frontier field {key} must be schema-declared"
            );
        }
        for candidate in output["assessment"]["policy_assessments"]
            .as_array()
            .expect("policy list is array")
        {
            for key in candidate.as_object().expect("policy assessment is object").keys() {
                assert!(
                    schema["$defs"]["policyAssessment"]["properties"]
                        .get(key)
                        .is_some(),
                    "policy assessment field {key} must be schema-declared"
                );
            }
        }

        fn assert_local_refs_resolve(value: &serde_json::Value, root: &serde_json::Value) {
            match value {
                serde_json::Value::Object(object) => {
                    if let Some(reference) = object.get("$ref").and_then(|item| item.as_str()) {
                        let mut target = root;
                        for path_segment in reference
                            .strip_prefix("#/")
                            .expect("schema references must be local")
                            .split('/')
                        {
                            target = target
                                .get(path_segment)
                                .unwrap_or_else(|| panic!("unresolved schema reference: {reference}"));
                        }
                        assert!(!target.is_null(), "schema reference must resolve: {reference}");
                    }
                    for child in object.values() {
                        assert_local_refs_resolve(child, root);
                    }
                }
                serde_json::Value::Array(items) => {
                    for item in items {
                        assert_local_refs_resolve(item, root);
                    }
                }
                _ => {}
            }
        }
        assert_local_refs_resolve(&schema, &schema);
    }
}
