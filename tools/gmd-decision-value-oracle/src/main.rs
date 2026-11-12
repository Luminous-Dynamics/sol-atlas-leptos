use gmd_decision_value_oracle::{evaluate, positive_voi_fixture};

fn main() {
    let problem = positive_voi_fixture();
    match evaluate(&problem) {
        Ok(result) => println!("{}", result.to_json()),
        Err(error) => {
            eprintln!("decision-value evaluation failed: {error}");
            std::process::exit(2);
        }
    }
}
