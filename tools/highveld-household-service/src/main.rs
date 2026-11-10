use highveld_household_service::validate_tracked_package;
use std::process::ExitCode;

fn main() -> ExitCode {
    match validate_tracked_package() {
        Ok(report) if report.is_valid() => {
            println!("suite=highveld_household_service_e1_synthetic_only");
            println!("package_id={}", report.package_id);
            println!("service_count={}", report.service_count);
            println!("architecture_count={}", report.architecture_count);
            println!("stress_case_count={}", report.stress_case_count);
            println!("violation_count=0");
            println!("result=package_valid_synthetic_only");
            println!("claim_ceiling=no_physical_product_building_safety_or_performance_validation");
            ExitCode::SUCCESS
        }
        Ok(report) => {
            eprintln!("suite=highveld_household_service_e1_synthetic_only");
            for violation in &report.violations {
                eprintln!("violation={violation}");
            }
            eprintln!("result=invalid");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("result=invalid");
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
