# Highveld passive cooling — pools, water features and thermal-water integration v0.1

**Parent candidate:** `NEIGHBORHOOD_RESILIENCE_COMMONS`  
**Maturity:** `DESIGN_HYPOTHESES_NOT_PHYSICALLY_EVALUATED`  
**Primary climate context:** Johannesburg / Highveld; other locations require their own weather and water-source qualification  
**Decision rule:** water is a scarce thermal resource; no cooling benefit is assumed from the presence of water

## 1. Design thesis

Yes—pools and water features should be integrated into the building and neighbourhood thermal model, but not simply added as decorative evaporative coolers. We should treat water as a multi-function thermal subsystem with two fundamentally different operating modes:

1. **Latent/evaporative cooling:** deliberately evaporate water into an airstream or courtyard. This can lower dry-bulb temperature toward the local wet-bulb temperature, while increasing humidity and consuming water.
2. **Sensible/thermal storage:** use the heat capacity of a pool, cistern or water store as a thermal reservoir coupled through a qualified heat exchanger, slab, radiant loop, ground loop or night-sky heat rejector. This can shift heat in time without requiring evaporation as the primary mechanism.

These functions must not be conflated. A deep pool can store substantial heat but does not automatically cool a room. A shallow rill or fountain may provide evaporative cooling but can waste water, increase humidity and add pump/filtration loads. Every candidate must beat a no-water passive baseline on declared thermal outcomes while staying inside a hard water, energy, health and maintenance budget.

## 2. Johannesburg-specific constraints

Johannesburg Water announced Level 1 restrictions effective **1 September 2026 through 31 March 2027**. During that period, filling swimming pools and watering gardens are prohibited between 06:00 and 18:00. This is a current operating constraint, not a reason to shift avoidable filling to restricted periods: every system must comply with the applicable municipal rules and source permissions at the time of operation. [Johannesburg Water notice, 15 September 2026](https://www.johannesburgwater.co.za/blog/2026/09/15/level-1-water-restrictions-are-here-lets-make-every-drop-count/).

A Johannesburg energy-efficient building guideline recommends passive/mixed-mode strategies such as natural ventilation, high thermal mass, shading and night cooling for many buildings; it also notes winter nighttime cold. It is an older 2008 guideline and must be checked against current codes and site data, but its climate logic remains a useful starting hypothesis. [City of Johannesburg design guideline (2008)](https://www.uj.ac.za/wp-content/uploads/2022/11/coj-design-guidelines-for-energy-efficient-buildings-in-johannesburg-2008.pdf).

ERA5-derived 1991–2020 grid-cell normals show a marked seasonal pattern: the wettest months are in the austral summer, while late winter/early spring are much drier and relatively less humid. This is modelled reanalysis, not a site weather station or a design-year file; use it only to motivate seasonal test cases, then obtain a site-specific EPW or measured weather series. [Johannesburg ERA5 climate summary](https://weathermatics.earth/weather/za/gauteng/johannesburg-993800/climate/).

**Implication:** do not run evaporative features continuously. The best opportunity may be selected hot, dry, ventilated shoulder-season periods; wet-season operation can have lower evaporative headroom and winter nights can make added moisture/cooling undesirable. Control must use measured dry-bulb temperature, humidity/dew point, local wind, occupancy and water availability—not a calendar-only schedule.

## 3. System archetypes to compare

| Archetype | Mechanism | Why test it | Principal risks / qualification need |
|---|---|---|---|
| `NO_WATER_PASSIVE_BASELINE` | Shading, orientation, cross/stack ventilation, night flushing and thermal mass; no added water feature | Essential matched reference; isolates the marginal value of water | Must be competently designed or water appears better by comparison to a deliberately poor baseline |
| `COVERED_POOL_THERMAL_MASS` | Existing/added pool is primarily a covered water mass; direct evaporation minimized outside intended use | Tests whether amenity + thermal storage can provide value without continuous evaporation | Pool may add no useful heat sink unless coupled to a real heat-transfer path; embodied water, leaks, treatment and safety remain |
| `SHADED_RECIRCULATING_RILL` | Small, low-flow, recirculating surface with a capture trough and controlled exposure | Tests a low-volume microclimate intervention | High surface-area/volume ratio can raise evaporation; humidity, splash-out, biological growth and pump energy |
| `EVAPORATIVE_DOWNSHAFT` | Air passes through wetted media or a controlled wet surface before entering a shaded courtyard/space | Tests a more directed evaporative path rather than cooling a whole outdoor area | Water use, humidity rise, aerosol/biological risk, drift, maintenance and wet-bulb ceiling |
| `CLOSED_LOOP_THERMAL_WATER_STORE` | Pool/cistern water exchanges heat through a closed hydronic circuit; heat is rejected to ground, night air or a radiative surface where qualified | Tests thermal shifting without routine water evaporation into occupied air | Heat exchanger/pump electricity, fouling, condensation, insufficient nighttime sink and seasonal heat accumulation |
| `POOL_COURTYARD_HYBRID` | Shaded covered pool + controllable exposure + night-flushed high-mass courtyard + optional closed-loop exchange | Tests staged dual-use: amenity, thermal mass and limited latent cooling | More interfaces, controls and maintenance; higher risk of double-counting cooling and water benefits |
| `UNBOUNDED_SPRAY_NEGATIVE_CONTROL` | Unmetered high-flow fountain/mist or splash feature | Negative control only; demonstrates why visible coolness is not enough | Expected to fail water accounting and/or humidity, aerosol, drift and energy constraints |

A no-water baseline is mandatory. Do not compare a water feature only against an unshaded, unventilated building.

## 4. Physical accounting that prevents false cooling claims

### 4.1 Water storage

For liquid water near ordinary building temperatures, a first-order thermal capacity is:

[
Q_{mathrm{store}} = ho_w c_{p,w} V Delta T
]

Using (ho_w approx 1000,mathrm{kg/m^3}) and (c_{p,w} approx 4.186,mathrm{kJ/(kg,K)}), this is approximately **1.163 kWh per cubic metre per kelvin**. Thus a 10 m³ store over a 1 K temperature swing has about 11.6 kWh of *thermal storage capacity*, not 11.6 kWh of guaranteed building cooling. Actual useful cooling depends on exchanger conductance, temperature lift, timing, stratification, losses and the external heat-rejection path.

A pool connected to a heat exchanger is a different system from a pool merely placed beside a building. Record the heat-flow path, not just volume.

### 4.2 Evaporation

The latent heat of vaporization near common outdoor temperatures is roughly 2.4 MJ/kg, or about **0.67 kWh per litre evaporated** as a thermodynamic upper-bound conversion. This is heat absorbed at the evaporation interface, not a claim that a litre of water yields 0.67 kWh of useful cooling at the occupant or room. Airflow, wet-bulb depression, heat and mass transfer, humidity limits and the fraction of the cooling that reaches the occupied zone all matter.

Record:
- measured make-up water and source;
- evaporation estimate and its method/uncertainty;
- splash-out, overflow, purge/backwash, cleaning and leaks separately;
- pump, filtration, treatment and control energy;
- useful heat removed from the occupied zone (not latent heat calculated from water loss alone);
- indoor and courtyard dry-bulb temperature, relative humidity, dew point, operative temperature and occupancy.

### 4.3 Closed water balance

Every feature needs a closed, unit-consistent balance:

[
Delta V = V_{mathrm{rain}} + V_{mathrm{makeup}} + V_{mathrm{transfer,in}}
- V_{mathrm{evap}} - V_{mathrm{splash}} - V_{mathrm{purge/backwash}}
- V_{mathrm{leak}} - V_{mathrm{overflow}} - V_{mathrm{transfer,out}}
]

Meter or instrument the major terms. Unknown loss is not zero loss. A feature with no trustworthy water balance cannot qualify as water-efficient cooling.

## 5. Architecture rules

1. **Shade first, evaporate second.** Reduce solar load on roof, glazing, walls and paving before spending water to cool the air.
2. **Default cover when not intentionally used.** Design the pool around a practical cover, cover storage and safe operation. Published municipal and WaterSense guidance identifies covers as a major way to reduce evaporation; the exact saving must be measured at the selected site and cover. [eThekwini water-conservation guidance](https://environment.durban.gov.za/uploads/0000/6/2025/09/23/water-conservation-guideline.pdf); [US EPA pool water efficiency](https://www.epa.gov/watersense/pool-water-efficiency).
3. **Avoid aerosol by default.** Prefer contained, low-splash, gravity-return features over jets/misters. A spray/mist system is a separate engineered subsystem with aerosol, drift, water-quality and maintenance controls—not an ornamental add-on.
4. **Use closed loops.** Recirculate feature water; capture splash and overflow; filter leaves/sediment; provide accessible drains and safe maintenance isolation.
5. **Keep water sources distinct.** Potable, harvested rainwater, greywater and treated reclaimed water are different source classes with different quality and legal constraints. Do not put untreated greywater into a bathing pool or aerosol-producing feature. Greywater used for landscape irrigation is not automatically suitable for pools, cooling coils or misting.
6. **Harvest without contaminating.** Roof runoff may feed non-potable storage or a feature only through a designed first-flush, debris filtration, overflow and water-quality path. Do not assume every paved runoff area is clean.
7. **Protect access and safety.** Include drowning prevention/barriers, child and animal safety, slip resistance, electrical isolation, chemical handling, accessibility, mosquito/vector management and public-health requirements in the design contract. A cover is not automatically a certified safety cover.
8. **Preserve household sovereignty.** Pool access, water allocation, telemetry and operating schedules require resident consent and human-governed allocation; no optimiser may deprive a household of its declared essential water floor.
9. **Winter/degraded mode is explicit.** Drain, isolate, cover or operate as a thermal store only within qualified frost, hygiene, structural and condensation limits. Do not assume summer control is safe year-round.
10. **No water-dependent essential floor.** Passive cooling is an optional comfort/resilience service; it must not consume water reserved for drinking, sanitation, hygiene, firefighting or other declared essential floors.

## 6. Control concept: water-budgeted latent cooling

The controller should select among `COVERED`, `THERMAL_STORAGE`, `EVAPORATIVE_ASSIST`, `BYPASS` and `SAFE_SHUTDOWN` modes. Mode selection is conditional on:
- current and forecast dry-bulb temperature, wet-bulb temperature, humidity/dew point and wind;
- occupied-zone humidity/condensation limits;
- measured thermal response versus no-water baseline;
- water account balance, quality class, reserve floors, restriction state and source permission;
- pump/filtration energy and equipment availability;
- maintenance, water-quality and aerosol monitoring state.

The evaporative mode is blocked when water availability, water quality, humidity/condensation, aerosol controls, maintenance or legal restrictions are unknown or fail. The controller may recommend, but residents/owners retain authority over discretionary use. No numeric humidity, wet-bulb or water budget threshold is universal: each building's comfort and health criteria must be established by its engineer and occupants.

## 7. Benchmark: prove the marginal value of water

### Freeze a matched building package

For every archetype, hold constant the weather file, orientation, geometry, envelope, shading opportunities, occupancy, internal gains, ventilation schedule, comfort criteria and control-information horizon. Keep service floors and water-source quality rules identical. Run:
- **water-limited primary case:** same declared non-potable/harvested-water budget and reserve floors;
- **capacity-matched case:** same installed thermal capacity where physically meaningful;
- **energy-matched sensitivity:** equal auxiliary electricity budget;
- **drought/curtailment case:** water availability falls or discretionary use is suspended;
- **humid-hot case:** latent cooling is constrained by humidity/condensation;
- **cold-night case:** verify added moisture/cooling does not worsen winter comfort or condensation.

Use a specialist building-energy solver (for example, EnergyPlus) for building heat balances; use a qualified airflow/CFD or reduced-order model for courtyard flows; use hydraulic/plant models for water circuits. Sol-Atlas must carry geometry, orientation, shading and versioned configuration. Symtropy owns common event traces, measured input timing, coupled service trajectories and stable recovery. Do not duplicate these physics engines inside the Commons qualifier.

### Required outcome vector

- occupied-hour operative-temperature and discomfort degree-hours;
- peak and duration above/below comfort bounds;
- indoor and courtyard relative humidity/dew point;
- mean radiant temperature and solar exposure;
- useful cooling delivered to occupied zones, separately from latent heat at the water surface;
- water make-up per useful kWh of cooling and per occupied-hour comfort improvement;
- leakage, evaporation, splash, purge/backwash and overflow losses;
- pump, filtration, treatment and control kWh;
- thermal storage charge/discharge, approach temperatures and heat-exchanger effectiveness;
- indoor air quality, condensation and water-quality compliance;
- maintenance labour, downtime, repairability, lifecycle cost and embodied impact;
- comfort/access distribution by household, with no average compensating for a household floor breach.

No percentage cooling reduction is claimed in advance. Success means a matched, uncertainty-qualified reduction in thermal discomfort or cooling energy without violating water, hygiene, comfort or household-floor constraints.

## 8. Adversarial and negative controls

The study must detect at least these false claims:
- nominal pool volume counted as useful room cooling without a measured/qualified heat path;
- latent heat inferred from evaporation reported as delivered building cooling;
- a tank/reservoir balance treated as household tap delivery;
- missing water-meter data interpreted as zero loss;
- rainwater or greywater reclassified as potable without treatment/rights qualification;
- untreated greywater used in pools, aerosol systems or occupied air paths;
- spray/mist feature claims cooling while ignoring humidity, drift or aerosol controls;
- a stagnant ornamental basin with no vector-control/maintenance path;
- a pool shown as covered for water savings while claiming simultaneous full-surface evaporation cooling;
- pool-cover safety assumed without the required local certification/barriers;
- water draw exceeds reserve floors or current restrictions;
- pump/filtration energy omitted from the cooling comparison;
- an adaptive controller uses future weather or information unavailable to the baseline;
- water feature receives better shade/ventilation than the no-water comparison without reporting that confounder.

Positive controls are equally important: a genuinely qualified low-water closed-loop or evaporative case must not be blocked simply because it contains water.

## 9. Proposed research hypotheses

- **H1 — thermal-water battery:** a covered pool/cistern with a qualified heat exchanger and a night-time heat-rejection path may deliver useful load shifting with much lower evaporative loss than an exposed water feature.
- **H2 — selective evaporation:** a small shaded recirculating surface can be useful during hot/dry/high-airflow windows, but will lose value during humid or water-constrained periods.
- **H3 — shade-water synergy:** shade plus high-mass surfaces and night flushing may outperform water alone; the incremental water contribution must be isolated by matched factorial experiments.
- **H4 — local-first resilience:** where water scarcity or maintenance access is severe, a no-water passive design may dominate any pool/water-feature intervention.
- **H5 — adaptive operation:** a water budget controller that shifts between cover, storage, evaporative assist and bypass can outperform fixed schedules, but only if it is causal, interpretable and governed.

## 10. Integration with the Luminous stack

- **Sol-Atlas:** solar access/shadow, building and landscape geometry, courtyard aspect ratio, pool/rill area and depth, wind exposure, roof catchment, paths, safety/access clearances, component/BOM/revision provenance.
- **Symtropy:** weather/event clock, paired control traces, thermal/water trajectories, metered outcomes, counterfactual evaluation and stable recovery under outages/drought.
- **Mycelix:** source rights and consent, household water floors, allocation commitments/appeals, instrument provenance/calibration/admission, water-account conservation and restrictions.
- **Specialist engines:** EnergyPlus/building energy; qualified airflow/CFD; hydraulic and heat-exchanger models; water-quality and public-health qualification. Benchmark wrappers must not impersonate their physics.
- **Highveld Commons:** compare shared versus household-local thermal/water services without substituting pooled assets for household essential-service guarantees.

## 11. Research sources and evidence boundary

- Johannesburg Water's current Level 1 restrictions, 1 Sep 2026–31 Mar 2027: [official notice](https://www.johannesburgwater.co.za/blog/2026/09/15/level-1-water-restrictions-are-here-lets-make-every-drop-count/).
- City of Johannesburg's 2008 efficient-building guideline: [PDF](https://www.uj.ac.za/wp-content/uploads/2022/11/coj-design-guidelines-for-energy-efficient-buildings-in-johannesburg-2008.pdf). It supports passive/mixed-mode, high mass, shade and night cooling; it is not a current code substitute.
- Johannesburg ERA5 1991–2020 monthly climate summary: [modelled climate reference](https://weathermatics.earth/weather/za/gauteng/johannesburg-993800/climate/). Not a site weather station/EPW.
- eThekwini Municipality water-conservation guidance (2010): [PDF](https://environment.durban.gov.za/uploads/0000/6/2025/09/23/water-conservation-guideline.pdf). Useful transferable water-saving design advice, not Johannesburg regulation.
- US EPA WaterSense pool efficiency: [pool water efficiency](https://www.epa.gov/watersense/pool-water-efficiency) and [technical sheet](https://www.epa.gov/system/files/documents/2023-08/ws-homes-TRM-20-PoolsandOutdoorWaterFeaturesTechSheet.pdf). Its quantified savings are guidance from other climates, not a Johannesburg guarantee.
- Water-feature integration in a 2025 World Bank urban-heat handbook: [Urban Heat Management Handbook](https://documents1.worldbank.org/curated/en/099090225111530574/pdf/P170027-8734c3d2-9b13-4cd7-9ea9-7cea8dab1706.pdf). Contextual design guidance, not proof for this site.
- Water-based envelope cooling review, 15 Nov 2025: [Journal of Building Engineering, DOI 10.1016/j.jobe.2025.114296](https://www.sciencedirect.com/science/article/pii/S2352710225025331).
- Courtyard evaporative-cooling CFD study (2024): [Building and Environment, DOI 10.1016/j.buildenv.2024.111588](https://www.sciencedirect.com/science/article/pii/S036013232400430X).

All thermal values and archetypes in this document are hypotheses or first-order physical relationships. No local pool evaporation rate, building cooling reduction, water saving, payback, hygiene compliance or resilience benefit has been physically measured for this programme.
