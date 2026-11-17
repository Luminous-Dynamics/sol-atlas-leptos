use exact_dc_opf_proof::{TWO_BUS_SOURCE, prove_frozen_fixture};
use std::process::ExitCode;

fn main() -> ExitCode {
    match prove_frozen_fixture(TWO_BUS_SOURCE) {
        Ok(proof) => {
            print!("{}", proof.to_json());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("exact DC-OPF proof refused: {error}");
            ExitCode::FAILURE
        }
    }
}
