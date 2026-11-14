# IEAC decision reliance continuity v0.1

**Status:** first standalone Rust composition evaluator and schemas implemented; exact-head Rust, schema and executable-fixture qualification pending.
**Issue:** [#509 — IEAC-DECISION-RELIANCE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/509).
**Composes with:** [#138](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/138), [#397](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/397), [#484](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/484), [#488](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488), [#490](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/490), [#492](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/492), [#494](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/494), [#496](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [#347](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347), and the pre-registered IBAP protocol [#338](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/338).

## 1. The architectural shift

The question is no longer only:

> Which nodes are reachable from a changed artifact?

It is:

> Given the exact evidence and dependencies that supported a decision, what did the decision-maker know then, what do we now know about the conditions that existed then, and what can still be relied upon now?

This requires separating three views that ordinary lifecycle dashboards tend to collapse:

| View | Valid time | Knowledge/system-time cutoff | Question |
|---|---|---|---|
| Historical knowledge | Decision time | Decision time | What evidence had actually arrived and been admitted when the decision was made? |
| Retrospective at decision | Decision time | Current evaluation time | What do we now know about the state of the world at decision time, including late-arriving corrections? |
| Current reliance | Current evaluation time | Current evaluation time | Which claims remain usable now, and which require revalidation, quarantine or withdrawal? |

The distinction is substantive. Suppose a source correction is effective on 1 November, a decision is made on 2 November, and the correction is not recorded until 4 November.

- The historical-knowledge view must not leak the 4 November event into the 2 November knowledge state.
- The retrospective view must account for the correction's verified 1 November effective time.
- Current reliance must account for the correction and any downstream obligations.

The historical decision remains an authentic record of what happened. Its current admissibility and the current truth of its supporting claims can change.

## 2. Why three views rather than one bitemporal boolean

A standard bitemporal record distinguishes valid time from transaction/system time. Decision continuity needs both axes *and* a stable decision-time anchor.

The evaluator therefore binds:
- the decision's exact timestamp and target artifact digest;
- the valid-time instant represented by each view;
- the knowledge cutoff used to compute that view;
- the receipt and verifier identities for every gate;
- the current event-chain and impact report references.

A late event has two independent questions:
1. When does the event claim the condition became effective?
2. When did the relevant party learn and record the event?

A trusted timestamp for the event's registration does not, by itself, prove the asserted effective time. An authenticated revoker can still provide a disputed or unverified historical date. The composition layer must quarantine such a claim rather than silently treating it as retroactive truth.

## 3. Ownership: compose existing authorities, do not duplicate them

The evaluator is a receipt composer. It is not another graph, lifecycle engine, coverage engine, trust registry, solver, or authorization service.

| Existing authority | Owns | Decision-reliance evaluator consumes |
|---|---|---|
| Sol-Atlas #138 | Case evidence, artifacts, engineering receipts and decision events | Exact decision and target artifact binding |
| IEAC #487 | Owner-native node identity contract | Identity and mapping receipt |
| #492/#496 | Source-owner descriptors and their internal consistency | Exact descriptor catalogue digest and disposition |
| #489/#490 / #488 | Source inventory, mandatory dependency-family coverage and bounded absence | Coverage receipt and missingness |
| IEAC #397 | Bitemporal event chain and graph impact report | Event-chain head and exact impact classifications |
| #484 | Independent full-scan impact oracle | Candidate/oracle agreement receipt |
| #347 | Trust roots, signatures, authority, revocation and canonicalization | Authentication and authorization receipts |
| Domain engineering verifiers | Solver, metrology, inspection, GD&T and professional review | Exact-input engineering and inspection receipts |
| #338 | Pre-registered benchmark run manifest | Frozen experiment identities and result provenance |

No layer upgrades another layer's output. For example, a coverage pass cannot authenticate a source owner; a valid SCITT receipt cannot establish that an engineering model is physically correct; a passing solver receipt cannot prove that the source inventory was complete.

## 4. Gate model

The normalized input supports these independently reported gates:

1. `source_coverage`
2. `native_identity`
3. `payload_integrity`
4. `source_authenticity`
5. `rights`
6. `lifecycle`
7. `impact_oracle`
8. `engineering`
9. `decision_authority`
10. `inspection`
11. `accepted_work`

The profile declares which gates are mandatory for the decision type. Each gate result binds:
- status;
- receipt SHA-256;
- verifier build SHA-256;
- exact decision-scope digest;
- checked time;
- when the evidence became available to the system;
- validity expiry, if any;
- reason codes.

Statuses are `qualified`, `failed`, `unknown`, `revalidation_required`, `quarantined`, and `not_applicable`. A mandatory gate cannot be bypassed by declaring it not applicable. Missing or unknown gates remain incomplete.

There is no weighted score. A high pass rate cannot compensate for a false source identity, missing mandatory dependency family, digest mismatch, unauthenticated revocation, or absent inspection.

## 5. Temporal event composition

The input may include exact references to lifecycle events. Each event binds:
- event digest and cause class;
- target node and target artifact digest;
- effective time and effective-time confidence;
- recorded time and trusted registration time;
- authority disposition;
- impact report digest and impact disposition;
- exact scope digest.

The composer applies events to each temporal view using two separate boundaries:
- an event cannot affect a view before it was recorded/known by that view's knowledge cutoff;
- an event cannot retrospectively affect a valid-time view unless its effective time is verified and it is in scope.

Unverified authority, unknown/disputed effective time or scope mismatch produces quarantine. A verified, authenticated, in-scope event whose effective time falls at or before a view's valid time can carry the upstream impact disposition into that view. An event effective after the decision does not rewrite decision-time history, but can change current reliance.

The composer **does not recompute dependency traversal**. It consumes #397's impact report and #484's independent oracle receipt. It checks exact receipt references and temporal/scope consistency only.

## 6. Disposition semantics

Each of the three views independently reports one of:

| Disposition | Meaning |
|---|---|
| `candidate_current_within_declared_scope` | All required gate receipts supplied to this composition are qualified for this view. This is not approval or proof of source-universe completeness. |
| `revalidation_required` | A receipt is expired or an upstream verifier requires a new check. |
| `historical_basis_incomplete` | The historical or retrospective view has a missing/unknown mandatory basis. |
| `claim_invalidated` | A qualified upstream result reports a failed claim or an authenticated in-scope event invalidates reliance for this view. |
| `quarantined_pending_review` | Authority, scope, effective time or supporting evidence is disputed or not qualified. |
| `indeterminate` | The current view cannot be established because a mandatory input is missing or unknown. |
| `invalid_input` | The input has contradictory digests/scopes/times, duplicate gate/event identity or future-information leakage. |

The output preserves:
- all three temporal views;
- exact gate receipts and reasons;
- applicable event references;
- revalidation obligations;
- report and input digests;
- `historical_record_preserved = true`;
- `input_authentication = not_performed`;
- `impact_recomputation = not_performed`;
- `action_authority = none`;
- `claim_ceiling = upstream_receipt_composition_only`.

The phrase `candidate_current_within_declared_scope` is intentionally a *candidate reliance assessment*, not an approval, permit, construction release, or command.

## 7. First executable synthetic scenario

Fixture: `schemas/examples/decision-reliance-late-source-correction-synthetic-v0.1.json`.

The fixture describes a synthetic decision on 2 November 2026, evaluated on 4 November 2026. A source correction has a verified effective time on 1 November but is recorded on 4 November.

The expected three-view result is:
- historical knowledge: `candidate_current_within_declared_scope`;
- retrospective at decision: `claim_invalidated`;
- current reliance: `claim_invalidated`.

The fixture demonstrates temporal semantics only. Its digests are synthetic placeholders and its upstream gate/authority dispositions are supplied inputs, not cryptographically authenticated evidence.

## 8. Benchmark programme

### P0: correctness and non-compensable controls

The frozen corpus must contain at least these 14 families:

| Scenario | Required property |
|---|---|
| Complete baseline, no later event | All three views remain candidates |
| Out-of-scope event | No unrelated claim is invalidated |
| Retroactive correction discovered late | Historical knowledge excludes it; retrospective and current views account for verified effective time |
| Prospective correction | Decision-time views remain unchanged; current view changes |
| Late-discovered mandatory dependency | Prior NoImpact is not retained without complete coverage |
| Partial source inventory | Missing coverage forces incomplete/indeterminate, not a clean negative |
| Purpose-scoped rights revocation | Only the affected purpose/use is blocked |
| Disputed rights effective time | Quarantine, not definitive retroactive invalidation |
| Key compromise with pre-boundary trusted registration | Historical authenticity may remain, subject to upstream trust proof |
| Post-boundary or ambiguous registration | Quarantine unless signing-time evidence establishes the relevant boundary |
| Verifier vulnerability disclosed later | Affected receipts require re-verification; unrelated receipts remain untouched |
| Engineering receipt exact-input mismatch | Dependent decision is invalidated or requires revalidation |
| Inspection failure after command completion | Command completion is not accepted work |
| Cross-case native identity conflict | Same native identity with conflicting digest is rejected |

Mandatory acceptance:
- 100% exact agreement with an independently authored three-view oracle;
- zero future-information leakage into historical knowledge;
- zero false-current dispositions with missing/stale/unknown mandatory gates;
- zero retrospective forgery claims based solely on late registration;
- zero unqualified NoImpact when coverage or bounded absence is incomplete;
- zero historical record deletion or mutation;
- 100% exact replay of input digests, receipt references, event references and output report;
- unknown/disputed/unauthenticated cases remain unknown or quarantined;
- no output carries action authority.

### P1: scale and selective revalidation

Run 10, 100, 1,000, 10,000 and 100,000 dependency nodes, with 1, 10, 100 and 1,000 decisions per graph. Include sparse and dense dependency families, shared owner-native records across projects, high fan-out, delayed event arrival and concurrent corrections.

Measure:
- p50/p95/p99 and worst-case end-to-end latency;
- peak RSS, allocations and bytes read/written;
- percentage of decisions needing revalidation;
- false-negative stale-decision rate;
- false-positive unnecessary-revalidation rate;
- time from correction arrival to affected-decision classification;
- time from revalidation obligation to accepted work;
- event-chain replay and recovery time;
- cost of full reconciliation versus sparse updates.

The correctness gate is zero false negatives against the independent full-scan oracle. Efficiency is optimized only after that gate passes.

### P2: held-out field evaluation

Use rights-cleared construction/manufacturing projects, multiple source-owner systems, independently curated dependency inventories, and professional review. Preserve every excluded or inaccessible source in the denominators. Synthetic qualification remains Level 1, never field proof.

## 9. Industry comparison and differentiation

The strategy is to compose with established platforms and prove the *decision-reliance properties* across them.

| Industry baseline | Existing capability to respect | Luminous comparison |
|---|---|---|
| ISO 23247-6:2026 | Integrated, unified and federated digital-twin composition | Independent identity, completeness and decision-reliance receipts across composed twins |
| NIST IR 8536:2026 | Interoperable, cross-organizational traceability, cryptographic links and selective disclosure | Whether traceability claims are authenticated, complete for declared scope and still valid for downstream engineering decisions |
| Bentley iTwin | Native engineering synchronization/export, changed-element feeds, clash/validation, project scheduling and workflow automation | Exact evidence-to-decision dependencies, independent impact replay, and temporal reliance across the exported scope |
| Siemens Digital Twin Composer | Large-scale product/plant/factory simulation, virtual commissioning and robot training | Preserve simulator input/version/assumption receipts and evaluate which decisions become stale after changes |
| Palantir Ontology | Objects, links, actions, logic and granular security for operational workflows | Portable, cross-owner, independently replayable reliance assessments without claiming to replace its operational ontology |
| buildingSMART IFC/IDS | Open BIM exchange and machine-readable information requirements | Cross-checker agreement plus explicit source coverage and downstream decision invalidation |

Official anchors:
- [ISO 23247-6:2026](https://www.iso.org/obp/ui?_escaped_fragment_=iso%3Astd%3Aiso%3A23247%3A-6%3Aed-1%3Av1%3Aen)
- [NIST IR 8536 (final, September 2026)](https://csrc.nist.gov/pubs/ir/8536/final)
- [buildingSMART IDS 2026 survey](https://www.buildingsmart.org/ids-survey-results-2026/)
- [Bentley iTwin APIs](https://developer.bentley.com/apis/)
- [Siemens Digital Twin Composer](https://www.siemens.com/en-us/company/digital-transformation/industrial-metaverse/introducing-digital-twin-composer/)
- [Palantir Ontology overview](https://www.palantir.com/docs/foundry/ontology/overview)

Do not use vendor feature-count comparisons. Compare identical exported source scopes, source snapshots, tasks, versions and hardware. If a vendor's relationship is inaccessible or the compared task is not equivalent, label it `unassessed` or `not_comparable`, never absent.

## 10. What would constitute a genuine SOTA advantage

A credible advantage would require all of these simultaneously:

1. **Correctness:** parity with the independent oracle on supported scenarios, including late events and temporal boundaries.
2. **Completeness:** fewer unknown/missing relationships on held-out sources, proven against independent gold inventories.
3. **Precision:** fewer unnecessary revalidations than naive descendant invalidation, with zero missed invalidations.
4. **Portability:** the same decision-reliance report can be independently replayed by another implementation from the same pinned receipts.
5. **Accountability:** every change to reliance maps to exact event, source, impact and verifier receipts.
6. **Operational value:** lower total cost and time-to-accepted-work after all correctness gates pass.

No single weighted score is permitted to hide a false-current decision. The primary headline metrics should be false-current rate, false-negative invalidation rate, false-positive revalidation rate, source-coverage recall, exact replay rate and median/p95 time-to-revalidated-accepted-work.

## 11. Nonclaims

The evaluator does not authenticate input receipts, fetch or hash original source bytes, verify signatures or rights, recompute the dependency graph or impact frontier, execute engineering solvers, perform physical inspection, establish that an engineering profile is adequate, or authorize action. Those capabilities remain with the existing independent authorities and must be referenced by exact digest.

Its product is a reproducible *reliance assessment*, with explicit limits—not a universal engineering approval.
