# Pool and water-feature cooling scale-transferability contract v0.1

**Study ID:** HIGHVELD-POOL-SCALE-TRANSFER-001  
**Parent:** HIGHVELD-POOL-WATER-THERMAL-001 / HIGHVELD-POOL-MICROCLIMATE-ATTRIBUTION-001  
**Maturity:** RESEARCH_PROTOCOL_NOT_EXECUTED  
**Climate context:** Johannesburg / Highveld; no local scale-transfer data admitted

## 1. Governing rule

A result at one physical or decision scale is evidence only at that scale. A cooler water surface does not prove a cooler courtyard; a cooler courtyard average does not prove lower pedestrian heat stress; a lower pedestrian temperature does not prove indoor cooling; a park cooling effect does not prove that residents have accessible, safe refuge service.

This contract makes the claim hierarchy explicit and prevents a metric from silently crossing a scale boundary.

## 2. Five evidence tiers

| Tier | What is being measured | Examples of sufficient evidence |
|---|---|---|
| COMPONENT_POOL | Water/material/storage state and physical balance | Layer-resolved temperature/enthalpy, volume, heat/water balances, exergy reference state, heat-sink path, usable headroom |
| COURTYARD_PEDESTRIAN | Local outdoor microclimate at occupied heights | Matched dry-shade control, upwind boundary, feature edge/downwind transect, air temperature, humidity, wind, MRT/globe, height and geometry |
| BUILDING_OCCUPIED_ZONE | Indoor thermal service and cooling demand | Measured/qualified building-side heat path, zone operative conditions, occupancy/schedule and cooling electricity |
| NEIGHBORHOOD_BLUE_SPACE | Spatially distributed urban effect and morphology | Representative distributed stations, urban form/sky-view/wind, activity-space exposure, spatial decay and uncertainty |
| THERMAL_SERVICE_REFUGE | Whether a person/group can actually obtain thermal service | Qualified refuge operating state, authority-defined safety envelope, accessible routes, time-specific capacity, water and outage dependencies |

## 3. Promotion rules

- COMPONENT_POOL → COURTYARD_PEDESTRIAN requires independently measured outdoor boundary/occupied-height response and a matched dry-shade control.
- COURTYARD_PEDESTRIAN → BUILDING_OCCUPIED_ZONE requires an independently qualified heat-transfer path to the building zone and indoor occupied-zone evidence. A water-surface or outdoor air-temperature change is insufficient.
- COURTYARD_PEDESTRIAN → NEIGHBORHOOD_BLUE_SPACE requires distributed spatial sampling and representative morphology/meteorology; a single courtyard or park-average point cannot establish neighborhood effects.
- NEIGHBORHOOD_BLUE_SPACE → THERMAL_SERVICE_REFUGE requires separate evidence of actual refuge operations, capacity, accessible reachability and authority-approved safety/service criteria. A cooling map is not refuge service.

There is no automatic transitive promotion. Passing a lower tier does not remove the evidence requirements of the next tier. The tier of every public claim must be stated.

## 4. Literature transferability controls

### 4.1 Urban water-body area threshold

A February 2026 dense-urban water-body study reports a 3.41 ha threshold for isolated water bodies under its study conditions and analyzes area, aggregation and surrounding morphology. The threshold is a study-specific empirical/model finding, not a universal physical constant. It is **not** a residential pool, backyard rill, courtyard or Johannesburg design threshold. Do not scale it down by area ratio or use it as an acceptance gate.

The study is useful as evidence that water-body cooling is scale- and morphology-dependent. The Highveld benchmark must test its own feature scale with its own measured boundary conditions.

### 4.2 Park-average cooling vs activity-space heat stress

A September 2026 field/model study of Mochou Lake Park in Nanjing reports that the park interior averaged 1.30 °C cooler than surrounding built-up areas during its study window, while each sampled internal point still showed at least strong UTCI heat stress; air temperature also differed by up to 2.22 °C among activity-space types. Its field campaign used one hot-summer observation day (28 August 2025) with hourly observations at selected points and an ENVI-met model validated against those observations.

The correct inference is not that a park or water body guarantees comfort. It is that park averages can hide persistent activity-space heat stress, and that air temperature alone is insufficient to characterize pedestrian thermal exposure. These Nanjing values and model-derived canopy references are context only, not Highveld performance targets.

### 4.3 Water channels and urban geometry

An August 2026 traditional water-channel study reports that wind, canyon geometry and channel configuration alter the local cooling response; some arrangements can increase daytime long-wave heat burden. The design space must therefore allow water features to perform neutrally or negatively, and must measure long-wave/radiant conditions rather than assume every added water surface improves comfort.

References:
- [Dense urban water-body cooling study, February 2026](https://www.sciencedirect.com/science/article/pii/S2210670726000363)
- [From Park-Wide Cooling to Heat Stress Across Activity Spaces, 6 September 2026](https://www.mdpi.com/2071-1050/18/17/9141)
- [Traditional water-channel cooling and canyon configuration study, August 2026](https://www.sciencedirect.com/science/article/pii/S2210670726004038)

## 5. Scale-aware descriptors

Register these descriptors where relevant; they are explanatory covariates, not universal thresholds:
- exposed water surface area / water volume;
- water surface area / courtyard plan area;
- water depth / characteristic feature length;
- sensor distance / characteristic feature length;
- sky-view factor and horizon obstruction;
- canyon height / width and local roughness/shelter;
- wind-to-buoyancy regime (e.g. Richardson number, only when its assumptions and required variables are available);
- evaporative latent heat / measured total heat input;
- evaporated volume / discretionary water budget;
- usable thermal headroom / representative heat-load rate (store autonomy time);
- spatial decay of measured pedestrian conditions away from the feature.

The descriptors must preserve their unit, derivation, uncertainty and applicability domain. Missing descriptors remain unknown; they are not inferred from an area ratio.

## 6. Required tests and hard claim gates

- No 3.41 ha study threshold transferred as a residential pool/courtyard rule.
- No land-surface-temperature-only evidence promoted to pedestrian comfort.
- No dry-bulb-only evidence promoted to MRT/UTCI/PET or heat-health safety.
- No park/courtyard mean used to hide hot activity spaces or downwind exposure.
- No courtyard evidence promoted to building cooling without indoor measurements and a qualified heat path.
- No neighborhood cooling map promoted to refuge availability without route/capacity/operating evidence.
- Every treatment has matched dry-shade and upwind controls.
- Report negative results where added water has no benefit or worsens humidity/radiant exposure.
- Use event/day-block holdouts; minute readings are not independent replications.
- 100% of promoted claims bind a scale-tier ID, source evidence, model/measurement revision and applicability domain.

## 7. Status and claim ceiling

**Status:** RESEARCH_PROTOCOL_NOT_EXECUTED. The literature cases are transferability controls only. No pool/courtyard/park/neighborhood field measurement, area threshold, or cooling/safety result is admitted for Johannesburg. No universal cooling coefficient is claimed.
