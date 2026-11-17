# Highveld paired courtyard thermal-water test cell protocol v0.1

**Study:** `HIGHVELD-PASSIVE-COOLING-WATER-001`  
**Maturity:** `FIELD_PROTOCOL_NOT_DEPLOYED`  
**Purpose:** test whether a small, low-splash recirculating water intervention creates a measurable occupied-zone thermal benefit over a competent no-water passive baseline.

## 1. Why start with a test cell rather than a swimming pool

A full pool retrofit is expensive, water-intensive and confounds thermal storage, evaporation, solar exposure, occupancy and amenity use. The first field question should be narrower: does a deliberately small, metered and controlled water intervention create a repeatable improvement in a shaded courtyard under Highveld weather—and what water and electrical cost does it impose?

Start with two comparable, unoccupied or supervised research bays/courtyard cells. Do not begin with spray/mist, public bathing, untreated greywater, a large pool or a water feature that can aerosolise contaminated water. A closed-loop low-splash rill or wetted test panel is sufficient for the first field question.

This protocol is a measurement design only. Installation requires site-owner permission, qualified engineering, electrical and water safety review, water-quality review, mosquito/vector controls, child/drowning prevention where relevant, and current municipal restriction compliance.

## 2. Paired, randomized crossover design

### Test arms

- **A — control:** competent no-water passive baseline with the same shade, geometry, surface, ventilation and control policy.
- **B — intervention:** same baseline plus a removable, metered, shaded, low-splash recirculating water feature with a closed return and measured pump power.

### Bias controls

1. Use two cells with matched geometry, orientation, surface finish, shade and ventilation. If two cells cannot be matched adequately, use one cell in randomized crossover periods and report the reduced ability to control weather drift.
2. Randomly assign which cell receives B during the first block.
3. Swap A/B between cells on a preregistered schedule to separate intervention effect from location bias.
4. Use concurrent measurements in both cells; match analysis periods by outdoor weather and solar exposure.
5. Predeclare minimum valid data coverage, sensor outage handling, wet/rain periods, occupancy exclusions and stop rules.
6. Conduct an initial shakedown to estimate variance. Determine confirmatory sample size/power from that variance and predeclared minimum practically important difference; freeze the confirmatory plan before examining outcome differences.
7. Keep the weather/control input horizon identical. Controllers must not use future information unavailable to the other arm.

Do not select only the hottest days after viewing the results. Report all preregistered valid/invalid periods and the reason for exclusions.

## 3. Instrumentation and placement

All devices need unique instrument IDs, manufacturer/model, range, resolution, calibration certificate or comparison procedure, calibration date, as-found/as-left status, placement coordinates/height, clock source/synchronisation, uncertainty and maintenance history.

### Ambient and comfort sensors

- Shielded air temperature and relative-humidity sensors at agreed occupied-zone heights (for example, near ankle, seated breathing-zone and standing breathing-zone positions where those use cases matter).
- Globe temperature or an appropriately specified operative-temperature measurement method.
- Surface temperature on principal shaded/unshaded surfaces and the wetted surface.
- Outdoor reference air temperature/RH, wind speed/direction, global solar irradiance, rainfall and barometric pressure where needed by the analysis.
- A sensor placement drawing with height and distance from the water edge, openings, walls and direct sun.

Sensor heights are study design choices, not represented as a universal code prescription. Select them to reflect the intended occupant/use case and document why.

### Water/plant sensors

- Water volume or calibrated tank-level measurement and total make-up water meter.
- Flow meter on the recirculation loop.
- Water temperature at inlet/outlet and multiple depths if stratification can occur.
- Calibrated temperature pair across any heat exchanger; record flow simultaneously so transferred heat can be estimated.
- Pump/filter/control electrical energy meter.
- Separate records for rain capture, evaporation estimate, splash-out, purge/backwash, leakage, overflow, transfer and drain-down.
- Water-quality test results appropriate to source and intended contact/aerosol exposure.

A single top-up meter cannot distinguish evaporation from leaks, splash, backwash or overflow. Log those terms separately or record them as unknown with uncertainty.

## 4. Sampling, clocks and data integrity

- Start with 1-minute sampling for fast-changing air/plant signals; retain higher raw sample rates if the instrument supports them. Aggregate to 5- or 15-minute analysis intervals only after preserving raw measurements.
- Synchronize all clocks to a documented time source; record timezone and daylight-saving handling.
- Retain raw readings, sensor quality flags, missingness, calibration, filtering/aggregation transforms and resulting datasets separately.
- Do not interpolate across sensor outages for hard-gate evidence. Missing water or safety evidence blocks the corresponding claim.
- Sign the daily data package with source IDs, schema revision, instrument IDs, calibration evidence, interval start/end conventions and SHA-256 digest.
- Symtropy must provide native event identity, ordering, clock and trajectory receipts; this field protocol does not invent event ticks.

## 5. Physical balances

### Water balance

For each interval:

[
Delta V = V_{rain} + V_{makeup} + V_{transfer,in}
- V_{evap} - V_{splash} - V_{purge/backwash}
- V_{leak} - V_{overflow} - V_{transfer,out}
]

Report measured, estimated and unknown terms separately. Propagate each term's uncertainty. The closing residual must be compared with the combined measurement uncertainty. A large residual cannot be assigned to evaporation simply to improve the energy/water story.

### Heat transfer

For a measured liquid stream:

[
dot Q = dot m c_p (T_{out}-T_{in})
]

Integrate over the exact sampling interval and report the sign convention. This estimates heat transferred by that stream; it does not establish whole-building useful cooling unless the receiving/rejecting side and occupied-zone outcome are also measured.

Report separately:
- water sensible storage change;
- heat exchanged through the measured loop;
- estimated latent heat at evaporation interface;
- radiant/solar/conductive terms if included in the chosen model;
- pump/filter/treatment electricity;
- measured occupied-zone and outdoor thermal response;
- unclosed balance and uncertainty.

## 6. Outcomes and causal estimate

Primary outcome:
- difference in occupied-zone operative-temperature/discomfort response between B and A over matched valid periods, stratified by outdoor dry-bulb temperature, humidity/wet-bulb depression, wind and solar radiation.

Secondary outcomes:
- occupied-hour comfort degree-hours and peak/duration out of declared comfort bounds;
- relative humidity/dew point and condensation events;
- water loss by category and litres of makeup per useful cooling kWh (only once useful cooling is independently qualified);
- pump/filter energy;
- mean radiant temperature and surface-temperature changes;
- time to response, persistence after shutdown and night-time heat rejection;
- maintenance labour, fouling, water-quality alarms and downtime;
- household/resident access and acceptability where a real shared setting is later studied.

Use paired difference-in-differences or a preregistered hierarchical model with cell, period and weather covariates. Report effect size, uncertainty interval, residual diagnostics and sensitivity to exclusions. Do not use a p-value as a substitute for thermal significance or water sustainability.

## 7. Safety and stop rules

Immediately stop operation and isolate the intervention if any of the following occurs:
- applicable municipal restriction or source permission prohibits operation;
- water source rights/quality becomes unknown, revoked or failed;
- essential water reserve would be consumed;
- unexpected spray/aerosol, contamination, stagnant water or vector-risk condition develops;
- electrical, slip, structural, child/drowning or access safeguard fails;
- leakage or water-balance residual exceeds the predeclared stop threshold;
- sensor or control fault removes a safety-critical observation;
- qualified maintenance or safe isolation is unavailable.

Use no public bathing and no aerosol/mist in this first test. No unqualified greywater is to enter the system.

## 8. Evidence packet and acceptance

Every run must have:
- preregistered protocol version and analysis plan;
- site geometry and orientation digest;
- weather/source file and restrictions snapshot;
- instrument registry, calibration records and sensor map;
- water source, rights and quality approvals;
- raw and processed time series with digests;
- water/energy balances and propagated uncertainty;
- intervention/crossover assignment record;
- missingness/exclusion register;
- model and script versions;
- independent reviewer identity and replay result.

Acceptance for a cooling claim requires all safety/rights gates to pass, water and thermal balances to close within declared uncertainty, a repeatable matched improvement in the primary occupied-zone outcome, no hidden humidity/comfort penalty, and a measured water/energy cost that satisfies the predeclared limits. If it fails any gate, report the result as not qualified rather than average the violation away.

## 9. What this first test cannot establish

It cannot establish:
- performance of a full swimming pool, all courtyard geometries or all Johannesburg neighborhoods;
- annual water savings or climate-change resilience;
- public-pool or water-feature health compliance;
- lifecycle cost, payback or social acceptability at scale;
- superiority over EnergyPlus, BOPTEST, CFD or other engines;
- essential-service resilience or water delivery.

Those claims need separate representative models, data, permissions and validation.
