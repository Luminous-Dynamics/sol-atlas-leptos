# Highveld thermal service continuity and neighborhood refuge benchmark v0.1

**Study ID:** HIGHVELD-THERMAL-SERVICE-REFUGE-001  
**Parent:** HIGHVELD-PASSIVE-COOLING-WATER-001 / HIGHVELD-THERMAL-SINK-CASCADE-001  
**Maturity:** BENCHMARK_PROTOCOL_NOT_EXECUTED  
**Climate context:** Johannesburg / Highveld; no site or occupant data admitted

## 1. Paradigm shift: benchmark thermal service, not equipment

The field often compares cooling devices, peak capacity, material surface temperatures or annual energy. Residents experience something different: whether the rooms and refuges available to them remain tolerable across a hot day, a hot night, a power outage, a water interruption and repeated events.

The benchmark therefore treats thermal comfort and refuge access as an essential service with two independent layers:

A. **Building thermal service:** can the dwelling maintain its declared occupied-zone thermal service envelope under matched weather, occupancy, grid and water conditions?

B. **Neighborhood refuge continuity:** when the dwelling cannot maintain that service, is a qualified, capacity-available, accessible refuge actually reachable and available through the same compound disruption?

Do not collapse these layers into a single average indoor temperature or a map of cooling assets. A refuge that has no backup power, no safe water/ventilation, no capacity, an inaccessible route or is closed during the event is not available service.

## 2. Seven building-intervention arms

| Arm | Intervention | Role |
|---|---|---|
| PASSIVE_ENVELOPE_SHADE_BASELINE | Competent envelope, external shading and passive load reduction; no added water | Primary control |
| PASSIVE_ENVELOPE_NIGHT_PURGE | Same envelope plus controlled night ventilation when outdoor enthalpy/air quality/security permit | Test |
| MECHANICAL_COOLING_REFERENCE | Properly sized conventional mechanical cooling | Industry reference |
| PCM_STORE_NIGHT_REGENERATION | PCM store with metered charge/discharge and qualified night regeneration | Test |
| RADIATIVE_DRY_SINK_STORE | Radiative panel and/or dry-air heat rejection coupled to a finite store | Test |
| CONDITIONAL_EVAPORATIVE_ASSIST | Water-coupled evaporative assist, only when separate rights/quality/reserve governor permits | Conditional test |
| LAYERED_MULTI_SINK_PORTFOLIO | Passive load reduction plus qualified store and multiple bounded sink paths | Portfolio test |

Each arm uses the same geometry, orientation, envelope boundary, occupied schedules, internal gains, weather, comfort/authority limits, outage trace, electrical boundary and evaluation horizon. Differences in capital budget, floor area or equipment capacity are separate matched sensitivity arms, never hidden normalizations.

The passive baseline must be competent. A weak baseline cannot be used to claim a technology victory.

## 3. Eight compound operating scenarios

1. HOT_TYPICAL_DAY — representative hot day and ordinary grid/water availability.
2. MULTIDAY_HEAT_WAVE_HOT_NIGHTS — high daytime load and weak night recovery.
3. GRID_OUTAGE_24H — cooling electricity interrupted for 24 hours.
4. GRID_OUTAGE_72H_DURING_HEAT — prolonged grid outage during a multi-day heat event.
5. WATER_SUPPLY_INTERRUPTION_OR_CURTAILMENT — water availability and discretionary use restricted.
6. COMPOUND_GRID_WATER_OUTAGE — simultaneous grid and water interruption.
7. HUMID_HOT_WEAK_EVAPORATIVE_POTENTIAL — high humidity limits evaporative strategies.
8. COLD_CLEAR_NIGHT_OVERCOOLING_AND_RECOVERY — prevent overcooling and test seasonal transition/recovery.

Building-service matrix: 7 arms × 8 scenarios = 56 registered comparisons. No event probabilities or outage times are invented here; actual traces must be owner-admitted, measured or explicitly synthetic.

## 4. Four refuge-network architectures

- HOME_ONLY — no shared refuge dependency; every dwelling is evaluated on its own declared service.
- CENTRAL_REFUGE_HUB — one shared refuge with declared capacity, opening hours, power, water, ventilation, staffing and route dependencies.
- DISTRIBUTED_REFUGE_PODS — several small local refuges with independently declared capacity and failure domains.
- FEDERATED_REFUGE_MESH — independently operated, mutually discoverable refuges with local authority and partition-tolerant coordination.

Six stress scenarios are crossed with these topologies:
- REFUGE_SITE_UNAVAILABLE
- GRID_AND_NETWORK_PARTITION
- CAPACITY_SURGE_AND_QUEUE
- ACCESSIBLE_ROUTE_DISRUPTION
- WATER_RIGHTS_OR_QUALITY_WITHDRAWAL
- STAFFING_OR_MAINTENANCE_OUTAGE

Refuge-network matrix: 4 topologies × 6 scenarios = 24 registered comparisons. Combined benchmark inventory = 80 cases, not executed.

## 5. Comfort, safety and access: do not conflate them

### Comfort compliance

Where the occupancy and building context fall within its applicability, use the current licensed version of ANSI/ASHRAE Standard 55 with the appropriate standard or adaptive method. Pin the edition, relevant addenda, calculation method and exact input fields. The current public addenda index lists addenda a and b to Standard 55-2023 dated 31 August 2026; recheck the official index before freezing an execution package.

Standard 55 addresses combinations of air temperature, thermal radiation, humidity and air speed with activity and clothing. Its stated scope is healthy adults in indoor occupied spaces; it is not by itself a public-health heat-safety limit, an emergency shelter standard or a guarantee for every person.

Record as applicable:
- operative temperature / mean radiant temperature;
- air temperature and vertical head-to-ankle gradient;
- humidity/dew point and air speed;
- activity/clothing assumptions and occupant control mode;
- occupied hours outside the selected comfort method's zone;
- duration of continuous discomfort and night recovery.

### Safety envelope

Any heat-health or emergency operating envelope must be set by a competent local public-health/engineering authority for the site and intended population. The benchmark accepts it as an externally governed input with source, applicability, version and owner. Do not invent universal safe temperatures, infer vulnerability from identity, or substitute ASHRAE comfort for a health assessment.

### Refuge access

Use actual accessible walking/network routes, travel impedance, refuge opening/availability, capacity, queue and outage dependencies. Straight-line distance is not a valid access substitute. Test accessibility using privacy-preserving needs categories and route constraints; do not retain identifiable movement histories or infer individual health status.

For each dwelling and scenario, record:
- eligible refuge(s) and route/network digest;
- reachable capacity at the event time;
- travel time under explicitly declared mode and accessibility class;
- opening/operating status, backup power, water, ventilation and staffing status;
- queue/overflow and fallback assignment;
- service gap when no qualified refuge is reachable.

A refuge is counted as available only if all of its mandatory operational dependencies are qualified and capacity remains. If any critical state is unknown, that refuge is unavailable for the relevant claim.

## 6. Metrics that expose false resilience

Report distributions and per-dwelling outcomes, not only neighborhood means:
- occupied hours within selected comfort envelope;
- discomfort degree-hours and maximum continuous exceedance;
- night recovery time and retained indoor heat;
- occupied-zone operative temperature / humidity / vertical gradient;
- service hours maintained during grid/water outage;
- refuge reachability, accessible travel time, usable capacity, queue and service gap;
- unmet refuge demand and dwellings with no reachable alternative;
- cooling electricity, peak demand and water use by source class;
- lifecycle and maintenance burden;
- worst-decile and worst-dwelling outcomes alongside P50/P90/P95.

A fair strategy win is a matched Pareto improvement in comfort-service continuity, water, energy, peak, cost and access—or a demonstrated reduction in worst-case service gaps—without violating any protected water, consent, safety or access floor.

Do not let a large number of well-served dwellings hide one dwelling with no safe refuge. Do not weight away a hard-floor failure.

## 7. Benchmark targets

- 0 false available-refuge claims when a critical dependency is absent or unknown.
- 0 route assignments that exceed qualified refuge capacity.
- 0 essential-water reserve violations caused by discretionary cooling/refuge operation.
- 0 building comfort claims made without the correct method/applicability and occupied-zone measurements.
- 0 ASHRAE comfort claims treated as a heat-health safety guarantee.
- 0 identity or sensitive health fields required by the benchmark.
- 100% of case results tied to weather, geometry, occupancy, controller, water/energy and refuge-topology digests.
- 100% of seeded capacity, route, opening-hour, grid, water and network negative controls detected.
- Preserve positive controls: reachable, open, accessible refuge with capacity and qualified services must not be falsely blocked.
- Report held-out uncertainty/calibration and out-of-domain performance; no arbitrary universal R² or single temperature threshold.

These are proposed benchmark acceptance targets, not achieved results.

## 8. Industry and SOTA comparison

- **ASHRAE Standard 55-2023:** use as a thermal-comfort evaluation reference where applicable; its current public page describes standard/adaptive methods and vertical-gradient discomfort treatment. Pin addenda a/b dated 31 August 2026 at execution.
- **EnergyPlus 26.2:** building heat/moisture and HVAC/thermal-storage specialist model; no neighborhood route or refuge-service authority.
- **BOPTEST 0.9:** controller comparison and reproducible building emulator pattern; extend with a custom Highveld case rather than claim its ready cases already represent water/right/route-aware refuge resilience.
- **Urban heat literature:** a 2026 systematic review reports empirical evaluation gaps and inconsistent integration of cooling access, energy burden, housing condition and intervention-scale spatial evidence. That makes the service/access layer an explicit research contribution rather than an optional dashboard.
- **Highveld:** compete on a cross-scale, evidence-carrying benchmark that couples specialist physics to occupied-zone service, water entitlements, local safe fallback, refuge capacity and accessible routing under common-cause outages.

Do not claim superiority over an incumbent by comparing unmatched buildings, comfort methods, weather or population assumptions. The claim requires a preregistered holdout, independent evidence, transparent distributional outcomes and exact replay.

## 9. Ownership and claim ceiling

- Sol-Atlas: geometry, routes, accessible network, refuge-site/configuration revision and spatial provenance.
- Symtropy: native event identity/time, matched weather/outage traces, per-dwelling trajectories, service chronology and replay.
- Mycelix: water rights/quality/reserves, consent, refuge capacity commitments, instrument admission and auditable allocation.
- EnergyPlus / Modelica / BOPTEST / qualified airflow solvers: their respective building and plant physics/control domains.
- Public-health authority / qualified local professionals: site-specific safety envelope and emergency refuge criteria.
- Resident/owner authority: participation, consent, discretionary resource allocation and appeals.

**Status:** BENCHMARK_PROTOCOL_NOT_EXECUTED. No location, resident, route, refuge, comfort observation, health data, EnergyPlus/BOPTEST run or service-superiority result is admitted by this protocol.
