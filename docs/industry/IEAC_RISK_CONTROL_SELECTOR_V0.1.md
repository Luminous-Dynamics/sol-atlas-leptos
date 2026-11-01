# IEAC risk-to-control selector v0.2

**Status:** Pure requirements selector and versioned policy. It does not qualify a profile, attest evidence, promote a decision tier or authorize an action.  
**Tracks:** [IEAC-RISK-CONTROLS-001 / #356](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/356).  
**Composes with:** IEAC profile/schema/validator #327/#331/#335; trust lifecycle #396/#397; outcome receipts #389/#390; cross-layer adversarial benchmark #391/#392.

## 1. Objective and paradigm shift

IEAC already contains six independent risk dimensions: consequence, model influence, exposure, reversibility, detectability and fallback.

The selector makes their implications executable:

risk vector + decision tier + policy version → minimum proof obligations + trigger reasons

It does not compute a weighted score or a new maturity class. A low value on one dimension cannot compensate for a catastrophic or poorly detectable failure on another.

The selector answers **what must be demonstrated and why**. The existing IEAC validator answers whether a profile's evidence, operating envelope, authority and artifact bindings meet the profile contract. These are distinct stages.

## 2. Standards and industry grounding

NIST AMS 100-85 (published 5 October 2026) recommends scaling VVUQ effort to model influence and decision consequence, and reviews variance/distribution propagation and adjoint sensitivity methods. This selector operationalizes the risk-to-proof-obligation part of that direction. It does not prescribe one UQ method for every model: domain and model characteristics still determine whether distributional propagation, sensitivity analysis, empirical calibration or other methods are justified.

The existing IEAC decision tiers preserve a separate authority boundary:
- D0 descriptive;
- D1 advisory;
- D2 delegated planning;
- D3 autonomous control.

A selector result is never permission to take the associated action. The D3 tier continues to require its separate safety case, independent assessment, operational authorization, fallback and security evidence; a D3 label or a larger requirement list is not an automatic approval.

## 3. Policy model

The versioned policy is schemas/ieac-risk-control-policy-v0.2.json. Its strict Draft 2020-12 schema is schemas/ieac-risk-control-policy-v0.2.schema.json.

The policy contains:
- five baseline controls;
- cumulative D0-D3 minimum control sets;
- reviewed risk-class overlays;
- per-value rules for each of the six dimensions;
- four explicit cross-dimensional interaction rules;
- a stable catalog of 38 controls;
- an explicit monotonicity order for all six dimensions.

Every control has a stable control_id, description, mapped IEAC evidence dimension, human-approval requirement and operational-obligation tags.

The control IDs are policy vocabulary, not a claim that the corresponding activity has been completed. Their evidence mapping makes the requirement traceable into the existing ten-dimensional IEAC evidence vector.

### 3.1 Baseline and tier controls

Every tier inherits the five baseline controls:
1. artifact identity and integrity;
2. schema/interface conformance;
3. source rights and lineage;
4. numerical known answers;
5. unit and semantic validation.

D1 adds physical reference validation, uncertainty calibration, out-of-domain abstention, accountable human review, operational monitoring and prospective outcome reconciliation.

D2 is cumulative over D1. It adds independent constraint reconstruction, group-disjoint holdout, compound-fault stress, rollback/recovery, dual-control authorization, shadow/HIL trial and tested fallback.

D3 is cumulative over D2. It adds independent safety case, hazard analysis, independent safety assessment, secure control boundary, HIL qualification, fail-safe/emergency stop, continuous requalification and field operational authorization.

These are internal minimums. Domain-specific regulations, safety engineering, jurisdiction and operator policy can require stricter controls.

### 3.2 Risk-axis overlays

| Dimension | Worsening direction | Examples of additional obligations |
|---|---|---|
| Consequence | negligible → minor → major → severe → catastrophic | physical reference validation, uncertainty credibility, independent safety work, fail-safe and dual control |
| Model influence | informational → advisory → constraint-gating → selection → direct actuation | human review, independent constraint reconstruction, generalization, compound faults and safety case |
| Exposure | single asset → site → multi-site → regional → national/larger | site/asset holdouts, cross-site replication, common-cause testing and regional failover |
| Reversibility | readily reversible → costly/delayed → difficult/irreversible | rollback, staged rollout, dual control, fallback and independent safety assessment |
| Detectability | immediate/reliable → delayed/partial → poor | continuous/independent monitoring, redundant measurement, incident response and fail-safe |
| Fallback | independent tested → human → degraded → no verified fallback | fallback qualification, response SLA, degraded-mode qualification, safe stop and field authorization |

The policy declares each sequence explicitly. Unknown maps to that axis's maximum known control set and marks the result indeterminate. The profile schema also supports conflicted; contradictory values use the same worst-case mapping and are independently reported as conflicted. A qualified profile cannot carry unknown or conflicted risk.

### 3.3 Cross-dimensional interactions

Single-axis overlays are not sufficient. The policy includes compound triggers, including:
- catastrophic consequence + direct actuation;
- poor detectability + difficult/irreversible action;
- regional/national exposure + degraded or absent fallback;
- severe/catastrophic consequence + selection/direct-actuation influence.

Each interaction adds stable control IDs and an exact reason code. All controls are unioned; no control can be subtracted by another dimension or trigger.

## 4. Runtime selector

The separate Rust package is tools/industry-risk-control-selector. Its binary is industry-risk-control-selector.

Usage:

    cargo run --locked --manifest-path tools/industry-risk-control-selector/Cargo.toml --       schemas/examples/industry-assurance-qualification-profile-synthetic-v0.1.json       schemas/ieac-risk-control-policy-v0.2.json       /tmp/ieac-risk-control-report.json

The selector:
1. strictly parses JSON and rejects duplicate object member names;
2. checks the IEAC profile version, tier and six risk values;
3. validates policy structure, control references, evidence mappings and action-authority ceiling;
4. validates set monotonicity across every dimension, reviewed-class order and D0-D3 tier order before selection;
5. merges baseline, cumulative tier, reviewed-class, per-dimension and compound-trigger controls;
6. maps each control to the corresponding IEAC evidence dimension;
7. reports established evidence rows, missing evidence, unknown, conflicted and not-applicable states;
8. emits sorted stable control IDs, per-control reason codes, human approvals and operational obligations;
9. emits canonicalized profile and policy digests plus a selector build ID;
10. always emits independent_qualification_attested = false and action_authority = none.

The core selector is pure: its risk mapping depends only on the supplied profile and policy. The CLI handles file I/O. The selector does not read its own clock, perform network/database operations or actuate controls. Unknown or conflicted model influence is blocked below D3; even a D3 result is only a proof-obligation plan, never an actuation grant.

### 4.1 Evidence semantics

A selected control is mapped to one existing IEAC evidence dimension. The report calls a control evidenced only when that dimension is declared established and has at least one structurally populated evidence record with artifact ID and result.

This is a **profile-reported evidence state**, not an independent cryptographic verification of the artifact or the truth of the result. The existing IEAC validator performs its separate schema, artifact-reference, rights, tier and authority checks. Independent evidence verification, source truth and physical validation remain with their owning validators.

A conflicted/unknown/not-applicable evidence dimension keeps the selector disposition indeterminate. A not-established dimension yields a missing requirement. Requirements satisfied is only the selector's statement that all selected control mappings appear established in the input profile; it does not mean the profile is qualified.

### 4.2 Authority boundary

If model_influence = direct_actuation but the declared tier is below D3, the report marks tier compatibility blocked and returns indeterminate requirements. Even at D3, the selector does not authorize a control action. Only the separate IEAC validator and accountable operational authority can evaluate those requirements.

## 5. Benchmarks

The implementation includes a frozen 50-case expected-control truth table at `schemas/examples/ieac-risk-control-truth-table-synthetic-v0.2.json`, with its strict Draft 2020-12 schema at `schemas/ieac-risk-control-truth-table-v0.2.schema.json`. The Rust test replays every case and compares the exact selected control IDs and risk-assessment state. The table covers 31 risk-axis values, six conflicted aliases, five reviewed classes, four decision tiers and four compound triggers. It is a frozen expected-output snapshot for the current policy, not a field safety dataset.\n\nThe implementation includes tests for:
- every adjacent transition across all six ordered dimensions, including the unknown worst-case state;
- ordered reviewed-class and D0-D3 monotonicity;
- unknown, conflicted and unassessed risk;
- direct actuation below D3, including unknown/conflicted model influence below D3;
- compound trigger controls;
- established evidence with no evidence rows;
- conflicted evidence;
- field-order stability and deterministic output;
- unknown control references, unmapped compound values and non-monotone policy edits;
- duplicate JSON members, exact file-byte digest binding and the SHA-256 known-answer vector.

### 5.1 Promotion targets

| Measure | Target |
|---|---|
| Monotonicity | Zero violations across all adjacent declared risk-state transitions and all tier/class transitions |
| Unknown/conflicted | Never produce a weaker control set than the worst known state; always indeterminate |
| False clearance | Zero unknown/conflicted/unassessed risk inputs reported as requirements-satisfied |
| Traceability | 100% of selected controls have at least one stable trigger reason and one evidence-dimension mapping |
| Determinism | Identical normalized profile/policy inputs produce byte-stable report fields |
| Policy safety | 100% rejection of unknown control references, unmapped interaction values and non-monotone policy edits |
| Authority | Zero selector outputs that authorize action, attest qualification or upgrade a tier |
| Utility | Report missing controls and human-review hours separately from required-control count and abstention share |

These are internal policy acceptance targets. They are not real-world safety probabilities or evidence that the control catalog is universally sufficient.

### 5.2 Comparative benchmark

Compare three policy strategies on the same frozen profile corpus:

1. **Reviewed-class-only baseline:** the risk class determines controls, without per-axis or interaction detail.
2. **Six-axis vector selector:** cumulative controls from all six dimensions, tier and interaction rules.
3. **Worst-known conservative baseline:** any unknown/conflicted dimension receives the maximum control mapping for that axis and an indeterminate result.

Measure false-clearance/under-assurance rate, requirement recall against an independently reviewed truth table, requirement precision and over-prescription, unknown/conflicted abstention share, worst risk-group coverage, number of controls, reviewer hours, time-to-ready and policy execution latency.

Do not optimize for the smallest control list alone. The intended trade-off is high safety-requirement recall with explainable, bounded over-prescription. Freeze the truth table and a holdout profile corpus before tuning policy mappings.

## 6. Industry comparison and differentiation

NIST AMS 100-85 grounds risk-scaled credibility effort in model influence and decision consequence. Conventional engineering risk matrices and internal safety processes often provide reviewed categories and assurance plans, but implementation detail varies by sector and operator. IEAC should not claim that a 38-control list is better than every proprietary safety process.

Its differentiator is the executable, profile-bound link between:
- six independent risk dimensions;
- decision tier;
- stable proof obligations;
- exact reason codes;
- the existing ten-dimension evidence vector;
- subsequent versioning/invalidation through the IEAC dependency graph.

This makes the selector's output reviewable and replayable rather than a scalar risk score or a prose checklist. The next comparative study should use matched industrial cases and independently reviewed assurance truth tables—not claim SOTA from synthetic unit tests.

## 7. Claim ceiling and next work

This selector does not estimate incident probabilities or compute a universal risk score; determine whether a physical model is valid; verify the cryptographic authenticity or truth of control evidence; make not-applicable decisions on behalf of a domain reviewer; qualify a profile or change its D0-D3 tier; or grant operational/actuation authority.

Next work: compare the selector with the reviewed-class-only baseline on a locked, independently adjudicated set of water/energy, telecom, manufacturing and maintenance profiles; then bind the chosen policy digest and control requirements into the temporal assurance lifecycle so a changed policy triggers targeted re-evaluation.
