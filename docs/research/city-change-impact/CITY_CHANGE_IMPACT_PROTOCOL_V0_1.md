# CITY-CHANGE-IMPACT-001 — Protocol v0.1

**Status:** research contract; synthetic-first; read-only city projection  
**Issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/643  
**Branch:** research/city-change-impact-v0.1  
**Parent:** CITY-MAKER-001 #549; CITY-REFERENCE-DISTRICT-001 #615

## 1. Research question

When an exact source, design, product, process, supplier, lot, calibration, model, policy or operating assumption changes, which city-level claims remain applicable, which require revalidation, which are invalid, and which cannot be assessed?

The CityMaker consumer must avoid both stale-evidence false negatives and unnecessary over-invalidation. It must preserve specialist-owner authority and use the already existing dependency, change, trust and decision-reliance systems.

This is not a second graph engine. The owner systems compute, authenticate and qualify their respective facts. Sol-Atlas projects their exact outputs into the city/district context.

## 2. Paradigm: change-aware city rather than static digital twin

A city model should be understood as a versioned set of claims over exact assets, source data, models, processes, products, policies and decisions. A claim is only current within its declared scope, evidence dependencies and evaluation time.

A change event should cause a *targeted revalidation frontier*, not:
- a blanket reset of the entire district;
- silent reuse of stale engineering or operational evidence;
- automatic revocation based only on graph reachability;
- a new approval or actuation decision.

A small change can have a large consequence; a large byte diff may have no semantic consequence. Domain-owner change semantics and exact dependency edges determine impact, not diff size alone.

## 3. Owner boundaries — compose, do not duplicate

| Owner | Existing authority | CityMaker role |
|---|---|---|
| Symthaea #10543 / #6117 | Engineering change impact, evidence applicability, design transformation/delta semantics | Consume exact change and applicability reports |
| Sol-Atlas #484 | Independent revalidation-frontier correctness/minimality | Consume qualified impact set and path |
| Sol-Atlas #396 | Bitemporal authenticated invalidation, correction, revocation and supersession | Consume event chain and trust-state reports |
| Sol-Atlas #475 | Exact-base graph-bound sparse impact deltas | Validate base/report identity through owner contract |
| Sol-Atlas #501 | Decision reliance at declared time and scope | Project current reliance state |
| Sol-Atlas #623/#624, #615 | Scope, owner-envelope binding and reference district | Bind claims to exact district and product scope |
| Sol-Atlas #500/#637 | Independent payload byte verification and adapter integration | Require exact byte report where applicable |
| Symtropy #4181 / #4043 | Service module and physical asset inspection/repair evidence | Consume product/process/asset claims |
| Mycelix #4814 / #5149 | Supplier, inventory, product-lot, custody, correction and revocation facts | Consume provenance and supply change evidence |

A missing integration or dependency edge is explicit Unknown. A successful byte rehash is not source authentication, legal rights, engineering correctness or completeness.

## 4. Change event and query contract

Each projection request must bind:
- exact event ID and event digest;
- exact changed subject node, subject revision and change class;
- source owner, schema and revision;
- authority/trust/revocation state from its owning system;
- effective/world time, recorded/knowledge time, trusted/receipt time and evaluation time;
- exact graph snapshot ID/digest and dependency contract;
- impact-report ID/digest, policy/build revision and query semantics;
- scope, asset, design/configuration revision and functional unit;
- coverage receipt, excluded domains and unresolved edges;
- any revalidation receipt proposed to discharge an obligation.

Do not infer event authenticity from a well-formed JSON object, digest syntax or caller assertion. Preserve the distinction between declared effective time and independently established trusted time.

### Bitemporal replay

A late correction can have an effective time earlier than the time it was recorded. The current view may therefore require revalidation, while a historical "what did the system know at time T?" query must reproduce the earlier knowledge state. Never rewrite history to make the old view appear to have known the later event.

### Exact-base sparse deltas

A delta is valid only against the exact base graph and prior report/event-chain head required by its owner contract. A stale-base delta, out-of-order event, rollback or duplicated sequence is not applied. Full recomputation remains a comparator and periodic reconciliation path.

## 5. Applicability states

Per claim, return exactly one of:

- **applicable_current** — owner-qualified evidence and coverage establish current applicability at the evaluation time and scope.
- **revalidation_required** — a known changed dependency reaches this claim, and no valid post-change revalidation receipt discharges the obligation.
- **invalid_evidence** — a required input is authoritatively revoked, contradictory, invalid or superseded such that reliance is blocked.
- **insufficient_evidence** — authority, bytes, rights, dependency coverage, time, path, schema, source revision or revalidation evidence is missing, stale, ambiguous or unsupported.
- **not_affected** — a qualified, coverage-bounded owner impact report establishes no relevant dependency path within its explicit query boundary.

An empty result is not proof of no impact. If dependency coverage is unqualified or an owner domain is excluded, return insufficient_evidence rather than not_affected. Preserve reason codes, owner refs, paths, graph snapshot, coverage and evaluation time.

A revalidation receipt only discharges the exact obligation when it binds the exact changed subject, post-change revision, claim scope, required verification method, owner, time and policy. A receipt for an older design or a sibling claim does not discharge the current claim.

## 6. E1 synthetic corpus

The companion JSON file contains 28 synthetic change/claim fixtures. They span:
1. Unrelated design changes and qualified no-impact results.
2. Panel geometry and service-cassette interface revisions.
3. Product/BOM revision and exact lot revocation.
4. Supplier capacity changes that affect future procurement but not installed-asset test results.
5. Supplier authority, signed/unsigned revocation and source trust.
6. Calibration validity before and after measurement.
7. Robot firmware, tooling and manufacturing fixture changes.
8. As-built clearance and coordinate-frame corrections.
9. Weather/EPD/operational source corrections with spatial, temporal and lifecycle boundaries.
10. Future-effective standard updates and late-arriving corrections.
11. Missing dependency edges and incomplete graph coverage.
12. Cycles, wrong-base deltas, duplicate/out-of-order events and rollback.
13. Partial revalidation, stale-revision receipts and downstream decision supersession.
14. Sibling claims where one remains applicable while another requires revalidation.

All fixture IDs, revisions, times, edges, digests and claims are synthetic. They are not local source facts, legal interpretations, physical performance claims or operational evidence.

## 7. Metrics

Report separately:
- impacted-node precision, recall, F1 and exact-set agreement against an independent reference graph oracle;
- false-negative impact rate (target zero for declared mandatory edges);
- false-positive invalidation rate (target zero for gold no-impact claims);
- impact-path and reason-code agreement;
- no-impact coverage qualification rate;
- revalidation obligation completeness and exact discharge rate;
- bitemporal replay and late-event correctness;
- stale decision-reliance false acceptance rate (target zero);
- unknown/abstention rate by cause;
- graph build/validation, full recomputation, sparse-delta and end-to-end consumer costs;
- human review time and solver/revalidation expenditure.

Do not optimize only for fewer revalidations. A low revalidation count obtained by missing critical dependencies is failure.

## 8. E1–E5 benchmark ladder

| Level | Method | Exit criterion |
|---|---|---|
| E1 | Frozen synthetic graph and corpus | 100% independent known-answer agreement; zero missed mandatory impacts; zero false no-impact results |
| E2 | Owner-method reproduction | Reproduce #396/#484/#475/#10543/#6117 semantics and standards crosswalks |
| E3 | Held-out multi-owner exports | IFC 4.3.2.0/IDS 1.0, product/BOM, asset, provenance and operations exports; independently audit semantic loss and dependency coverage |
| E4 | Controlled supervised project/workcell changes | Verify exact work packages and decisions gated after controlled changes |
| E5 | Sustained operational evidence | Repeated change, correction, revalidation and recovery cycles independently replicated |

No level grants statutory, professional or operational approval. E3/E4 physical work requires competent professionals, rights-cleared data, safety review, suitable instruments and accountable acceptance.

## 9. Standards and industry crosswalk

- **ISO 23247-6:2026** (published 14 July 2026) identifies integrated, unified and federated digital-twin composition and provides guidance for interoperation between independently developed twins. CityMaker should classify each owner interface and test authority, consistency, latency, coverage and change propagation. This is a manufacturing twin standard; adapting it to city/building systems is a hypothesis, not a certification claim. https://www.iso.org/standard/87426.html
- **NIST IR 8536** (final 9 September 2026) provides an industry-neutral traceability meta-framework for linking, querying and selectively disclosing manufacturing provenance across ecosystems. Reuse its traceability patterns for products/lots; do not duplicate its framework. https://csrc.nist.gov/pubs/ir/8536/final
- **IETF RFC 9943 / SCITT** specifies signed-statement transparency, append-only history and receipts. A receipt demonstrates inclusion of a signed statement, not engineering truth, completeness or applicability. https://www.rfc-editor.org/rfc/rfc9943
- **ISO 55000:2024** provides asset-management vocabulary, principles and lifecycle/outcome orientation. It is relevant to portfolio and asset value, not an impact-graph algorithm. https://committee.iso.org/cms/live/live/en/sites/isoorg/contents/data/standard/08/30/83053.html
- **ISO 37123:2019**, confirmed current in 2025, supplies city-resilience indicators. It measures outcomes; it does not prove evidence applicability. https://www.iso.org/standard/70428.html
- **IFC 4.3.2.0 / ISO 16739-1:2024** is the current official IFC release. **IDS 1.0** supports machine-readable IFC alphanumeric information requirements but does not express geometry. Geometric/as-built change impacts require additional owners. https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/ https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/

## 10. Competitive comparison

Use identical exported dependency graphs and change sets to compare:
- full recomputation after every change;
- qualified owner-native incremental/revalidation-frontier tools;
- conventional BIM/PLM/CDE version and change workflows, where equivalent graph semantics can be exported;
- CityMaker's federated consumer of owner-native change/impact receipts.

Measure misses, over-invalidation, path explanation, historical replay, data-rights/selective-disclosure behavior, human review time, solver expense and end-to-end latency. Include graph building and validation costs. Do not infer proprietary competitor limitations or compare unequal input coverage.

The competitive hypothesis is that CityMaker can preserve specialist authority and local autonomy while making cross-owner city-level consequences explainable and replayable. The benchmark must establish this, not assume it.

## 11. Claim boundary

A synthetic PASS proves only the declared change-impact semantics over the frozen graph and corpus. It does not prove all real dependencies were modeled, that a correction/revocation is true, that a design is safe or compliant, that physical outcomes changed, or that an authorized decision may proceed. Action and release authority remain with existing human, professional, owner and governance processes.
