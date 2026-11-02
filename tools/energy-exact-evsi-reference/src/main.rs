use energy_exact_evsi_reference::{
    DecisionProblem, ObservationCandidate, ObservationDesign, WeightedState, exact_evsi,
    rank_observation_candidates,
};

fn main() {
    let problem = DecisionProblem {
        states: vec![
            WeightedState {
                id: "normal".to_owned(),
                prior_probability_ppm: 500_000,
            },
            WeightedState {
                id: "storm".to_owned(),
                prior_probability_ppm: 500_000,
            },
        ],
        actions: vec!["do-nothing".to_owned(), "prepare".to_owned()],
        losses: vec![vec![0, 40], vec![100, 40]],
    };
    let candidates = vec![
        ObservationCandidate {
            id: "high-bound-low-information".to_owned(),
            design: ObservationDesign {
                id: "high-bound-low-information".to_owned(),
                observations: vec!["signal-normal".to_owned(), "signal-storm".to_owned()],
                likelihoods_ppm: vec![vec![500_000, 500_000], vec![500_000, 500_000]],
            },
            heuristic_avoidable_loss_lower_bound_units: 10_000,
            observation_cost_units: 0,
            compute_cost_units: 1,
        },
        ObservationCandidate {
            id: "lower-bound-informative".to_owned(),
            design: ObservationDesign {
                id: "lower-bound-informative".to_owned(),
                observations: vec!["signal-normal".to_owned(), "signal-storm".to_owned()],
                likelihoods_ppm: vec![vec![900_000, 100_000], vec![100_000, 900_000]],
            },
            heuristic_avoidable_loss_lower_bound_units: 100,
            observation_cost_units: 0,
            compute_cost_units: 1,
        },
    ];
    let ranking = rank_observation_candidates(&problem, &candidates).expect("rank candidates");
    println!("# schema=energy-exact-evsi-reference-report-v0.1");
    println!("heuristic_order={}", ranking.heuristic_order.join("|"));
    println!("exact_evsi_order={}", ranking.exact_evsi_order.join("|"));
    for candidate in &ranking.evaluations {
        println!(
            "candidate={},evsi_numerator_ppm2={},evpi_numerator_ppm2={},net_positive={}",
            candidate.id,
            candidate.exact_evsi.evsi_numerator_ppm2,
            candidate.exact_evsi.evpi_numerator_ppm2,
            candidate.exact_evsi.net_value_nonnegative
        );
    }
    let perfect = ObservationDesign {
        id: "perfect".to_owned(),
        observations: vec!["normal".to_owned(), "storm".to_owned()],
        likelihoods_ppm: vec![vec![1_000_000, 0], vec![0, 1_000_000]],
    };
    let exact = exact_evsi(&problem, &perfect, 0).expect("perfect information");
    println!(
        "perfect_information_evsi_numerator_ppm2={}",
        exact.evsi_numerator_ppm2
    );
}
