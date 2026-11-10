# Highveld household service outcome binding v0.1

Parent: [HIGHVELD-HOUSEHOLD-SERVICE-OUTCOME-BINDING-001 #700](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/700).

## Purpose

Bind the synthetic WaterLoop, HeatBank and AirCare service-package outputs to the exact *existing* read-only post-occupancy outcome contract and deterministic service-availability timeline contract. This package owns only the mapping and unbound-state rules. It is not a new observation ledger, meter registry, timeline engine, service evaluator, physical solver or device authority.

## Owner-native contracts

### Sol-Atlas CITY-IN-USE-OUTCOMES-001

Exact owner source audited: `tools/city-in-use-outcomes-bench/src/lib.rs` on `research/city-in-use-outcomes-v0.1`.

Observation fields consumed by the owner are preserved verbatim: `metric_id`, `value`, `unit`, `evidence_state`, `evidence_kind`, `source_owner`, `source_contract`, `source_revision`, `source_receipt_id`, `source_subject_digest`, `scope_digest`, `design_revision`, `bom_revision`, `observation_window`, `context`, `instrument_id`, `calibration_state`, and `coverage_fraction`.

Requirement fields are preserved verbatim: `metric_id`, `unit`, `direction`, `threshold`, `evidence_class`, `definition_revision`, `expected_source_owner`, `expected_source_contract`, and `expected_source_revision`.

Evidence distinctions are exact: requirement class `meter` expects observation kind `meter_measurement`; `experience` expects `consented_survey` plus `consent_status=authorized`; `service` expects `service_receipt`. Do not convert meter, experience or service evidence into one another.

Owner contract version: 0.1.0. The owner evaluates comparability/eligibility only. It does not calculate savings, causal impact, regulatory compliance or physical sensor calibration.

### Symtropy SPATIAL-RESILIENCE-TIMELINE-ORACLE

Exact owner source audited: `tools/spatial-resilience-qualifier/src/lib.rs` and `docs/research/spatial-design/SPATIAL_RESILIENCE_TIMELINE_ORACLE_V0_1.json` on `research/spatial-resilience-trajectory-benchmark-2026-10`.

Event fields are `id`, `generation`, `tick`, `sequence`, `kind`; event kinds are lowercase `restored` / `lost`. The timeline evaluator computes continuous availability and stable recovery over a declared horizon. It has no water-flow, energy, pollutant, temperature, quality or meter quantity.

Each service receives its own timeline evaluation. `service_id` is crosswalk wrapper metadata and is not added to the owner-native event object. A restore event is not proof of measured physical delivery.

## Service mapping state

Ten output/input resource metrics are mapped from exact fields in the Highveld package to Sol-Atlas's measurement-observation contract shape. Their source owner, source receipt/revision, service floor, instrument profile and calibration remain unbound. Therefore every current mapping has state `UNBOUND`; no actual `observations[]` record or numeric threshold is emitted.

Two AirCare outputs remain explicitly unbound: the current aggregate `clean_air_delivery` lacks pollutant-specific and occupied-zone identity; condensate/residual has no selected unit. Do not promote either.

## Fail-closed disposition hierarchy

- Unknown service/metric/unit or mismatched scope/revisions/window/context -> `NOT_COMPARABLE`.
- Event wrapper or event payload invalid -> `REJECTED_EVENT_SCOPE` / `REJECTED_EVENT_CONTRACT`.
- Event used as measurement, or model/vendor assertion used as measured physical truth -> `REJECTED_CLAIM_PROMOTION`.
- Any command request -> `REJECTED_NO_DEVICE_AUTHORITY`.
- Wrong meter evidence class, consent or purpose -> rejected, never silently converted.
- Missing calibration, observation coverage, source owner/receipt, service floor or admitted instrument profile remains a specifically named `UNKNOWN` / `UNBOUND` state.
- Fully satisfied synthetic gates yield only `ELIGIBLE_FOR_INDEPENDENT_OWNER_REVIEW_ONLY`, never PASS and never evidence of physical outcome.

## Claim ceiling

All 36 tests are synthetic crosswalk logic. No real measurement, meter, calibration, resident requirement, water/air quality, delivered hot water, thermal comfort, service continuity, savings, causal benefit, safety, compliance, or device command is qualified.
