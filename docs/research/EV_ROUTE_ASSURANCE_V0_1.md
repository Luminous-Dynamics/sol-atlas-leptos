# EV route assurance v0.1

**Tracking issue:** Sol Atlas #664 — EV-ROUTE-ENERGY-001  
**Related site interface:** Sol Atlas #665 — EV-CHARGING-SITE-001  
**Status:** first executable proposal-audit slice; synthetic fixture only

## Purpose

This increment adds an independent, read-only auditor for a proposed route and charging-stop sequence. It checks whether declared per-leg energy intervals, pack capability, connector identity, charger status, site import limits and charging duration are mutually consistent.

It is deliberately not a route optimizer, road-graph implementation, EV energy model, battery estimator, charging-network operator or vehicle controller. It consumes upstream model outputs and evidence references instead of recreating their owners.

## Four-project ownership

- **Symthaea:** supplies the versioned pack capability vector, including usable-energy and charge-acceptance intervals. This adapter does not infer pack SOH or derive capability from cell averages.
- **Symtropy:** supplies the exact EV configuration/energy-model identity and conservative per-leg energy intervals. This module does not estimate consumption or create a second EV-REF-01 model.
- **Mycelix:** supplies durable vehicle and charger configuration/lifecycle references. This module does not create asset identity, a second lifecycle ledger or a Battery Passport.
- **Sol Atlas:** supplies road-state, charger operational-state, site connection-stage and site import-capacity evidence. The map renderer remains read-only; this module does not build a road graph or infer an energized site from a map pin.

## Assurance rules

1. Energy is propagated as an interval. A route is not declared feasible when the lower remaining-energy bound crosses the pack reserve floor.
2. Connector compatibility is checked against the exact vehicle configuration.
3. A charger must have a current operational observation. Unknown, forecast, synthetic or expired status is not treated as available.
4. Site import power is usable only when the connection stage is energized, accompanied by its own current evidence and a site-import interval. An application, budget quote, allocation or signed agreement is not energization.
5. Conservative connector power is bounded by the minimum of (a) site import multiplied by the declared site-to-connector efficiency, (b) EVSE output power and (c) pack charge acceptance. Charging energy is limited by the planned dwell duration.
6. Missing, synthetic, forecast, scenario or unknown evidence yields INDETERMINATE; an evidenced closed road, offline charger, incompatible connector, impossible charge target or definite reserve breach yields INFEASIBLE.
7. Events must be chronologically ordered and non-overlapping. Input interval and digest shape errors are rejected.

## Result semantics

- FEASIBLE_UNDER_DECLARED_BOUNDS: all required evidence classes and freshness windows satisfy this software contract, and conservative interval arithmetic stays above reserve.
- INFEASIBLE: at least one evidenced hard contradiction or definite bound violation exists.
- INDETERMINATE: the proposed plan may be feasible, but evidence, freshness, model support or interval bounds do not establish it.

A feasible result is not a statement that an EV trip is physically safe, that an advertised charger is genuinely available beyond its evidence window, that grid import has been legally approved, or that the model is empirically calibrated. It is a proposal audit under declared bounds only.

The receipt hashes the exact input bytes and compact assessment JSON. These hashes bind identity/replay; they do not authenticate a source, prove truth or substitute for a signature or authority check.

## CLI

Run from the repository root:

    cargo run -p sol-atlas-core --bin ev_route_assess --locked -- schemas/examples/ev-route-assurance-v0.1.synthetic.json

The frozen example deliberately uses synthetic evidence and must return INDETERMINATE, while still exposing the arithmetic path. It is not a field-data result.

## Validation ladder

1. **Software contract:** strict schema, hostile fixtures, exact interval arithmetic, chronology, connector and site-stage negatives.
2. **Matched route-energy evaluation:** consume a frozen Symtropy EV-REF-01 model/config and compare on identical route, grade, weather, payload, speed and temperature inputs to a separately implemented reference.
3. **Measured route calibration:** use rights-cleared vehicle/pack logs and metered route energy, split by physical vehicle/configuration and route campaign. Report MAE, RMSE, P95 absolute error and prediction-interval coverage by vehicle/terrain/temperature strata.
4. **Charger/site validation:** bind dated operator status, exact EVSE model, connector, power curve, approved import/energization records and independent metering. Keep failed sessions and stale/missing observations in the denominator.
5. **Corridor demonstration:** Johannesburg–Durban/N3 in both directions, with each charger/site and vehicle observation individually sourced. One vehicle or one journey is never a fleet distribution.

## Prospective benchmark targets (not achieved)

- Zero false-feasible results on the frozen exact and adversarial software corpus.
- 100% abstention/rejection for wrong connectors, closed routes, offline/unknown chargers, non-energized sites, expired status evidence and reserve-boundary violations.
- 100% of reports expose event-level energy intervals, input evidence references and a claim ceiling.
- On measured holdouts, target at least 95% empirical coverage for declared 90% prediction intervals; report calibration by strata and sample counts.
- Report false-feasible rate as a hard non-compensatory metric. Do not hide it inside a weighted score.
- Report planning/audit latency, memory, exact-oracle agreement and performance against shortest-distance, fastest-time, nearest-charger and fixed-reserve heuristics on matched input information and compute budget.

No SOTA or field-performance claim is made by this v0.1 software slice. Promotion requires matched external baselines, rights-cleared measured data, independent numerical references and field qualification.
