# GMD decision-value oracle

A dependency-free Rust 2024 reference implementation for a narrow question:

**Does a particular qualified observation improve the decision enough to justify its cost, once its arrival delay, action feasibility, physical admissibility, service floors, rights and human authority are considered?**

This is the executable increment tracked by [GMD-DECISION-VALUE-001 #723](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/723) and specified in [GMD_DECISION_VALUE_BENCHMARK_V0.1.md](../../docs/research/GMD_DECISION_VALUE_BENCHMARK_V0.1.md).

## Current v0.1 boundary

The crate implements a single-step, finite-state, one-observation expected-value oracle. It is deliberately not a general partially observable Markov decision process, grid simulator, forecast model, physical GIC model, scheduler, optimizer or operational controller.

It accepts already-qualified upstream inputs:
- a frozen scenario manifest and digest;
- a finite state set with prior probabilities;
- service-specific loss units and explicit owner-supplied decision weights;
- candidate actions with state-specific physical admissibility and service-floor preservation;
- independent human-approval, resource, evidence-freshness and scope gates;
- a single observation option with source, rights and likelihood-model evidence;
- a joint observation-outcome likelihood model, which must include known common-mode/correlated error;
- acquisition latency, conservative completion duration, safety buffer and decision deadline.

The oracle chooses the lowest expected weighted-loss action among actions that are feasible in **every state still supported by the evidence**. This robust support rule is intentionally conservative. A low-probability state is not discarded simply because it is inconvenient.

For each observation result, the evaluator recalculates the supported state set and selects an outcome-conditioned action. It then compares expected service-loss vectors and the owner-weighted decision score with the ACT_NOW baseline. Observation cost is supplied in the same explicitly owner-defined utility units. No default cross-service weights are invented.

## Exact arithmetic and receipt

Probabilities use integer parts per million (PPM). Prior weights must sum exactly to 1,000,000. For every physical state, the likelihoods across all observation outcomes must also sum exactly to 1,000,000.

Expected values are emitted as exact rational numerators with a common denominator of 1,000,000,000,000. Large integers in the JSON receipt are strings to prevent downstream floating-point truncation.

The CLI runs a small positive-VOI known-answer fixture:

```sh
cargo run --manifest-path tools/gmd-decision-value-oracle/Cargo.toml --locked --offline
```

The fixture should select HOLD before observation. Once a qualified observation arrives, it selects a state-specific mitigation in the moderate-impact state and HOLD in the severe-impact state. The expected weighted loss numerator is 40,000,000,000,000 before observation and 20,000,000,000,000 afterwards. Acquisition cost is 2,000,000,000,000, leaving a positive net value numerator of 18,000,000,000,000. The denominator is 1,000,000,000,000.

The receipt always sets `execution_authorized=false`. It is evidence about a decision calculation, not a command.

## Safety and evidence boundaries

- Unknown, stale or rights-ineligible evidence cannot make an action more admissible.
- A likelihood model that does not qualify known common-mode/correlated errors is rejected.
- A hard-floor service blocks an intervention if the floor is not preserved in any supported state. HOLD does not require intervention approval, deployment resources or intervention-specific freshness/scope gates, but still requires the current physical state to be represented. HOLD remains reportable even if the current service trajectory is already below a floor; the loss remains visible.
- An intervention needs physical admissibility, fresh evidence, available resources, verified scope and human approval.
- An intervention must complete using its maximum declared duration plus safety buffer before the decision deadline.
- An observation arriving at or after the decision deadline is not used.
- A late but pre-deadline observation can still have negative value if waiting for it removes the ability to complete an action.
- Ties resolve lexicographically by action ID.
- No runtime network access or third-party crate dependencies are required.

The output preserves per-service expected loss. The owner-supplied scalarization is used only to select among actions; it must not replace per-service reporting or conceal a breached service floor.

## Test coverage

The Rust unit tests cover:
1. positive value of a state-discriminating observation;
2. zero value for an uninformative, free observation;
3. negative value when waiting causes a feasible action to miss its completion window;
4. rights-ineligible observations;
5. unqualified common-mode likelihood models;
6. pending human approval;
7. unavailable resources;
8. hard service-floor violations;
9. stale action evidence;
10. unresolved action scope;
11. observations arriving after the decision deadline;
12. deterministic receipts;
13. HOLD remains available when intervention-specific gates are closed;
14. malformed prior probabilities.

The machine-readable scenario index is in `fixtures/GMD_DECISION_VALUE_FIXTURES_V0.1.json`.

## What this does not prove

Passing these tests establishes only software behavior for the declared synthetic contracts (E2). It does not validate the likelihood model, sensor calibration, South African station inventory, Earth conductivity, geoelectric fields, network topology, GIC, transformer response, AC dynamics, restoration policy, service data, operator procedures or field safety.

The caller is responsible for providing qualified physical-admissibility and likelihood evidence. The tool does not fetch data, infer rights, authenticate a utility owner, authorize a person, dispatch equipment or certify that essential services recovered.

## Next extensions

- multiple candidate observations and sequential value of information;
- joint correlated sensor observations with explicit common-cause groups;
- uncertainty distributions for acquisition and completion times;
- robust/chance-constrained policy families;
- explicit service-floor trajectory ingestion from the #717 contract;
- operator-policy baselines and event/geographic holdouts;
- independent benchmark reproduction against public GMD and restoration datasets.
