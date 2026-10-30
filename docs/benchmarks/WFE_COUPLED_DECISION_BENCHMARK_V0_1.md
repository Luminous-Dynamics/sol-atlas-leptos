# Water–Food–Energy Coupled Decision Benchmark 0.1

**Status:** synthetic composition contract and golden fixtures; no cross-project evaluator or field pilot is claimed.  
**Date:** 2026-10-30  
**Parent:** [WFE-COUPLING-001 / Sol-Atlas #326](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/326)

## 1. Purpose

Define a reproducible bundle that composes water-network, pump-energy, soil-water, irrigation-policy and crop-response artifacts while retaining the authority and limits of each source model.

The benchmark asks:

> Given a declared water demand, pump operating envelope, terminal electrical supply, soil-water balance inputs and irrigation allocation, is the proposed allocation physically and operationally feasible under all declared hard constraints—and what remains unknown?

This version is a **cross-model contract and known-answer fixture pack**. It does not implement or validate a hydraulic, soil-flow, crop-growth or irrigation optimizer.

## 2. Reuse boundaries

| Domain | Existing owner | Reuse and limitation |
|---|---|---|
| Water hydraulics and water service | Sol-Atlas EPANET pilot [PR #112](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/112), service resilience [PR #116](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/116) | EPANET remains the hydraulic reference. The Sol-Atlas max-flow/service evaluator is not a pressure/head/pump-curve solver. |
| Pump power / water coupling | Sol-Atlas issues [#144](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/144) and [#147](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/147) | Reuse pump-terminal power and specific-energy bounds; do not infer feeder-to-pump losses without a validated mapping. |
| Soil-water accounting | Symtropy [SOIL-TWIN-001 PR #2848](https://github.com/Luminous-Dynamics/symtropy/pull/2848) | Consumes supplied interval fluxes and computes storage balance. It does not predict ET, runoff, drainage, Richards flow or crop growth. |
| Irrigation policy and trial design | Symthaea [CropFoundry PR #8046](https://github.com/Luminous-Dynamics/symthaea/pull/8046) | Reuse existing synthetic policy candidates, hard caps, sensing perturbations and trial-power work; do not implement another selector. |
| Farmer choice and outcome | Mycelix [FARMER-UX-001 #5194](https://github.com/Luminous-Dynamics/mycelix/issues/5194), [FARMER-UX-004 #5202](https://github.com/Luminous-Dynamics/mycelix/issues/5202), Symthaea [AGRI-ADVICE-001 #8027](https://github.com/Luminous-Dynamics/symthaea/issues/8027) | Keep advice, farmer choice, physical action, acceptance and later outcome distinct. |
| Cross-project evidence envelope | Luminous Platform OMSI agriculture profile [PR #158](https://github.com/Luminous-Dynamics/luminous-platform/pull/158) | Reuse portable evidence and source-rights semantics; do not duplicate source registries or farm identity. |

## 3. Model/reference lanes are deliberately separate

- **EPANET 2.2**: hydraulic reference for supported pressurized-water-network cases under matched topology, demand, boundary conditions, pump curves and controls. A pass on a graph-flow model is not hydraulic validation.
- **Pump energy bound**: a transparent operating-point constraint. Given pump-terminal available power (P) in kW and specific energy (e) in kWh/m³, energy-limited flow is (P/e) in m³/h. Intersect with mechanical-flow and actuator bounds. This is not a pump curve or electrical load-flow model.
- **Symtropy SOIL-TWIN-001**: supplied-flux conservation accounting. For one interval, storage after = initial storage + precipitation + irrigation − ET − runoff − drainage. It does not independently validate the measured/supplied fluxes.
- **FAO AquaCrop 7.3**: separate crop-water response reference only for supported herbaceous crops and matched management/soil/climate inputs. A run output stays modelled, not harvested yield. v0.1 fixtures explicitly mark the crop response as not run; they do not invent a reference result.
- **FAO WaPOR v3**: optional regional water-productivity context with level, pixel support and time aggregation preserved. It is not plot-level soil moisture or yield ground truth.

If crop, soil, boundary conditions, forcing, units, output definition, time step, spatial support or model applicability do not match, the result is `not_comparable`; do not manufacture a converted score.

## 4. Contract semantics

### 4.1 Input artifact references

Each domain artifact reference has:
- stable local artifact ID;
- producer and exact model/policy identifier;
- version;
- evidence class;
- execution state;
- upstream artifact IDs that were actually used.

The benchmark bundle is not itself a validation of those referenced artifacts. A future materialized run must bind the exact input, executable, configuration and output bytes with digests in the run receipt.

### 4.2 Power-to-pumping relation

For bounded intervals:

- energy-limited lower flow = lower pump-terminal power / upper specific energy;
- energy-limited upper flow = upper pump-terminal power / lower specific energy;
- effective pump capacity is bounded by the minimum of the mechanical-flow interval and the energy-limited interval.

A missing power interval is **indeterminate**, not zero power and not unlimited power. Power at a generator, feeder, substation or grid connection cannot be used as pump-terminal power without a declared and validated conversion/loss model.

### 4.3 Water and service constraints

A candidate must satisfy all declared caps and service floors. Water-volume caps, flow rates, actuator limits and service demand must use their declared units and compatible time intervals. A high service score cannot compensate for violating a hard water, power, actuator or safety constraint.

### 4.4 Soil accounting

Depths refer to the same explicitly declared control-volume depth, plot area and interval. Fluxes in v0.1 are supplied inputs. If any required flux or basis is absent, no storage balance is manufactured. The balance residual and final storage must be checked without silently clamping negative storage or exceeding a declared storage capacity.

### 4.5 Crop response

The crop response record may be:
- `not_run`;
- `modelled` with exact model, version, inputs and output definition;
- `measured` only when an actual independently sourced measurement and method exist.

Do not replace `not_run` with a synthetic yield number and do not turn modelled yield into a measured outcome.

## 5. Golden scenarios

The three fixtures are:
- **WFE-FEASIBLE-001:** feasible boundary case. A 50 m³ request over 5 h requires 10 m³/h. Pump mechanical flow is 12 m³/h; pump terminal power is 5 kW; specific energy is 0.5 kWh/m³, so energy-limited flow is 10 m³/h. Water cap is 60 m³, actuator capacity is 10 m³/h. Soil balance: 90 + 2 + 8 − 5 − 1 − 2 = 92 mm, below a 100 mm storage limit. Essential-service fraction is 50/60 = 0.8333 against a declared 0.8 floor. The exact boundary is feasible because all limits are inclusive.
- **WFE-INFEASIBLE-001:** mechanically feasible but energy-limited to 8 m³/h against a 10 m³/h request. The candidate is infeasible; no soil or crop outcome is imputed for a plan that cannot be supplied.
- **WFE-INDETERMINATE-001:** pump-terminal power is unknown. No positive feasibility or service-adequacy claim is permitted, even though mechanical flow and the water cap alone appear sufficient.

All values are synthetic and hand-computable. The fixtures are not an agronomic trial, field observation, validated pump performance record or SOTA result.

## 6. Benchmark metrics

Report separate dimensions:
- hydraulic parity: head, pressure, link flow, convergence and continuity residuals against EPANET;
- pump constraint: energy-limited flow interval, mechanical interval, terminal measurement point and requested flow;
- soil accounting: conservation residual, initial/final storage and storage-bound status;
- crop reference: modelled yield/water-productivity error only on matched, supported AquaCrop tasks and held-out sites/seasons;
- decision quality: feasibility, constraint violations, regret on tiny exhaustive cases, abstention/measurement requests;
- service equity: per-community/plot floor compliance, unmet demand and unknown denominator weight;
- resource efficiency: water and energy per accepted output only where the measured output denominator exists.

Do not compress water, energy, yield, service equity, quality and cost into a single unweighted-looking score. If a stakeholder supplies a preference function, preserve it as a versioned policy input and report the original objective vector.

## 7. Negative controls

- feeder/generator power mislabeled as pump-terminal power;
- missing or non-positive specific-energy lower bound;
- a non-finite or negative interval;
- pump request greater than mechanical, energy or actuator capacity;
- irrigation volume exceeding declared water cap;
- impossible soil-water overdraw or storage overflow;
- mismatched time interval, control depth, area or flux unit;
- modelled crop output relabelled as measured harvest;
- missing/quarantined data-rights status promoted to training permission;
- same farm/plot/sensor/source experiment leaking across held-out splits;
- an unknown farmer outcome treated as success or failure;
- a crop-response model output derived from the same inputs being presented as independent ground truth.

Every invalid or incomplete case must be rejected or remain indeterminate/not-comparable for the intended reason.

## 8. Promotion gates

- **E1 composition contract:** schema validates and the three golden fixtures reproduce their declared known-answer classifications.
- **E2 reference validation:** hydraulic and crop lanes match pinned external reference runs under identical parameters, supported operating domain and units; soil arithmetic and pump bounds pass independent hand/reference checks.
- **E3 external data:** source rights confirmed; genuinely held-out sites/seasons; calibration and uncertainty; all denominators and missingness reported.
- **E4 operational integration:** authorized sensors, farm/service records and model inputs; data protection and operational owner accepted.
- **E5 shadow/controlled trial:** preregistered outcome and comparator, no unsafe actuation, farmer consent, independent measurement.
- **E6 production assurance:** deployment-specific security, safety, recovery, operator authority and continuing monitoring.

These are internal stages, not certifications. Promotion is per claim: a balance identity may be E2 while crop yield and policy efficacy remain E0/E1.

## 9. Next implementation

Implement a small Rust bundle validator in Sol-Atlas that:
1. validates the versioned JSON bundle;
2. checks interval arithmetic and cross-domain units;
3. verifies every reference ID and upstream derivation reference is closed and acyclic;
4. reconstructs the pump flow bounds and soil balance independently of candidate policy selection;
5. applies hard water/power/actuator/service floors and returns feasible, infeasible, indeterminate or not-comparable;
6. binds the report to exact input/evaluator/profile IDs.

The validator must not reimplement EPANET, AquaCrop, CropFoundry policy selection, Symtropy soil prediction or farmer workflow logic. It validates their declared output contract and the joins between them.

## 10. Claim ceiling

This package is a synthetic composition benchmark only. It does not establish:
- a valid real pump curve, site energy supply or hydraulic field model;
- crop yield, farm productivity, amendment efficacy, nutrient outcome or food safety;
- policy superiority, ROI, farmer usability or South African transfer;
- operational deployment or control authority.

No model, source or benchmark can be promoted by inclusion in this document.
