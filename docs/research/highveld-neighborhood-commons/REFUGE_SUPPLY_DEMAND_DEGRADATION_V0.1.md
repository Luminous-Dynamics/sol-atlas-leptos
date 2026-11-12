# Highveld refuge service supply-demand degradation benchmark v0.1

**Study ID:** HIGHVELD-REFUGE-SUPPLY-DEMAND-DEGRADATION-001  
**Parent:** HIGHVELD-REFUGE-NETWORK-FEASIBILITY-001  
**Maturity:** TEMPORAL_SYNTHETIC_BENCHMARK_NOT_FIELD_VALIDATED

## 1. Paradigm shift: static coverage → state/process resilience

A map of cooling sites or a static accessibility score says little about whether the network continues to provide service through a heatwave. Demand can surge while site capacity declines because of power, ventilation, water, staffing, route, queue or maintenance failures. A site can also reopen only after requalification, while demand remains unmet.

The benchmark evaluates the refuge network as an ordered service trajectory. It reports both service state (feasible and unmet demand at each timepoint) and process (unmet-place-hours, accessible unmet-place-hours, worst-timepoint gap and cohort service degradation).

A 2026 state/process study of urban heat resilience explicitly examines cooling-service supply-demand mismatch under heatwave perturbations and the degradation of fixed-location service supply. A separate 2026 urban-heat systematic review identifies operational twins linked to real-time sensing, outage risk, transit conditions and cooling-site occupancy as a research direction. The Highveld extension tests this dynamic layer while keeping real clock/event identity with Symtropy.

References:
- [Urban heat resilience from a state–process perspective: Heatwave-induced cooling service supply–demand mismatch and hierarchical spatial allocation of greening, 15 September 2026](https://www.sciencedirect.com/science/article/pii/S2210670726004762)
- [Urban heat, vulnerability, and mitigation in U.S. cities: A systematic review, 2026](https://doi.org/10.1016/j.scsadv.2026.100077)
- [Where to cool off: a geospatial framework for placement of cooling centers, May 2025](https://www.ornl.gov/publication/where-cool-geospatial-framework-placement-cooling-centers)
- [A two-dimensional framework for building-level heat risk assessment and equity, September 2026](https://www.sciencedirect.com/science/article/pii/S2212420926003535)

## 2. Time and event ownership

The time-series input is an ordered sequence of timepoint IDs and durations. It does not invent timestamps, native event IDs, clock ticks or causal event order. At physical execution:
- Symtropy supplies native event identity, clock/sequence, duration, outage/weather traces and site-service chronology.
- Sol-Atlas supplies route/accessibility evidence for the relevant timepoint.
- Mycelix supplies rights, capacity commitments, water and instrument admission.
- The oracle evaluates a snapshot per timepoint and aggregates service-gap metrics.

The committed example uses symbolic synthetic timepoint identifiers only.

## 3. State/process metrics

For each timepoint report:
- requested, feasible and unmet total places;
- accessible and general feasible/unmet demand;
- per-cohort aggregate service ratio;
- qualified site and route counts;
- scenario and duration.

Across the sequence report:
- maximum unmet demand at any timepoint;
- minimum cohort service ratio at any timepoint;
- total unmet-place-hours;
- accessible unmet-place-hours;
- aggregate unmet snapshot count (labelled as a sum of snapshots, not unique people).

No individual identity or movement history is present. The aggregate flow output does not assign groups to named sites or book places.

## 4. Synthetic known-answer sequence

- **TP-001 / hot-event onset:** baseline refuge supply; 20 anonymous places requested, six accessible.
- **TP-002 / compound failure:** one refuge loses qualified thermal service and another's water falls to its committed/protected boundary; the network is partitioned but local directory is known.
- **TP-003 / recovery:** synthetic baseline state restored after hypothetical requalification.

The case is intended to prove that the same nominal network may move from partial service to near-zero service and back. It does not claim these failures occurred at any real site.

## 5. Acceptance rules

- Timepoint IDs must be unique, ordered and accompanied by positive durations.
- Missing/unknown site/route/service/water states are excluded, never imputed as available.
- Accessible demand uses qualified accessible routes and capacity.
- Water-limited effective capacity is computed after commitments and protected reserve.
- Per-timepoint capacities are not summed across time as if they were simultaneous.
- Unmet-place-hours are computed from the supplied timepoint duration; the oracle never creates native ticks.
- Preserve valid local availability during network partition where the local directory and all required evidence remain qualified.
- Report minimum cohort service ratio and accessible unmet-place-hours alongside total served demand.

**Status:** TEMPORAL_SYNTHETIC_BENCHMARK_NOT_FIELD_VALIDATED. No real route, refuge, time series, resident, weather or outage data is admitted.