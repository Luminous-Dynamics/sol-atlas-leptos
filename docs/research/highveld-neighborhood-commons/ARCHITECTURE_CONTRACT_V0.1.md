# Highveld — Neighborhood Resilience Commons architecture contract v0.1

**Architecture ID:** \`NEIGHBORHOOD_RESILIENCE_COMMONS\`  
**Maturity:** \`PROPOSED_NOT_EVALUATED\`  
**Evidence class:** \`SyntheticControlledDesign\`  
**Parent:** Sol-Atlas #694; proposal #737  
**Owner boundary:** Sol-Atlas defines the candidate and its spatial/service boundary. Symtropy owns native service trajectories, event timing, stable recovery and adaptive measurement. Mycelix owns allocation governance/commitments and instrument/source authority. Specialist building and utility solvers retain their own physics domains.

## 1. Research hypothesis

A neighborhood-scale commons may improve resilience through optional pooling of non-exclusive services, repair capability and logistics. It may also create new failure modes: access barriers, queues, common-cause failures, resource contention, inequitable allocation, privacy loss and dependency on central coordination.

The candidate is **not** a larger battery or a shared utility plant. It changes the system boundary from one dwelling to a governed service network of households, shared assets, local fallback resources and optional mobile/distributed support.

The benchmark must be capable of falsifying the hypothesis. It must be possible for the commons to perform worse than household-only designs because of access, capacity, common-cause, repair or governance constraints.

## 2. Three-layer architecture

### Layer H — household guarantee

Every household retains its declared essential-service floors and qualified household-local fallback. These floors must remain protected during hub loss. The local fallback must not depend on reaching, communicating with, or obtaining authorization from the commons.

A household-local fallback is a service capability, not automatically a particular appliance. Its engineering basis, capacity, autonomy, boundary conditions and qualification evidence must be declared by the relevant owner. A rendering, inventory entry or nominal storage capacity is not proof of delivered service.

### Layer C — commons service contracts

Each shared service is exposed as a versioned contract with:
- declared service, unit, capacity and demand window;
- provider asset and failure domain;
- access mode, accessible route/delivery mechanism and qualification evidence;
- queue/wait policy and unmet-demand disposition;
- source/rights/revision and owner references in the implementation packet.

A hub being online is not equivalent to a resident receiving a service. Record requested, admitted, delivered, refused, delayed, inaccessible and unknown states separately. A service must not be marked delivered from capacity, telemetry or proximity alone.

### Layer G — resident-governed allocation and repair

Any resource allocation policy requires explicit resident oversight, a recorded approval state, a human final authority and an appeal/reversal path. An optimizer may produce transparent options; it may not determine moral priority or become the final authority over household essential services.

Maintenance and repair capacity are part of the service contract: operator, spares, access, tooling, expected repair window, fallback and re-verification. Unknown repair capacity is not infinite repair capacity.

## 3. System boundary and interfaces

The reference manifest uses synthetic IDs only. A production adapter must separately identify:
- household nodes and resident-defined service floors;
- shared hubs, distributed nodes, mobile assets and external utility interfaces;
- explicit failure domains and common-cause dependencies;
- service contracts, access paths and queues;
- resource accounts, external inflows/losses and one-counted transfers;
- allocation policies, approval records and appeals;
- evidence IDs, scope, rights, revision, availability time and claim ceiling.

Do not collapse upstream infrastructure state into service outcome. A grid interruption is an infrastructure event. It becomes a household service failure only if the service-specific state/trajectory establishes the transition. Likewise, grid restoration does not prove stable service recovery.

The resource ledger is separate from the service ledger. Reservoir volume is not tap delivery. Battery state of charge is not energy delivered to a household task. A transfer is recorded once by source-event identity, debited once at its origin and credited once at its destination. Unit conversion requires an explicit, versioned conversion contract; silent unit coercion is prohibited.

## 4. Non-negotiable invariants

1. **No hub-only essential service.** Every household essential floor has a qualified local fallback that survives hub loss.
2. **Failure-domain separation.** A fallback and its shared provider cannot silently share the same declared failure domain.
3. **Access is a service precondition.** A shared service is not allocatable while access/route/delivery is unqualified.
4. **Queueing is explicit.** Peak demand above capacity requires bounded queue semantics and an unmet-demand disposition. Queue wait must not exceed any affected household's declared floor.
5. **Human-governed allocation.** Allocation requires resident oversight, approval, appeal and human final authority.
6. **Resource conservation.** Account balances include opening quantity, external inflow, incoming/outgoing transfers, committed outflow and known losses. Required reserves cannot be spent by nominal optimization.
7. **No duplicate observations.** Source-event identity is unique across resource transfers; duplicate ingestion must not create additional usable resource.
8. **No implicit units.** Resource and unit must match both ledger accounts. Conversions are explicit and versioned.
9. **No aggregate compensation.** A neighborhood aggregate cannot hide one household's essential-floor failure.
10. **No data-rights shortcut.** Household telemetry, occupancy, needs and usage are not shared merely because the commons is shared.
11. **Local degraded mode.** Loss of internet, hub controller, communications or governance coordination does not revoke household local fallback.
12. **Claim ceiling travels.** The reference architecture remains synthetic and unevaluated until evidence, access, physics and governance gates are separately passed.

## 5. Required adversarial experiment families

| Family | Seeded condition | Required detection |
|---|---|---|
| Hub outage | shared asset unavailable | local household floors remain independently satisfiable or candidate is rejected |
| Shared common cause | hub and fallback share one failure domain | reject false independence |
| Access failure | route/delivery inaccessible or unqualified | no shared service delivery claim |
| Queue overload | demand exceeds capacity | bounded wait and unmet demand recorded; no hidden service floor breach |
| Allocation capture | automated final authority or no appeal | block allocation |
| Consent/rights withdrawal | required household evidence no longer usable | block affected claims; no authority inferred from stale evidence |
| Resource exhaustion | reserve below floor | block further commitment |
| Duplicate telemetry | same source event used twice | reject duplicated resource credit |
| Unit mismatch | L and m³, Wh and kWh, or other unmatched units | reject until explicit conversion contract |
| Repair failure | no access, spares or maintainer | repair/recovery remains unknown or blocked |
| Communications loss | hub coordination unavailable | local fallback remains operable |
| Simultaneous stress | hub and one or more dwellings stressed together | measure per-household floor, not only hub availability |
| Inaccessible refuge | facility exists but is unreachable by a declared resident requirement | count as inaccessible, not delivered |
| Allocation dispute | residents contest a priority decision | pause/escalate through governed process; do not automate final resolution |
| Repeated fault | apparent restoration then recurrence | stable recovery remains open until native recovery criterion passes |

These controls are not probabilities or representative household frequencies. They are deliberately designed failure challenges.

## 6. Benchmark protocol

Compare the commons candidate only after the three household-scale candidates are separately frozen. Use common exogenous traces and declared functional units, then extend the system boundary explicitly.

**Primary non-compensatory results**
- per-household, per-service essential-floor violations and hours preserved;
- fraction of essential demand served without hub access;
- inaccessible/late/refused service by household;
- local fallback independence under declared common-cause scenarios;
- reserve balance and resource conservation;
- allocation disputes, appeals, reversals and unresolved cases.

**Secondary vector**
- hub reachability, travel/access time, queue delay and service radius;
- service continuity and stable recovery, using Symtropy's native evaluator;
- energy/water per delivered service unit and resource losses;
- maintenance, spares, repair and re-verification time;
- shared asset utilization, lifecycle burden and cost where source data supports it;
- privacy exposure and data minimization burden.

Do not publish one resilience score. Report hard floors first, then the Pareto set among candidates that satisfy them. Separate hub availability from delivered household service.

## 7. Benchmark ambition and industry comparison

The competitive position is compositional:
- use EnergyPlus/ASHRAE 140 for qualified thermal/building-model capability;
- use BOPTEST for eligible standardized control/emulator comparisons;
- use specialist microgrid/network solvers for electrical dispatch and protection;
- use qualified hydraulic/network tools for water service and pressure;
- use GIS/network analysis for reachability and access;
- use Symtropy for matched event/service trajectories, recovery, Pareto and adaptive measurement;
- use Mycelix for evidence/rights/commitment and governed allocation boundaries.

Highveld's proposed contribution is the **cross-domain assurance contract**: all the specialist outputs must remain tied to the same candidate revision, residents' declared floors, trace, source rights, capacity/resource accounting, access conditions and independent decision record. This is a hypothesis to test, not an established SOTA victory.

## 8. Admission and promotion gates

Promotion from proposal to evaluated candidate requires:
1. accepted candidate boundary in Sol-Atlas #694;
2. accepted household/service requirement profiles from the proper owner;
3. a finite synthetic service/resource oracle with independent review;
4. common-cause and access/queue controls with exact known answers;
5. resource-ledger balance and idempotency tests;
6. Symtropy native adapter and exact event/service identity;
7. Mycelix allocation governance/consent review;
8. primary source and instrument admission for any empirical claims;
9. model-specific VVUQ and held-out references before physical claims;
10. reviewed safe physical protocol before occupied or unoccupied test claims.

The supplied reference manifest and Rust qualifier test only structural/semantic invariants over a synthetic design. They do not evaluate a real neighborhood, household, microgrid, water network, building or allocation process.
