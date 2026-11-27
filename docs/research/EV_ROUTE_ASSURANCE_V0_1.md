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
6. **SOC-taper boundary:** the v0.1 pack charge-acceptance interval is a session-wide conservative minimum over the complete SOC and battery-temperature envelope expected during the dwell. It must not be a nameplate rating, peak power, or a point sample at arrival SOC. The input digest/evidence reference must bind the underlying envelope calculation. A pointwise or unscoped value is not eligible for a qualified-model evidence class.
7. The v0.1 schema does not carry the piecewise SOC/temperature curve itself. Before production route qualification, v0.2 should consume the banded acceptance curves being developed in Symtropy EV charging-curve work (#2116), plus a scoped thermal/capability envelope from Symthaea, and verify that the full predicted SOC/temperature trajectory stays inside the supplied curve domain.
6. Missing, synthetic, forecast, scenario or unknown evidence yields INDETERMINATE; an evidenced closed road, offline charger, incompatible connector, impossible charge target or definite reserve breach yields INFEASIBLE.
7. Events must be chronologically ordered and non-overlapping. Input interval and digest shape errors are rejected.
8. Evidence can contribute to a FEASIBLE result only when it is authenticated by an independently supplied, role-scoped, non-revoked and time-valid Ed25519 trust-store key. Missing, malformed, mismatched, revoked or unauthorized signatures remain non-qualifying.

## Result semantics

- FEASIBLE_UNDER_DECLARED_BOUNDS: all required evidence classes and freshness windows satisfy this software contract, and conservative interval arithmetic stays above reserve.
- INFEASIBLE: at least one evidenced hard contradiction or definite bound violation exists.
- INDETERMINATE: the proposed plan may be feasible, but evidence, freshness, model support or interval bounds do not establish it.

A feasible result is not a statement that an EV trip is physically safe, that an advertised charger is genuinely available beyond its evidence window, that grid import has been legally approved, or that the model is empirically calibrated. It is a proposal audit under declared bounds only.

The receipt hashes the exact input bytes, the separately supplied trust-store bytes and the compact assessment JSON. The trust store is external to the route plan: a plan cannot insert its own trusted keys or expand its own issuer authority.

### Evidence admission and trust roots

Each evidence reference may carry an Ed25519 attestation binding the exact reference ID, SHA-256 digest, evidence class, observation/validity window, evidence role, subject ID, issuer ID/role, key ID and issue time. Signatures use a route-protocol domain separator and a versioned typed claim encoding.

The independent trust-store input contains the admitted public keys, issuer identities/roles, authorization by evidence role, validity windows and revocation state. The caller must also supply an **expected trust-store SHA-256 pinned by deployment policy outside the route plan and trust-store file**. The auditor rejects a trust-store snapshot whose exact bytes do not match this pinned digest. A signature is accepted only when the exact claim, signer, scope, role and assessment-time policy checks pass. The assessment output records each consumed reference's authentication status and issuer/key identity, and its receipt separately hashes the trust-store snapshot. The synthetic CLI example uses a fixed public test-fixture digest only; production deployments must pin their actual trust-store digest through their protected deployment configuration.

**A signature authenticates an issuer-bound claim; it does not establish sensor accuracy, calibration correctness, model validity, grid approval, vehicle safety or the state of the physical world.** The route auditor does not fetch referenced source artifacts or independently re-hash their underlying bytes. Those must be verified by the evidence-admission and source-specific owner before a claim is classified as suitable for this contract. Never place production private keys or real trust anchors in fixtures or public examples. The example trust store is deliberately synthetic and must not be used for production evidence.

## CLI

Run from the repository root:

    cargo run -p sol-atlas-core --bin ev_route_assess --locked -- --input schemas/examples/ev-route-assurance-v0.1.synthetic.json --trust-store schemas/examples/ev-route-trust-store-v0.1.synthetic.json --expected-trust-store-sha256 939cec28ec4f240505600f96bd4275f4f9f17cfe9428225cc06e8d8b16488b72

The frozen route example deliberately carries no attestations and must return INDETERMINATE, while still exposing the arithmetic path and the per-reference UNATTESTED statuses. The trust-store example contains only a deterministic public test key; it cannot authenticate the example plan because no evidence signatures are supplied. Neither file is field evidence.

## Validation ladder

1. **Software contract:** strict schema, hostile fixtures, exact interval arithmetic, chronology, connector and site-stage negatives.
2. **Matched route-energy evaluation:** consume a frozen Symtropy EV-REF-01 model/config and compare on identical route, grade, weather, payload, speed and temperature inputs to a separately implemented reference.
3. **Measured route calibration:** use rights-cleared vehicle/pack logs and metered route energy, split by physical vehicle/configuration and route campaign. Report MAE, RMSE, P95 absolute error and prediction-interval coverage by vehicle/terrain/temperature strata.
4. **Charger/site validation:** bind dated operator status, exact EVSE model, connector, power curve, approved import/energization records and independent metering. Keep failed sessions and stale/missing observations in the denominator.
5. **Corridor demonstration:** Johannesburg–Durban/N3 in both directions, with each charger/site and vehicle observation individually sourced. One vehicle or one journey is never a fleet distribution.

## Known v0.1 limitation: charging taper

The route arithmetic uses a single acceptance interval. Its safety meaning therefore depends on the upstream producer supplying a conservative minimum across the full charge session; this implementation does not reconstruct or independently validate a piecewise SOC-dependent taper curve. Do not feed a battery nameplate maximum or initial-SOC point into this field and then interpret a pass as an end-to-end charging validation. The v0.2 target is explicit piecewise SOC acceptance, temperature/health derating, session integration and domain-coverage checks against the actual charger/site power ceiling.

## Prospective benchmark targets (not achieved)

- Zero false-feasible results on the frozen exact and adversarial software corpus.
- 100% abstention/rejection for wrong connectors, closed routes, offline/unknown chargers, non-energized sites, expired status evidence and reserve-boundary violations.
- 100% of reports expose event-level energy intervals, input evidence references and a claim ceiling.
- On measured holdouts, target at least 95% empirical coverage for declared 90% prediction intervals; report calibration by strata and sample counts.
- Report false-feasible rate as a hard non-compensatory metric. Do not hide it inside a weighted score.
- Report planning/audit latency, memory, exact-oracle agreement and performance against shortest-distance, fastest-time, nearest-charger and fixed-reserve heuristics on matched input information and compute budget.

No SOTA or field-performance claim is made by this v0.1 software slice. Promotion requires matched external baselines, rights-cleared measured data, independent numerical references and field qualification.
