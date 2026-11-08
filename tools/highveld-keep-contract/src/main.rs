use highveld_keep_contract::{
    validate_tracked_documents, validate_tracked_resilience_binding,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    let design = validate_tracked_documents();
    let resilience = validate_tracked_resilience_binding();

    match (design, resilience) {
        (Ok(report), Ok(binding)) if report.is_valid() && binding.is_valid() => {
            println!("suite=synthetic_definition_and_resilience_binding_e1_only");
            println!("scenario_id={}", report.scenario_id);
            println!("alternative_count={}", report.alternatives);
            println!("weather_candidate_count={}", report.weather_candidates);
            println!("continuous_parameter_count={}", report.continuous_variables);
            println!("categorical_parameter_count={}", report.categorical_variables);
            println!("functional_relationship_count={}", report.functional_relationships);
            println!("alternative_climate_method_count={}", report.alternative_climate_methods);
            println!("station_metadata_crosscheck_count={}", report.station_metadata_crosschecks);
            println!("station_network_profile_count={}", report.station_network_profiles);
            println!("resilience_binding_id={}", binding.binding_id);
            println!("resilience_scenario_family_count={}", binding.scenario_families);
            println!("resilience_weather_candidate_count={}", binding.weather_candidates);
            println!("resilience_service_domain_count={}", binding.service_domains);
            println!("resilience_owner_binding_count={}", binding.owner_bindings);
            println!("resilience_invariant_mapping_count={}", binding.invariant_mappings);
            println!("resilience_vvuq_stage_count={}", binding.vvuq_stages);
            println!("resilience_measurement_question_count={}", binding.measurement_questions);
            println!("violation_count=0");
            println!("result=definition_and_resilience_binding_valid_synthetic_only");
            println!("claim_ceiling=no_geometry_buildability_safety_or_compliance_validation");
            ExitCode::SUCCESS
        }
        (Ok(report), Ok(binding)) => {
            eprintln!("suite=synthetic_definition_and_resilience_binding_e1_only");
            eprintln!("scenario_id={}", report.scenario_id);
            eprintln!("design_violation_count={}", report.violations.len());
            for violation in report.violations {
                eprintln!("design_violation={violation}");
            }
            eprintln!("resilience_binding_id={}", binding.binding_id);
            eprintln!("resilience_violation_count={}", binding.violations.len());
            for violation in binding.violations {
                eprintln!("resilience_violation={violation}");
            }
            eprintln!("result=invalid");
            eprintln!("claim_ceiling=no_geometry_buildability_safety_or_compliance_validation");
            ExitCode::FAILURE
        }
        (Err(error), _) => {
            eprintln!("result=invalid");
            eprintln!("violation=SOURCE_JSON_PARSE_FAILED");
            eprintln!("detail={error}");
            ExitCode::FAILURE
        }
        (_, Err(error)) => {
            eprintln!("result=invalid");
            eprintln!("violation=RESILIENCE_BINDING_SOURCE_JSON_PARSE_FAILED");
            eprintln!("detail={error}");
            ExitCode::FAILURE
        }
    }
}
