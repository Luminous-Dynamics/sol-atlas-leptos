# Highveld Neighborhood Commons — topology variants and matched comparison v0.1

**Parent architecture:** \`NEIGHBORHOOD_RESILIENCE_COMMONS\`  
**Maturity:** \`PROPOSED_NOT_EVALUATED\`  
**Purpose:** define distinct, falsifiable neighborhood network topologies without creating five disconnected candidate programmes or changing PR #736's frozen v0.1 matrix.

## 1. Why topology is a separate design variable

A neighborhood commons is a network of household guarantees, service providers, access paths, resource accounts, repair capacity and resident-governed allocation. The location and coupling of shared capacity changes the failure surface even when total resource capacity is identical.

A central hub may be efficient in normal operation but creates a concentrated failure/access point. Distributed nodes may improve proximity and local islanding but increase maintenance burden and coordination complexity. A federated mesh may preserve local agency and partition tolerance but introduce coordination conflicts and incomplete knowledge. Mobile support can reach isolated households but is constrained by routes, travel time, weather, fuel/energy, staffing and replenishment. A layered hybrid may combine these capabilities but has more interfaces and common-cause paths to qualify.

The topology comparison must be able to conclude that a simpler household-only architecture is better for a declared service and stress envelope. The existence of shared infrastructure is not itself evidence of resilience.

## 2. Five topology families

| Variant ID | Physical/organizational pattern | Hypothesis worth testing | Primary failure exposure | Distinctive measurements |
|---|---|---|---|---|
| \`CENTRAL_HUB\` | One primary shared facility provides selected services; households retain local fallback | Concentrated maintenance, storage and treatment may reduce duplicated assets and simplify servicing | Hub outage, single access route, queue overload, shared controller/utility dependence | Hub common-cause cut sets, route accessibility by household, queue tail, islanding/black-start, hub repair/requalification time |
| \`DISTRIBUTED_NODES\` | Two or more local shared nodes provide overlapping or geographically partitioned services | Multiple nodes may reduce travel/queue burden and permit local islanding | correlated node failures, hidden shared feeder/control, unequal coverage, spares spread too thin | per-node service catchment, independent failure domains, node-removal service continuity, inter-node transfer loss, coverage gaps |
| \`FEDERATED_PEER_MESH\` | Independently governed household/community nodes exchange explicit service/resource offers under a shared protocol; no central node is authoritative for all allocation | Federation may preserve local decision autonomy and continue some functions through network partitions | stale reservations, double commitments, split-brain coordination, replay/revocation ambiguity, trust/identity outage | partition consistency, stale-state age, conflict/appeal resolution, duplicate-credit rejection, offline local service continuity |
| \`MOBILE_SUPPORT\` | Mobile water/energy/repair/communications/refuge assets supplement stationary household and shared infrastructure | Mobile capacity may reach isolated or temporarily displaced demand without permanently duplicating all assets | road closure, travel delay, unavailable operators, fuel/charging limits, weather, maintenance and resupply | request-to-arrival time, reachable households per route state, route failure, payload/service capacity, staff/spares availability, return/replenishment time |
| \`LAYERED_HYBRID\` | Household-local floor + distributed nodes + central strategic reserve + mobile repair/logistics; federated coordination only where qualified | Combining passive/local capability with redundant shared service layers may improve graceful degradation | interface complexity, correlated dependencies across layers, hidden resource double-count, governance fragmentation | all per-household metrics plus interface failure count, dependency cut sets, coordination overhead, maintenance labour and lifecycle burden |

These are five variants within one proposed system-boundary family. They are not five already-evaluated architectures. Keep \`NEIGHBORHOOD_RESILIENCE_COMMONS\` as one candidate identity until the parent architecture owner decides whether variant identities should become separate rows.

## 3. Non-negotiable invariants across all variants

1. Household-declared essential service floors remain locally supported and do not depend solely on a hub, network, vehicle, neighbouring household or allocation approval.
2. A variant may augment a household floor but may not weaken or replace it without explicit resident decision and proper owner review.
3. All provider/fallback dependencies are identified by versioned asset and failure-domain references; common-cause independence is not asserted from labels alone.
4. A service is delivered only by the service owner's qualified outcome/trajectory evidence, never by asset availability, installed capacity, location or resource stock alone.
5. Route/access, queue, operating hours and service quality must be within each affected household's declared requirement.
6. Resource ledgers and delivered-service ledgers remain separate; no unit conversion without an explicit conversion contract.
7. Consent, allocation authority, appeals and resource commitments remain human/resident governed under the appropriate Mycelix contracts.
8. Network partition and central coordination loss cannot revoke household-local manual fallback.
9. Repair capacity includes maintainers, access, spares, repair bound and requalification evidence.
10. No neighbourhood aggregate or weighted resilience score may conceal a household hard-floor breach.

## 4. Matched experiment design

### 4.1 Freeze a common reference system

Before variant comparison, freeze:
- the same synthetic household/task/service set and each household's floor;
- the same geometry and spatial/access requirements;
- the same exogenous hazard and disruption trace bytes/digest;
- the same initial resource state and resource/service units;
- the same operator, compute, information and repair/spares assumptions;
- the same common-cause dependency oracle and service-transition oracle;
- the same model versions, configuration digests and evidence rights;
- the variant's declared boundary and what is intentionally out of scope.

Run two complementary comparisons, not one:
- **Capacity-matched:** same total installed service/resource capacity and declared quality limits, to isolate the effect of topology.
- **Budget-matched:** same installed/lifecycle resource and maintenance budget, with explicit assumptions and uncertainty. Do not claim cost superiority from list prices alone.

A third operational comparison can be **maintenance-capacity-matched**, holding technician-hours, spare inventory and response coverage equal. If these cannot be held constant, report them as measured confounders rather than quietly attributing the result to topology.

### 4.2 Mandatory topology stresses

| Stress | Central hub | Distributed nodes | Federated mesh | Mobile support | Layered hybrid |
|---|---|---|---|---|---|
| Primary hub/node loss | Hub outage | single-node loss | peer loss | depot/vehicle loss | each layer removed separately |
| Correlated upstream power loss | hub + local reserve coupling | shared feeder common cause | peer islanding | charge/fuel dependence | cross-layer common cause |
| Geographic access cut | hub unreachable | local node catchment loss | local neighbour reach loss | road/route cut | both node and mobile access |
| Simultaneous demand peak | queue/overload | node hot spots | contention/reservation conflict | fleet scheduling | cross-layer allocation contention |
| Network partition | central control loss | inter-node coordination loss | split-brain/stale reservations | dispatch communication loss | federated coordination loss |
| Repair/spares shock | hub maintainer/spares loss | distributed technician dilution | peer repair commitment failure | mobile operator/payload loss | repair capacity across layers |
| Repeated fault before stable recovery | hub recurrence | node recurrence/cascade | replayed stale service state | asset returns unavailable | recurrence across layers |
| Consent/authority withdrawal | hub analytics/allocation | cross-node sharing | peer scope leak | mobile telemetry leak | federated data reuse |
| Reserve/resource exhaustion | central reserve depleted | stranded local stocks | double-committed stock | mobile stock/replenishment | inter-layer double count |
| Climate/road compound event | hub and route stress | multiple catchments fail | isolated peers | vehicle route blocked | several layers degrade together |

Symtropy must instantiate all time-varying events, service trajectories and stable-recovery windows using its native event identity, clock and sequence. This table defines challenges; it does not manufacture timestamps or trajectories.

## 5. Measures and acceptance gates

### Hard gates (synthetic finite oracle)

- Per-household essential-floor false passes: 0.
- Unqualified or inaccessible shared-service delivery claims: 0.
- Hub/node/vehicle availability falsely substituted for delivered household service: 0.
- Common-cause dependencies missed against the finite independent cut-set oracle: 0.
- Duplicate or double-committed resource credits: 0.
- Silent unit conversions or reserve-floor violations: 0.
- Future-information, revoked-consent or wrong-purpose access: 0.
- Critical unknown state treated as PASS: 0.
- Positive-control preservation: 100% for cases the independent oracle marks admissible.
- Exact finite known-answer agreement: 100%.

### Comparative vectors (no universal scalar)

Report for every household and service:
- delivered/requested quantity and unit;
- time to first floor breach, duration unserved, access delay and queue delay;
- locally supported essential demand without hub/network/vehicle dependence;
- share of demand missed, refused, inaccessible or unknown;
- stable recovery time and repeated-fault count;
- resource reserve and balance residual by account;
- repair/spares/maintainer coverage and technician-hours;
- route reachability, operating windows and accessible service options;
- common-cause cut sets and cross-layer dependency counts;
- consent scope, denied requests, appeals, reversals and unresolved disputes;
- installed/lifecycle cost, embodied/operational burden and service per unit cost only where evidence supports it.

First eliminate variants that breach a household floor. Only then compare the Pareto set on cost, resource use, repair burden, accessibility, privacy and autonomy. A superior neighbourhood mean cannot offset a household loss.

## 6. Research hypothesis and staged evaluation

**Hypothesis A:** distributed nodes may reduce access and single-hub exposure relative to \`CENTRAL_HUB\`, but may increase technician-hours, spares fragmentation and hidden shared-feeder dependencies.

**Hypothesis B:** \`FEDERATED_PEER_MESH\` may improve partition tolerance and resident autonomy, but only if resource reservation is idempotent, conflicts are explicit and local decisions remain safe under stale information.

**Hypothesis C:** \`MOBILE_SUPPORT\` may reduce service interruption for some remote or isolated households, but route, staffing and replenishment may dominate nominal payload capacity.

**Hypothesis D:** \`LAYERED_HYBRID\` may provide the broadest graceful degradation, but could be inferior after accounting for maintenance, interface failures, lifecycle cost and coordination burden.

Staging:
1. Keep the current central-hub synthetic reference as the contract smoke test.
2. Add a distributed-node counterfactual with explicit independent and shared feeder failure domains.
3. Add federation only after Mycelix's offline coordination, resource commitment and trust/appeal contracts are accepted.
4. Add mobile support only after route/time/capacity and maintenance contracts can be evaluated by an appropriate spatial/network owner.
5. Compose a layered hybrid only after each component is independently qualified and cross-layer common-cause analysis is available.

The first result may well be "do not build a commons for this service; strengthen household-local passive capacity instead." That is a valid research result.

## 7. Status / claim ceiling

This is a topology experiment plan, not an evaluated result. No variant currently has measured access, queue, service, repair, cost, equity, safety, climate performance or resilience evidence. The current synthetic Rust qualifier checks only the manifest semantics it implements; it does not simulate these topologies or perform a common-cause graph analysis.


## 8. Long-horizon variant check

The topology comparison should eventually be repeated across a non-stationary scenario ensemble, not just one event trace. Compare whether central, distributed, federated, mobile and layered topologies preserve local essential-service floors as climate stress, component aging, maintenance backlog, spare-part availability and household requirements evolve. Where occurrence probabilities are not supportable, report robust feasibility, worst-case/tail deficits and hindsight regret against a clearly marked oracle; do not fabricate probability weights.

This is a future benchmark horizon, not a new claim about current variants. Keep the event-trace benchmark and long-horizon policy benchmark versioned separately.
