# Landscape measurement value-of-information benchmark v0.1

**Status:** draft, synthetic-only Rust benchmark. It is not a field measurement plan, not a qualified Bayesian value-of-information model, and not evidence that a landscape intervention works.

## Paradigm shift

Traditional monitoring often starts by spreading sensors uniformly or sampling where it is easiest. A decision-centric alternative asks: **which feasible, authorized observation is most likely to distinguish the decision-relevant landscape alternatives, given the budget?**

Recent research supports studying measurement design as an optimization problem, without implying that a result from one setting generalizes everywhere. A 2026 urban-temperature study used terrain/land-cover-informed sensor placement in a WRF observing-system simulation experiment and reported comparable reconstruction accuracy with about 200 optimized stations versus more than 300 uniformly distributed stations in that setting: https://doi.org/10.1007/s44327-026-00308-x. An earlier heat-monitoring study compared entropy, prediction error, and value-of-information objectives and found that different objectives lead to different sensor layouts: https://doi.org/10.1016/j.buildenv.2018.05.059. An uncertainty-aware built-environment digital-twin position paper argues for linking data quality, model-form assumptions, calibration, benchmark residuals and decision tolerance throughout the lifecycle: https://doi.org/10.1016/j.dte.2026.100140.

Those studies motivate the question; they do not validate this planner or justify transplanting their layouts/accuracy into Johannesburg.

## v0.1 contract and algorithm

The frozen fixture represents candidate site/metric/time measurements with:

- Separate baseline and alternative predictive means and standard deviations.
- A measurement-noise standard deviation.
- An explicit decision weight and integer cost.
- A digest-bound opaque site reference, spatial/temporal stratum, and a declared correlation group.
- Explicit consent and access states. Only `consent_status=granted` plus `access_status=authorized` is selectable; denied/unknown status fails closed.

For the baseline and alternative predictive distributions, observation noise is added to the respective variances. The planner computes the **symmetrized KL divergence** between these two Gaussian distributions and multiplies it by the explicitly supplied decision weight. This is a distribution-separation proxy, expressed in nats; it is not mutual information, expected posterior decision regret, or real-world utility.

An exact dynamic program selects the maximum total weighted discrimination score under the integer budget while selecting at most one candidate per declared correlation group. The result uses deterministic tie-breaking, records exclusions/rejections with reasons, and hashes the exact fixture bytes.

The one-per-correlation-group rule is a conservative v0.1 anti-redundancy constraint. The planner does not estimate spatial covariance; group definitions must be supplied by a domain-appropriate method. It also assumes candidate scores can be added across different groups. That assumption needs to be replaced by a calibrated covariance-aware design before making claims about real sensor-network optimality.

## Frozen synthetic benchmark

Budget: 6 cost units. The report calculates a score-per-cost greedy baseline as well as the exact constrained optimum. In this fixture the greedy baseline selects the cost-4 candidate with score 7; exact budgeted selection chooses two cost-3 candidates with total score 10, a synthetic gain of 3 discrimination-nats. These are algorithm oracles, not empirical performance, cash costs, or a landscape benefit.

Known-answer checks cover the Gaussian symmetric-KL equation, exact budgeted selection, one-per-group selection, fail-closed consent/access, duplicate IDs, the synthetic-only claim ceiling, input digests and deterministic reports. An independent exhaustive-enumeration oracle checks the dynamic program across 24 deterministic six-candidate problem variants, enumerating all 64 subsets per case (1,536 subsets total) across differing budgets, costs, decision weights, and correlation assignments.

## Benchmark ladder

1. **Contract integrity:** malformed distributions, missing source digests, unknown consent/access, duplicate IDs, and zero-variance values are rejected.
2. **Small exact oracle (implemented in v0.1):** compare the dynamic program with exhaustive enumeration across 24 deterministic generated six-candidate problems and budgets; require identical optimal score and selected IDs. Extend coverage with property-based generation and independent oracle implementations before scaling candidate counts.
3. **Heuristic baselines:** compare exact selection with uniform/coverage sampling, max predicted variance, raw mean-gap ranking, and greedy discrimination-per-cost. Report total weighted score, cost, stratum coverage and runtime separately.
4. **Redundancy/uncertainty:** add validated spatial covariance, conditional information gain, uncertainty calibration, measurement error and model-form discrepancy. Keep correlated observations from being counted as independent information.
5. **Decision utility:** only after the objective is preregistered, compare expected decision loss/regret against baseline plans in held-out simulations and prospective field trials.
6. **Field qualification:** use a consented pilot, a declared sampling frame, instrument calibration, weather/time matching and an independent outcome reference. Report coverage/missingness and uncertainty. Never relabel synthetic inputs as observations.

Target: zero unauthorized/unconsented picks, zero budget/correlation-group violations, exact agreement with exhaustive small-instance oracles, deterministic reports, and measured superiority over uniform and greedy baselines on frozen held-out synthetic problem families before testing real deployments.

## Reproduction

```sh
cargo fmt --manifest-path tools/landscape-measurement-planner/Cargo.toml -- --check
cargo test --locked --manifest-path tools/landscape-measurement-planner/Cargo.toml
cargo run --locked --manifest-path tools/landscape-measurement-planner/Cargo.toml
```

The default CLI loads `fixtures/landscape/measurement-planning-v0/fixture.json`. All fixture values are synthetic, and every generated report states `SYNTHETIC_SELECTION_ONLY`.
