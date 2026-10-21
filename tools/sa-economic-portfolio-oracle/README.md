# SA Economic Portfolio Oracle

A small, standalone Rust reference evaluator for [SA-PORTFOLIO-OPT-001](../../docs/research/SA-ECONOMIC-PORTFOLIO-001.md).

This is a **synthetic contract benchmark**, not a production national-investment planner.

## Run

From the repository root with Rust/Cargo installed:

```sh
cargo test --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml
```

The default command evaluates the frozen synthetic input, checks its exact feasible set and Pareto frontier against the expected manifest, and prints a JSON receipt with the input SHA-256 digest. Optional positional arguments can select an alternate input and expected manifest:

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- INPUT.json EXPECTED.json

# Generalized v0.2 ten-project exact reference (the separate branch-and-bound
# search is exercised through the Rust library tests)
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.expected.json

# Exact dependency-aware frontier search, with a reference frontier
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --search \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.expected.json

# Adversarial 24-project search, compared to the frozen eight-project frontier
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --search \
  docs/research/fixtures/sa-economic-portfolio-v0.2-24-project.json \
  docs/research/fixtures/sa-economic-portfolio-v0.2-24-project.search-expected.json

# Repeat exact-oracle vs search timings on the same ten-project problem (21 repetitions)
cargo run --release --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --compare-bench \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.expected.json 21

# Repeat search timing on the 24-project adversarial case (21 repetitions)
cargo run --release --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --bench-search \
  docs/research/fixtures/sa-economic-portfolio-v0.2-24-project.json \
  docs/research/fixtures/sa-economic-portfolio-v0.2-24-project.search-expected.json 21
```

### Scenario-set sensitivity (v0.1)

The `--scenario-omission INPUT.json SCENARIO_ID EXPECTED.json` mode removes one scenario from a v0.2 synthetic input, computes the exact reduced-scenario frontier, then rescales each selected portfolio against the *full reference scenario universe*. It reports full-reference hard-feasibility failures separately from scenario-wise and worst-case objective regret. A known-answer manifest is mandatory; a report mismatch exits nonzero.

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --scenario-omission \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  normal_reference \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-omit-normal.expected.json

cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --scenario-omission \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  heat_dry_power_stress \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-omit-heat-dry.expected.json

cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --scenario-omission \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-floor-challenge.json \
  heat_dry_power_stress \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-floor-challenge-omit-stress.expected.json
```

The two original v0.2 omission fixtures are deliberately useful controls: omitting `normal_reference` keeps 82 feasible portfolios and its eight frontier IDs; omitting `heat_dry_power_stress` admits 90 feasible portfolios and changes frontier membership to seven IDs. The dedicated floor challenge changes one synthetic stress-service contribution so that omitting that scenario produces seven reduced-frontier candidates, three of which fail the full-reference hard floor. These are arbitrary test units, not claims about real service delivery or scenario likelihood. The expected JSON manifests are frozen from a separate exhaustive enumeration, not calculated by the Rust scenario-omission function.

### Resource-capacity sensitivity sweep

`--capacity-sweep INPUT.json CONFIG.json EXPECTED.json` varies one period's cost cap or crew cap over a strictly descending finite sequence. The first value must equal the input's existing cap, making the no-shock baseline an explicit negative control. Each cap is applied independently to a fresh copy of the unchanged input and evaluated exactly. The receipt includes all removed portfolio IDs, the exact frontier project-ID sets before and after each cap, frontier-ID intersection/union, attempted values and any remaining values. It verifies that tightening a cap never introduces feasible portfolios.

The config selects one of two modes:

- `stop_on_first_counterexample: true` identifies the first cap that excludes any baseline-feasible portfolio and lists the untested suffix.
- `stop_on_first_counterexample: false` computes the entire cap-to-feasibility curve, unless the evaluation budget is exhausted. A partial curve reports `BUDGET_EXHAUSTED` even when an earlier cap already excluded portfolios; the receipt still records `first_counterexample_capacity`.

```sh
# Early-stop threshold search
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --capacity-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-resource-capacity.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-cost-config.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-cost.expected.json

# Full cost-cap sensitivity curve
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --capacity-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-resource-capacity.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-cost-full-config.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-cost-full.expected.json

# Full crew-cap sensitivity curve
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --capacity-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-resource-capacity.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-crew-full-config.json \
  docs/research/fixtures/sa-portfolio-capacity-sweep-v0.1-p1-crew-full.expected.json
```

The purpose-built synthetic fixture makes cost and crew constraints non-interchangeable. It changes the P1 cost cap to 6 and P02 cost to 3 while holding every scenario outcome fixed. The baseline has 82 feasible portfolios and an eight-member frontier. Exhaustively checking four cap values gives:

| Dimension | Capacity | Feasible portfolios | Excluded vs. baseline | Frontier size | Frontier ID intersection / union vs. baseline |
|---|---:|---:|---:|---:|---:|
| P1 cost | 6 | 82 | 0 | 8 | 8 / 8 |
| P1 cost | 5 | 72 | 10 | 8 | 6 / 10 |
| P1 cost | 4 | 30 | 52 | 6 | 0 / 14 |
| P1 cost | 3 | 30 | 52 | 6 | 0 / 14 |
| P1 crew | 3 | 82 | 0 | 8 | 8 / 8 |
| P1 crew | 2 | 42 | 40 | 6 | 0 / 14 |
| P1 crew | 1 | 22 | 60 | 4 | 0 / 12 |
| P1 crew | 0 | 5 | 77 | 2 | 0 / 10 |

The early-stop searches require two evaluations each: baseline plus first binding cap. Full curves require all four; the complete manifests freeze every excluded portfolio ID and both frontier ID sets at each point. These values are synthetic test constraints, not field-calibrated capacity estimates.

### Bounded counterexample sweep

`--counterexample-sweep INPUT.json TEMPLATE_PATCH.json CONFIG.json EXPECTED.json` explores an ordered, finite list of non-negative integer service-baseline values using the existing exact evaluator. Each attempted mutation receives a stable scenario ID suffix; the receipt hashes the input, template, config and expected report. It stops on the first portfolio removed by the newly added scenario, or reports `DOMAIN_EXHAUSTED_NO_COUNTEREXAMPLE` only after every configured value is checked. If the budget ends first, status is `BUDGET_EXHAUSTED` and unattempted values are listed. The sweep is not a global real-world robustness proof.

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --counterexample-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-compound-template.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-compound-config.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-compound.expected.json

cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --counterexample-sweep \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-no-effect-control.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-no-effect-config.json \
  docs/research/fixtures/sa-portfolio-counterexample-sweep-v0.1-no-effect.expected.json
```

A further metamorphic test varies each period's cost cap and crew cap by one unit. For every dimension, tightening a cap must yield a feasible portfolio set that is a subset of baseline; relaxing a cap must preserve every baseline-feasible portfolio. This checks monotonicity of hard resource constraints, not the predictive validity of any resource forecast.

The synthetic compound fixture checks baseline 10 first (no feasibility change), then baseline 9 (two formerly feasible portfolios violate the protected service floor). It stops after two of nine configured values and reports the seven remaining values; it does not claim the domain was exhausted. The exact duplicate-scenario control checks baseline 3, preserves all 82 feasible portfolios and all eight frontier IDs, and only then returns `DOMAIN_EXHAUSTED_NO_COUNTEREXAMPLE`. A separate unit test confirms that a one-evaluation cap on a larger no-effect domain returns `BUDGET_EXHAUSTED`, never a robustness pass.

### Withheld-scenario addition

The `--scenario-addition INPUT.json PATCH.json EXPECTED.json` mode adds a fully declared withheld scenario, recomputes exact feasibility/frontier on the augmented scenario universe, compares the original and augmented feasible sets, and emits a receipt with SHA-256 digests for the input, patch and expected report. It fails closed unless the complete report exactly matches the frozen independent manifest.

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --scenario-addition \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-withheld-compound-stress.json \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-withheld-compound-stress.expected.json
```

The frozen compound-stress patch adds explicit outcomes for all ten projects and both interaction rules, combining heat/drought service pressure with grid scarcity without assigning a probability. This first patch does not change the period crew caps; resource-capacity uncertainty is a separate perturbation family still to implement. A second patch, `sa-portfolio-scenario-robustness-v0.1-no-effect-control.json`, duplicates the `normal_reference` coefficients exactly as a negative control; its expected report verifies 82 feasible portfolios, the same eight frontier IDs, and zero feasible-set or frontier turnover.

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- --scenario-addition \
  docs/research/fixtures/sa-economic-portfolio-v0.2-10-project.json \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-no-effect-control.json \
  docs/research/fixtures/sa-portfolio-scenario-robustness-v0.1-no-effect-control.expected.json
``` The independent exhaustive manifest records 82 feasible portfolios and 8 frontier members in the original two-scenario input; after adding this synthetic scenario, 41 feasible portfolios and 5 frontier members remain, with 41 previously feasible combinations failing the expanded constraints. Four frontier IDs are retained; the ID-set union contains nine. These are synthetic units only. The metamorphic test verifies that the augmented feasible set is a subset of the original set and that the reported removed portfolios reconcile exactly.

The exact oracle enumerates all subsets up to 20 projects and rejects over-cap inputs rather than silently truncating. V0.1 freezes eight project IDs, two named stress scenarios, P1/P2 periods and interaction rules. V0.2 accepts unique project and scenario IDs plus ordered Pn periods (while retaining the four typed maximize objectives and null scenario weights/probabilities); the committed ten-project fixture freezes 1,024 subsets, 82 feasible portfolios and its exact frontier.

The library also exposes `evaluate_frontier_search(&Input)`, a dependency-aware depth-first frontier search. It prunes only provably infeasible include branches (non-PASS gate, missing prerequisite, period cost or crew cap), retains the full multi-scenario objective vector, and returns node/subset accounting. It is exact, not an approximation, but remains exponential in the worst case; the 63-project u64 mask bound is not a practical runtime promise. The frozen v0.1 and generated v0.2 tiny instances are cross-checked against exhaustive enumeration. The CLI flag `--search INPUT.json [REFERENCE_FRONTIER.json]` emits a separate search receipt. Supplying a reference file verifies the frontier only; on a large instance, this is not a proof of the entire feasible set or a replacement for the small exact-oracle tests. The current suite also has deterministic generated 12 / 16 / 18-project tests and a 24-project decoy case; the latter is intentionally easy to prune and is an adversarial regression, not broad scaling evidence. The `--compare-bench INPUT.json EXPECTED.json [REPEATS]` mode times exact enumeration and frontier search in the same process over an identical input, verifies both outputs against the expected manifest, and emits sorted samples plus nearest-rank p50/p95/max latency. It runs two warm-ups per algorithm, alternates the order within timed pairs, checks output stability, and records runner OS/architecture, CPU model, logical CPU count, memory and process high-water RSS where available. The `--bench-search INPUT.json REFERENCE.json [REPEATS]` mode measures search when exhaustive enumeration is impractical; it also performs two warm-ups and verifies frontier equality, not the full feasible set. Both default to 21 measured repetitions (allowed range 1–1000), hash input/reference manifests and label outputs as synthetic. Still treat CI results as smoke benchmarks until representative workload families and a matched independent solver are included.

## Important limits

- All costs and objective values are arbitrary synthetic units.
- Scenario cases are stress cases, not probabilities.
- The frontier preserves non-dominated alternatives; it does not choose society's priorities or a single winner.
- Every rejected subset receives exactly one **primary** rejection reason in the receipt. The fixed precedence is: non-PASS gate, missing prerequisite, period budget exceeded, crew cap exceeded, protected service floor. When multiple constraints fail, only the first reason is counted; these counts partition rejected subsets and must sum to the rejected-subset count. They are not multi-label incident totals.
- Both v0.1 (8 projects / 256 subsets) and v0.2 (10 projects / 1,024 subsets) exact fixtures include an UNKNOWN-gate decoy assigned implausibly high values; it must never enter a feasible portfolio.
- Frontier-search counters partition the entire subset space into visited leaf evaluations and subtrees pruned for a proved infeasibility. They are candidate-search metrics, not a runtime/SOTA result until exact-head runs and larger matched baselines are published.
- Input hashing identifies the exact bytes evaluated. It does not authenticate a publisher or establish the truth of data.
- Passing the tests does not establish real ROI, causal service improvements, jobs, engineering validity, legal permission or investment authority.
