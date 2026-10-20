# AI Compute–Energy Trace Intake and Benchmark Protocol v1

**Status:** protocol draft; the Alibaba and Google Borg manifests are both planned metadata only. No trace artifact has been downloaded or evaluated by this work item.  
**Related work:** [AI-COMP-ENERGY-001 / evaluator PR #203](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/203) and [AI-COMP-ENERGY-002 / follow-up issue #212](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/212).  
**Machine-readable contract:** `schemas/ai-compute-energy-trace-intake-v1.schema.json`.

## 1. Paradigm shift: evidence is part of the benchmark

A benchmark should not start with a large dataset and only later ask what its fields mean. It should start with an explicit **claim ceiling**: what is justified by the evidence that was actually collected?

The intake manifest records source/release identity, license review, raw and normalized artifact hashes, time semantics, unit mappings, quality-gate status, data-cleaning counts, energy attribution, transformation identity, and prohibited inferences. A planned source can be registered without pretending it has been retrieved or validated.

This makes four evidence states distinct:

1. **Source registered** — a source is identified; no quantitative claim follows.
2. **Workload trace replay** — arrivals, dependencies, demand, deadlines and schedule outcomes are reconstructible, but power/quality may remain unknown.
3. **Metered system result** — defined wall-power boundary, instrument, sampling method, repeated runs and uncertainty are available.
4. **Quality-accepted work per measured kWh** — the same model/workload/quality gate is paired with measured energy at a named boundary.

These are not interchangeable levels of confidence. A utilization time series does not become GPU energy, model quality, facility power, or grid-service performance by renaming its columns.

## 2. First trace candidate and its traps

The [official Alibaba `cluster-trace-v2018` release documentation](https://github.com/alibaba/clusterdata/blob/master/cluster-trace-v2018/trace_2018.md) describes approximately 4,000 machines over eight days and six tables, including machine/container usage and batch-workload records. It documents time fields as seconds relative to the beginning of the sampled period and warns that some normalized memory/disk values are scaled and invalid values may appear as `-1` or `101`.

This makes the release a useful candidate for **arrival/dependency/resource-replay research**, not a direct AI model-quality benchmark and not, by itself, a trustworthy GPU/facility-energy dataset. It must not be used to infer GPU wattage or model quality from generic CPU utilization. The planned manifest at `schemas/examples/ai-compute-energy-trace-intake-v1.planned.json` records the source while leaving licensing, exact artifact identity, source columns, and data-quality counters unresolved.

Before retrieval, record the license decision against the exact release and acquisition route. The official page has historically routed downloads through a survey; a public repository page alone does not establish that a mirror, redistributed archive, or transformed copy can be republished. Never commit raw data or credentials as part of the benchmark by default.

### Complementary candidate: Google ClusterData2019 plus power traces

A second, potentially better first **workload-to-power replay** source is the official [Google Borg cluster-data repository](https://github.com/google/cluster-data). Its documentation says that version 3 (`ClusterData2019`) covers eight Borg cells during May 2019 and separately provides power traces for 57 power domains from that same month. It states that the data and documentation are available under CC-BY. See the [official README](https://github.com/google/cluster-data) and [Google's trace overview](https://research.google/blog/yet-more-google-compute-cluster-trace-data/).

The planned manifest `schemas/examples/google-clusterdata2019-power-trace-intake.planned.json` registers this source under `metadata_only`. The presence of power traces does **not** establish that every power domain maps one-to-one to a Borg cell, that the values are metered facility-boundary power, or that the jobs are AI inference. Those semantics must be established from the chosen files and official power documentation before model replay or stronger claims. The source is a priority candidate for general workload/power response analysis, while Alibaba remains a separate candidate for workload arrivals and batch dependencies. Neither alone proves GPU energy per accepted AI token or real grid-service benefit.

The tracking plan for the paired Google workload/power path is [AI-COMP-ENERGY-004 / issue #257](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/257). Its data, license, alignment, and power-attribution gates are intentionally not treated as complete by registering the manifest.

## 3. Required intake receipt

For each source and each materially different transformed dataset:

- **Identity:** publisher, dataset, release, canonical source URI, source kind, artifact state.
- **License:** status, decision, identifier/terms where available, review timestamp and note. An unreviewed source permits metadata-only planning, not dataset use.
- **Immutable bytes:** SHA-256 and byte count for raw archive and normalized output; retrieval timestamp. A normalized artifact additionally requires transformer version, immutable source commit, and configuration digest.
- **Time/unit semantics:** event versus snapshot/interval time, source epoch/time zone, source units, UTC normalization, aggregation interval, and timezone-aware bounds. Do not assume a relative-second timestamp is UTC wall time.
- **Field lineage:** each canonical field maps to an exact source table/column, unit, conversion, status and missingness count. “Planned” mappings carry no observed record counts.
- **Data quality:** read/kept/excluded records, duplicates, missing/out-of-scope timestamps, missingness, and the checks actually executed.
- **Workload quality:** measured, unknown, not applicable, or a separate quality gate required. Generic cluster traces should normally require an independent task-quality benchmark for AI serving claims.
- **Energy attribution:** `metered`, `calibrated_model`, `proxy`, or `unknown`, with boundary, instrument/model and uncertainty. Unknown remains unknown—not zero.
- **Claim policy:** highest evidence-supported claim and explicit forbidden inferences.

The schema rejects unknown fields. Its cross-field rules restrict planned artifacts to metadata-only claims; prevent proxy/unknown energy from supporting metered claims; require normalized artifacts, metered energy, an approved license review with terms, and a non-synthetic source kind for metered-energy claims; and require mapped canonical fields, recorded data-quality checks, a measured quality rule, and facility-boundary metering for quality-accepted work per facility kWh. These schema rules check required declarations and structure, not whether those declarations are truthful; that requires independent evidence review and measurement.

## 3A. Executable schema validation

The CI-only script `tools/validate_ai_compute_trace_intake.py` uses the pinned `jsonschema[format]` validator to validate the schema document against the Draft 2020-12 meta-schema, actively check `date-time` and `uri` formats, accept the planned metadata fixture and a fictional in-memory positive-control object, and reject deliberate mutations that attempt to overstate evidence. It is development tooling only; Sol-Atlas production code remains Rust. The positive control is explicitly fictional and exists only to exercise conditional schema branches—it is not a dataset, meter record, license review or benchmark result. This target-specific test does not replace the JSON Schema project's full language-agnostic conformance suite.

## 4. Fair comparison protocol

### Baselines

Use the same trace slice, allowed deferral windows, deadlines, protected-floor requirements, energy/carbon boundary and hardware assumptions for every policy:

- **B0 — FIFO / earliest-ready:** simple and reproducible, with no claimed look-ahead.
- **B1 — capacity-aware greedy:** respects interval headroom and hard floors.
- **B2 — objective-aware greedy:** cost or carbon ranking with unknown inputs held out of the relevant objective.
- **B3 — independent optimizer:** separately implemented and version-pinned LP/MILP/CP-SAT or another suitable solver, reporting solver version, gap, runtime, stop condition and infeasibility diagnostics.
- **Oracle — exhaustive enumeration:** mandatory for small generated cases in domains where the exact solution can be enumerated.

The three policies in evaluator v0.1 are candidate baselines, not a global Pareto optimum. On small supported cases, both objective schedules must agree with an independent exact oracle; on larger cases report solver gaps rather than describing a heuristic as optimal.

### Metrics

Keep each dimension visible instead of hiding trade-offs in one weighted “green AI” score:

| Dimension | Report | Guardrail |
|---|---|---|
| Useful work | requests/tokens/jobs that pass a named quality gate per IT kWh and facility kWh | Same model, dataset, precision, concurrency/SLO, and quality criterion |
| Serving quality | p50/p90/p95/p99 latency where appropriate; TTFT, inter-token latency, throughput, deadline misses | Never count rejected/low-quality output as accepted work |
| Protected service | per-interval floor violations, unserved protected work, recovery time | Target zero violations; violations cannot be offset by carbon or cost gains |
| Grid interface | peak import, ramp rate, hours above headroom, response time/duration, rebound after deferral | Compare against a declared passive-load counterfactual and relevant grid constraints |
| Energy and carbon | wall-metered joules/kWh, measurement boundary, grid-carbon factor and vintage, location/time granularity | Do not label provisioned-power estimates as metered energy or annual procurement as hourly deliverability |
| Water and heat | water by source and local stress context; heat actually delivered to an available customer; auxiliary energy | No credit for theoretical heat, unavailable customers, or unknown water factors |
| Robustness | results under outage, derating, missing-data, weather/resource and correlated-failure scenarios | Publish failed/unsupported cases and uncertainty, not only successful runs |
| Economics | energy cost, demand charges if included, interconnection/upgrades where modeled | State tariff vintage, counterfactual, and cost boundary |

The relevant quality-throughput frontier should remain comparable to the [MLPerf Endpoints rules and metrics](https://docs.mlcommons.org/endpoints-submission-docs/understand/what-is-mlperf-endpoints/). Its v1.0 `system_tps_per_kw` normalizes throughput by **provisioned power**, derived from system specifications and an overhead model, rather than directly by the power meter reading during a run ([glossary](https://docs.mlcommons.org/endpoints-submission-docs/reference/glossary/)). Preserve that official metric and add wall-metered energy per quality-accepted work as a separate axis; do not relabel either measure.

### Test matrix

At minimum run these paired scenarios, retaining each run's immutable manifest and seed:

1. Baseline load and ordinary grid headroom.
2. Tight headroom with an explicit protected-service floor.
3. Low-carbon hours that are expensive, and cheap hours that are carbon-intensive (objective conflict).
4. Water restriction that becomes binding while grid capacity remains available.
5. Missing/unknown carbon or energy data (no fabricated zero).
6. Workload deadline, DAG dependency and non-interruptible job counterexamples.
7. Outage/derating and rebound after a flexibility window.
8. Out-of-range sentinels, duplicated rows, missing and out-of-order timestamps.
9. Extreme finite values that overflow derived objectives.
10. Multiple sites only after data-transfer capacity, latency, failure-domain and data-residency constraints are modeled.

## 5. Acceptance gates and benchmark targets

These are engineering targets for this project, not claims that an external standard mandates the exact thresholds:

- **G0 — intake integrity:** strict manifest validation, license decision, raw/normalized digests and fully specified transformation lineage before quantitative replay.
- **G1 — deterministic replay:** identical inputs/configuration produce byte-identical canonical outputs or a documented reason that output canonicalization is not stable.
- **G2 — exactness:** 100% agreement with exhaustive enumeration on all small supported instances; zero hard-floor violations.
- **G3 — independent optimizer:** publish objective gap, solve time and failure modes against the exact oracle; candidate target is within 1% of the independently solved objective on larger tractable instances, with exact feasibility still mandatory.
- **G4 — metrology:** report instrument, boundary, sample interval and uncertainty; repeat each hardware condition and disclose run-to-run variation. Do not promote from modeled energy to measured-energy claims without wall-power evidence.
- **G5 — quality-aware efficiency:** demonstrate improvement in quality-accepted work per measured facility kWh against matched hardware/workload baselines, with uncertainty intervals and no service-floor/SLO regression.
- **G6 — system value:** show cost/carbon/peak/flexibility trade-offs against the same-site passive-load counterfactual and disclosed grid model. A scheduler-level reduction alone does not prove avoided generation or transmission capacity.

A credible “better than baseline” result should show a repeatable benefit on a named scenario family without violating hard constraints. A superiority or SOTA claim requires apples-to-apples traces, quality targets, hardware/meter boundaries, baseline implementations, versions, solver status and uncertainty. Otherwise publish a hypothesis and the test needed to falsify it.

## 6. Why location and counterfactual matter

The June 2026 ACM study [*To Defer or To Shift?*](https://doi.org/10.1145/3765611.3815593) reports that flexibility benefits vary with location and grid conditions; more flexibility did not consistently reduce generation-capacity needs, and longer deferral had diminishing returns in its scenarios. The June 2026 perspective [*A net-grid-benefit test for interconnecting AI data centres*](https://www.nature.com/articles/s44432-026-00013-5) proposes auditable grid contributions against a passive-load counterfactual. The latter is a proposal, not a universal adopted standard. Together they motivate measuring location-, time-, response- and rebound-specific results rather than assigning a general “flexible load” credit.

## 7. Scope boundaries

This protocol does not itself download a trace, grant rights to use a dataset, establish ground-truth energy, schedule real infrastructure, solve AC power flow, simulate thermal-fluid dynamics, or verify physical grid services. Those require separate permissions, measurements and qualified tools. Storage dispatch, multi-site optimization, and production control remain unsupported until their state, constraints, recovery, uncertainty and failure semantics are implemented and independently tested.
