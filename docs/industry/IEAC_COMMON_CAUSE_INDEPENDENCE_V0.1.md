# IEAC common-cause-aware evidence independence v0.1

**Status:** Synthetic registry-backed dependency model and independent-set auditor. This is a deterministic admission layer over a declared dependency registry, not a probabilistic reliability model and not proof that the registry is complete.  
**Tracks:** [IEAC-CCF-001 / #406](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/406).  
**Stacked on:** [evidence-acquisition planner #402](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/402), which currently counts distinct declared source-domain IDs only.

## 1. Why unique organization IDs are not independence

Two different companies, labs or vendor names do not establish independent evidence. Their measurement paths may still share:
- a calibration chain or reference standard;
- an upstream sensor feed or data service;
- firmware, software build or test harness;
- operator, reviewer, procedure or sign-off authority;
- facility, power supply, communications or maintenance process;
- environmental or site hazards.

The reliability-engineering literature has treated common-cause and dependent failures for decades. The NRC/INL CCF database classifies common-cause events and groups components with shared mission, manufacturer, maintenance or environmental characteristics. It distinguishes β-factor, α-factor and other parameterized CCF models. The General Dependency Model (GDM) proposed by O'Connor and Mosleh uses Bayesian networks to represent causal dependency and defenses, addressing limits of classic models that focus on identical redundant components.

Useful references:
- Vaurio (2008), *Common Cause Failure Modeling*, DOI: https://doi.org/10.1002/9780470061596.risk0497
- O'Connor & Mosleh (2016), *A general cause based methodology for analysis of common cause and dependent failures in system risk and reliability assessments*, DOI: https://doi.org/10.1016/j.ress.2015.06.007
- NRC/INL, NUREG/CR-6268 Rev. 1 (2007), *Common-Cause Failure Database and Analysis System: Event Data Collection, Classification, and Coding*: https://www.nrc.gov/regulations-legislation/nureg-series-publications/publications-prepared-by-nrc-contractors/cr6268
- NRC PRA Methods Database, which lists NUREG/CR-5485, NUREG/CR-5497, NUREG/CR-6268, the INL/EXT-21-62940 2020 CCF parameter-estimate update and EPRI inter-system CCF work: https://prastandards.sandia.gov/pra-methods-report/
- INL/NRC CCF parameter estimates 2020 update: https://nrcoe.inl.gov/publicdocs/CCF/Causalccfparamest2020.pdf

The β-factor and α-factor models quantify probabilistic common-cause failure under their assumptions; GDM models probabilistic causal dependencies. This v0.1 graph does **not** replace those methods, estimate failure probabilities or claim a novel reliability theory. It makes shared-source relationships explicit in the evidence-admission contract before probabilistic reliability analysis or assurance aggregation.

## 2. Paradigm shift

Replace the v0.1 rule:

`count(distinct independence_domain_id) >= k`

with:

`source domain → dependency closure → shared cause groups → conflict graph → maximum independent evidence set`

A source is credited as independent only when its declared dependency closure and common-cause group membership are disjoint from the other credited sources under one pinned, current registry model.

This changes the question from “How many vendor IDs do we have?” to “How many evidence paths remain pairwise disjoint under the declared causal/dependency model?”

## 3. Registry contract

The versioned Draft 2020-12 registry contains:

- `common_cause_groups`: stable group IDs and typed cause classes (upstream feed, calibration lineage, reference standard, software build, test harness, operator process, facility, power, communications, shared environment, maintenance, other);
- `dependency_nodes`: active/revoked dependency objects with typed classes and common-cause group membership;
- `dependency_edges`: directed edges `dependency_id → depends_on_dependency_id`, which define transitive dependency closure;
- `domains`: domain identity, organization identity, active/revoked status, dependency-completeness status, direct dependency IDs, direct common-cause groups and validity interval;
- registry ID/version, authority ID, issued time and validity interval.

The request binds the exact registry digest. v0.1 computes this over compact `serde_json` serialization of the registry JSON value (object keys are serialized in sorted order; array order remains significant). This is a deterministic byte-binding profile, not RFC 8785 JCS, and the digest is not a signature.

**Authentication boundary:** the auditor checks registry identity, digest, schema semantics, validity and internal graph consistency. It does not authenticate the registry authority, its key, its signature, the truth of externally supplied dependency declarations or the completeness of the dependency inventory. Those assertions require an upstream trust-root/signature verifier and accountable registry governance.

## 4. Conflict semantics

For each source, the auditor resolves:
- its source domain;
- the transitive dependency closure, including the domain's direct dependency nodes and all recursively referenced dependencies;
- the union of direct domain common-cause groups and all groups on nodes in that closure;
- dependency-completeness state.

Two source vertices conflict when one or more of the following holds:
1. They claim the same source-domain ID.
2. They claim the same source artifact/action identity digest.
3. Their transitive dependency closures intersect.
4. Their common-cause group sets intersect.
5. Either domain has unknown or disputed dependency completeness.

The report preserves every conflict edge and its reason codes, shared dependency IDs and shared group IDs. Distinct organization names or source-domain IDs do not override a shared dependency or common-cause group.

The registry graph must have unique nodes/groups/domains, known edge endpoints and no self-edge or cycle. Every active dependency node is checked, including nodes that are not reached by the fixture's current evidence sources. A source referring to a revoked/inactive domain, a revoked dependency in its closure, an expired domain, stale registry or registry-digest mismatch is rejected.

### Unknown and disputed dependencies

Unknown/disputed closure is conservative. Such a source conflicts with every other source vertex, because the auditor cannot establish that the dependencies are disjoint. A single such source may remain a singleton candidate, but it cannot increase an independent set together with another source.

If an exact calculation fails the required threshold and unresolved dependency sources are present, the disposition is `indeterminate_under_registry_model`, not `not_qualified_by_registry_model`. The report makes the uncertainty explicit rather than turning a missing dependency inventory into evidence of independence.

## 5. Source receipt contract

Each candidate source binds:
- stable `source_id`;
- control ID;
- source kind (`existing_evidence` or `planned_action`);
- source artifact/action SHA-256 identity;
- registry source-domain ID;
- evidence state (`verified` for existing evidence, `planned` for a not-yet-executed action);
- rights status, rights-evidence digest, verification time and valid-until.

Only current, rights-verified sources are accepted. Existing evidence and planned actions remain distinguishable in the report. A planned action is not evidence; the auditor only checks the dependency independence of the proposed source path. The source artifact/action and rights receipts are not cryptographically authenticated by this package; upstream verifiers must establish those facts.

## 6. Independent-set algorithm

For each mandatory control, the auditor:
1. gathers that control's candidate source vertices;
2. restricts the global conflict graph to those vertices;
3. solves for the maximum pairwise-independent source subset;
4. compares its cardinality with the control's required minimum;
5. reports the selected source IDs, conflict pairs, exactness and disposition.

For at most 20 candidate sources **per control**, the auditor exhaustively enumerates subsets and reports `proven_optimal`. Ties are resolved deterministically by lexicographically smallest sorted source-ID vector.

Above 20 candidates for a control, v0.1 uses a deterministic low-conflict-degree greedy algorithm and reports a feasible independent-set **lower bound** with `lower_bound_not_proven`. It does not call that set a maximum. If the lower bound meets the required threshold, that concrete independent subset is sufficient to show the threshold is met under the registry model. If the lower bound does not meet the threshold, the result is `indeterminate_under_registry_model`, not a proof that no larger set exists.

## 7. Frozen synthetic fixture

The committed fixture has:
- 7 distinct source domains from 7 nominal organizations;
- 7 typed dependency nodes;
- a transitive calibration-chain → reference-standard dependency;
- a shared upstream sensor feed and operator group;
- a shared production test harness and regional power dependency;
- one source domain with unknown dependency completeness;
- 2 mandatory controls with minimum independent source count 2;
- a mix of existing verified evidence and planned acquisition actions.

Hand-calculated expected sets:
- `numerical_known_answers`: `num-alpha` + `num-gamma`; `num-alpha` and `num-beta` conflict through transitive calibration/reference and upstream feed/operator dependencies.
- `physical_reference_validation`: `physical-delta` + `physical-zeta`; `physical-delta` and `physical-epsilon` conflict through shared test harness and power supply.

These are synthetic conformance vectors, not physical test results, production CCF probabilities or evidence about actual vendors.

## 8. Benchmark programme and promotion targets

### Frozen test matrix
- direct and transitive shared dependency edges;
- common-cause group overlap without a shared dependency node;
- same artifact digest presented under two different domains;
- same domain ID represented by multiple source rows;
- distinct organizations with shared upstream feed;
- separate labs sharing calibration/reference chain;
- separate labs sharing test harness/firmware and power/site dependencies;
- unknown and disputed dependency completeness;
- revoked domain, revoked dependency node and expired registry/domain;
- unknown edge endpoint, duplicate edge, self-edge and dependency cycle;
- registry digest mismatch and duplicate JSON members;
- exact graph instances at 5/10/15/20 sources per control;
- more than 20 sources with a deterministic lower-bound result;
- source thresholds that pass, fail exactly, or remain indeterminate.

### Targets
- Zero false independence credit on frozen common-cause cases.
- 100% agreement with an independent brute-force maximum-independent-set oracle for bounded cases.
- 100% detection of shared dependency/group overlaps, including transitive closures.
- Zero credit to sources with unknown/disputed dependency membership when paired with another source.
- Zero acceptance of revoked domains/dependencies, cycles, stale registry or digest substitution.
- Exact method explicitly distinguished from lower-bound heuristic.
- Zero reports that authorize actions or attest profile qualification.
- Measure conservative false rejection separately from false independence credit.

For larger cases, add an external SAT/MaxSAT or MILP oracle and publish lower/upper bounds, time-to-first-independent-set and gap; do not conceal unknown optimality.

## 9. Industry comparison

| Method | What it models | What it does not provide by itself |
|---|---|---|
| Unique organization/domain count | Nominal source separation | Shared cause and transitive dependency detection |
| β-factor | Aggregate fraction/rate attributable to common-cause failure, under model assumptions | General evidence lineage or arbitrary heterogeneous dependency graph |
| α-factor / multiple Greek-letter | Multi-component failure combinations in common-cause groups | Evidence rights, source provenance and proof-receipt semantics |
| General Dependency Model / Bayesian network | Causal probabilistic dependency structures and defenses | A deterministic, portable source-independence receipt for every evidence item |
| IEAC CCF graph v0.1 | Deterministic registry-declared dependency closure, shared groups and exact/lower-bound independent source sets | Failure probabilities, registry truth/completeness or unmodeled causes |

The differentiator is the portable assurance contract: source artifact identity, rights receipt, registry version, transitive dependency closure, conflicts and selected independent subset travel with the plan and can be rechecked by an independent auditor. Reliability models should consume these source relationships and add probabilistic quantification where appropriate.

## 10. Claim ceiling and next work

`qualified_by_registry_model` means only that the selected source subset is pairwise disjoint under the exact pinned registry's declared dependency and common-cause model. It does not establish:
- the registry is complete or free from false declarations;
- two organizations have no undiscovered shared people, infrastructure or suppliers;
- physical/environmental causes not represented in the registry do not exist;
- planned tests will pass or existing evidence is true;
- a safety case or IEAC profile is qualified;
- an action or industrial intervention is authorized.

Next: cryptographically authenticate registry versions and source receipts; add a causal/hypergraph dependency representation for common causes not captured by simple shared groups; compare deterministic conflict filtering against β/α-factor and GDM-based probabilistic sensitivity analysis without conflating their semantics; build public, license-respecting common-cause test data from published event classifications; then feed registry-qualified source counts into the proof planner and calendarized RCPSP scheduler.