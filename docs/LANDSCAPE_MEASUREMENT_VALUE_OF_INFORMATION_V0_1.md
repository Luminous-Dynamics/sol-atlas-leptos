# Landscape measurement value-of-information benchmark v0.1

**Status:** draft, synthetic-only Rust benchmark. It is not a field measurement plan, not a qualified Bayesian value-of-information model, and not evidence that a landscape intervention works.

## Paradigm shift

Traditional monitoring often starts by spreading sensors uniformly or sampling where it is easiest. A decision-centric alternative asks: **which feasible, authorized observation is most likely to distinguish the decision-relevant landscape alternatives, given the budget?**

Recent research supports studying measurement design as an optimization problem, without implying that a result from one setting generalizes everywhere. A June 2026 urban-temperature study used terrain/land-cover-informed placement in a 250 m WRF observing-system simulation experiment and reported comparable reconstruction accuracy with about 200 optimized stations versus more than 300 uniformly distributed stations in that Sofia case: https://doi.org/10.1007/s44327-026-00308-x. A January 2026 PM2.5 study explicitly formulated placement around value of information and misclassification costs: https://doi.org/10.1088/1748-9326/ae2b89. An earlier heat-monitoring study compared entropy, prediction error, and value-of-information objectives and found different objectives can produce different layouts: https://doi.org/10.1016/j.buildenv.2018.05.059. An uncertainty-aware built-environment digital-twin position paper argues for linking data quality, model-form assumptions, calibration, benchmark residuals and decision tolerance throughout the lifecycle: https://doi.org/10.1016/j.dte.2026.100140.

Those studies motivate the question; they do not validate this planner or justify transplanting their layouts/accuracy into Johannesburg.

## v0.1 contract and algorithm

The frozen fixture represents candidate site/metric/time measurements with:

- Separate baseline and alternative predictive means and standard deviations.
- One shared model-family/version for both alternatives within each metric/unit, plus distinct SHA-256 digests for the baseline and alternative scenario snapshots. Every candidate in the same campaign must bind to that same scenario pair; this prevents a model-version change from being mistaken for a design effect.
- A measurement-noise standard deviation.
- An explicit decision weight and integer cost.
- A digest-bound opaque site reference, spatial/temporal stratum, and a declared correlation group.
- Explicit consent and access states, each bound to an evidence digest when positive. `consent_status=granted` requires a valid `consent_evidence_sha256`; `access_status=authorized` requires a valid `access_authorization_sha256`; denied/unknown states must not claim positive authorization evidence. Only granted + authorized candidates are selectable. Fixture digests are synthetic placeholders—not valid permission to access any real site. Mycelix is adding a scoped authorization record in [draft PR #5298](https://github.com/Luminous-Dynamics/mycelix/pull/5298) that distinguishes measurement consent, site access and publication consent. Cross-repository digest resolution, signature verification and expiry/revocation enforcement remain integration work.

For the baseline and alternative predictive distributions, observation noise is added to the respective variances. The planner computes the **symmetrized KL divergence** between these two Gaussian distributions and multiplies it by the explicitly supplied decision weight. This is a distribution-separation proxy, expressed in nats; it is not mutual information, expected posterior decision regret, or real-world utility.

An exact dynamic program selects the maximum total weighted discrimination score under the integer budget while selecting at most one candidate per declared correlation group. The result uses deterministic tie-breaking, records exclusions/rejections with reasons, reports eligible spatial-temporal stratum coverage, emits both paired scenario SHA-256 digests, and hashes the exact fixture bytes. Coverage is descriptive in v0.1—not a minimum-coverage constraint—so a high discrimination score cannot be mistaken for representative sampling.

The one-per-correlation-group rule is a conservative v0.1 anti-redundancy constraint. The planner does not estimate spatial covariance; group definitions must be supplied by a domain-appropriate method. It also assumes candidate scores can be added across different groups. That assumption needs to be replaced by a calibrated covariance-aware design before making claims about real sensor-network optimality.

## Frozen synthetic benchmark

Budget: 6 cost units. The report calculates both a score-per-cost greedy baseline and 64 deterministic seeded uniform-priority feasible plans. In this fixture the greedy baseline selects the cost-4 candidate with score 7; the uniform baseline averages 8.78125 nats and averages 0.53125 stratum coverage; the exact budgeted selection chooses two cost-3 candidates with score 10 and coverage 2/3. Its synthetic gain is 3 nats over greedy and 1.21875 nats over the seeded-uniform mean. These are algorithm oracles, not empirical performance, cash costs, or a landscape benefit.

Known-answer checks cover the Gaussian symmetric-KL equation, exact budgeted selection, one-per-group selection, fail-closed consent/access, duplicate IDs, the synthetic-only claim ceiling, input digests and deterministic reports. An independent exhaustive-enumeration oracle checks the dynamic program across 24 deterministic six-candidate problem variants, enumerating all 64 subsets per case (1,536 subsets total) across differing budgets, costs, decision weights, and correlation assignments. The frozen report also checks the coverage denominator (three eligible spatial-temporal strata) and selected-stratum fraction (two thirds).

## Benchmark ladder

1. **Contract integrity:** malformed distributions, missing source digests, unknown consent/access, duplicate IDs, and zero-variance values are rejected.
2. **Small exact oracle (implemented in v0.1):** compare the dynamic program with exhaustive enumeration across 24 deterministic generated six-candidate problems and budgets; require identical optimal score and selected IDs. Extend coverage with property-based generation and independent oracle implementations before scaling candidate counts.
3. **Heuristic baselines (partial v0.1):** the report compares exact selection with greedy discrimination-per-cost and 64 deterministic seeded uniform-priority plans, reporting score and stratum coverage. Next add coverage-constrained selection and metric-aware baselines; do not rank raw mean gaps or raw variances across incompatible metrics/units.
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
