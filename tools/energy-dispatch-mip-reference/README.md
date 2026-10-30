# Energy Dispatch Independent MIP Reference v0.1

This standalone Rust workspace emits 12 independent integer LP files from the frozen dispatch request: minimize and maximize each of the six objective components separately.

## Run

    cargo test --locked
    cargo run --locked -- ../../schemas/examples/energy-dispatch-pv-outage.synthetic.json ./target/dispatch-mip

## Independence

The crate duplicates the JSON request DTOs and the mathematical model. It does not import the production exact search, the production objective calculation, the production Pareto reducer, the schedule verifier, or the reference enumerator.

## Exact integer semantics

For charge input (c_t), charge gain (q_t), and ratio (n_c/d_c):

    d_c q_t <= n_c c_t
    n_c c_t <= d_c q_t + d_c - 1

For SOC withdrawal (w_t), delivered energy (d_t), and ratio (n_d/d_d):

    d_d d_t <= n_d w_t
    n_d w_t <= d_d d_t + d_d - 1

Because every variable is integer, these inequalities encode the exact floor quotient. No floating-point efficiency coefficient is emitted.

Charging and discharging are mutually exclusive through a binary interval mode. The generator also preserves chronology, SOC transitions, hard terminal reserve, outage import bounds, per-interval bus balance, PV partition, service partition and separate objective dimensions.

## Claim ceiling

The generator emits model inputs only. It has not solved them, checked an optimality certificate, or established physical validity. SCIP exact mode plus VIPR/CakeML checking and CP-SAT status/bound checks are separate downstream steps. Six separate scalar optima do not alone prove full Pareto-frontier completeness.
