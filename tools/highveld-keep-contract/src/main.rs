use highveld_keep_contract::validate_tracked_documents;
use std::process::ExitCode;

fn main() -> ExitCode {
    match validate_tracked_documents() {
        Ok(report) if report.is_valid() => {
            println!("suite=synthetic_definition_contract_e1_only");
            println!("scenario_id={}", report.scenario_id);
            println!("alternative_count={}", report.alternatives);
            println!("weather_candidate_count={}", report.weather_candidates);
            println!("continuous_parameter_count={}", report.continuous_variables);
            println!(
                "categorical_parameter_count={}",
                report.categorical_variables
            );
            println!(
                "functional_relationship_count={}",
                report.functional_relationships
            );
            println!(
                "alternative_climate_method_count={}",
                report.alternative_climate_methods
            );
            println!(
                "station_metadata_crosscheck_count={}",
                report.station_metadata_crosschecks
            );
            println!("violation_count=0");
            println!("result=definition_valid_synthetic_only");
            println!("claim_ceiling=no_geometry_buildability_safety_or_compliance_validation");
            ExitCode::SUCCESS
        }
        Ok(report) => {
            eprintln!("suite=synthetic_definition_contract_e1_only");
            eprintln!("scenario_id={}", report.scenario_id);
            eprintln!("violation_count={}", report.violations.len());
            for violation in report.violations {
                eprintln!("violation={violation}");
            }
            eprintln!("result=invalid");
            eprintln!("claim_ceiling=no_geometry_buildability_safety_or_compliance_validation");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("result=invalid");
            eprintln!("violation=SOURCE_JSON_PARSE_FAILED");
            eprintln!("detail={error}");
            ExitCode::FAILURE
        }
    }
}
