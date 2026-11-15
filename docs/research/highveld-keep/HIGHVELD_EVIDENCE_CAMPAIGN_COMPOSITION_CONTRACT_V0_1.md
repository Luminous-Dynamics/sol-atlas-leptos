# Highveld Evidence-Campaign Composition Contract v0.1

**Status:** proposed benchmark-only interface; no production owner schema binding claimed  
**Parent:** [HPT-CROSS-OWNER-REPLAY-001 #878](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/878)  
**Related owners:** Symtropy [#5164](https://github.com/Luminous-Dynamics/symtropy/issues/5164), Mycelix [#6252](https://github.com/Luminous-Dynamics/mycelix/issues/6252), Symthaea [#12046](https://github.com/Luminous-Dynamics/symthaea/issues/12046)

## 1. Why this contract exists

Two active owners solve adjacent but different problems:

- **Mycelix #6252** plans admissible acquisition of evidence for unresolved assurance obligations. Its hard gates include rights/source-use, authority, safety, schedule, physical realizability and data access. It distinguishes a task that is capable of producing evidence from evidence that is actually acquired, verified and accepted.
- **Symtropy #5164** plans multiple physical observations under spatial, temporal and shared-resource constraints. It checks candidate actions against existing reservation and occupancy state and against one another, on scratch planning state. A selected observation plan does not clear work or prove coverage.

These are complementary decision layers, not two competing next-best-scan implementations.

The composition problem is:

> Given unresolved proof obligations and candidate ways to resolve them, which admissible evidence-acquisition tasks should be selected, which of those require physical observation, can the corresponding observation campaign be scheduled feasibly, and what is the least-burden set of plans without disguising hard-gate failures or double-counting evidence?

The benchmark must not replace either owner, copy either planner's internal records, or invent a common authority.

## 2. Non-goals and binding status

This is a **benchmark-only normalized envelope**, not a production contract and not a new evidence or resource planner.

Before a real adapter is accepted, it must record:
- source repository and exact commit SHA;
- owner profile/schema identifier and version;
- exact source artifact/reference and content digest;
- transformation/adapter version and its own digest;
- semantic mapping review and owner approval;
- rights to use and disclose each source projection.

Until all of those are known, the projection is **UNBOUND**. A matching field name, JSON shape or issue title does not establish semantic equivalence.

No values in this contract authorize an inspection, resource booking, site entry, material receipt, work release, robot command, acceptance decision, professional approval, construction or occupancy.

## 3. The two-layer composition model

### Layer A — assurance acquisition admissibility (Mycelix owner)

For each unresolved proof obligation, consume the Mycelix owner's result about candidate acquisition tasks. Preserve each independent gate and its original owner evidence:

- obligation coverage / relevance;
- authority and signer scope;
- legal rights and permitted source use;
- safety permission;
- physical realizability;
- schedule admissibility;
- data access / disclosure permission;
- prerequisite closure;
- source competence and scope.

Only a candidate whose required gates are affirmatively admissible can enter a complete candidate plan. A Blocked or Unknown task remains visible as rejected, with its owner-provided reason; it must not be made selectable by adding a cost or information-value bonus.

Some evidence tasks are not physical observations: for example a source audit, clerical review or authority review may be remote. They remain in the assurance-acquisition layer and do not need a Symtropy observation action unless the owner mapping proves that they do.

### Layer B — physical observation feasibility (Symtropy owner)

Only acquisition tasks that actually require field measurement or inspection are candidates for observation-action binding.

For those bound tasks, consume Symtropy's existing owner result for:
- observation coverage against declared work needs / regions;
- action/site/workcell / epoch identity;
- time interval and uncertainty semantics;
- crew, instrument, hoist and other resource requirements;
- spatial occupancy and pairwise conflict;
- declared cost currency/minor units, crew minutes, duration and energy;
- search budget, nodes visited, completeness and optimality claim.

Do not recalculate or weaken Symtropy's reservation or occupancy semantics in the Sol-Atlas composition layer. Do not mutate live reservation/occupancy books; the owner planner's scratch-state semantics remain authoritative.

### The binding between layers

A physical evidence-acquisition task and a Symtropy observation action may be joined only by an explicit reviewed owner binding. The binding must preserve exact asset/component, design/configuration revision, site frame/epoch, measurement purpose, source, calibration/configuration and decision scope.

A source-review task without a physical observation remains a Mycelix acquisition. A physical observation with no admissible assurance obligation may be useful operationally but cannot be counted as satisfying a proof obligation in this benchmark.

An absent, ambiguous, many-to-one or stale binding is **UNBOUND / NOT COVERED** until resolved. Never synthesize a relationship from similar titles, IDs, timestamps or geometric proximity.

## 4. Benchmark-only normalized envelope

The following are semantic concepts for test fixtures, not asserted fields in Symtropy or Mycelix production schemas.

| Concept | Purpose | Authority / owner |
|---|---|---|
| Decision episode | Exact decision scope and cutoff time | Decision owner; source refs only |
| Unresolved obligation | Claim/proof-obligation ID, criticality, exact subject/revision and missing evidence type | Mycelix |
| Candidate acquisition | How evidence might be obtained; cost/currency; prerequisites; source and scope | Mycelix |
| Acquisition admissibility | Separate gate dispositions and reasons; Unknown remains Unknown | Mycelix |
| Observation action | Physical task, coverage, site/epoch, times, crew/instrument/resource use and energy | Symtropy |
| Acquisition/action binding | Reviewed relation for physical acquisitions only | Owner-approved adapter |
| Existing reservations / occupancy | Current context and constraints used for scratch admission | Symtropy |
| Candidate plan | Selected acquisition and observation references, rejected alternatives, remaining obligations | Composition result |
| Observation result | Actual acquired measurement with source, time, calibration, coverage and digest | Physical observation owner |
| Qualification result | Independent decision whether acquired evidence satisfies the obligation | Relevant claim/acceptance owner |

The envelope carries references to these records; it does not copy or replace them. Exact property names, enumeration values, temporal conventions and identifiers must be mapped from pinned owner profiles before any non-synthetic adapter can exist.

## 5. Planning and optimization semantics

### Hard gates first

Hard-gate eligibility is non-compensatory. An action with an Unknown or Blocked authority, rights, safety, physical-realizability, schedule or data-access condition is excluded from the admissible set. It remains in the report with the owner's reason.

Neither lower cost, higher information gain, criticality, nor increased coverage can compensate for a failed hard gate.

### Complete coverage

Let U be the set of unresolved assurance obligations. A candidate plan is **obligation-complete** only when its selected admissible acquisition tasks can cover all elements of U under the declared benchmark mapping.

This is planned obligation coverage only. It is not a successful measurement, evidence verification, claim satisfaction or acceptance.

Where a physical observation is necessary, the plan is complete only if the required owner binding exists and the selected Symtropy campaign is resource/space-time feasible. Do not count an unbound or infeasible observation as coverage.

### Separate objective dimensions and expose the Pareto set

Do not collapse different owners' objectives into an opaque weighted score.

Report separately:
- evidence-acquisition cost by currency and minor units;
- physical observation cost by currency and minor units;
- crew minutes, separately by role where the owners expose them;
- instrument / shared-resource occupancy;
- elapsed campaign span;
- energy;
- number of acquisition tasks and observation actions;
- uncovered obligations and rejected actions.

Never add costs across currencies. Convert currencies only with an explicitly versioned rate source, rate time, uncertainty and user-approved accounting basis; otherwise report separate ledgers.

For the joint finite benchmark, first exclude all hard-gate and resource-infeasible subsets, then return the **Pareto frontier** over the declared burden dimensions. Preserve tied and dominated plans with reasons. A single selected plan requires an explicit decision-owner objective or policy; the benchmark must not invent weights.

The Pareto report complements—not replaces—the owners' own objective contracts:
- Mycelix can report its minimum acquisition-cost and minimum acquisition-count optima over its declared universe.
- Symtropy can report its existing lexicographic observation-campaign objective: complete coverage, crew minutes, same-currency observation cost, energy, wall-clock span, action count and stable IDs.
- The composition evaluator reports the joint frontier and exact feasibility facts without silently rewriting either ordering.

### Search truthfulness

Distinguish:
- complete plan found and optimality proven over the exact declared candidate universe;
- complete plan found but search is incomplete / optimality not proven;
- partial plan with explicit uncovered obligations;
- no feasible complete plan in the exhaustively searched declared universe;
- no admissible candidates;
- input invalid or owner projection UNBOUND.

A node-budget exhaustion is not infeasibility. A candidate universe not fully enumerated cannot support a global optimality claim.

## 6. Required adversarial fixtures

Start with at least 15 deterministic synthetic fixtures. Each should carry an independent expected outcome and be tested under input permutation.

1. **Admissible remote source audit:** resolves one obligation but has no physical observation; it must remain a valid acquisition without a fake Symtropy action.
2. **Admissible physical survey:** explicit owner binding exists and a feasible campaign covers the declared obligation.
3. **Cheap authority-blocked acquisition:** excluded even though it would make the nominal plan cheaper.
4. **Rights-Unknown source:** excluded; no public redistribution or use by virtue of a digest.
5. **Safety-Unknown observation:** excluded even if its geometry/coverage looks complete.
6. **Unbound acquisition/action pair:** physical task has no reviewed owner binding and contributes no obligation coverage.
7. **Two individually feasible observations with a shared instrument conflict:** joint selection must reject the conflict.
8. **Spatially incompatible observations:** both are excluded from one simultaneous campaign under the owner's occupancy semantics.
9. **Shared resource capacity exceeded:** individually valid actions cannot be selected together.
10. **Prerequisite blocked:** a dependent action cannot be selected until the prerequisite is admissibly covered.
11. **Common-cause sensor evidence:** two reports sharing one calibration/controller/source cannot be counted as independent corroboration.
12. **Currency mismatch:** aggregate comparison is rejected or reported in separate ledgers; no arithmetic across currencies.
13. **Budget/node cap exhausted:** report SEARCH_INCOMPLETE and retain uncovered obligations; never claim infeasible or optimal.
14. **No feasible complete plan:** partial coverage and exact blockers are reported.
15. **Complete plan exists only through non-greedy combination:** exact tiny oracle finds it; greedy baseline reports its actual gap.
16. **Inconclusive measurement:** planning coverage is recorded, but the obligation remains unresolved after the outcome.
17. **Late correction or revocation:** the as-of-decision result is recomputed; historical events are preserved.
18. **Duplicate action / obligation / binding ID:** typed input rejection, not silent overwrite.
19. **Plan permutation:** objective values and Pareto frontier remain invariant under row/action order; digest still binds the exact byte representation.
20. **Owner output tampering:** changed source digest, schema version, site epoch or adapter digest invalidates the bound projection.

The first 15 form the v0.1 tiny-oracle minimum. Cases 16–20 expand to outcome-aware and adversarial owner-replay tests.

## 7. Baseline arms and benchmark method

Run the exact same frozen obligation set, candidate catalogue, rights/authority/safety dispositions, cost ledger, site/epoch and reservation books through:

- **B0:** no campaign / all obligations remain unresolved.
- **B1:** Mycelix assurance-acquisition planner alone.
- **B2:** Symtropy observation-campaign planner alone.
- **B3:** sequential composition (Mycelix first, then Symtropy).
- **B4:** joint exact exhaustive oracle for small finite fixtures.
- **B5:** candidate joint system, with no access to oracle labels.

Use separately implemented oracle logic. The oracle must not call the same planner implementation's feasibility or objective functions to generate its expected answers. For small fixtures, enumerate all subsets independently and compare every selected/rejected action, objective vector, uncovered obligation and search-completeness state.

Report performance by fixture family. Do not aggregate away a hard-gate violation.

## 8. Metrics and prospective targets

These are proposed targets, not results.

| Measure | Target |
|---|---:|
| Exact agreement with independent joint oracle on all tiny fixtures | 100% |
| Blocked/Unknown/rights-ineligible acquisition selected | 0 |
| Unbound physical acquisition/action relation counted as coverage | 0 |
| Spatial/resource conflict selected | 0 |
| Currency arithmetic across unlike currencies | 0 |
| Search-budget exhaustion misreported as infeasible or optimal | 0 |
| Planned observation promoted to acquired evidence | 0 |
| Acquired evidence promoted to qualified proof without verifier | 0 |
| Production reservation/occupancy mutations from planning | 0 |
| Same input, same semantic report under input permutation | 100% |

Also report:
- exact joint Pareto-front agreement on tiny cases;
- Pareto regret of sequential/greedy baselines against the independent oracle;
- percentage of critical obligations with a genuinely admissible, feasible evidence path;
- number of requests per defensible decision;
- evidence-acquisition and observation labour/cost/energy separately;
- proportion of acquired outcomes that are inconclusive, stale, rejected or not decision-sufficient;
- time-to-defensible-decision, reviewer effort and rights/disclosure burden;
- abstention correctness when no admissible, feasible plan exists.

## 9. Execution sequence

1. **Reference-only:** complete the benchmark envelope and tiny independent oracle without connecting production owner internals.
2. **Owner mapping:** obtain exact Symtropy #5164 and Mycelix #6252 projection schemas, profile versions, branch/head commits and tests. Mark every unknown mapping UNBOUND.
3. **Synthetic adapter replay:** map the same synthetic Highveld decision episode into the approved projections, consume owner results and record only their references/digests.
4. **Composition tests:** run B0–B5 and compare to the independent joint oracle; verify no owner state mutation.
5. **Adjudication:** independently review expected decisions and freeze gold labels and hidden holdout under a trusted custodian.
6. **Physical observation:** only after a separate worksite/cell safety and competent metrology process; planning remains non-authorizing.

## 10. Claim ceiling

A perfect benchmark result shows only that the candidate reproduces the independently specified planning result for the frozen synthetic candidate universe and owner projections. It does not prove actual observation coverage, instrument calibration, legal admissibility, source authenticity, material acceptance, robot safety, physical installation, building compliance, professional approval, or fitness for occupancy.

