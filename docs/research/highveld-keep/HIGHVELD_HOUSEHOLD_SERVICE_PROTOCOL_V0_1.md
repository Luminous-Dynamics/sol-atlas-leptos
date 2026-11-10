# Highveld household-service package v0.1

Parent: [HIGHVELD-HOUSEHOLD-SERVICE-001 #694](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/694).

## Purpose

Bind three existing Symthaea household-service research profiles—WaterLoop, HeatBank and AirCare—to the synthetic Highveld Keep dwelling without claiming that their products, interfaces, capacities or performance are selected or qualified.

This package is a **definition and fixture-integrity artifact**. It is not a building model, energy/water simulator, product certification, resident requirements assessment, engineering approval or device-control plan.

## Frozen package

- Scenario: `HIGHVELD-HOUSEHOLD-SERVICE-SYNTHETIC-001`
- Schema: `luminous.highveld.household-service-package/0.1`
- Design reference: `HIGHVELD-KEEP-SYNTHETIC-001`
- Household profile: synthetic 4–6 occupant benchmark assumption only.
- Service products: WaterLoop #10323, HeatBank #10329, AirCare #10330.
- Architectures: independent services, shared utility and hybrid local autonomy.
- Stress definitions: 14 synthetic case families.
- Execution authority: none.

Every performance quantity, input-quality state, service floor, schedule, duty cycle, calibration state and maintenance interval remains explicitly Unknown or Not established. The fixture intentionally contains no assumed service rates or product capacities.

## Service contract rules

1. A service is a named outcome over a declared observation window, not a product nameplate.
2. Service-specific output quality and acceptance remain owned by the relevant product/technical owner.
3. Resource quantities carry a unit and source state. An Unknown quantity has a null value and no source reference.
4. Shared inputs and outputs must be counted once by their physical accounting owner; no duplicated credit for energy, water or heat.
5. A missing service floor is not zero demand and is not a pass.
6. A planned manual fallback or local-control requirement is not evidence that a working fallback exists.
7. A product profile or vendor observation is not physical performance evidence.
8. No service can inherit approval from another service.

## Architecture comparison

The three arms are deliberately not ranked:

- **Independent services:** discrete systems and explicit service boundaries.
- **Shared utility:** shared interfaces only where a qualified physical boundary exists; common-mode failure and contamination are explicit.
- **Hybrid local autonomy:** passive-first design, bounded storage and local/manual fallback; no assumed autonomy duration.

The comparison protocol requires identical event-trace bytes, information budget and human-time budget. The actual event trace bundle and budgets are not yet materialized. No architecture can therefore be scored or claimed superior from this package.

## Adversarial workload

Fourteen case definitions cover concurrency, hot/cold weather, two grid-interruption positions, water interruption, outdoor-air constraints, component faults, common-mode failure, delayed consumables/spares, observation failure, household change, resource-allocation conflict and repeat disturbance.

These are scenario *definitions*, not calibrated event durations, local weather records or probabilities. Event traces must be created and source-classified separately before executable policy comparison.

## Claim ceilings

- **E0:** this document defines a synthetic interface and Unknown states.
- **E1:** an independent checker may establish schema/semantic consistency of these exact files.
- **E2+:** require separately admitted source data and owner-native adapter qualification.
- **E3+:** require controlled bench evidence, metrology, independent endpoint assessment and repeatability.
- **E4/E5:** require professional/statutory review and appropriately governed non-occupied / occupied evidence.

No value in this package may be used as evidence of energy savings, water autonomy, air-quality protection, comfort, safety, code compliance, affordability, resident benefit or buildability.

## Required future benchmark metrics

Report separately: service delivered/unmet by service; energy by source/end use; useful thermal service and storage losses; water withdrawal/reuse/delivery/residual; indoor conditions by named pollutant and measurement method; availability and stable recovery; repair and operator labour; lifecycle cost/carbon/materials with declared boundaries; resident agency/privacy/accessibility; and model uncertainty/calibration/evidence class.

Do not collapse these into one weighted score. A hard safety, accessibility, service or consent failure cannot be compensated by savings elsewhere.

## Validation protocol

The independent Rust oracle must reject:

- unknown fields at any object depth;
- duplicate JSON object member names;
- missing or additional service/architecture/stress-case identities;
- non-null resource quantities without a declared evidence upgrade;
- any evaluated/approved architecture state in this E0 package;
- materialized trace claims without a trace bundle and digest;
- any non-empty execution authority or device-command capability;
- any claim-ceiling change.

The checker reports only `package_valid_synthetic_only` or `invalid`. It does not call external services, inspect weather files, calculate engineering outputs or authorize physical effects.
