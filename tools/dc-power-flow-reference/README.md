# DC Power-Flow Reference v0.1

A dependency-free Rust reference kernel for small, fixed-injection DC power-flow feasibility tests.

## Model profile

For each in-service branch from bus f to bus t:

P_ft = baseMVA / (x * tap) * (theta_f - theta_t - phase_shift_rad)

Reactance is per unit. A zero transformer tap field means unity. Phase shift is positive in the declared from-to orientation. One unspecified-injection reference bus per energized island is fixed at zero radians; its balancing injection is calculated from the solved branch flows.

The kernel validates topology, reference-bus count, branch endpoints, reactance, tap ratio, phase shift, optional Rate A limits and nodal residuals. It reports thermal overloads separately from solvability.

## Explicit non-goals

This is not AC power flow, OPF, unit commitment, a production grid simulator or an actuation path. It does not model voltage magnitude, reactive power, resistance losses, voltage control, generator capability curves, frequency response, protection, dynamic stability or transient behavior. It treats Rate A MVA as an active-MW limit under the DC approximation.

The dense reference solver is capped at 256 buses and 4,096 branches. It is intended for small independent test cases, differential validation and inspection—not large-system performance.

## Run

cargo test --manifest-path tools/dc-power-flow-reference/Cargo.toml --locked --offline

cargo clippy --manifest-path tools/dc-power-flow-reference/Cargo.toml --locked --offline --all-targets -- -D warnings

## Promotion path

1. Add hand-calculated 2–5 bus fixtures and negative controls.
2. Differentially compare the same DC model with MATPOWER, PowerModels.jl and/or PyPSA on pinned cases.
3. Bind supported PGLib cases through a separately audited MATPOWER importer.
4. Only after parity, use the kernel as a small-instance feasibility reference for a DC-OPF optimizer.

Passing these tests proves only the declared linear model and fixtures, not real-network accuracy or operating authority.
