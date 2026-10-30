# Energy Dispatch Policy DSL v0.1 — design contract

**Status:** Draft design; no policy compiler or runtime is claimed  
**Issue:** [ENERGY-DISPATCH-POLICY-DSL-001 #315](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/315)  
**Parent benchmark:** [ENERGY-DISPATCH-REGRET-001 #310](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/310)

## 1. Decision

Define a bounded, declarative, data-only Energy Dispatch Policy format that selects among schedules already returned by the exact finite dispatch oracle.

The compiler does not generate schedules, change the physical model, or authorize dispatch. It transforms a frozen policy document into a typed policy IR. A separate selector ranks only the oracle's complete Pareto alternatives. The existing independent schedule verifier then rechecks the selected schedule.

~~~text
policy source
  -> strict parse / schema validation
  -> typed policy IR
  -> registry and model-scope resolution
  -> deterministic policy digest
  -> select from a complete oracle Pareto frontier
  -> independent schedule verification
  -> decision / regret report
~~~

A policy that requests an unsupported objective or constraint must be Blocked or NotComparable. It must not be approximated by silently dropping the field or assigning a zero value.

## 2. Ownership

| Concern | Existing owner |
|---|---|
| Study/case identity, datasets, model/policy revision and config digests | EnergyStudyManifestV1 (#82) |
| Finite dispatch feasibility, complete Pareto frontier and search caps | Exact dispatch oracle (#309) |
| Transparent baseline and objective-vector regret | Dispatch regret benchmark (#311) |
| Mapping candidate subgraphs into the finite model | Energy design adapter (#314) |
| Policy authoring, validation, objective order and deterministic selection | This DSL |
| Recalculation of schedule feasibility and balances | Existing independent schedule verifier |

This DSL extends the existing EnergyPolicyRefV1 identity. It must not create a second study manifest, physical model, solver, evidence format or schedule verifier.

## 3. Why a policy DSL is valuable

The exact oracle already exposes a Pareto set over separate quantities. The baseline PR introduces a transparent fixed heuristic. What remains unclear and hard to review is how a decision-maker chooses one alternative from a complete frontier, and whether a policy change has altered critical-service priority.

The policy DSL makes explicit which constraints define feasibility, which objectives are optimized and in what order, how each objective is aggregated, what happens when the data required by an objective are absent, how ties are resolved deterministically, and exactly which policy artifact selected a schedule.

No single weighted sum should hide an essential-service failure behind a cost or carbon improvement.

## 4. v0.1 grammar and semantics

Use a structured, versioned data document (JSON for exchange; TOML may be added as a reviewed authoring frontend). Do not add an arbitrary expression language.

The normalized IR consists of:
- policy ID, schema version, revision and stable policy kind;
- declared oracle/model schema scope;
- hard-constraint registry IDs supported by that exact model revision;
- ordered objective tiers, each with a registered objective ID, direction and named aggregation;
- missing-input behavior for every objective;
- deterministic tie-break sequence, limited to declared stable fields;
- source/semantic identity, limitations and claim ceiling.

### Hard constraints are not objectives

Hard feasibility checks are owned by the oracle. In v0.1, the policy can require the existing declared hard constraints, but it cannot invent a new physical constraint and pretend the oracle enforced it.

If a future policy asks for a minimum essential-service floor or maximum longest-deficit duration that the oracle does not yet represent, compilation returns Unsupported until the model and independent verifier both implement that rule.

### Objective priority is explicit

The current bounded model reports:
- total essential energy unserved;
- total flexible energy unserved;
- operating cost units;
- grid carbon grams;
- curtailed PV energy;
- terminal battery SOC.

The first five are minimized and terminal SOC is maximized by the current reference ordering. A policy document may select and order only objective IDs supported by the exact model revision.

Do not infer that a metric is available merely because a field exists in a report. Its required input factors must be present, source-qualified and included in the case identity.

### Aggregation is named, not free-form

For v0.1, support only aggregation operations already available and exactly recomputable from the current oracle result. Initially this is the oracle's current horizon-total objective vector and terminal SOC. Per-interval maxima, longest deficit, CVaR, stochastic expectations and worst-case scenario aggregation remain Unsupported until independently implemented and validated.

### Missing information

If an objective needs tariff or carbon factors that are missing or not comparable, the policy must emit NotComparable or Blocked for that objective. It must not impute zero, inherit a previous tariff, or quietly remove the objective from the ranking.

### Tie-breaking

If the declared objective vector ties, use a fixed, versioned tie-break sequence over stable schedule fields. The tie-break must not change feasibility or convert a physically meaningful objective into a hidden preference. The tie-break profile is part of policy identity.

## 5. Example normalized policy document

Illustrative only; exact objective IDs and registry versions must be frozen against the selected oracle revision before implementation.

~~~json
{
  "schema_version": 1,
  "policy_id": "critical-service-first-v1",
  "model_scope": {
    "request_schema": "energy-dispatch-v0.1",
    "objective_registry": "energy-dispatch-objectives-v1"
  },
  "hard_constraints": {
    "require_all_oracle_feasibility_checks": true
  },
  "priority": [
    { "objective": "essential_unserved_quanta", "direction": "minimize", "aggregation": "horizon_total" },
    { "objective": "flexible_unserved_quanta", "direction": "minimize", "aggregation": "horizon_total" },
    { "objective": "operating_cost_units", "direction": "minimize", "aggregation": "horizon_total" },
    { "objective": "grid_carbon_grams", "direction": "minimize", "aggregation": "horizon_total" },
    { "objective": "curtailed_pv_quanta", "direction": "minimize", "aggregation": "horizon_total" },
    { "objective": "terminal_soc_quanta", "direction": "maximize", "aggregation": "terminal_value" }
  ],
  "missing_data": "not_comparable",
  "tie_break": "stable_schedule_v1",
  "claim_ceiling": "selects among schedules in the exact declared finite model only"
}
~~~

The policy selects among an already-computed complete frontier. It does not change the frontier's feasible region or claim that the declared finite model represents the whole physical site.

## 6. Identity and change analysis

Keep separate commitments for:
1. exact authoring source bytes;
2. normalized semantic policy IR;
3. resolved objective and constraint registry versions;
4. exact model/oracle revision and case identity.

Harmless formatting may change the source digest but must not change the semantic policy digest. Changing objective order, aggregation, missing-data behavior, tie-break, model scope or registry version must change semantic identity.

A policy-change report should list prior and candidate policy digests, exact case and Pareto-frontier digests, selected alternative before/after, objective-vector differences, changed objective priority and tie-break behavior, newly NotComparable/Blocked requests, and whether the change is a pure ranking change or changes eligibility.

Policy equivalence is only asserted over the declared finite case/frontier universe. General behavior outside that universe remains Unknown.

## 7. First implementation fixture

Represent the existing pv-grid-battery-priority-v0.1 baseline as a frozen source policy. Do not alter its decisions during the first adapter.

Run the same policy against:
- the frozen PV-surplus-then-outage known-answer case;
- the one-interval grid-first case where the heuristic incurs avoidable cost;
- the terminal-reserve conflict case;
- outage, partial-PV, zero/full-SOC, lossy-storage and impossible-reserve cases;
- the full generated regret corpus as it becomes available.

Every policy/case pair remains in the denominator, including policy infeasibility, unsupported objective and NotComparable outcomes.

## 8. Conformance corpus

Positive:
- exact representation of the current baseline policy;
- service-first then cost-first policy selecting the same alternative as a manually declared known answer;
- carbon objective present only when every interval has a comparable carbon factor;
- deterministic tie among equal objective vectors.

Negative:
- unknown objective/constraint ID;
- objective not supported by the named oracle revision;
- invalid direction or aggregation;
- missing tariff/carbon factor used by a requested objective;
- duplicate objective IDs or tie-break fields;
- hard constraint downgraded to a soft penalty;
- policy attempts to trade essential shortfall against cost using weights;
- unknown field or policy schema version;
- incomplete Pareto frontier, candidate cap reached or truncated result;
- policy/model/case digest mismatch.

Metamorphic:
- source formatting change preserves semantic digest;
- objective-order change changes digest and possibly selected alternative;
- missing required factor produces NotComparable/Blocked, never a free objective;
- incomplete frontier cannot produce a result marked policy-optimal;
- every selected alternative passes the independent schedule verifier;
- changing case/model/oracle/registry identity changes the final selection receipt identity.

## 9. Benchmarks and release targets

### Correctness

- 100% agreement with hand-calculated known-answer selections.
- Zero selected schedules with independent hard-constraint violations.
- Zero missing/unqualified objective factors treated as zero.
- 100% rejection of unknown objectives, unsupported aggregations and incomplete frontiers.
- 100% reproducibility for identical policy, case, frontier and registry inputs.
- 1,000 deterministic policy mutations before v0.2; zero surviving known mutations that alter hard-constraint or essential-service priority semantics.

### Decision quality

On matched small instances report, separately:
- essential energy unserved;
- longest essential deficit and outage-only deficit when supported by the evaluator;
- flexible energy unserved;
- cost and carbon only when source-qualified;
- curtailment and terminal SOC;
- feasibility failure rate;
- vector regret against the exact frontier.

Do not collapse these into one weighted score. Compare the Pareto alternatives and chosen policies on common cases and held-out seeds.

### Performance

Benchmark policy parse, validation, registry resolution, ranking over an already-built frontier and receipt construction separately from exact-oracle enumeration. Report cold/warm p50, p95, p99, throughput, allocations/RSS, frontier size and number of alternatives. Include 1, 16 and 64 concurrent policy evaluations.

## 10. Industry comparison

The external tools have broader model scopes:
- NREL REopt provides techno-economic sizing/dispatch and resilience analysis for buildings, campuses, communities and microgrids.
- PyPSA supports large chronological multi-energy systems, storage, sector coupling and linearized network power flow.
- GenX jointly models capacity expansion, hourly dispatch, unit commitment, reserves, transmission and policy constraints.

The Sol-Atlas exact oracle is not meant to replace those systems. It is a calibration and policy-selection laboratory for small finite instances. Compare only when the same physical/model boundary, timestep, energy units, storage efficiencies, terminal state, constraints and objective hierarchy are representable. Publish mismatches and Unsupported results instead of bending either model to manufacture parity.

## 11. Non-goals

- No dispatch execution or equipment actuation.
- No guarantee of site safety or physical model completeness.
- No AC power flow, feeder/inverter protection, thermal/process flows, multi-site networks, capacity investment, stochastic scenario trees or degradation model in v0.1.
- No replacement of the existing EnergyStudyManifestV1, exact oracle, verifier or external optimizers.

**Exit criterion:** a reviewer can inspect a policy file and know exactly which finite feasible schedules it prefers, why it selected one, what information it relied on, what it cannot evaluate, and which exact policy/case/model artifacts produced that selection.
