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

The synthetic noise sweep uses sigma fractions 0, 0.05 and 0.10 of each site's training-fold standard deviation in normalized pressure-change space, with five deterministic replicates. These fractions are stress-test knobs, **not measured transducer error specifications**. The same seeded noise realization is reused across strategies for a given fold/replicate so comparisons are paired. At zero noise, deterministic layouts intentionally repeat the same clean score across replicate IDs; these rows preserve a rectangular paired-results contract and must **not** be treated as five independent observations for significance testing. Seeded-random layouts can still differ by replicate. Layout result rows retain per-scenario target label, predicted top-1, actual rank, top-1/top-3 hit flags, reciprocal rank, and graph-hop error, alongside aggregated top-1, top-3, MRR, mean/p95/worst topological localization error, and disconnected predictions. The retained per-scenario outcomes enable paired comparisons instead of trying to reconstruct uncertainty from aggregate rates.

## Acceptance and reporting rules

A generated result is eligible for inspection only when the runner reports a fully completed corpus, all report artifacts match their declared hashes and byte lengths, and output records bind to the exact run-manifest and topology digests. The expected result count for the 92-junction Net3 experiment is:

\`4 strategies × 3 budgets × 3 noise levels × 5 replicates × 2 held-out magnitudes = 360 rows.\`

These rows are a reproducibility contract, not a success metric. CI must not enforce desired accuracy or silently drop failed scenarios. Publish per-fold/per-noise/per-budget strata; do not collapse them into one hidden scalar score. Any comparison that changes a scenario denominator or uses a different corpus must be labelled non-comparable.

The paired-comparison CLI `water-sensor-placement-compare` compares each strategy to the **predeclared** `fixed-predeclared` reference, using matched scenario IDs and actual target labels. It rejects aggregate/per-case metric contradictions, missing or duplicated strategy strata, and any reference/candidate scenario mismatch. It reports paired deltas for top-1, top-3 and MRR with a deterministic 2,000-resample target-cluster percentile bootstrap. If more than one scenario belongs to a target in a stratum, outcomes are first averaged within that target, so bootstrap resampling treats the target as the cluster.

These intervals are conditional on the one supplied topology and scenario family. They are not network-level confidence intervals. All grid comparisons are exploratory and unadjusted for multiple comparisons; a positive interval in a selected row is not a confirmatory efficacy result. Confirmatory work must preregister one primary sensor budget/noise/metric/strategy contrast, or apply a declared multiplicity correction, and must hold out operating regimes and multiple topologies. Repeated clean deterministic rows are not independent evidence.

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

## Provenance hardening amendment (2026-10-16)

The evaluator also verifies that the manifest's declared observation pressure readings exactly match those same nodes in the hash-verified raw EPANET report. It rejects missing/duplicate sensor IDs, unavailable or null values, non-finite values, a declared sensor set that differs from the frozen plan, and any mismatch between the manifest reading and the report value. The output is bound to all three original identities: run-manifest SHA-256, frozen scenario-plan SHA-256, and source-EPANET-INP SHA-256. Scenario labels and withdrawal magnitudes must match the frozen plan byte-for-byte via its digest and semantically via each scenario record.

The response-separation layout is selected from the **clean normalized training fold**, then held fixed across paired noise replicates. That prevents a noise draw from silently changing the layout being evaluated and helps isolate the sensitivity of a fixed layout to the declared synthetic measurement-noise stress. The end-to-end test includes a deliberately inconsistent declared pressure field and expects a fail-closed rejection.

These are authored protections and tests. Their execution is not established until a run of the workflow for the exact current pull-request head reports success.

## Paired comparison / uncertainty increment (2026-10-16)

The localization result contract now retains per-case ranks and hit metrics, allowing downstream checks to reconstruct every aggregate rather than accepting a mean in isolation. A separate Rust comparator produces 270 paired strata for the Net3 design (three non-reference strategies × three budgets × three noise levels × five replicate IDs × two held-out magnitudes). Each stratum reports deltas against the predeclared fixed layout and percentile intervals from 2,000 target-cluster resamples, using the same bootstrap draws across top-1, top-3 and MRR.

The comparator refuses a comparison if two strategies do not share the same scenario IDs and ground-truth targets, if aggregate metrics contradict the per-case records, or if a result stratum is missing. This is analysis infrastructure; the result itself remains unrun until exact-head CI produces the hydraulic and evaluation artifacts. Per-stratum bootstrap intervals are not corrected for the large comparison grid and are explicitly exploratory.

