use highveld_service_outcome_binding::validate_tracked;
use std::process::ExitCode;

fn main()->ExitCode{
 match validate_tracked(){
  Ok(count)=>{println!("suite=highveld_service_outcome_binding_v0_1");println!("cases={count}");println!("result=all_known_answers_matched");println!("claim_ceiling=synthetic_owner_contract_crosswalk_only");ExitCode::SUCCESS}
  Err(error)=>{eprintln!("result=invalid");eprintln!("{error}");ExitCode::FAILURE}
 }
}
