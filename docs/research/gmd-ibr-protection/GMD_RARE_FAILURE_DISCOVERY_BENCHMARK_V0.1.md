# GMD rare-protection-failure discovery benchmark v0.1

**Date:** 12 November 2026  
**Tracking:** [GMD-TRAJECTORY-FALSIFICATION-001 #740](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/740)  
**Parent:** [trajectory/timebase verifier PR #734](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/734), [protection envelope verifier PR #731](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/731)  
**Claim ceiling:** E2 synthetic sampling-algorithm benchmark only.

## Purpose

Measure how efficiently different sampling strategies discover disconnected rare-failure regions under an identical evaluation budget. This benchmark does not run an electrical simulator and does not certify any operating region.

The oracle is an idealized signed-margin function over four normalized synthetic axes:
- GIC pulse width;
- fault inception phase;
- CT remanence;
- IBR current-limit response.

Twelve disjoint ellipsoidal failure regions are frozen in the challenge corpus. The Rust harness compares:
1. uniform 8x8x8x8 grid;
2. seeded Latin hypercube sampling;
3. Halton low-discrepancy sampling;
4. margin-guided local refinement with continuing global exploration.

Each method receives exactly 4,096 oracle evaluations per seed across five declared seeds. The margin-guided method uses the same oracle budget and no access to region centers or IDs; it can see only the scalar signed margin returned by the synthetic oracle.

## Metrics

The deterministic receipt reports:
- regions found and region recall;
- failure hits;
- occupied 8x8x8x8 diagnostic bins (not a volume-coverage estimate);
- best observed synthetic margin;
- evaluation count;
- non-cryptographic deterministic sample fingerprint;
- certified cell count, hard-coded to zero because discovery is not qualification.

The benchmark has no false-safe rate claim because it does not certify cells. Its required safety property is simpler: the discovery harness must never emit or imply a certified safe cell.

## Important limitations

- All regions and values are synthetic, hand-authored algorithm-challenge fixtures.
- The signed-margin oracle is an idealized diagnostic function, not a transformer/CT/relay model.
- The four axes are static normalized controls in v0.1; multi-rate trajectories, clock uncertainty and event-order ambiguity enter only in later challenge versions.
- This does not establish physical protection behavior, model accuracy, or superiority over industry methods.
- Equal evaluation budgets do not by themselves establish equal computational cost; future reports must include wall-clock, CPU and simulator resource costs.
- Results must be compared across held-out seeds and, eventually, with an independent EMT/HIL oracle.

## Industry and SOTA positioning

Active learning and boundary-adaptive sampling already exist in power-system planning research. This benchmark therefore does not claim novelty for "using active learning." The research hypothesis is that trajectory-aware, evidence-qualified rare-event discovery can improve detection efficiency when failure regions depend on GIC pulse history, fault phase, CT state, IBR dynamics and uncertain clocks.

The next version should add:
- binary-only oracle baselines vs signed-margin oracle;
- importance sampling / subset simulation;
- Gaussian-process or other calibrated surrogate acquisition;
- event-time and sample-clock perturbations;
- held-out region families and interaction-only failures;
- matched EMT/HIL budget accounting;
- model-disagreement escalation and proof-qualified envelope output.

## Reproduce

```sh
cargo fmt --manifest-path tools/gmd-rare-failure-benchmark/Cargo.toml -- --check
cargo clippy --manifest-path tools/gmd-rare-failure-benchmark/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tools/gmd-rare-failure-benchmark/Cargo.toml --locked
cargo run --manifest-path tools/gmd-rare-failure-benchmark/Cargo.toml --locked
```
