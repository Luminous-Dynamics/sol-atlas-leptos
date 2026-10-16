# Water sensor-placement benchmark v0.1

## Purpose

This experiment asks whether sensor-layout strategies differ in synthetic junction-withdrawal localization quality under a matched pressure-response corpus. It does **not** claim leak detection, physical leak localization, or deployment readiness.

The benchmark reuses the full node-pressure tables in a completed, hash-verified EPANET run corpus. That is the main design shift: candidate layouts consume the same solved scenarios, rather than requiring a separate hydraulic run for every layout. This removes one major source of comparison unfairness and makes layout search reproducible.

## Frozen evaluation design

The Rust CLI \`water-sensor-placement-evaluate\` reads:

1. the completed scenario-corpus manifest;
2. the directory containing its report artifacts; and
3. the exact source EPANET INP.

It verifies the source topology SHA-256 against the run manifest, checks native units, validates each report's relative path, byte count and SHA-256, and requires every junction to have a finite pressure row in the first reported period. Missing values are errors, not zeros. The baseline must be unique and every target must appear at every held-out withdrawal magnitude.

The evaluator compares four layout strategies:

- **Fixed predeclared:** the four manifest observation sites first, then a deterministic junction-ID ordering for larger budgets. This is a baseline, not an asserted optimal layout.
- **Seeded random:** a deterministic random layout, with replicate-specific seeds.
- **Graph-coverage greedy:** farthest-first selection under unweighted graph-hop distance.
- **Response-separation greedy:** greedily increases the minimum pairwise distance between target-class centroids in the *training fold only*.

Proposed sensor budgets are 2, 4 and 8 sensors (clamped to the available junction count for small test networks). The split is leave-one-withdrawal-magnitude-out. Normalization scales, response-driven placement, and model comparisons are computed from the training fold; held-out target labels are read only after candidates are ranked.

The synthetic noise sweep uses sigma fractions 0, 0.05 and 0.10 of each site's training-fold standard deviation in normalized pressure-change space, with five deterministic replicates. These fractions are stress-test knobs, **not measured transducer error specifications**. The same seeded noise realization is reused across strategies for a given fold/replicate so comparisons are paired. Layout result rows report top-1, top-3, mean reciprocal rank, mean/p95/worst topological localization error, and disconnected predictions.

## Acceptance and reporting rules

A generated result is eligible for inspection only when the runner reports a fully completed corpus, all report artifacts match their declared hashes and byte lengths, and output records bind to the exact run-manifest and topology digests. The expected result count for the 92-junction Net3 experiment is:

\`4 strategies × 3 budgets × 3 noise levels × 5 replicates × 2 held-out magnitudes = 360 rows.\`

These rows are a reproducibility contract, not a success metric. CI must not enforce desired accuracy or silently drop failed scenarios. Publish per-fold/per-noise/per-budget strata; do not collapse them into one hidden scalar score. Any comparison that changes a scenario denominator or uses a different corpus must be labelled non-comparable.

## Proposed research targets

These are project targets, not industry standards and not achieved results.

- **Pilot integrity:** 185/185 all-junction Net3 solver cases complete; exact input, report and manifest hashes verified.
- **Baseline breadth:** at least four strategies compared at 2/4/8 sensor budgets on a shared corpus, with every row and replicate retained.
- **Robustness:** report clean (0), moderate (0.05), and severe (0.10) normalized-noise strata; no accuracy threshold is hard-coded into the evidence plumbing gate.
- **Statistical efficacy:** on a later preregistered multi-topology corpus, seek at least a 5-percentage-point improvement in held-out top-3 localization over the strongest matched baseline, using paired bootstrap intervals with the lower 95% confidence bound above zero. This is meaningful only with grouped held-out operating regimes and adequate sample size.
- **Topology-aware loss:** in addition to classification metrics, track mean/p95/worst graph-hop error and disconnected predictions. Do not call link-hop distance physical distance or repair cost.
- **Operational value:** only after calibrated real-world data and a defensible loss/cost model exist, estimate decision regret and expected value of sample information.

## Limitations and next gates

The experiment currently has one example topology, synthetic added nodal demand rather than a calibrated leak emitter, two perturbation magnitudes, one first-period pressure snapshot, and an illustrative normalized-noise model. It does not evaluate pressure-dependent demand, daily demand patterns, sensor bias/drift/clock skew, topology uncertainty, pipe-segment hypotheses, cross-topology transfer, actual sensor cost, field false alarms, detection delay, or real service outcomes. Sensor placement and localization remain coupled to the hydraulic model assumptions.

Before claims of research efficacy, add varied demand patterns/operating states and pressure-dependent demand; vary leak/emitter semantics separately from legitimate demand changes; include controlled sensor bias and dropout; freeze grouped train/validation/test regimes across multiple networks; add pipe-segment hypotheses; and compare on at least three topologies with independent reproduction. Before a field pilot, use authorized utility data, calibrate and validate the hydraulic model on held-out observations, quantify sensor uncertainty and missingness, obtain qualified engineering review, and retain human authorization for any physical intervention.

## Research basis

- Cheng and Li, *Water Research* (2023), [optimal sensor placement with graph signal processing](https://doi.org/10.1016/j.watres.2023.120313), studies graph-based feature selection and sensor redundancy, including noise-perturbed datasets.
- Ranganathan et al., *Journal of Water Process Engineering* (May 2026), [benchmarking leak detection and localization architectures](https://doi.org/10.1016/j.jwpe.2026.110114), reports a 47,000+ Net3-scenario benchmark and distinguishes high binary-detection performance from the harder localization task.
- [Pressure sensor placement using information theory](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/) considers information relevance and redundant measurements.
- Steffelbauer and Fuchs-Hanusch, *Water Resources Management* (2016), [sensor placement under demand uncertainty](https://doi.org/10.1007/s11269-016-1504-6), motivates testing uncertainty in addition to nominal hydraulic responses.

These references motivate the experiment design; none validates this implementation or implies that Sol Atlas has outperformed prior methods.
