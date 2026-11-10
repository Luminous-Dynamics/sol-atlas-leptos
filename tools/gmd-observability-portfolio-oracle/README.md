# GMD observability portfolio oracle

An isolated, dependency-free Rust reference implementation for small linear-Gaussian sensor portfolio design problems.

## What it does

Given:

- A positive-definite prior covariance over declared physical state variables.
- A supplied linear observation operator for each candidate instrument.
- A full positive-definite measurement-noise covariance, including cross-sensor correlated noise.
- Positive lifecycle cost units for candidate instruments.
- Positive decision weights for state dimensions.
- A deployment budget and explicit single-instrument-outage risk aversion.

The oracle exhaustively evaluates portfolios (up to 12 candidate instruments and 16 state variables) and minimizes a weighted posterior-variance objective that balances nominal performance with the worst single-instrument-outage case.

For selected observations S:

```
P_post = (P_prior^-1 + H_S^T R_S^-1 H_S)^-1
```

The measurement covariance submatrix R_S is selected from the full supplied R, so correlated measurement errors are not silently treated as independent.

## Important boundary

This is an optimal-experimental-design calculation, not a geomagnetic field model or sensor-placement truth engine.

It does not create magnetometer spatial covariance, MT impedance, transformer observation operators, sensor noise, prior uncertainty, instrument costs or utility topology. Those must be supplied by a qualified physical model or measured evidence. In particular, distance is not used as a proxy for representativeness.

A mathematical posterior reduction is conditional on the supplied prior, observation matrix and noise covariance. Missing or unqualified physical observation operators must block use of this oracle for real deployment recommendations.

The current optimizer handles at most one sensor loss at a time. It is not an arbitrary multi-failure guarantee, and lifecycle costs are user-normalized units rather than prices.

## Run

From the Sol-Atlas repository root:

    cargo fmt --manifest-path tools/gmd-observability-portfolio-oracle/Cargo.toml -- --check
    cargo test --manifest-path tools/gmd-observability-portfolio-oracle/Cargo.toml --locked --offline
    cargo clippy --manifest-path tools/gmd-observability-portfolio-oracle/Cargo.toml --locked --offline --all-targets -- -D warnings

## Evidence tier

Passing analytic fixtures establish E1 mathematical/software correctness under the linear-Gaussian assumptions. P1+ regional GMD claims require real station holdouts, calibrated observation operators, measured cross-covariance, rights-qualified data and comparison against actual operator instrumentation.
