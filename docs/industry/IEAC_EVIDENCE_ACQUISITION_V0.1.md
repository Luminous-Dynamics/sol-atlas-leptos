# IEAC Evidence Acquisition v0.1

**Status:** Contract and schema tranche. It does not implement a source connector, acquisition executor, calibrated value-of-information model, cryptographic trust verifier or admission authority.

## 1. Core idea

The current runtime gate correctly distinguishes admission, abstention and rejection. Its next evolution should make abstention actionable without turning a missing observation into a permission to guess.

The proposed flow is:

`request → admission → abstention → evidence obligations → ranked or unranked acquisition plan → operator-mediated evidence collection → new request → new admission report`

An evidence plan is never an admission report. It never executes a query, changes an operational system, modifies a profile, renews a qualification or raises the decision tier. The original abstention remains immutable.

## 2. Why this is a paradigm shift

A digital twin normally provides a representation, simulation, data integration or operating workflow. This layer should specify the conditions under which a proposed decision can be trusted enough to be considered, then identify what evidence is still needed when those conditions are not met.

The shift is from "unknown, stop" to "unknown, here is the minimal auditable path to reduce the uncertainty." That path must be bounded by source rights, identity, provenance, uncertainty, freshness, source dependencies and the original profile. It should not maximise data volume.

## 3. Contract objects

### Evidence obligation

An obligation refers to one prior request and binds profile, asset and decision identity. It carries:

- stable reason codes and a resolution state;
- the exact variable and accepted unit;
- the unchanged operating envelope and maximum uncertainty;
- an explicit observation window and maximum age;
- profile-permitted source IDs/classes and data-rights requirements;
- declared dependency dimensions and source-independence constraints;
- the authorization source and required trust state.

A revoked artifact, denied authorization, malformed timestamp or out-of-domain request is not automatically "fixable by fetching another sample." Only abstention reasons may create acquisition obligations. Rejections remain rejections unless a separate authorized review establishes a new profile or request.

### Acquisition plan

The plan provides candidate sources, obligation coverage, rights state, dependency declarations, acquisition cost/latency, and information-value estimate. It records eligible and excluded candidates and an ordered selection only when ranking is defensible.

The schema has hard invariants: D0/D1 only; advisory-only; execution authorization false; profile mutation false; ranked plans require calibrated planner state, assessed independence and complete rights; eligible candidates must have verified rights and declared independence; unknown/dependent sources cannot be ranked as eligible. If these preconditions are unknown, the planner must abstain from ranking.

### Evidence response and re-admission

This v0.1 tranche does not yet define a connector or permit evidence collection. The next runtime tranche should require a response to bind the obligation ID, prior request ID, source artifact ID, raw payload digest, source identity, timestamp, unit, uncertainty, rights, provenance and signer/trust-root status. A response creates a new admission request and report. It never changes the old report.

## 3.1 Structural schema versus semantic verification

Draft 2020-12 validation is a structural gate, not the complete acquisition policy. A future runtime verifier must additionally prove that:

- every obligation request/profile/asset/decision identity matches the prior admission request and profile;
- every permitted source ID is actually declared by that profile and every candidate maps to one or more declared obligations;
- every ordered candidate exists exactly once and is eligible; no excluded, unknown-rights, dependent or unassessed candidate can be selected;
- the candidate's source class, rights, digest, freshness and dependency graph satisfy the original profile and obligation;
- observation-window ordering, numeric bound ordering, uncertainty width and age calculations are valid;
- acquisition costs use a common, declared unit and calibrated cost model before comparisons are made;
- the planner's decision-value model is calibrated for the relevant vertical, decision tier and operating context;
- no evidence acquisition output mutates the prior admission report, profile, decision tier or operating envelope.

These cross-object invariants cannot be established by the schemas alone. A schema-valid plan must not be treated as an executable or authorized plan. All acquisition stays operator-mediated until a separate runtime verifier and trust path are implemented.

## 4. Value-of-information discipline

Do not rank sources by a single guessed "information gain" number. A usable ranking needs a calibrated decision-value model and measured acquisition cost/latency. For small bounded fixtures, compare candidate selection with an exhaustive oracle. For realistic deployment, calibrate the value model prospectively and report uncertainty.

The planner must abstain from ranking when any required source's rights, source dependencies, acquisition cost, latency, or decision-value calibration are unknown. It may still return an unranked obligation set and request human review.

## 5. Benchmark design

Compare three strategies using the same frozen requests, sources, rights, budgets and observations:

1. Conservative baseline: abstain without proposing acquisition.
2. Naive baseline: request every missing/uncertain channel allowed by the profile.
3. Candidate: select the smallest cost-bounded evidence set that resolves the obligation under the declared dependency and uncertainty rules.
4. Bounded oracle: exhaustively enumerate small synthetic source sets to establish a lower bound on acquisition cost/cardinality and an upper bound on achievable resolution coverage.

Measure false recovery/readmission, obligation precision/recall, cost-normalized minimality gap, valid resolution coverage, rounds-to-resolution, time-to-abstention, human escalation, source-dependency detection, stale/revoked evidence rejection, verifier disagreement, replay and worst-group outcomes. Report per vertical and failure family. Never pool failure families to hide a weak stratum.

Proposed gates:

- zero false recoveries in the frozen generated negative corpus;
- 100% exact reason-code-to-obligation mapping on hand-calculated cases;
- 100% of eligible candidates satisfy profile source, rights and independence constraints;
- no ranking where calibration, rights, cost or dependency status is unknown;
- zero changes to profile bounds or decision-tier ceiling;
- every evidence response produces a new request and immutable report;
- 100% replay/checker agreement on the frozen software contract panel.

Sample-size confidence bounds describe the declared generated corpus only. They are not real-world risk rates. A field claim requires a rights-authorized, independently reviewed shadow pilot with measured source quality and reconciled outcomes.

## 6. Industry positioning

This does not compete with the rendering, simulation, ontology or connectivity features of major digital-twin stacks. It should integrate with them.

- Siemens Digital Twin Composer: consume its engineering/simulation context and make the evidence/decision boundary portable.
- Palantir Foundry Ontology: complement its objects, links, actions and dynamic governance with a separate, replayable admission evidence record.
- Bentley iTwin: consume infrastructure engineering, reality, GIS and IoT evidence without requiring source migration.
- OPC UA FX: use standardized field connectivity; do not invent a parallel field protocol.
- ISO 23247-6: align with integrated/unified/federated twin composition rather than assume a single canonical twin.

## 7. Claim limits

The synthetic example is a schema illustration. Its cost and information-value numbers are fictitious. A valid JSON document does not prove that an observation is true, that sources are independent, that a signer is authorized, or that an action is safe. Cryptographic trust remains blocked on the IEAC trust workstream (#347), and physical model credibility remains a separate VVUQ obligation.

## 8. Cross-project boundary

Sol-Atlas owns evidence obligation semantics, source constraints, decision re-admission and replay. Symthaea may propose information-gain estimates but cannot determine admissibility or authority. Symtropy may produce bounded what-if evidence only under an independently qualified model envelope. Mycelix can later carry provenance and source attestations after the trust-root and anti-replay contract is implemented. Operational execution remains in customer-controlled systems.
