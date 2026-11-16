# Deep-Time Infrastructure Viability Kernel
## Benchmark protocol v0.1

**Status:** 12 finite-state cases, nine hand-computable references and three specification-only controls. No real infrastructure qualification.  
**Owner:** [DEEP-TIME-VIABILITY-KERNEL-001 #898](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/898).  
**Composes:** #890 service continuity; #894 topology renewal; #896 model robustness; Symthaea #7636 adaptive policy; Symtropy #3894 service cascades; Mycelix #5923/#5762 dependency semantics.

## 1. Paradigm: infrastructure as a viability space

Asset redundancy asks how many paths exist now. Recovery asks how quickly service returns after a disruption. Adaptive planning asks which action should be taken as observations arrive.

A viability kernel asks a stronger question:

**From which system configurations does there exist a policy that keeps every mandatory service, safety, rights and ecological floor satisfied under all declared disturbance sequences?**

This captures the ability to reorganize infrastructure before an irreversible threshold is crossed. A city or region is not robust merely because it has two routes today; it is robust when it retains enough time, buffer, authority, resource and topology options to reach another viable configuration.

## 2. Finite robust-viability semantics

For finite state set S, admissible actions A(s), and disturbance set D(s,a), define terminal states by explicit hard floors. The backward recursion is:

`V_T = {s | every mandatory terminal floor passes}`

`V_t = {s | floors(s) pass AND exists admissible a in A(s) SUCH THAT for all d in D(s,a), T(s,a,d) belongs to V_(t+1)}`

The action choice is existential; the disturbance quantifier is universal. No event probabilities are required.

Distinct results:
- **Weak viability:** some path succeeds for at least one disturbance sequence.
- **Robust viability:** one admissible policy preserves every mandatory floor across every declared disturbance sequence.
- **Adaptive viability:** policies may branch only on observations available at that decision time.
- **Non-anticipativity:** histories with identical observable evidence receive identical actions; hidden future model labels are not policy inputs.
- **Eligibility:** authority, rights, consent, safety and ecology are non-compensatory gates.
- **Unknown:** missing evidence, action preconditions or disturbance outcomes produce UNRESOLVED, never optimistic viability.
- **Reachability is not viability:** a state can be reachable through a transition that breaches a service or ecological floor; that path is invalid.

## 3. State, action and disturbance contract

A state should carry:
- topology/configuration identity and time step;
- active, commissioning, degraded, retired and restored asset states;
- service level and beneficiary-group floor vectors;
- buffers/storage and exhaustion times;
- ecological state/threshold variables;
- rights, authority and consent gates;
- common-cause failure groups;
- observation and evidence-frontier state;
- resource, cost, material and carbon budget state.

An action should carry:
- preconditions and explicit admissibility gates;
- affected configuration and service;
- lead time, duration and temporary capacity effects;
- required crew/equipment/energy/material;
- lifecycle cost and carbon/material ledger deltas;
- ecological, decommissioning and restoration consequences;
- trigger and evidence availability time.

A disturbance should be a named branch in an explicit scenario set. Do not infer probabilities from the list or assume independence between branches.

## 4. Twelve initial cases

The machine-readable register contains 12 unique cases:
- DVK-001 — finite single-path reachability;
- DVK-002 — one-period versus two-period buffer for a two-step relocation;
- DVK-003 — future identity leakage / non-anticipative decisions;
- DVK-004 — two nominal routes share one power dependency;
- DVK-005 — failed rights/authority gate removes the only transition;
- DVK-006 — freight pass cannot compensate for ecological-flow failure;
- DVK-007 — weak viability versus robust viability;
- DVK-008 — old corridor retires before replacement commissioning;
- DVK-009 — shared restoration crews/equipment;
- DVK-010 — disturbance-set expansion monotonicity;
- DVK-011 — delayed signpost and action lead time;
- DVK-012 — matched fixed/contingency/adaptive policy comparison.

Nine cases have hand-computable outcomes. Three remain specifications requiring fuller finite action/observation encoding. The exact-state register is a reference contract, not a real-world model.


## 5.1 Executable Rust reference kernel

The shared core now contains [sol-atlas-core/src/viability_kernel.rs](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/sol-atlas-core/src/viability_kernel.rs), exported by sol-atlas-core.

The exact reference implementation provides:
- finite-horizon backward recursion using existential controller choice and universal disturbance branches;
- conjunctive mandatory state floors and separate action admissibility gates;
- explicit VIABLE / INFEASIBLE / UNRESOLVED results;
- a witness action for every viable non-terminal state;
- an input-order-independent lexicographic witness tie-break;
- rejection of malformed graph references and non-advancing transitions;
- non-anticipativity checking across identical observable histories;
- checked observation-delay + action-lead-time arithmetic;
- configurable horizon/state/action/transition-branch caps, returning BOUNDED with no partial viability classifications when exceeded.

The default caps are 128 stages, 50,000 states, 250,000 actions and 2,000,000 estimated transition branches. These are engineering defaults for a reference implementation, not performance claims or universal safe limits. The lower-level limits API supports stricter or larger declared caps.

The Rust module currently has 21 unit tests written, including the frozen DVK exact cases, unknown/failing evidence controls, deterministic witness ordering, explicit BOUNDED execution and a 64-game exhaustive truth-table comparison. The tests still require exact-head execution in CI before being described as passed.

## 5.2 Exhaustive finite viability oracle

The machine-readable panel is frozen:
- [DVT exhaustive oracle JSON](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/research/deep-time-infrastructure/DEEP_TIME_VIABILITY_KERNEL_EXHAUSTIVE_ORACLE_PANEL_V0_1.json)
- [DVT exhaustive oracle schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/research/deep-time-infrastructure/DEEP_TIME_VIABILITY_KERNEL_EXHAUSTIVE_ORACLE_PANEL_V0_1.schema.json)

It enumerates all 64 one-stage games with one initial state, two terminal states (one passing and one failing), two controller actions and three disturbance branches. Each action has eight possible successor patterns. The independent truth-table oracle states that the initial state is viable iff at least one action maps every disturbance to the passing terminal. The test compares the finite kernel with that independent oracle for all 64 games.

DVK-003 (non-anticipativity) has also been promoted from specification-only to a hand-computable reference because the Rust kernel now directly rejects different actions for identical observable histories. DVK-009 (shared crews/equipment) and DVK-012 (matched comparison across policy families) remain specification-only.

## 5. Key exact reference answers

**DVK-001:** The single start→commissioned transition passes all floors under the only disturbance. Both states are viable.

**DVK-002:** A two-step relocation is viable with two periods of service buffer, but not one. The replacement must be commissioned before the buffer expires.

**DVK-004:** Two nominal routes share SUB-1. Because the robust disturbance set includes SUB-1 outage and both routes depend on it, the initial state is not robustly viable (though it has a successful normal branch).

**DVK-005:** The only transition fails its mandatory authority gate. No admissible action remains; the state cannot be declared viable.

**DVK-006:** Freight service passes but environmental flow fails. Conjunctive hard floors mean the transition is not viable.

**DVK-007:** A single action succeeds in the normal disturbance and fails in the compound disturbance. Weak viability is true; robust viability is false.

**DVK-008:** The future terminal is service-capable, but the intermediate state breaches the freight floor. The initial state is not viable.

**DVK-010:** Under {normal}, the initial state is viable. After adding power-failure to the disturbance set, the robust kernel shrinks. It cannot expand solely because the disturbance set got larger.

**DVK-011:** Signal crosses at t=4; observable at t=6; relocation completes at t=9; service floor deadline t=8. The action is one time unit too late.

## 6. Required baselines

Compare:
- fixed topology / no adaptation;
- fixed master plan plus contingency list;
- greedy one-step repair / lowest immediate cost;
- shortest recovery time;
- unweighted minimax/minimax regret;
- exact exhaustive backward viability oracle on small finite state spaces;
- adaptive policies proposed by Symthaea #7636, evaluated independently here;
- service consequence trajectories from Symtropy #3894.

Do not build a second policy generator or service-cascade simulator.

## 7. Metrics

Report separately:
- robust viable-state count and share of declared finite state universe;
- weak-only versus robust viability gap;
- witness actions/policies for accepted states;
- worst-case service floor margin and deficit area;
- buffer reserve and lead-time margin;
- reconfiguration latency and shared-resource contention;
- reachable distinct terminal configurations;
- common-cause cut sets that empty the viable path set;
- rights/authority/ecology/safety failure dispositions;
- non-anticipativity violations;
- exact-oracle agreement, state-space size, runtime, memory and any search bound.

Never compare raw viable-state counts across state spaces with different encodings or universes. Do not collapse all dimensions into a single resilience score.

## 8. Proposed acceptance targets

- 100% exact agreement with an independently implemented backward oracle on enumerated small fixtures.
- Zero false-viable results on rights, ecology, authority, common-cause, transition-gap and hidden-future controls.
- Zero future-information leakage.
- Zero optimistic viability from unknown required evidence.
- Adding a disturbance cannot enlarge the robust viability set without a declared change to actions/observations.
- Deterministic replay for pinned finite cases.
- Resource caps produce explicit BOUNDED/INCOMPLETE, not false exactness.

## 9. Claim ceiling

A passing finite-state benchmark proves only the stated transition and policy semantics on its synthetic state universe. It does not qualify a real city, port, canal, rail corridor, water allocation, ecological response, construction sequence, consent process or investment decision.
