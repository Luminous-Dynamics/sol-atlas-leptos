# Deep-Time Infrastructure Belief-State Viability
## Benchmark protocol v0.1

**Status:** finite partial-observation reference kernel committed; ten cases registered; source tests are written but not executed on the latest exact head.  
**Owner:** [DEEP-TIME-BELIEF-VIABILITY-001 #903](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/903).  
**Composes:** #898 perfect-information viability; #901 solver scaling; Symthaea #7636 adaptive pathway policy; Symtropy #3894 service outcomes.

## 1. Why belief-state viability is a distinct problem

A perfect-information controller observes the modeled state. A real long-horizon infrastructure planner often does not observe which future is occurring, which subsurface condition is true, or which disturbance model is correct. It must act on what has actually been observed.

The correct decision object is the **belief state**: the set of hidden worlds still consistent with the observation history. An action must be shared by all members of that belief. Future actions may branch only after observations distinguish successor beliefs.

This is not merely a post-hoc non-anticipativity audit. It changes the action quantifier in the solver.

## 2. Finite semantics

At each decision time, let B be the non-empty set of hidden physical/model states consistent with current observations. A belief is viable when there exists one action a such that:

1. a is admissible in every hidden state in B;
2. every mandatory service, safety, authority, rights and ecological floor passes in every hidden state;
3. for every hidden state and every declared disturbance, the successor is represented;
4. successors are grouped by the *next observable history*, not hidden model identity;
5. each complete next-observation group matches an explicitly declared successor belief;
6. every possible successor belief is viable.

One common action is chosen before hidden state and disturbance quantifiers. The solver then groups hidden successors by what the controller observes, and only then allows a new decision.

`Viable(B_t) iff exists a: admissible(a,x) for every x∈B_t AND every (x,d) successor observation-belief B' is viable.`

This is finite-horizon robust belief viability, not probabilistic POMDP planning. Disturbance and hidden-world lists are not probabilities.

## 3. Implemented reference module

[sol-atlas-core/src/belief_viability.rs](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/sol-atlas-core/src/belief_viability.rs) is exported from the shared core.

Typed inputs:
- `HiddenWorldState`: one hidden state and its mandatory floors; no policy-visible future model label;
- `BeliefState`: the set of hidden states consistent with one observable-history ID;
- `BeliefAction`: one action ID selected for the full belief, with gate dispositions per hidden member;
- `BeliefTransition`: one hidden-state/action/disturbance row plus successor hidden state and next observable-history ID.

The solver requires a complete exact successor belief for every observable successor group. Two hidden successors that produce the same observable history are merged into one belief before the next policy decision. A missing row, missing observation ID or successor-belief membership mismatch is UNRESOLVED. A known hard-floor failure in any hidden member makes the belief or action infeasible.

The limit-aware API returns BOUNDED without partial policy labels when hidden states, beliefs, actions, transition rows or expansion branches exceed declared caps.

## 4. Initial frozen cases

The [machine-readable register](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/research/deep-time-infrastructure/DEEP_TIME_BELIEF_VIABILITY_CASE_REGISTER_V0_1.json) includes ten cases:
- DBV-001 common action preserves all hidden states;
- DBV-002 hidden futures prefer conflicting actions before observations distinguish them;
- DBV-003 one common action now, separate actions after a later distinguishing observation;
- DBV-004 same-observation successor states must merge;
- DBV-005 unknown authority in one hidden state;
- DBV-006 one hidden ecological-floor failure;
- DBV-007 missing observation mapping;
- DBV-008 future hidden labels cannot split identical observable histories (specification-only);
- DBV-009 adding a failing hidden state removes viability;
- DBV-010 bounded execution negative.

Eight are executable hand-computable scenarios; DBV-008 is a specification-only control and DBV-010 checks the bounded contract.

## 5. Exhaustive differential challenge

The Rust tests include 256 tiny partial-observation games:
- one initial belief with two indistinguishable hidden states;
- two candidate common actions;
- two disturbance branches per hidden state;
- binary pass/fail terminal outcomes for each of the eight hidden-state × action × disturbance rows.

The independent truth table says the initial belief is viable iff at least one common action has all four terminal outcomes passing. The test checks the belief solver against this oracle across all 256 combinations.

Ten unit tests are written, including this exhaustive comparison. They have not yet run on the current exact head.

## 6. Baselines and metrics

Compare:
- perfect-information finite-horizon viability as an optimistic upper bound;
- open-loop fixed action sequences;
- scenario-tree policies with explicit non-anticipativity constraints;
- the exact belief-state recursion;
- Symthaea-proposed adaptive policies, independently checked by this solver.

Report separately:
- viable initial beliefs / declared belief universe;
- perfect-information gap;
- size of the belief graph and hidden-state memberships;
- action gate coverage across hidden members;
- observation partitions and merged successor members;
- policy-tree size and witness actions;
- unknown/missing evidence rate;
- false-viable and non-anticipativity violation rates;
- exhaustive oracle agreement, runtime, memory and cap disposition.

No probability-weighted result should be added unless model and disturbance weights are separately justified, source-qualified and disclosed.

## 7. Acceptance targets

- 100% exact agreement on frozen tiny games and generated cases within declared caps.
- Zero future-label leakage.
- Zero false-viable beliefs when any member fails a hard floor.
- One shared action per observable belief.
- No branching before an observation makes the belief partition distinguishable.
- Missing action/transition/observation evidence yields UNRESOLVED.
- Belief expansion cannot turn a known non-viable belief into a viable one.
- Deterministic replay; explicit BOUNDED results.

## 8. Non-goals and claim ceiling

No stochastic probabilities, model likelihoods, site physical model, autonomous actuation or investment recommendation. A passing software test proves only the stated finite belief transition semantics.