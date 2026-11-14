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

## 7. Cohort cumulative service-debt ledger

The snapshot and network-wide unmet-place-hour metrics are now reconciled against an independent cohort ledger. For each anonymous cohort and time slice, the evaluator records:
- requested, served and unmet total places;
- accessible and general requested/served/unmet places;
- duration-weighted requested, served and unmet place-hours;
- accessible and general unmet place-hours;
- minimum snapshot service ratio and cumulative service ratio across the ordered sequence.

The cumulative service ratio is:

\[
R_g^{cum}=\frac{\sum_t S_{g,t}\Delta t_t}{\sum_t D_{g,t}\Delta t_t}
\]

and cumulative service debt is:

\[
B_g=\sum_t(D_{g,t}-S_{g,t})\Delta t_t
\]

where \(D_{g,t}\) is requested places, \(S_{g,t}\) is feasible places and \(\Delta t_t\) is the supplied duration. B is measured in place-hours. It is not a count of distinct people and must not be presented as one.

The ledger has two conservation checks:
- sum of cohort unmet place-hours equals network-wide unmet place-hours;
- sum of cohort accessible unmet place-hours equals network-wide accessible unmet place-hours.

It reports the cohort with the largest cumulative absolute service debt. It also reports normalized cumulative service ratios so smaller and larger cohorts can be compared without relying on absolute place-hour totals alone.

This is an audit metric, not an optimizer instruction. It does not automatically direct future capacity or priority toward the cohort with the largest debt. Any debt-aware or rotating policy must be separately specified, owner-governed, consent-aware and compared against the static max-min and throughput policies.

## 8. Resilience-curve service recovery and continuity

Place-hours quantify cumulative shortfall, but they do not describe when the shortfall occurs or how long service takes to recover. The sequence oracle now emits an independent continuity curve for each anonymous cohort.

For each cohort, the first snapshot defines a pre-disruption baseline service ratio:

\[
R_g^{base}=\frac{S_{g,0}}{D_{g,0}}
\]

A cohort is considered degraded in a later interval when its feasible service ratio falls below that baseline. The time axis is relative elapsed duration formed only by summing the supplied interval durations; it does not create native event timestamps.

Reported metrics:
- baseline service ratio;
- first degradation start (elapsed hours);
- first return to baseline (elapsed hours);
- time from first degradation to first baseline recovery;
- number of degraded episodes and completed recovery episodes;
- total hours below baseline;
- longest continuous degraded episode;
- whether an episode remains unresolved at sequence end and its elapsed duration.

A return to baseline is not necessarily full service: when a baseline already has unmet demand, recovery means return to that pre-event level. The absolute unmet-place-hour and accessible unmet-place-hour ledgers remain separately reported, so a poor baseline cannot be mistaken for adequate service. This benchmark should compare both the resilience curve and its service floor.

The temporal study requires stable anonymous cohort identities and owner-governed tie-break ranks across snapshots. A changed cohort set or fairness rank fails the comparison rather than silently moving the baseline or changing priority mid-sequence. New demand groups must be represented by a separately versioned benchmark case with an explicit cohort model.

This follows resilience-curve literature that distinguishes performance magnitude, duration, integral impact, recovery rate and threshold adherence; no single scalar captures every resilience property. A review of 273 publications recommends explicit milestones and performance targets and cautions against condensing diverse trajectories into one expected curve. A 2026 urban heat resilience study likewise emphasizes state–process supply/demand mismatch under heatwave perturbations rather than relying solely on static capacity maps.

Sources:
- [Poulin & Kane, Infrastructure resilience curves: Performance measures and summary metrics (2021)](https://doi.org/10.1016/j.ress.2021.107926)
- [Urban heat resilience from a state–process perspective: heatwave-induced cooling service supply–demand mismatch (15 September 2026)](https://doi.org/10.1016/j.scs.2026.107593)

**Interpretation rule:** a cohort with no return to baseline within the sequence remains UNRECOVERED_AT_SEQUENCE_END; recovery time is undefined, not zero or extrapolated. These are aggregate service metrics, not unique people, health risk or an instruction to allocate capacity.

## 9. Cross-event fairness research target

A further research increment is to compare:
1. static accessible-first maximum throughput;
2. discrete max-min cohort coverage at each snapshot;
3. cumulative service-debt-aware progressive filling with a declared rotating tie-break;
4. a no-history baseline that resets service state each event.

The primary tests should use repeated and unequal-duration heat events, changing route/site availability, demand surges and recovery windows. Report both per-event and cumulative place-hour distributions, including the worst cohort and worst timepoint. No synthetic duration or ordering is to be promoted to real chronology; Symtropy supplies native time/event identity for any real replay.

Acceptance targets:
- 100% conservation between cohort and network-wide place-hour ledgers;
- zero duplicate cohort/timepoint identities silently merged;
- preserve accessible unmet-place-hour accounting separately;
- no snapshot sum represented as unique people;
- disclose cumulative service-ratio and absolute-debt distributions together;
- no debt-derived priority or allocation without owner/resident governance.

**Claim ceiling:** this is a synthetic ordered-sequence audit; it does not identify individuals, estimate unique affected people, prove actual refuge operations or allocate future capacity.


## 10. Temporal evidence freshness and event binding

Each time slice must carry a snapshot evaluation event ID equal to that slice's Symtropy timepoint ID. Site and route freshness is evaluated against that snapshot's event, not against a wall clock generated by the spatial application. Source event IDs and elapsed age are supplied by Symtropy from its native sequence; maximum age and the policy reference are supplied by the responsible service/route/water owner.

This makes staleness a time-varying service degradation mechanism. When a capacity, water, thermal-service or route observation becomes stale, its site/edge is excluded in that snapshot; the resulting unmet-place-hours and accessible unmet-place-hours accrue to the temporal service-debt ledger. A subsequent fresh observation does not automatically restore service unless the owner has requalified it.

The synthetic sequence now includes explicit site status/capacity/service/water and route/accessibility freshness. Its age values are test fixtures, not observed data or recommended maximum-age policies.
