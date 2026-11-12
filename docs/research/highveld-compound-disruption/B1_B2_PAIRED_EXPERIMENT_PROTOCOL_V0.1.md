# Highveld B1/B2 paired experiment protocol v0.1

**Status:** preregistration draft; synthetic software-audit tooling only.  
**Owner boundary:** Sol-Atlas #735/#736 own the corpus and experiment packet; Symtropy #1848/#1849 own native service trajectories, recovery and Pareto evaluation. Other owners retain their respective model, source, instrument and rights authorities.

## 1. Question and falsifiable claim

Does adding the Luminous evidence-composition layer to the same incumbent specialist-tool outputs reduce critical evidence and decision errors, without changing the underlying engines or hiding extra information, compute or operator effort?

The primary contrast is B1 versus B2:

- **B1:** eligible specialist engines and their documented conventional decision workflow.
- **B2:** the same specialist engines, versions, engine inputs and raw engine outputs, followed by Luminous evidence identity, scope/rights/revision controls, owner-native service binding, scoped invalidation and reproducible receipts.

This isolates whether evidence composition improves cross-tool decision integrity. It does not claim better building physics, superior architectural design or superior physical service outcomes.

## 2. Experimental units and candidates

The first preregistered synthetic round uses the three existing Sol-Atlas #694 candidates:

1. INDEPENDENT_SERVICES
2. SHARED_UTILITY
3. HYBRID_LOCAL_AUTONOMY

Use the seven SYNTHETIC_CORPUS_V0.1.json scenarios for compound traces and the 12 SERVICE_EVENT_HANDOFF_CONTROLS_V0.1.json cases for owner-boundary/adversarial controls. Candidate/scenario/control combinations must be declared before execution. Do not treat the number of synthetic fixtures as a probability distribution.

A fourth, neighborhood-scale candidate is proposed separately in #694 follow-up work. It is not silently included in this v0.1 run or treated as evaluated.

## 3. Blinding and preregistration

Before generating B1/B2 decisions:

1. Freeze candidate designs, resident/task requirements, corpus bytes, scenario/control selection, functional units, time windows, resource budgets, information budgets, tool versions and exclusion rules.
2. Publish a preregistration packet with the full case matrix, oracle authorship/independence statement, scoring rules, stopping rule and planned analyses.
3. Assign opaque reviewer labels to B1/B2 exports. Keep the arm-to-label key outside reviewer packets until decisions, error classifications and operator-time records are frozen.
4. Have the independent oracle author derive expected dispositions from the closed synthetic controls before reviewing either arm's output.
5. Record deviations, missing outputs, retries and exclusions. Do not silently drop difficult cases or change the oracle after unblinding.

The audit CLI is a custodian-side integrity check, not the blinding mechanism. Human review must retain the separate arm key.

## 4. Mandatory paired-input invariants

Every B1/B2 pair must share:

- design revision and geometry digest;
- resident/task requirement digest and per-service floor profile;
- exact event-trace bytes and SHA-256 digest;
- initial-condition digest and boundary-condition revision;
- source snapshot and rights revision;
- resource budget, information budget, functional unit and time window;
- exact specialist tool identities, versions, configurations, licences and exclusions.

B1 and B2 must preserve identical raw specialist-engine input and output digests. The composition layer may change the evidence-qualified decision packet, but must not quietly change EnergyPlus, BOPTEST, Forma, TestFit or another underlying engine's inputs or outputs. Any necessary engine change is a separate experiment.

Record operator minutes, rework minutes, compute, runtime, manual edits, retries and tool availability per arm. Compare human effort only at equivalent decision quality.

## 5. Expected output and independent oracle

For each named service, both arms emit:

- decision state: PASS, FAIL, BLOCKED or UNKNOWN;
- hard-floor state: SATISFIED, VIOLATED, UNKNOWN or NOT_APPLICABLE;
- evidence references and their availability times;
- decision timestamp and information actually available then.

B2 additionally emits the composition-layer name/version, a receipt bound to exact matched inputs and engine I/O, source-rights revision, and per-service evidence validation (rights_state, scope_join_state). The receipt uses format `highveld-b2-composition-receipt-v0.1`; its SHA-256 is recomputed over the fixed field sequence `event_trace_sha256`, `geometry_sha256`, `resident_requirements_sha256`, `initial_state_sha256`, `source_snapshot_sha256`, `engine_input_sha256`, `engine_output_sha256`, `source_rights_revision`, with each UTF-8 value length-prefixed in bytes under the `HIGHVELD_B2_COMPOSITION_RECEIPT_V0.1` domain tag. A changed value with an unchanged digest is rejected.

The independent oracle defines expected decision and hard-floor state per service. B2 must match the oracle exactly on finite synthetic known-answer cases. B1 is measured against the same oracle; its false passes and mismatches are outcomes, not hidden exclusions.

No decision may use information unavailable at decision time. A forecast can describe a future event only when the forecast itself was available at that decision time; future observations cannot be relabelled as forecasts.

## 6. Non-compensatory acceptance gates

These are hard software/contract gates for the synthetic round:

| Measure | Acceptance |
|---|---:|
| Paired geometry, requirements, trace, initial-state and budget identity | 100% exact |
| Underlying engine version/input/output identity across B1 and B2 | 100% exact |
| B2 receipt binding to matched inputs and raw engine I/O | 100% exact |
| B2 oracle agreement on finite synthetic controls | 100% |
| B2 critical false passes | 0 |
| Future-information leaks in either arm | 0 |
| Unqualified/revoked/out-of-scope evidence used to support B2 PASS | 0 |
| Hard-floor violations/unknowns hidden by B2 PASS | 0 |
| Missing, duplicate or unregistered service decisions | 0 |

These are exact synthetic contract targets, not claims that zero physical risk can be demonstrated. B1 is not required to pass; it is the baseline whose observed errors and human effort are compared with B2.

## 7. Secondary measures

Report by candidate, scenario family and evidence-control family:

- B1/B2 exact oracle mismatch counts;
- critical false-pass and false-reject counts, separately;
- false-feasible design decisions after approved higher-fidelity reference;
- scoped invalidation precision/recall against an independent dependency oracle;
- Pareto precision/recall against the exhaustive finite oracle;
- decision/rank reversals between low- and high-fidelity evaluation;
- evidence reconstruction time, human-hours, rework and operator interventions;
- compute/runtime and data-acquisition burden;
- missingness, coverage, applicability and uncertainty for every non-synthetic input.

Do not collapse service availability, repair, equity, cost, carbon, comfort or privacy into one weighted score. Report each resident's declared hard floors first; then report the Pareto set among candidates that satisfy them.

## 8. Multi-fidelity and decision-sufficient measurement

Use staged evaluation:

1. Cheap screening across the full synthetic candidate set.
2. Symtropy's existing decision-impact and adaptive-measurement qualifiers to identify measurements capable of changing robust feasibility, a critical blocker or the Pareto set.
3. Higher-fidelity simulation only for candidates/claims whose decision could change.
4. Physical measurement only after instrument admission, approved reference method, safety review and the unoccupied test-cell gate.

A measurement that cannot change any surviving decision is not automatically useful. Conversely, measurement cannot be skipped while a critical blocker remains unresolved or while the terminal decision changes across surviving states.

## 9. Fourth architecture proposal: neighborhood resilience commons

The proposed NEIGHBORHOOD_RESILIENCE_COMMONS is a system-boundary extension, not another version of local battery autonomy. It introduces an optional shared community layer for resilience services, repair capacity and logistics. It should be evaluated separately from the three dwelling topologies and included in a later experiment only after Sol-Atlas #694 accepts its definition.

The household retains a declared local essential-service floor. The commons may augment it with shared energy, water-access, cooling/clean-air refuge, communications, charging, repair tools, spares and trained support. No resident's critical personal service may depend solely on reaching the hub. A fixed hub, distributed nodes and mobile support can be tested as variants of this one architecture family.

Adversarial cases must include hub outage; islanding/black-start failure; shared plant or controller failure; cyber/network isolation; resource depletion; queue contention; inaccessible travel route; mobility/access needs; unequal allocation; maintenance/spares failure; rights/consent withdrawal; and simultaneous household and hub stress.

Measure per-resident service floors, coverage distance/time, queue/wait, allocation fairness, common-cause exposure, local fallback, shared-resource utilization, lifecycle cost and repair burden. Never assume a community resource is equally accessible merely because it is geographically nearby.

## 10. Industry comparison and standards discipline

Compose specialist tools; do not claim to replace them:

- Autodesk Forma: schematic design and early-stage daylight/site workflows.
- TestFit: rapid site feasibility and configuration.
- EnergyPlus: whole-building energy and thermal simulation.
- BOPTEST: standardized emulator and control/FDD cases.
- ASHRAE Standard 140: established software capability/applicability testing.

Reuse standards and test suites in their proper scope. Add an evidence-chain benchmark on top; do not reimplement or mislabel incumbent capability tests. Pin product versions and public feature claims on execution date. ASHRAE 223P must be described as proposed/in development until finalized.

## 11. Claim ceiling and promotion

A passing paired audit demonstrates that the submitted synthetic run packet satisfies matching, receipt, timing and decision-oracle checks. It does not mean the benchmark actually ran with incumbent tools, that an architecture won, or that a dwelling is resilient.

Empirical promotion requires rights-cleared primary sources with declared geographic/time coverage; source admission through named owners; model-specific VVUQ and held-out reference observations; admitted physical instruments; and an approved, safe, unoccupied physical test. Survey prevalence is not an event trace; system-level utility notices are not household service observations.
