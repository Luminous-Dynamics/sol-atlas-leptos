use building_semantic_interop_qualifier::validate_tracked;
use std::process::ExitCode;

fn main() -> ExitCode {
    match validate_tracked() {
        Ok(count) => {
            println!("suite=city_building_semantic_interop_v0_1");
            println!("cases={count}");
            println!("result=all_known_answers_matched");
            println!("claim_ceiling=synthetic_mapping_admission_and_semantic_loss_checks_only");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("result=invalid");
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
