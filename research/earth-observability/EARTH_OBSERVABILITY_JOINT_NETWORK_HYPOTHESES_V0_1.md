# EARTH-OBSERVABILITY-JOINT-NETWORK-HYPOTHESES-001
## Joint posterior hypotheses for evidence-qualified utility-network reconstruction v0.1

**Parent:** Sol-Atlas #771 — EARTH-OBSERVABILITY-QUAL-002  
**Implementation branch:** research/earth-observability-joint-network-hypotheses-v0-1  
**Status:** Rust structural contract and deterministic analytic fixture added; probability calibration, graph scoring and physical validation NOT_RUN  
**Date:** 2026-11-14  
**Claim ceiling:** The fixture is abstract and non-geographic. It does not represent a real utility, field survey, calibrated posterior, qualified network geometry or excavation clearance.

---

## 1. Research proposition: infer a distribution over worlds, not one best-looking map

The current candidate-graph contract permits multiple event-to-asset association marginals and explicit abstention. That is useful, but independently meaningful marginal probabilities do not necessarily form a coherent joint network.

For example, separate edge probabilities can each look plausible while their combination creates an impossible network: a single event is assigned to two incompatible assets, a crossing is treated as a junction, or two mutually exclusive asset partitions are simultaneously represented as facts.

The next abstraction is a **joint network hypothesis set**:

- one immutable evidence snapshot and one investigation scope;
- a finite set of mutually exclusive candidate network worlds;
- one complete event disposition per observed event in every world;
- candidate assets, geometry claims, nodes and relations that are internally coherent within each world;
- a probability for each whole world;
- explicit residual probability for worlds not enumerated.

This is not a claim that a Bayesian posterior has been calibrated. It is a data and validation contract that makes the intended probability semantics explicit and testable.

The decision product becomes:

> Here are the plausible physical network configurations, their relative support, the evidence that separates them, and the observation most likely to change the engineering decision.

## 2. Contract semantics

The executable contract is in `sol-atlas-core/src/network_hypotheses.rs`; the strict JSON Schema Draft 2020-12 interchange profile is `research/earth-observability/NETWORK_HYPOTHESES_V0_1.schema.json`. The Rust validator enforces cross-reference and cross-field invariants that JSON Schema alone cannot express.

### 2.1 One hypothesis is one complete candidate world

A NetworkHypothesis contains:

- a unique hypothesis identity and whole-world probability;
- exactly one assignment record for every event in the set;
- explicit null assignment for events left unassociated;
- candidate asset instances and their event membership;
- optional geometry-claim references;
- candidate nodes and their associated candidate assets;
- evidence-supported relations between nodes.

A candidate identifier is local and non-authoritative. It is not a MUDDI ObjectID, owner-system SystemID or utility-owner asset identifier.

### 2.2 Mutually exclusive world mass

The probability invariant is:

sum(P(hypothesis_i)) + P(residual_unknown) = 1

within a fixed numerical tolerance.

The residual is not a negative detection, absence probability or permission to redistribute probability among the listed hypotheses. It represents unenumerated or unresolved possible worlds.

The fixture contains three alternatives:

| World | Probability | Interpretation |
|---|---:|---|
| world-one-utility | 0.62 | Two events belong to one candidate utility |
| world-two-utilities | 0.23 | The events belong to distinct candidate utilities |
| world-crossing-unresolved | 0.10 | Two distinct assets have an unresolved crossing relation |
| Unenumerated worlds | 0.05 | Explicit residual unknown mass |

The values are deliberately illustrative, not learned or calibrated.

### 2.3 Evidence closure

Every event has a source-evidence reference. Every assigned event must be represented in its candidate asset's event membership and supported by that exact source evidence. Every node and relation must resolve its references and carry admitted evidence.

A null event assignment is a valid explicit abstention. It is not silently converted into an absent utility.

### 2.4 Topology is not visual intersection

The relation types distinguish continuation, junction, unresolved crossing and unresolved near-intersection.

An unresolved crossing is not a graph edge establishing physical connection. It remains a spatial relationship awaiting evidence.

### 2.5 Calibration status is separate from structural validity

The validator can check that a calibration receipt identifier was supplied when a caller claims that one is referenced. It cannot authenticate that receipt or establish that the probabilities are calibrated.

The code deliberately does not derive a posterior from event marginals or multiply cross-modal likelihoods.

## 3. The benchmark should score whole worlds and their components

A single graph-edit score is not sufficient. A nearly correct map can still contain a dangerous false connection, and an accurate event detector can still over-split one utility into several assets.

The benchmark therefore needs multiple independent score layers.

| Layer | Required measures |
|---|---|
| World posterior | Joint log score and multiclass Brier score on frozen finite-world cases; residual-mass scoring |
| Asset identity | Asset-instance precision/recall; pairwise or B-cubed clustering scores; overmerge and oversplit rates |
| Event association | Event-to-asset assignment precision/recall; unassigned-event rate; association calibration |
| Geometry | Lateral and depth error P50/P95; endpoint error; route completeness; false inferred length; unsupported length |
| Topology | Node/edge precision and recall; false junction and false connection rates; component-count error |
| Uncertainty | Reliability diagrams; proper scores; interval coverage and width; site-clustered uncertainty |
| Domain transfer | Leave-project/site/instrument/acquisition/geology-out performance; selective risk and coverage |
| Decision value | Expected decision loss/regret and cost at equal eligible investigation budgets |

### 3.1 Proper probabilistic scoring

For finite synthetic worlds with unambiguous adjudicated truth, score the complete probability distribution, not only the maximum a-posteriori world.

- Use the executable `score_network_hypothesis_set` scorer in `sol-atlas-core/src/network_hypotheses.rs` to compute negative log probability and multiclass Brier score over every listed world plus the residual-unknown category. A zero-probability truth is an explicit scoring error (infinite log loss), not silently clipped or renormalized.
- Include the residual/other category. If the true world is not represented by a listed hypothesis, it must not be forced onto the nearest listed graph.
- Score asset existence, event association, edge existence and junction status as separately labelled marginal outcomes.
- Use geometry-appropriate distributional scores and interval coverage for uncertain positions/depths.

Graph edit distance, geometric distance and asset matching are valuable diagnostic losses. They are not substitutes for proper scoring of the submitted probabilities.

Report calibration by independent project/site clusters, not radar traces. The probability bins, matching tolerances, graph edit costs and treatment of missing truth must be frozen before blind scoring.

### 3.2 No metric compensation

High event recall cannot compensate for false asset creation. High asset F1 cannot compensate for dangerous false junctions. Excellent geometry cannot compensate for a model that is overconfident outside its observability envelope.

Keep each layer visible in the scorecard and define consequence-specific promotion gates.

## 4. New challenge corpus

Build the first executable corpus around adversarial alternatives:

1. Repeated reflections from one asset versus two distinct assets.
2. One observation compatible with several assets.
3. Two crossing utilities versus a true junction.
4. Close parallel utilities versus one merged asset.
5. Intermittent observations versus a genuine network break.
6. Correlated GPS/IMU/route-registration drift.
7. A legacy record that conflicts with independent survey evidence.
8. Utility-like clutter with no physical asset.
9. Unseen instrument or soil domain requiring residual mass expansion or abstention.
10. A blind region where non-detection cannot establish absence.

Each case needs a complete hidden reference graph, not only event boxes. Asset IDs, event assignments, node/edge topology, geometry and uncertainty should be versioned separately so the benchmark can identify which layer failed.

## 5. Matched competitive baselines

Run every comparator on the same permitted observation pack, truth, budget and split:

1. Experienced human interpretation, with documented time and adjudication.
2. Conventional event detector plus expert association.
3. Event detector plus spatial clustering and RANSAC-style continuity.
4. Multi-view fusion with registration covariance.
5. Independent event-association marginals followed by a deterministic graph builder.
6. Joint network-hypothesis inference.
7. Physics-guided joint inference with declared model discrepancy.
8. Hidden-truth oracle, reported only as an upper bound.

The most important ablation is number 5 versus number 6. It isolates whether coherent joint hypotheses add value beyond independently plausible event-to-asset marginals.

Do not compare an asset-level result to a paper reporting only hyperbola or pixel-level detection accuracy.

## 6. Decision-oriented survey selection

The hypothesis set should be passed to the existing Sol-Atlas EVSI engine rather than starting another decision engine.

For each eligible survey action, estimate how the possible observation outcomes would update the joint-world distribution and whether the update changes the engineering decision.

Compare at equal cost and access budgets:

- expert-selected actions;
- fixed-grid;
- random eligible;
- cost-only;
- entropy-greedy;
- existing EVSI / ENBS;
- hidden-truth oracle.

Include a case where an action has high information gain but cannot change the decision. Entropy-greedy may choose it; a correctly specified decision-value policy should not.

Access, permits, owner rights, safety obligations and ecological constraints remain hard feasibility filters, not soft rewards.

## 7. Data and physical validation ladder

- T0: structural contract, serialization and negative-control tests.
- T1: deterministic analytic worlds with exact known topology.
- T2: physics-based sensor scenes with declared forward-model assumptions and discrepancy.
- T3: retrospective physical datasets with independently documented reference truth.
- T4: prospective blind campaigns with qualified operators and independent surveys.

A solver comparison is not ground truth. A synthetic radargram is not a physical field observation. A source record or drawing is not necessarily an accurate as-built.

Use gprMax as a possible GPR forward-model partner, but preserve its distinction between analytical validation, inter-code comparison, backend consistency and regression testing. Symtropy remains the owner of physical forward-model assumptions and sensor qualification.

## 8. Proposed acceptance gates

### Structural gates — deterministic

- Every event has exactly one disposition in each listed world.
- Every assigned event has exact source-evidence support.
- Every candidate node and relation resolves within its world.
- No duplicate IDs, unbound evidence or invalid probability values.
- Listed-world mass plus residual unknown mass equals one.
- No unresolved crossing is emitted as a physical junction.

These are software invariants, not field performance thresholds.

### Empirical gates — preregister per consequence class

- Calibrated joint-world probabilities improve on independent-marginal and simple baseline scores.
- Asset overmerge, oversplit and false-connection rates are reported separately.
- Nominal 90% and 95% geometry intervals are assessed on independent holdouts, with interval width reported.
- Domain shift leads to empirically justified uncertainty expansion or abstention.
- No high-confidence physical absence claim is made outside the qualified observability envelope.

### Decision stretch target

Retain the programme's existing hypothesis: at least 20% lower out-of-sample expected decision loss at the same eligible investigation budget, or at least 20% lower investigation expenditure at equivalent preregistered decision quality.

This is a future target. It is not an achieved result.

## 9. Execution state

Implemented in this increment:

- a Rust joint-network hypothesis contract;
- complete event assignment and explicit abstention;
- per-world asset/event consistency;
- evidence-reference and topology-reference guards;
- explicit residual probability;
- deterministic analytic fixture with three mutually exclusive network alternatives;
- unit tests for mass conservation, assignment completeness, source evidence, asset membership, topology references and calibration-receipt declarations.

Not implemented or qualified by this increment:

- posterior generation or calibration;
- joint graph scoring;
- graph matching against physical ground truth;
- geometry reconstruction;
- physics simulation;
- physical validation;
- EVSI update implementation.

This increment establishes the next data abstraction and structural safeguards. Exact-head Rust tests and formatting remain pending.
