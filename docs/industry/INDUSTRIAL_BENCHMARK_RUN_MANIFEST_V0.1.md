# Industrial Benchmark Run Manifest v0.1

**Status:** Contract/schema draft. The included profile is synthetic fixture data, not an executed benchmark.  
**Parent:** [IBAP-RUN-001 / #338](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/338)  
**Related:** [Industrial Benchmark Assurance Protocol v0.1](INDUSTRIAL_BENCHMARK_ASSURANCE_PROTOCOL_V0.1.md), [IEAC v0.1](INDUSTRY_ENGINEERING_ASSURANCE_CONTRACT_V0.1.md).

## 1. Decision

Every benchmark comparison must begin with a frozen, machine-readable run manifest. The manifest pins the task, estimand, system boundary, data, split, methods, budgets, metrics, protected constraints, environment, replication and independent-evaluation plan before results exist.

The manifest is a **pre-run contract**, not a result record. Results, logs, execution failures and independent evaluation are stored as separate immutable artifacts. A result must never edit the manifest that authorized its run.

Schema: [industrial-benchmark-run-manifest-v0.1.schema.json](../../schemas/industrial-benchmark-run-manifest-v0.1.schema.json)  
Synthetic example: [industrial-benchmark-run-manifest-synthetic-v0.1.json](../../schemas/examples/industrial-benchmark-run-manifest-synthetic-v0.1.json)

## 2. The paradigm shift

Do not benchmark an algorithm in isolation and then infer industrial value from a score.

Freeze an **experiment contract** that states what decision is being evaluated, what counts as a good outcome, what information and resources each method can use, which conditions are held out, what cannot be traded away, and what independent implementation will recompute.

This creates three distinct records:

1. **Run manifest — before execution.** Pre-register the comparison and freeze its identities and constraints.
2. **Execution receipt — after execution.** Record actual inputs, executable digests, runtime environment, resource use, exit/status semantics, logs, outputs, exclusions and raw result artifact digests. Never change the manifest.
3. **Independent evaluation report — after verification.** Recompute eligible metrics and constraints from raw artifacts without calling candidate-selection or candidate-optimization logic. Record discrepancies and the claim ceiling.

The v0.1 deliverable defines the first record and its schema. The receipt and evaluation-report schemas are subsequent work, but their identity boundary is mandatory now.

## 3. Freeze protocol and identity

Before any tuning or evaluation:

- Resolve every dataset/source to an immutable version, retrieval snapshot or exact Git commit.
- Compute SHA-256 over the exact source bytes used by the run. A URL or release label alone is not an identity.
- Pin each method's source/model artifact and configuration digest, plus solver/runtime version.
- Freeze the split manifest and the leakage audit.
- Declare the estimand, metric definitions, units, denominators, aggregations and subgroup axes.
- Declare equal-access rules and all deviations for information, tuning, compute, physical resources and time.
- Declare non-compensable constraints before inspecting candidate results.
- Freeze the independent verifier identity and the outputs it will independently reconstruct.
- Record the preregistration timestamp and reviewers before execution begins.

Use RFC 8785 JSON Canonicalization Scheme (JCS) to derive a stable canonical representation before hashing/signing a JSON manifest: https://www.rfc-editor.org/rfc/rfc8785.html. Produce a **detached freeze receipt** containing the manifest SHA-256, canonicalization algorithm/version, schema digest, frozen timestamp and reviewer signature/identity. Keep the digest outside the manifest so the manifest does not contain a self-referential hash.

A raw artifact digest proves byte identity, not publisher authenticity, data rights, scientific validity or independence. Those require separate evidence and review.

## 4. The baseline ladder

Where the task supports them, include distinct method roles:

- **Null baseline:** persistence, mean/constant, no intervention, or simple balance.
- **Incumbent:** the current operator rule, work practice or established production policy. It must be represented faithfully, not weakened.
- **Domain reference:** a specialist simulator, solver or model configured for the same formulation and boundaries.
- **Candidate:** the system under evaluation.
- **Bounded exact oracle:** exhaustive solution for a declared small problem. It is a bound for those instances, not an industrial competitor.

Do not force two tools into an incompatible formulation. Declare `not_comparable` when tasks, system boundaries or inputs cannot be matched. A solver's status, a predictor score or a synthetic objective value is not a complete industrial outcome.

## 5. Comparability is explicit

The manifest records `matched`, `declared_deviation` or `not_comparable`.

A `matched` comparison requires the task boundary, data access, information access, tuning budget, compute budget and physical resource budget to be matched. The schema makes these declarations explicit; semantic validation must check the methods and actual run receipts rather than trust booleans alone.

A `declared_deviation` requires a reviewer, a reason and a complete deviation list. Report the comparison as qualified only for the narrower claim that remains fair. An unreviewed or material deviation bars an unqualified superiority claim.

A `not_comparable` task has no winner. Preserve independent task results without a fabricated ranking.

## 6. Split and leakage contract

Random-row splits are inadequate when windows or samples from the same asset, machine, survey, site or campaign are correlated.

Select split axes that match the intended transfer claim:

- Machine/asset holdout for equipment generalization.
- Site/topology holdout for infrastructure transfer.
- Season/crop/geography holdout for agricultural transfer.
- Process-family and operating-mode holdouts for process engineering.
- Campaign/study holdout for repeated campaigns and field trials.
- Forward-time and pre-registered prospective holdouts for temporal claims.

The frozen split manifest must identify membership and carry a digest. An independent leakage audit must record the axes checked, overlap count and audit artifact digest. A pass requires zero overlap on every declared group axis. Holdout membership must be locked before candidate tuning.

## 7. Budgets and metrics

Record per-method wall time, CPU core-seconds, peak memory and computational energy. Record physical resource budgets separately: water, process energy, crew hours, spare parts, throughput horizon, inventory or other task-specific resources.

Compute energy is not the same quantity as industrial process energy. Never subtract one from the other or present compute efficiency as physical process savings.

Every metric pins:
- stable metric ID and definition/version;
- unit and direction;
- denominator and exclusions;
- aggregation and replication level;
- subgroup axes and worst-group reporting.

For stochastic methods, freeze the seed/replication protocol and report between-run variability. For deterministic methods, state whether byte-identical replay is expected. Timeouts and unknown solver states remain exact statuses; do not relabel them as optimal or successful.

## 8. Protected constraints are not weights

Safety, essential service, product quality, environmental and legal/privacy constraints are declared separately from optimization metrics.

A weighted score must not compensate for violating a hard floor. The evaluator first filters infeasible options; only then does it compare objectives among the remaining feasible candidates.

When a category is genuinely not applicable, record a reviewer and rationale. It is not permissible to omit a difficult constraint merely because the candidate fails it.

## 9. Independent evaluation

The verifier must reconstruct declared outputs from raw artifacts and pinned task semantics without invoking the candidate selector or optimizer.

Depending on the task, recompute:
- objective values and denominators;
- constraint and service-floor compliance;
- confusion counts, detection delay and false-alarm exposure;
- interval coverage and proper uncertainty scores;
- event/maintenance timeline outcomes;
- resource utilization and economic outcomes;
- subgroup and worst-group metrics;
- run-status and exclusion semantics.

Different executable binaries do not by themselves establish scientific independence. Record implementation separation, evidence independence and reviewer independence as distinct claims; IEAC/Mycelix's evidence-independence profile remains the owner for the latter.

## 10. Vertical benchmark mappings

| Vertical | External benchmark/reference lane | Transfer and outcome focus | Important limit |
|---|---|---|---|
| Maintenance | PHM Society challenge and datasets; NASA PCoE; DCASE 2026 machine-anomaly task; NIST SimPROCESD/Simantha for matching discrete-event production policy tasks | Machine/campaign holdouts, low-severity faults, lead time, calibration, false alarms, good output, crew burden and downtime | PHM/DCASE leaderboard scores are task-specific; SimPROCESD is early development and does not represent every factory |
| Process engineering | Tennessee Eastman and other named process benchmarks; independent simulator/reference cases | Operating-mode and process-run holdouts, lower fault magnitudes, detection delay, compute cost and fault-specific performance | Simulator success is not plant validation |
| Telecom | ITU-R propagation tasks; ns-3/5G-LENA packet/network simulation; declared topology/power-autonomy cases; HIL where available | Site/topology/time holdouts, outage duration, recovery, QoS, essential service, common-cause failures | RF propagation, packet network, backhaul, power and HIL are separate task lanes |
| Water–food–energy | EPANET hydraulic cases; AquaCrop field-scale crop-water response; WaPOR observations | Site/season/crop holdouts, hydraulic error, water per accepted output, yield/quality preservation, pumping energy | Specialist tools have different spatial/physical scopes; do not rank unlike tasks |
| Buildings | DOE BOPTEST | Building/climate holdouts, comfort, energy, peak demand and fault detection | Simulation and KPI agreement do not establish field readiness |

## 11. Current benchmark research implications

- The PHM Society 2026 challenge publishes separate test and validation leaderboards. This is useful as an external prognostics task lane, but its score cannot establish maintenance-policy value or field reliability: https://data.phmsociety.org/phm-2026-data-challenge-submission-area/
- A 2026 systematic review/meta-analysis of Tennessee Eastman fault-detection studies identifies selective reporting and evaluation protocol as important evidence-quality issues. It recommends leakage-free run partitions, more challenging low-magnitude faults, comparable reporting, detection delay and compute cost: https://www.mdpi.com/2504-4990/8/11/349
- DCASE 2026 uses development and evaluation machine types that differ, supporting explicit machine-type transfer rather than only random recording-window splits: https://dcase.community/challenge2026/task-first-shot-unsupervised-anomalous-sound-detection-for-machine-condition-monitoring
- NIST says SimPROCESD is early development and may change incompatibly; it supports bounded maintenance-policy and finite-buffer manufacturing experiments, not universal manufacturing claims: https://www.nist.gov/services-resources/software/simprocesd
- NIST's 2026 UQ report recommends scaling credibility assessment by model influence and decision consequence: https://www.nist.gov/publications/methods-uncertainty-quantification-computational-modeling-and-simulation-harmonizing

The run manifest therefore treats benchmarks as task-specific evidence packages and refuses a universal score across incompatible physical and operational tasks.

## 12. Acceptance targets

These are proposed internal gates, not achieved results or universal industry standards:

- 100% of runs accepted by the harness bind one frozen manifest digest, exact source snapshots and exact method/configuration artifacts.
- 100% of reported metrics have pre-registered units, denominators, aggregations and exclusions.
- 100% of decision comparisons bind budgets and non-compensable constraints before execution.
- Zero cross-split overlap on all declared holdout group axes.
- Zero unreviewed substitutions after freeze; a changed dataset/model/configuration requires a new manifest.
- 100% of frozen negative controls (baseline substitution, missing data rights, split leakage, hidden metric denominator, unreviewed budget deviation, verifier reuse and protected-floor removal) reject or become explicitly indeterminate.
- 100% of independently recomputable metrics agree within predeclared tolerances or the report records a discrepancy.
- No superiority/SOTA claim where comparability is `not_comparable`, a material deviation is unreviewed, a protected floor fails, or the independent evaluation is absent.
- No production, safety, regulatory, field-validity or service-level claim from synthetic fixtures or a public leaderboard score alone.

## 13. Claim ceiling and next tranche

Schema conformance proves only that the pre-run manifest conforms to the wire contract. It does not establish that dataset rights are valid, that split membership is honest, that comparisons are scientifically fair, or that model outputs are correct.

Next implementation tranche:
1. Add a deterministic Rust manifest parser/semantic validator and strict JSON Schema 2020-12 evaluator.
2. Add mutation tests for every post-freeze substitution and comparability failure.
3. Add a detached freeze receipt with JCS/SHA-256 and reviewer identity.
4. Add a post-run execution receipt schema.
5. Add an independent evaluation report schema and verifier that never calls candidate decision code.
6. Wire exact-head CI to check schema meta-validation, positive and negative fixtures, lockfile integrity and frozen digest stability.

This work composes IEAC, existing artifact-chain and evidence-independence systems. It does not create a new provenance graph, capability ontology, solver or universal benchmark metric.
