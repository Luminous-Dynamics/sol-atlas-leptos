use energy_solver_compute_calibration::{MAX_TRACE_CSV_BYTES, parse_trace_csv, render_summary_csv};
use std::io::Read;
use std::process::ExitCode;

fn run() -> Result<ExitCode, String> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: energy-solver-compute-calibration <solver-traces.csv>".to_owned())?;
    let file = std::fs::File::open(&path).map_err(|error| format!("open trace file: {error}"))?;
    let mut bytes = Vec::new();
    file.take((MAX_TRACE_CSV_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read trace file: {error}"))?;
    if bytes.len() > MAX_TRACE_CSV_BYTES {
        return Err(format!(
            "trace CSV exceeded {MAX_TRACE_CSV_BYTES} byte limit"
        ));
    }
    let input =
        std::str::from_utf8(&bytes).map_err(|error| format!("trace CSV is not UTF-8: {error}"))?;
    let traces =
        parse_trace_csv(input).map_err(|error| format!("invalid solver trace: {error}"))?;
    let summaries = energy_solver_compute_calibration::summarize_traces(&traces)
        .map_err(|error| format!("cannot aggregate solver traces: {error}"))?;
    print!("{}", render_summary_csv(&summaries));
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("solver trace calibration failed closed: {error}");
            ExitCode::from(2)
        }
    }
}
