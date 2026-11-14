# EARTH-OBSERVABILITY-INVESTIGATION-TOURNAMENT-001
## Constrained investigation policy scoring and equal-budget comparison v0.1

**Implementation:** `sol-atlas-core/src/investigation_tournament.rs`  
**CLI:** `investigation_tournament_eval`  
**Input schema:** `INVESTIGATION_TOURNAMENT_V0_1.schema.json`  
**Synthetic fixture:** `fixtures/investigation-tournament-synthetic-v0.1.json`  
**Pinned baseline:** `baselines/investigation-tournament-baseline-v0.1.json`  
**Parent:** Sol-Atlas #771 / decision-grade tournament protocol  
**Status:** structural evaluator, CLI, fixture and pinned baseline authored; exact-head CI pending  
**Claim ceiling:** Synthetic policy scoring only. This is not an EVSI engine, physical survey result or field authorization.

---

## 1. Paradigm shift: score the decision, not the amount of information

A policy can collect highly informative data and still make a poor engineering decision. Entropy reduction and expected information gain are not interchangeable with expected value of sample information or realized decision quality.

This evaluator makes that difference observable:
- action-level information-gain estimates supplied by a planner are retained as diagnostics;
- policy decisions are scored against an adjudicated decision-loss table;
- the same action catalogue and the same cost/crew/disturbance budget apply to every policy in a case;
- access, method, permit, rights, safety and required custodial review are hard feasibility gates.

The evaluator does **not** calculate EVSI/ENBS. It scores frozen outputs from policies such as expert-led, fixed-grid, random eligible, cost-only, entropy-greedy, expected-value-of-information and learned planners. A separate decision engine may supply the action selection and decision.

## 2. Common action catalogue and budget

Every action is bound to:
- unique action ID;
- spatial support and modality;
- currency and direct cost;
- crew hours;
- an ordinal disturbance class;
- optional planner-supplied information-gain estimate;
- authority/eligibility declarations.

The common case budget specifies:
- one currency for all actions in the case;
- maximum direct expenditure;
- maximum crew hours;
- maximum permitted disturbance class.

The evaluator rejects:
- unknown or duplicate actions;
- duplicate action selection within a plan;
- ineligible actions;
- actions above the permitted disturbance class;
- cost or crew-hour budget overruns;
- inconsistent currency;
- invalid action/decision metrics;
- an oracle label that is not explicitly marked as a non-deployable upper bound.

The action authority flags are declarations supplied to the evaluator. They are not authenticated permits or legal determinations. Real field deployment still requires independent authority, rights, safety and custodial review.

The disturbance class is an ordinal hard limit, not a monetized externality. It does not model cumulative ecological, cultural or community harm. Physical campaigns must add the domain-specific hard constraints and review processes they require.

## 3. Decision scoring

Each synthetic or blind case supplies:
- an adjudicated truth-state reference;
- a set of candidate decisions;
- the loss of each decision under that reference truth;
- multiple frozen policy plans.

For each policy, the evaluator reports:
- selected action count;
- direct cost and crew hours;
- highest selected disturbance class;
- planner-submitted information-gain sum (diagnostic only);
- chosen decision and realized decision loss;
- minimum available decision loss (oracle lower bound);
- realized regret relative to that lower bound;
- budget utilization.

No opaque composite score combines money, crew time, disturbance and decision loss. Publish a Pareto comparison. A policy should not be declared superior by quietly spending more, using more crew, exceeding authority or transferring disturbance to a community.

## 4. Synthetic counterexample: information gain is not decision value

The committed fixture contains one abstract case with:
- a shared budget of USD 150 and four crew-hours;
- a low-disturbance wide-area GPR action with submitted information gain 0.95;
- a targeted cross-line EM action with submitted information gain 0.55;
- an archival-record review action with submitted information gain 0.20;
- a prohibited trial-hole action with higher submitted information gain 0.99 but missing access, method, permit and safety authority.

The synthetic policy outputs demonstrate:
- entropy-greedy selects the 0.95 information-gain action but chooses a decision with loss 100;
- the decision-aware comparator selects the 0.55 information-gain action and chooses a decision with loss 10;
- the hidden-truth oracle has loss 0 and is marked as a non-deployable upper bound;
- the prohibited trial-hole action cannot be selected even though its submitted information-gain estimate is highest.

These are intentionally constructed values for a negative-control fixture. They are not measured sensor values, empirical action outcomes or evidence that one real policy is superior.

## 5. Benchmark baselines and matched comparison

For physical or retrospective tournaments, use the same:
- frozen case and evidence snapshot;
- action catalogue and instrument capability declarations;
- money, crew-hours and elapsed-time boundary;
- maximum disturbance/permit/access constraints;
- reference truth and decision-loss function;
- model and policy submission deadline;
- scoring executable and evaluator version.

Recommended policy families:
1. Existing expert investigation practice, with time and assumptions recorded.
2. Fixed-grid or fixed-spacing acquisition.
3. Random eligible sampling.
4. Cost-only selection.
5. Entropy/information-gain greedy selection.
6. EVSI or expected-net-benefit decision policy.
7. Learned policy trained only on fit/calibration projects.
8. Hidden-truth oracle, reported solely as an upper bound.

The evaluator currently handles one case per input. A multi-case campaign should aggregate per-case receipts without pooling away project-level variation. Compare policy Pareto frontiers and publish per-case and per-project results.

## 6. Blind truth and evidence integrity

In a real blind tournament, the policy must receive only the public case/action catalogue and admissible evidence. The adjudicated truth and decision-loss table are held by the evaluator until all policy plans, versions and input digests are frozen.

The current synthetic fixture contains both submissions and truth in one file for deterministic unit testing. The module cannot prove that a deployed policy did not inspect hidden truth. Production execution therefore needs a separate truth store, access-control receipt, frozen policy submission and scorer-side reveal procedure.

The receipt SHA-256 binds the exact input bytes. It does not authenticate the source data, policy implementation, human authority or reference truth.

## 7. Acceptance gates and stretch target

Structural gates:
- one common budget and catalogue per case;
- every selected action exists and is unique;
- no selected action fails authority declarations;
- no disturbance or resource limit is exceeded;
- all policy decisions resolve in the loss table;
- the oracle is clearly labelled as non-deployable;
- information gain is not used as a substitute for decision loss.

Empirical gates must be preregistered by consequence class and use independent project/site holdouts. Report realized loss, regret, direct cost, crew-hours, elapsed time, access burden, disturbance and selective abstention.

Retain the programme's stretch hypothesis: at least 20% lower out-of-sample expected decision loss at the same eligible investigation budget, or equivalent decision quality at 20% lower cost. Do not claim the target until a matched physical or otherwise admissible blind tournament demonstrates it.

## 8. Nonclaims

This v0.1 evaluator does not:
- calculate EVSI or ENBS;
- authenticate authority, permits, rights or safety approvals;
- prove policy blindness;
- establish a correct physical truth or loss model;
- estimate field performance;
- authorize survey or excavation;
- certify PAS 128 or ASCE survey quality.

The executable capability is a policy-submission and decision-loss scorer with explicit feasibility constraints. Physical and statistical qualification remain separate.
