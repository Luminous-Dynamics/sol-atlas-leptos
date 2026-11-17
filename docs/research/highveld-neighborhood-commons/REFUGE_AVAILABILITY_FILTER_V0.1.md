# Thermal refuge availability filter v0.1

**Study ID:** HIGHVELD-REFUGE-AVAILABILITY-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001  
**Maturity:** SYNTHETIC_ELIGIBILITY_CONTRACT_NOT_FIELD_VALIDATED  
**Purpose:** prevent a refuge from being counted as available when its operating window, thermal/ventilation service, safe-water, route, accessibility or capacity evidence is missing or failed.

## 1. Boundary and authority

This is a deterministic eligibility filter. It consumes site operating/service evidence and route/capacity evidence from the owning systems. Sol-Atlas owns spatial/route calculation and provenance. Mycelix owns resource rights, consent and capacity commitments. Symtropy owns event time and service trajectories.

The tool does not compute routes, publish personal location data, allocate named people, direct evacuation, reserve capacity, operate refuges or actuate equipment. It returns sites eligible for human/operator capacity planning only. It is not an emergency dispatch system.

A network partition does not automatically make a refuge unavailable when a qualified local directory and independently available route/site evidence remain. Conversely, central connectivity does not make an unknown or unavailable refuge eligible.

## 2. Demand contract

The input represents an anonymous group-level demand envelope, not named residents:
- total places requested;
- subset of accessible places requested;
- safe water demand in litres;
- required operating duration;
- maximum permitted travel time under an externally declared accessibility/travel policy.

No name, identity, health status, device ID or movement history is accepted or required.

## 3. Refuge eligibility gates

A site is eligible only when all of the following are qualified:
- site is open for the complete required time window;
- thermal service is qualified under the scenario being evaluated;
- ventilation and safe-water service are qualified;
- safe-water rights and quality are admitted;
- staffing/maintenance state is qualified;
- route is reachable and accessible under the declared access profile;
- route travel time is within the declared policy bound;
- route/site/capacity/service/water evidence references are present;
- total and accessible capacity remaining after existing occupancy and commitments covers the anonymous demand;
- safe water remaining after existing commitments and protected reserve covers the demand.

Unknown or failed states block the site. A site with negative remaining capacity is not rounded into availability. A network partition may be tolerated only if local discovery state is qualified.

## 4. Capacity arithmetic

For a site:

\[
C_{free}=C_{total}-O_{current}-R_{committed}
\]

\[
C_{accessible,free}=C_{accessible}-O_{accessible}-R_{accessible,committed}
\]

\[
W_{free}=W_{safe,available}-W_{committed}-W_{protected,reserve}
\]

The site is eligible only when free total places, free accessible places and free safe water meet the declared group request. The filter does not reserve those quantities. A subsequent commitment/booking must be performed by the responsible authority with an auditable transaction and a re-read of current capacity.

## 5. Synthetic known-answer case

The committed test fixture contains:
- one local pod that remains eligible during a network partition because its local directory, route, site services and spare capacity are all qualified;
- one central hub with insufficient free capacity;
- one pod with a blocked route;
- one pod with unknown ventilation state;
- one pod with unknown drinking-water rights.

These are synthetic states and quantities. They are not mapped sites, measured capacity, public operating information or an actual refuge recommendation.

## 6. Acceptance targets

- zero false-available claims with unknown/failed critical dependencies;
- zero over-capacity or over-reserve eligibility;
- zero accessible demand satisfied only by non-accessible capacity;
- zero route claims from straight-line distance or absent route evidence;
- zero site counted open outside its declared operating window;
- preserve positive controls during network partition when qualified local state and route evidence remain available;
- 100% output traceability to demand, site, route, service, capacity and water evidence references;
- zero personal identity, health data or identifiable movement history required.

**Status:** SYNTHETIC_ELIGIBILITY_CONTRACT_NOT_FIELD_VALIDATED. No refuge site, route, capacity or service observation is admitted, and the filter does not assign or direct people.
