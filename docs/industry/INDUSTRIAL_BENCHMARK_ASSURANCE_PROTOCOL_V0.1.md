# Industrial Benchmark Assurance Protocol v0.1

**Status:** Research protocol; targets are proposed, not achieved results.  
**Related:** [IEAC v0.1](INDUSTRY_ENGINEERING_ASSURANCE_CONTRACT_V0.1.md) and [IND-ASSURANCE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/327).

## 1. Purpose

Create a fair, reproducible way to compare Luminous industrial capabilities with established specialist tools and incumbent operating policies—without treating unlike tasks as one leaderboard.

The programme has two separate questions:

1. **Model credibility:** does the physical, statistical or operational model reproduce the behaviour it claims to represent within a declared validity domain?
2. **Decision value:** does using the model improve a real decision under the same information, time, compute, cost, safety and resource constraints as the incumbent?

A result may pass the first and fail the second. A highly accurate model may not improve outcomes, and a useful decision policy does not make an invalid physical model acceptable.

## 2. Required baseline ladder

Each benchmark task must include, where applicable:

1. **Null/trivial baseline:** persistence, constant/mean, no intervention, or simple physical balance.
2. **Incumbent baseline:** current operator procedure, named industrial rule, spreadsheet, or published policy. Describe it faithfully; do not construct a straw-man.
3. **Established domain reference:** e.g. EPANET for matched hydraulic cases, AquaCrop for suitable crop-water cases, NIST SimPROCESD/Simantha for appropriate manufacturing DES tasks.
4. **Candidate system:** the Luminous model or decision policy.
5. **Bounded exact oracle:** for small cases, enumerate all feasible solutions and independently verify feasibility/optimality. An oracle is an upper/lower bound for that bounded problem, not a deployable competitor.
6. **Independent verifier:** a separate implementation reconstructs constraints, selected outputs and reported metrics without invoking the candidate selector.

If a valid domain reference cannot be configured to the same task and boundary conditions, label the comparison `not_comparable` rather than manufacturing a ranking.

## 3. Freeze comparability before running

Every benchmark manifest must pin:

- task definition, estimand and intended decision;
- input corpus, source rights, data dictionary and raw-byte digest;
- asset/site/group/time split manifest and leakage audit;
- operating conditions, unit conversions and spatial/temporal support;
- reference solver/tool, exact version, configuration and executable identity;
- candidate model, training/tuning protocol and hyperparameters;
- hardware, compiler/runtime, thread/GPU policy and numerical tolerances;
- seeds or stochastic replication protocol;
- wall time, memory and computational energy budget;
- physical resource budget (e.g. water, energy, crew hours) where decision value is measured;
- output schema, metric definitions, denominators and exclusion rules;
- required hard service, safety, product-quality and environmental floors.

All candidate/baseline methods receive equal information, tuning opportunity, compute budget and resource constraints unless a difference is part of the declared experiment.

## 4. Separate benchmark lanes

### A. Numerical and physical verification

Use analytical known answers, conservation identities, reference solutions, mesh/time-step convergence, boundary-condition checks and metamorphic relations.

Report normalized bias/error, conservation residuals, convergence order where meaningful, numerical stability and independent-reference discrepancy. Never present a unit test as physical validation.

### B. Physical validation and uncertainty

Use measurements with known instrument, calibration, method, sample, unit and time support. Distinguish calibration data from held-out validation.

Report:
- bias, MAE/RMSE or task-appropriate normalized error;
- interval coverage at predeclared nominal levels;
- mean interval width and proper interval score;
- calibration by site, asset, operating regime and subgroup;
- parameter uncertainty and model-form uncertainty separately;
- sensitivity to boundary and measurement uncertainty.

Do not rely on one aggregate metric where a subgroup or worst-case regime can fail.

### C. Generalization and shift

Use held-out machines, tools, sites, networks, farms, seasons, batches, process families or geographic regions. Group related records before splitting. Random-row splits are diagnostic only when records from one asset or operating campaign are correlated.

Report per-group and worst-group performance, not only pooled averages.

### D. Abstention and validity-domain behaviour

Inject:
- missing, stale and out-of-calibration sensors;
- corrupted units, values and source digests;
- unknown network dependencies;
- unseen topology and process regimes;
- population/geographic shift;
- conflicting observations;
- invalid or withdrawn data permission.

Measure selective risk versus coverage, abstention precision/recall, false-confident rate, out-of-domain detection and the value/cost of requested additional measurements.

### E. Decision quality

Compare against named incumbent policies and bounded exact oracles using matched budgets.

Report:
- constrained decision regret;
- value of information;
- robust service/quality-floor compliance;
- good output/service rather than gross throughput alone;
- downtime, repair/crew burden, stockouts or missed demand;
- energy, water, materials and consumables per accepted unit/service;
- lifecycle cost and resource/cost Pareto frontier;
- intervention stability under model/parameter/scenario uncertainty.

Never collapse protected service, safety, quality or environmental constraints into a weighted score. First identify feasible alternatives; only then compare preferences among them.

### F. Operational readiness

Assess human review time, override frequency, workflow completion, observability, alarm load, access control, recovery and rollback. A good offline model does not establish that a plant, utility or farm can operate it.

Prospective shadow evaluation precedes any action authority. Real operational value requires outcome reconciliation against a predeclared comparator.

### G. Computational and environmental cost

Report compute runtime, peak memory and computational energy separately from the physical process energy savings being claimed. State the hardware and measurement boundary. Compare end-to-end workflow cost, not just a fast inner kernel.

## 5. Three flagship reference matrices

| Vertical | Specialist/reference tools | Core benchmark tasks | Decision outcomes |
|---|---|---|---|
| Water–food–energy | EPA EPANET; FAO AquaCrop; FAO WaPOR; authorized local measurements | Hydraulic known answers, water balance, crop-water response, seasonal/site holdout, water-energy coupling | Water per accepted output, yield/quality preservation, pumping energy, service floors, decision regret |
| Industrial maintenance | NIST SimPROCESD / Simantha; NASA PCoE C-MAPSS; PHM Society datasets; real plant telemetry only with permission | Time-to-failure/prognostics, calibration, machine-disjoint holdout, crew-constrained maintenance DES, finite-buffer parity | Good units, downtime, false alarms, missed actionable lead time, crew utilization, cost and quality |
| Telecom resilience | Declared capacity topology; later ITU-R P.1812-8 for RF, 5G-LENA/ns-3 for network simulation, Colosseum for HIL | Reachability/capacity, power autonomy, common-cause backhaul stress, separate RF/5G/HIL lanes | Essential service floors, outage duration, recovery, capex/opex trade-off, measured QoS |

The tools in a row are not all interchangeable. Each is used only for its declared purpose and matched input domain.

## 6. How to make a defensible SOTA claim

A claim of superiority must specify:

- task and dataset version;
- reference methods, exact versions and configurations;
- frozen test split and whether results are independent;
- hardware and equal compute/tuning budget;
- metric, denominator and statistical uncertainty;
- subgroup and worst-case results;
- hard-constraint violations;
- independent verifier and its separation from candidate logic;
- limitations and the claim ceiling.

A model is not better because it has a larger score on one synthetic corpus. A decision policy is not better if it violates a protected floor. An exact oracle is not a fair competitor outside the problem size it exhaustively solves. A cross-domain result is not SOTA without independent site/topology/season/asset transfer.

Prefer paired comparisons, pre-registered hypotheses and confidence intervals. When results are not statistically or practically distinguishable, report a tie or inconclusive outcome.

## 7. Proposed acceptance targets

These are internal engineering targets for declared fixtures and trial panels, not universal industry standards:

| Target | Proposed gate |
|---|---|
| Known-answer panel | 100% of frozen analytical fixtures pass within predeclared tolerances |
| Negative controls | 100% of seeded false-promotion/invalid-input controls reject or return indeterminate |
| Evidence traceability | 100% of reported outputs map to exact input, model and evaluator identities |
| Deterministic replay | Byte-identical output for the same deterministic input/configuration/executable where deterministic execution is specified |
| Split integrity | Zero asset/site/study/group overlap across held-out train/test groups |
| Hard constraints | Zero reported candidate passes that violate declared protected floors |
| Independent verification | 100% agreement on the independently checkable outputs of bounded fixtures |
| Uncertainty | Report empirical coverage and proper interval score; do not impose one universal coverage number across different tasks |
| Decision value | Demonstrate lower regret or greater good-output/service at equal budgets without worsening any protected constraint |
| Transfer | Report leave-one-site/asset/topology/season results and worst-group errors; no random-row-only SOTA claim |

## 8. Research and standards references

- NIST AMS 100-85 (5 October 2026), *On Methods of Uncertainty Quantification in Computational Modeling and Simulation: Harmonizing Modeling and Metrology Practices*: explicitly discusses risk-based VVUQ scaled by model influence and decision consequence. https://www.nist.gov/publications/methods-uncertainty-quantification-computational-modeling-and-simulation-harmonizing
- ASME's published VVUQ portfolio includes VVUQ 1-2022 terminology, VVUQ 20.1-2024 multivariate validation, VVUQ 40-2018 for medical-device model credibility and VVUQ 60.1-2025 on selecting computational physics software. https://www.asme.org/codes-standards/vvuq-standards
- ISO/AWI 23247-8 was approved and registered as a work item on 23 October 2026 and is under development; it is not yet a published standard. https://committee.iso.org/standard/94858.html
- W3C PROV-O provides existing provenance vocabulary. https://www.w3.org/TR/prov-o/
- OPC UA Asset Administration Shell provides standardized asset representations and submodels as an interoperability option where relevant. https://reference.opcfoundation.org/specs/OPC-30270/4.1

## 9. Execution contract

The protocol is instantiated by the [Industrial Benchmark Run Manifest v0.1](INDUSTRIAL_BENCHMARK_RUN_MANIFEST_V0.1.md) and its [JSON Schema](../../schemas/industrial-benchmark-run-manifest-v0.1.schema.json).

The manifest freezes the task, estimand, source dataset bytes, split, method versions, equal-access budgets, metric denominators, protected constraints, replication plan and independent-verifier identity **before execution**. It is a pre-run contract and contains no benchmark result fields.

A post-run execution receipt and independent evaluation report are separate immutable artifacts. Neither may rewrite the frozen manifest. Comparability is explicitly `matched`, `declared_deviation` or `not_comparable`; the last state has no leaderboard winner. A detached freeze receipt binds the canonical manifest digest and reviewer identity.

The checked-in example is synthetic contract data only. Its placeholder digests are not evidence of acquired datasets or completed runs.

## 10. Release rule

A benchmark result can advance a specific evidence dimension for a specific profile. It does not automatically advance every dimension, grant authority, certify compliance or establish production readiness.

The report must retain any failing, unknown or conflicted dimension. A benchmark run is evidence; qualification is a separate, reviewable decision.
