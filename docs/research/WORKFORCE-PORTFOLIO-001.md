# SA-WORKFORCE-PORTFOLIO-001 — Shared-resource intervention portfolio benchmark v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/285  
**Composes with:** diagnosis #268, source profile #272, outcomes/additionality #277, active evidence #279, source robustness #281, chain conformance #283.  
**Status:** synthetic exact-oracle benchmark; no real allocation or service impact claim.

## Paradigm shift: optimize the portfolio, not isolated cases

A low-cost intervention for one case can consume the only certified supervisor, tool kit, qualified shift or dispatch slot needed by a more critical task. A system that ranks or solves each case independently can therefore produce a globally infeasible portfolio even when each local choice appears reasonable.

This benchmark treats an intervention as an indivisible, evidence-bounded bundle with a resource-demand vector and a service-outcome vector. It enumerates combinations against shared resource caps and mandatory service floors, then returns the non-dominated portfolio frontier. That exposes real trade-offs among service restored, cash, staff effort, delay and worker burden without inventing a single weighted score.

## Decision pipeline

1. Candidates arrive from the diagnostic/intervention layer with explicit prerequisite, funding, authorization and job-quality status.
2. The portfolio evaluator filters ineligible, forbidden or indeterminate candidates before optimization.
3. Exact subset enumeration enforces one bundle per case, dependency closure, resource capacities and declared worker-burden ceilings.
4. Essential-service floors are enforced as hard feasibility constraints.
5. Among the feasible portfolios, Pareto dominance compares all declared service benefit dimensions against every declared cost/burden dimension.
6. If no portfolio can meet the floor, the evaluator returns an explicit infeasibility code; it never relaxes the floor or presents an incomplete cheap plan as feasible.

This is not an autonomous dispatcher or work authorization system. A human decision owner chooses among non-dominated portfolios under a predeclared policy. Unknown evidence never becomes permission. A digest or a solver result is not authority.

## Hard constraints versus objectives

**Hard constraints:** eligibility, prerequisites, confirmed funding, authorization, viable work conditions, one selected bundle per case, declared dependencies, shared resource capacities, service floors and any explicit worker-burden ceiling.

**Pareto objectives:** service units by category (maximize) and resource use, cash, staff minutes, delay and worker burden (minimize). The solver preserves service dimensions independently; it must not trade away an essential floor for a lower price or faster completion.

Resource caps constrain qualified person-shifts, supervisor hours, tool kits, dispatch hours and cash. The reported vector also includes staff minutes, elapsed delay and worker burden hours. Counts and sums use integer arithmetic with checked overflow.

## Research rationale and industry context

Multi-objective personnel rostering literature recognizes distinct employer, worker and service-recipient interests and warns that weighted-sum scalarization dominates much of the research even though it hides preference trade-offs. A 2025/2026 review surveyed 52 multi-objective personnel-rostering papers; a separate 2025 fairness-aware workforce resource-selection paper applies Pareto-based multi-objective optimization. These are method comparators, not evidence this implementation beats them.

- Böðvarsdóttir & Stidsen, *A review of multi-objective optimization methods for personnel rostering problems* (published 2025; journal volume 2026): https://link.springer.com/article/10.1007/s10951-025-00845-0
- *Workforce Management and Resource Selection with Fairness* (IFAC-PapersOnLine, 2025): https://doi.org/10.1016/j.ifacol.2025.09.364
- Microsoft Dynamics 365 Resource Scheduling Optimization proof-of-concept guidance identifies resource counts/types, skills, travel, priorities and work constraints as essential success criteria: https://learn.microsoft.com/en-us/dynamics365/field-service/rso-proof-concept
- NIST TEVV-Athlon for use-case-specific evaluation: https://www.nist.gov/artificial-intelligence/ai-research/tevv-athlon-framework-evaluating-ai-systems

Commercial schedulers already handle skills, availability, travel, shift limits and priority constraints. This portfolio benchmark targets the complementary upstream question: which already-authorized intervention bundles can fit a jointly constrained resource budget while retaining essential service and worker protections? A direct industry comparison must use the same cases, constraints, resources, objective dimensions and verified outcomes.

## Frozen v0.1 baselines and measurements

- B0: independently select the cheapest candidate per case (negative control; no global coordination).
- B1: essential-service priority first, then cheapest feasible bundle.
- B2: transparent resource-aware greedy heuristic.
- B3: exact subset/Pareto oracle for the tiny corpus.

The first executable slice implements B3 and tests it against the hand-authored portfolio frontier. Later heuristics should be compared against this oracle on small cases before larger stress cases are introduced.

Report feasible portfolio count, non-dominated portfolio IDs, service benefit vector, each resource used, staff minutes, delay and worker burden. In infeasible cases, report unmet essential-service floors and the resource/worker-burden constraints that prevent a feasible solution. Include explored subset count and the exact-oracle cap; exceeding that cap must be explicit, never silently truncated.

## Acceptance gates

- Exact frontier and feasibility status match all independent tiny-instance references.
- Zero unauthorized, forbidden, unknown, unfunded, prerequisite-failing or nonviable bundles selected.
- Zero shared-resource cap, worker-burden cap or essential-service-floor violations.
- One bundle per case; every selected dependency is present.
- A service floor is never traded for lower cost or less delay.
- An infeasible case returns named codes and no feasible portfolios, with no auto-relaxation.
- Input row permutation yields identical canonical report and frontier.
- Fixture/schema digests and a synthetic-only claim ceiling appear in the receipt.

## Claim ceiling

The fixture values are synthetic. A passing oracle demonstrates consistency with hand-enumerated tiny portfolio cases only. It does not estimate real service levels, assign real staff, authorize work, prove job creation or beat a production scheduler.