# Shared amenity spatial-provisioning evaluator v0.1

**Tracking:** [#952](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/952)  
**Parent protocol:** [#940](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/940) / [draft PR #941](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/941)  
**Maturity:** `SYNTHETIC_ORACLE_FIXTURES_ONLY`  
**Fixture set:** four contexts × five provision arms = 20 arm evaluations  
**Claim ceiling:** no live Johannesburg geography, residents, provider evidence, measured performance or allocation claim

## Why this package exists

PR #941 intentionally freezes 50 *registered design cases* without numerical inputs. This package is a separate, small numerical test corpus used to verify comparison semantics and arithmetic. It does **not** execute, replace, or change the state of those 50 cases. They remain `NOT_EXECUTED / NOT_SUPPLIED / NO_RESULT`.

The package compares the same five arms:

- `PRIVATE_DUPLICATION`
- `CENTRAL_SHARED`
- `DISTRIBUTED_SHARED`
- `HYBRID_SHARED_PRIVATE`
- `EXTERNAL_COMMERCIAL`

Across four deliberately synthetic contexts:

- `COMPACT_HIGH_DENSITY`
- `ACCESSIBLE_ROUTE_DISCONTINUITY`
- `EXTERNAL_PROVIDER_DISTANCE_OR_CLOSURE`
- `UNEQUAL_POPULATION_DISTRIBUTION`

The only measured quantities are fixture values authored for software tests. They are not local observations or estimates.

## Canonical owner boundaries

This evaluator composes owner outputs; it is not another GIS, capacity engine, operating ledger or allocator.

| Owner | Consumed responsibility | Not implemented here |
|---|---|---|
| Sol-Atlas [#623](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/623) | Common geography/population/service/time scope | A replacement scope/evidence registry |
| Sol-Atlas [#555](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/555) | Effective-access outcome and generalized access burden | Route finding, walking/cycling/transit graphs, isochrones or step-free routing |
| Mycelix [#6328](https://github.com/Luminous-Dynamics/mycelix/issues/6328) | Canonical amenity, asset, service and beneficiary semantics | A second amenity/service/ownership ledger |
| Symtropy [#3768](https://github.com/Luminous-Dynamics/symtropy/issues/3768) | Owner-qualified service capacity/resource feasibility | Physical capacity, queueing or building-physics simulation |
| Sol-Atlas [#816](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/816) | Whole-household affordability | Financial accounting or household affordability recalculation |
| Mycelix [#5901](https://github.com/Luminous-Dynamics/mycelix/issues/5901) | Matching and constrained allocation | Reservations, assignments, priority or allocation decisions |

The links above identify the intended owner contracts. The nine amenity functional-unit candidates in parent #940 remain `CANDIDATE_NOT_OWNER_APPROVED` pending Mycelix #6328 review. The synthetic fixtures do not contain actual owner receipts and do not establish that those upstream systems have emitted qualified data.

## Input and evaluation rules

Each scenario defines one common scope and resident-group population. Every arm repeats and must match the same:

- scope identifier and population identifier;
- service and exact functional unit (`COMMUNITY_KITCHEN` / `ACCESSIBLE_SHARED_KITCHEN_HOUR`);
- evaluation period and quality profile;
- resident-declared maximum generalized access burden.

The evaluator rejects any arm whose bindings differ. The symbolic `synthetic-scope:` and `synthetic-population:` identifiers are test identities, not cryptographic owner digests. The access record and the facility's period-specific service state are deliberately separate: a route can be qualified while the service is closed or unknown. An effective opportunity requires both a qualified route within tolerance and `service_state=AVAILABLE`; a route to a closed amenity is not counted as service access.

Access rows are already-qualified owner outputs. The evaluator does not calculate a route. A resident-group opportunity requires at least one facility row to be `QUALIFIED`, within the common resident tolerance, and compliant with any mandatory access condition, **and** that facility's period-specific `service_state` to be `AVAILABLE`.

### Metrics and denominators

All values remain a vector; no composite score or winning topology is emitted.

| Output | Definition |
|---|---|
| Effective-access coverage | Population with at least one effective access option ÷ total declared population; emitted in basis points |
| P50 / P90 burden | Weighted quantiles of generalized burden among the **effective-access population only** |
| Worst-decile mean | Population-weighted mean burden of the highest-burden 10% of the effective-access population (rounded down to whole seconds) |
| No-access population | Population for which all explicit access rows are known failures/blocks |
| Unknown-access population | Population for which no effective option exists and at least one required access input is unknown |
| Outside-tolerance population | Population with a known route but no route within the declared resident tolerance |
| Mandatory-access gate | `PASS`, `FAIL`, `UNKNOWN` or `NOT_APPLICABLE`; a mandatory failure is never averaged away |
| Aggregate capacity | Sum of each facility's owner-supplied deliverable capacity **once per arm**, independent of how many groups can reach it; a facility known unavailable in the period contributes zero, while unknown service/capacity state makes the aggregate unknown |
| Capacity gap | Aggregate demand minus aggregate capacity, reported as shortfall/surplus; unknown facility capacity makes the aggregate unknown |
| Minimum independent failure domains | Minimum count of distinct, explicitly declared failure domains reachable by any one resident group; not inferred from facility count |

The P50/P90/worst-decile values are conditional on effective access, so they must always be read alongside the coverage and no-access/unknown/service-unavailable/outside-tolerance populations. A low P90 among the few served residents is not a good outcome if the rest of the population cannot access the service.

Reachable capacity is not allocated supply. Group-level access records are not used to multiply facility capacity. This package does not infer a resident's booking, queue position, entitlement, or allocation.

### Fail-closed input controls

- duplicate JSON object keys are rejected before map deserialization;
- unknown JSON fields are rejected by typed Rust structures;
- route qualification and period-specific Mycelix service availability are separate; both must pass for an effective opportunity;
- unknown/invalid owner states are not promoted to effective access;
- a known-closed facility can be reachable but is not counted as an available service opportunity;
- unknown facility service availability or capacity makes the aggregate deliverable capacity unknown, not zero; a known-unavailable facility contributes zero;
- mandatory access `FAIL` or `UNKNOWN` remains visible;
- mixed scope/population/service/period/unit/quality bindings are rejected;
- duplicate facility IDs and duplicate group/facility rows are rejected;
- missing access rows are rejected instead of silently assuming access;
- negative counts/times cannot be deserialized into unsigned quantities;
- integer overflow in population, demand, capacity and weighted time calculations is rejected;
- output includes the exact fixture-byte SHA-256.

## Run

From repository root:

```bash
cargo test --locked --manifest-path tools/shared-amenity-spatial-eval/Cargo.toml
cargo clippy --locked --manifest-path tools/shared-amenity-spatial-eval/Cargo.toml --all-targets -- -D warnings
cargo run --locked --manifest-path tools/shared-amenity-spatial-eval/Cargo.toml
```

The CLI defaults to the frozen JSON fixture file. It exits non-zero if the input violates a contract or any computed output differs from the frozen golden result.

## Industry and research comparator protocol

The goal is **not** to replace specialist routing engines or claim a superior travel-time model.

| Reference | Strength to preserve | Luminous comparison layer |
|---|---|---|
| Conveyal R5 | Network-based isochrones, opportunity access and schedule-sensitive travel-time distributions/percentiles | Consume its route/time outputs; compare owner-qualified service usability, capacity, affordability, mandatory access and distributional failure states |
| OpenTripPlanner 2 | Multimodal trip planning and wheelchair-routing configuration | Consume network results; retain explicit unknown accessibility states and add resident/service/owner evidence without recoding the router |
| PEAI (2026) | Active-mobility accessibility beyond time-only thresholds, including service availability, infrastructure quality, housing affordability and user-centred insight | Reproduce its published methodology/data before any adaptation; preserve the dimensions separately rather than importing universal weights |
| UN SDG 11.2.1 / 11.7.1 | Comparable public-transport and public-space indicator baselines | Report the applicable indicator method with data vintage and limitations; do not equate an indicator with full amenity-service accessibility |

R5 publishes percentile-sensitive travel-time analysis and opportunity access. OTP documents both the limits of binary wheelchair-accessibility data and configurable treatment of unknown trip/stop accessibility. PEAI's 2026 Vancouver application reports that threshold-based time results can materially exceed its multidimensional result. These are reasons to compose and qualify specialized evidence, not claims that this Rust evaluator outperforms those products.

## Research targets — not achieved results

| Benchmark | Target |
|---|---:|
| Matched scope across compared arms | 100%; otherwise `NOT_COMPARABLE` |
| Golden-oracle agreement on frozen fixtures | 100% |
| Unknown/stale/wrong-scope input optimistically promoted | 0 |
| Duplicate facility-capacity accounting | 0 |
| Mandatory-access failure hidden by an aggregate | 0 |
| Registered 50-case benchmark state changed by this package | 0 changes |
| Real-world claims made from synthetic fixtures | 0 |

A future empirical comparison may test whether a distributed or hybrid topology improves P90 access burden by 20% versus a matched central-only baseline, **only where it is feasible and without degrading any mandatory access floor**. That remains a hypothesis, not a fixture pass criterion or a reason to prefer distributed provision.

## Current evidence ceiling

A passing exact-head CI run would establish only that this evaluator reproduces the frozen synthetic golden values and passes its adversarial tests. It would not establish live accessibility, safe routes, service opening, actual capacity, resident preference, affordability, legal authority, physical performance, or a preferred neighborhood design. No Johannesburg GIS, resident, facility, transit, price or operator data is present in this corpus.
