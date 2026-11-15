# Fixed-Injection DC Contingency Sweep v0.1

A bounded research evaluator for the intact case plus every single in-service branch outage in a small linear DC network.

For each scenario it:

1. clones the declared input network and disables exactly one branch (or none for the base case);
2. runs the separate dc-power-flow-reference solver;
3. passes every returned candidate to the separate dc-power-flow-independent-auditor;
4. classifies independently checked limit/residual violations separately from solver failures and auditor rejections;
5. reports complete branch-outage coverage, or rejects the sweep before running if the configured hard budget is too small.

## Why this is useful

This introduces a falsifiable contingency-screening baseline before adding redispatch, security-constrained OPF or learned outage policies. It can expose overloads, islanding and model-domain failures without promoting a missing solution to a safe outcome.

## Run

    cargo test --manifest-path tools/dc-fixed-injection-contingency-sweep/Cargo.toml --locked --offline
    cargo clippy --manifest-path tools/dc-fixed-injection-contingency-sweep/Cargo.toml --locked --offline --all-targets -- -D warnings

## Deliberate boundary

This is **not** a full N-1 security assessment and must not be labelled "N-1 secure". The present model:

- fixes all specified bus injections; it does not redispatch generators;
- uses one balancing/reference injection per energized island as defined by the solver input;
- does not model generator P/Q capability, reserve, ramping or corrective action;
- uses DC active-power approximations and Rate A values as MW limits;
- omits voltage magnitude, reactive power, resistance losses, AC power flow, frequency response, protection, transient stability, switching policy and grid-code criteria;
- has no telemetry, source-authentication or actuation path.

A branch outage that creates an island without an admissible reference bus is an explicit solver unknown, not a pass. A solved overload is an audited violation, not a feasible case. A solver or auditor failure is never counted as a pass.

The network identity label is a caller-supplied correlation label only. The caller must bind it to the upstream source bytes, parser profile, import receipt and exact model inputs. This crate does not authenticate or hash those bytes.

The hard sweep budget is 512 active branch outages. The default budget is 256. If the active branch count exceeds the configured budget, the function returns an error and no partial report.

## Research status

This increment provides a software benchmark instrument and synthetic known-answer/negative-control tests. It does not establish PGLib field parity, AC feasibility, real-grid transfer, N-1 security, physical reliability, professional approval or operating authority. Matched PGLib, PowerModels.jl, PyPSA and independent system-operator comparisons remain future gates.
