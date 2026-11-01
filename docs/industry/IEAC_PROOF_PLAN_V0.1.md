# IEAC proof-obligation acquisition planner v0.1

**Status:** Resource-bounded action-set planner. It is not yet a calendar/resource-concurrency scheduler. It does not execute tests, verify the evidence those tests may later produce, qualify an IEAC profile or authorize operations.  
**Tracks:** [IEAC-PROOF-PLAN-001 / #399](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/399).  
**Upstream:** risk-to-control selector #398 and its fixed mandatory control set.  
**Next tranche:** full calendarized evidence-acquisition scheduling and external RCPSP benchmarking.

## 1. The paradigm shift

The object being planned is not an ordinary project task list. It is the acquisition of **proof obligations** attached to a specific industrial decision.

`risk vector → required controls → admissible existing evidence → candidate proof actions → resource-feasible acquisition plan → execution → independent evidence verification → IEAC re-evaluation`

The planner can optimize the order and combination of evidence-acquisition actions. It cannot optimize away a mandatory control, lower the evidence-domain threshold, substitute a cheaper test for independent evidence, or convert a planned action into completed evidence.

The planner's job is therefore lexicographic:
1. Preserve the exact mandatory control set, decision tier, rights and independence constraints.
2. Find a set of actions whose prerequisites, conflicts, rights and evidence-domain requirements are satisfied.
3. Keep every resource total within its declared hard budget.
4. Minimize declared cost in the frozen objective order.
5. If no admissible plan exists, return exact blockers for bounded cases; return no-feasible-plan-found for a larger heuristic case without claiming proven infeasibility.

## 2. Current implementation boundary

The standalone Rust package is `tools/industry-proof-plan`.

**v0.1 solves a resource-bounded action-selection problem.** Each candidate action has aggregate costs for:
- money in micro-units of an explicitly declared three-letter currency;
- reviewer minutes;
- field-interruption minutes;
- compute in micro-units of an explicitly declared compute unit;
- duration minutes.

The request pins `currency_code` and `compute_unit_id`; budgets and all action costs share those declared units. No currency or compute conversion is performed.

The planner validates the prerequisite graph, requires symmetric conflict declarations, includes prerequisite closure, checks hard aggregate budgets, enforces the minimum count of distinct evidence-independence domains per control, and emits a deterministic topological action order.

Duration is conservatively summed in v0.1. The order is serial and prerequisite-respecting. There are no start-time variables, resource-capacity profiles, calendars, shifts, deadlines, parallel execution or time-dependent site windows. It is therefore **not yet a full resource-constrained project scheduling problem (RCPSP)**. The next scheduler tranche is explicitly separate.

### 2.1 Input anchors

Every request binds:
- the risk-control report digest;
- the underlying IEAC profile digest;
- the risk-control policy digest;
- one pinned independence-registry digest;
- one pinned control-mapping-policy digest;
- a normalized planning time;
- a fixed, mandatory control set with trigger reasons and minimum independent domains;
- current existing evidence receipts;
- candidate action specifications and their rights receipts;
- resource-unit identities and hard aggregate budgets.

All existing-evidence and candidate-action registry rows must match the request-pinned independence registry. Every candidate action's control mapping digest must match the request-pinned mapping policy. These pins allow an independent checker to replay the exact assumptions. They do not authenticate the registry, the mapping policy or the rights receipt; their signatures, authority and content must be verified by their owning trust/evidence validators.

### 2.2 Evidence and action semantics

Existing evidence is counted only when:
- verification status is `verified`;
- rights status is `verified`;
- verification time is not in the future;
- validity has not expired at the request's planning time;
- the evidence artifact, verifier build, independence domain, registry and rights-evidence digest are bound.

Candidate actions are not evidence. They remain planned until an action is executed, produces an artifact and is accepted by the independent verifier. Their declared independence domain is a prospective evidence source domain, not proof that the source is truly independent.

A control is considered *planned-covered* only when the set of distinct verified existing domains plus distinct domains represented by selected actions reaches its `minimum_independent_domains`. The report labels the domains as planned-or-existing and preserves existing artifact/verifier/rights identities separately from candidate action identities.

The planner does not authenticate whether two organizational domains are truly independent. That requires a trusted, versioned independence registry and an independent verifier. The planner only requires all rows to refer to one pinned registry digest and does not silently treat duplicate action count as independent domains.

### 2.3 Rights and time

Candidate actions require an explicit rights status, rights-evidence digest, verification time and validity horizon. An action whose rights have expired, are future-dated or are not verified is rejected before planning. Timestamps are real UTC calendar values, not merely strings that happen to sort lexicographically.

Existing evidence that is expired, revoked, restricted or unverified does not count toward closing a mandatory proof obligation. It remains visible to upstream evidence review.

### 2.4 Human approvals

The report separates:
- control-level approvals inherited from the mandatory proof-obligation set;
- action-level approvals required before the proposed evidence acquisition runs.

The combined approval list prefixes each item as `control:<id>` or `action:<id>`. A plan is not an authorization to execute any listed action.

## 3. Algorithms and honesty about optimality

### 3.1 Exact bounded solver

For at most 20 candidate actions, v0.1 performs exhaustive subset enumeration. For each subset, it checks prerequisite closure, mutual conflicts, independent-domain coverage and aggregate budgets. It uses this lexicographic objective:

1. money;
2. field-interruption minutes;
3. reviewer minutes;
4. serialized duration minutes;
5. compute units;
6. action count;
7. lexicographic action IDs.

The exact planner reports `proven_optimal` only after exhausting the complete candidate subset space. It reports `proven_infeasible` only when no subset satisfies the fixed requirements and hard constraints. An empty action catalog is valid: existing evidence can produce an optimal zero-action plan, while remaining gaps with no candidate actions produce proven infeasibility.

### 3.2 Large-set heuristic

Above 20 candidate actions, v0.1 uses deterministic greedy selection over dependency closures. Its progress metric is the remaining *independent-domain deficit*, not merely the number of controls still open; this matters when a control needs a second or third independent source.

A feasible result is labelled `feasible_heuristic` and `not_proven`. Failure to find a plan is labelled `no_feasible_plan_found`, not `infeasible`. The heuristic does not claim a lower bound or optimality gap yet.

The solver never changes the mandatory control set to improve its score. If its budget or candidate set cannot close the obligations, the exact case returns infeasible; the heuristic case returns no-feasible-plan-found and the current unmet controls.

## 4. Benchmark programme

The current authored Rust tests cover:
- exact action-set sizes of 5, 10, 15 and 20;
- deterministic heuristic sizes of 50, 100 and 500;
- the frozen 11-action hand-calculated fixture;
- resource-budget boundaries;
- independent-domain requirements, including the case where repeated actions use the same domain;
- prerequisite inclusion and topological order;
- cyclic prerequisites and asymmetric conflicts;
- absent action catalog with open proof gaps versus a fully evidence-satisfied zero-action plan;
- expired/revoked evidence and candidate action rights;
- invalid calendar values and unsupported decision tiers;
- request-pinned registry and control-mapping-policy mismatch;
- control-level versus action-level human approval gates;
- report binding to exact request bytes and identities;
- duplicate JSON object members and a SHA-256 known-answer vector.

### 4.1 Promotion targets

| Measure | Target |
|---|---|
| Exact solver | 100% agreement with an independently implemented exact checker on every bounded case |
| Infeasibility | 100% correct proof of no admissible subset for bounded cases |
| Mandatory controls | Zero omitted or silently weakened controls |
| Resource budgets | Zero overruns |
| Independence | Zero credit for duplicate actions within the same declared source domain |
| Rights freshness | Zero expired, revoked, future-dated or rights-unknown candidate actions accepted |
| Dependencies | Zero cycles, missing prerequisites, or dependent-before-prerequisite execution orders |
| Approval gates | 100% preservation of control-level and action-level approval requirements |
| Authority | Zero plan reports that authorize actions or attest qualification |
| Heuristic integrity | Never claim optimality or infeasibility without a proof |
| Utility | Report reviewer time, field interruption, compute, cost and blockers separately |

These are software and policy acceptance targets, not incident probabilities or physical safety guarantees.

### 4.2 Frozen and comparative cases

The current fixture is a synthetic bounded instance with six mandatory controls, eleven candidate actions, a verified rights-evidence row, two-control independent-domain requirements, a prerequisite chain, mutually exclusive alternatives, one multi-control candidate action and five hard resource budgets. The hand-calculated optimum selects eight actions and uses the full declared budget. This is a regression fixture, not an industrial performance dataset.

The next dataset tranche should use:
- 5/10/15/20-action exact instances with frozen oracle solutions;
- 50/100/500-action heuristic scaling cases;
- varying prerequisite depth and fan-out;
- shared-source, common-cause and correlated-evidence traps;
- multiple domain thresholds per control;
- revoked and expired rights;
- insufficient capacity and no-plan cases;
- reviewer, compute, field-interruption and cost budget sweeps.

For the full calendarized scheduler, use PSPLIB as an external generic RCPSP reference and add assurance-specific constraints. PSPLIB was created as a systematic benchmark library for single- and multi-mode resource-constrained project scheduling. The University of Ghent project-data catalogue also lists newer and larger RCPSP sets, including RG300, hard exact-method cases and heuristic-method cases. Do not report only solve time: compare feasible rate, objective gap to best-known/lower bound, constraint violations, robustness and reproducibility.

## 5. Industry and SOTA comparison

Google OR-Tools CP-SAT already models interval variables, precedence constraints, no-overlap and cumulative resource capacities. It is an appropriate baseline for the next calendarized scheduler, not a target to beat using the current subset-selection version.

| Capability | Generic project scheduling / CP-SAT | IEAC proof planner v0.1 |
|---|---|---|
| Dependencies and conflicts | Strong generic scheduling primitives | Explicit DAG and mutual-exclusion validation |
| Aggregate resource budgets | Modelable | Built into exact/heuristic subset selection |
| Time intervals and concurrency | Native interval/no-overlap/cumulative modelling | Not implemented yet; serial order and total-duration budget only |
| Multiple evidence sources | Not an intrinsic scheduling concept | Distinct independence domains per control |
| Control/evidence mapping | Application-specific modelling | Pinned mapping-policy digest on every action |
| Rights freshness and revocation | Application-specific modelling | Explicit rights receipt and valid-time checks |
| Mandatory safety/control set | Must be encoded by caller | Fixed, non-compensating proof-obligation set |
| Independent verification | Caller must build it | Separate auditor is the next required implementation step |
| Qualification/actuation | Outside generic solver | Explicitly prohibited by report contract |

The differentiator is not a claim that the planner solves RCPSP better than OR-Tools. It is that a generic solver's output can be treated as an untrusted proposal and independently checked against the assurance contract: mandatory control coverage, rights, evidence independence, approvals, exact input digests and no authority escalation. A future scheduler can use CP-SAT or another solver to propose times; the independent Rust auditor should verify the schedule without trusting the solver.

The target is to outperform current industry practice in **assurance integrity and reproducibility**, then compare scheduling efficiency under matched resource/time constraints.

## 6. Paradigm shifts and next architecture

1. **Proof-carrying plans:** the plan binds request, profile, policy, registry, mapping policy, actions, approvals, resource totals and exact source digests.
2. **Independent optimizer verification:** the optimizer is untrusted. A separately implemented auditor checks feasibility, objective and exact optimality claims.
3. **Evidence independence as a constraint:** source domain and registry identity are part of control coverage, not an unexamined metadata count.
4. **Rights-aware scheduling:** data, facility, software and measurement rights are validity-bounded inputs to action admissibility.
5. **No-budget-laundering:** a plan never reduces mandatory controls to fit a cost envelope. A budget shortfall is a blocker, not a score penalty.
6. **Adaptive evidence acquisition:** later work can use expected net value of information to choose the next measurement or test. Its probabilities, calibration and costs must themselves be evidenced; entropy gain alone is not the objective.
7. **Outcome reconciliation:** after a proof action and subsequent industrial decision run, compare predicted and observed outcomes and trigger targeted requalification when evidence drifts.

## 7. Full RCPSP scheduler tranche

The next scheduler should introduce:
- start/end variables and precedence lags;
- per-resource capacities over time (reviewers, labs/HIL rigs, field crews, compute, downtime);
- calendars, shifts, access windows and deadlines;
- optional action modes and alternative qualified providers;
- non-overlap and cumulative resource constraints;
- staged/conditional action branches for pass/fail/unknown results;
- a makespan objective after hard assurance constraints;
- an external CP-SAT baseline and a separately implemented Rust feasibility checker.

Benchmark it against PSPLIB and newer RCPSP datasets, then against matched assurance-specific instances. The assurance-specific contribution is the proof-obligation/evidence semantics; generic scheduling quality must be measured using the established OR benchmarks.

## 8. Claim ceiling

The current report establishes only that a set of declared acquisition actions is structurally feasible under supplied assumptions and aggregate budgets, or that no subset exists in the bounded candidate set. It does not prove that a planned test will pass, that a source is genuinely independent, that the action rights evidence is authentic, that an industrial site has authorized the work, or that a resulting IEAC profile is qualified.

No Rust compilation, Clippy result or CI pass is claimed until the workflow runs on the exact PR head. Static inspection and hand-calculated fixture reconciliation must not be presented as executable verification.
