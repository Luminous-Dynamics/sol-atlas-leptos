# Johannesburg weather package acquisition and qualification v0.1

**Study:** HIGHVELD-PASSIVE-COOLING-WATER-001  
**Maturity:** WEATHER_SOURCE_REGISTER_ONLY_NOT_ADMITTED  
**Purpose:** build a rights-cleared, spatially defensible weather package for the water/thermal benchmark without confusing a grid product, typical year or distant airport with the courtyard's observed microclimate.

## 1. Why this is a first-order physics input

The thermal sink cascade depends on more than dry-bulb temperature. Radiative regeneration needs atmospheric long-wave conditions, cloud and sky view; dry coolers need air temperature, humidity and airflow; evaporative regeneration needs wet-bulb depression, wind and humidity; night flushing requires safe outdoor enthalpy and air quality. A source with temperature only cannot qualify all these paths.

One typical meteorological year (TMY) is useful for repeatable design comparison but is not enough for long-horizon resilience, extreme heat, hot nights, dry spells or future climate. A coarse reanalysis/grid cell is useful for context and gaps but is not local ground truth. The benchmark therefore needs a source ensemble and an explicit validation hierarchy.

## 2. Candidate source register (not yet acquired)

The associated JSON register describes these candidates and their intended roles:

| Candidate | Type / useful fields | Access and limitation |
|---|---|---|
| South African Weather Service (SAWS) | Quality-controlled station archive; hourly wind direction/speed, temperature, humidity, pressure and sunshine are described as available from 1950 onwards, with station-dependent coverage | Order specific station/time-step data; commercial-use fees may apply; acquired data is not for resale and must follow SAWS data policy |
| Agricultural Research Council (ARC) Agrometeorology | Station data portal advertises historical hourly through annual products; AWS and manual stations; rainfall, temperature, wind, humidity/ET0 where station instrumentation supports it | Ad-hoc data reports/subscription; record station and sensor history; archive coverage and current active status need confirmation per station |
| Department of Water and Sanitation (DWS) Hydrological Services | Daily rainfall and pan evaporation observations | WRO describes these as free; not a complete building-energy weather record and not a substitute for solar, humidity, wind or sky-radiation measurements |
| Water Research Observatory (WRO) catalog | Discovery/metadata across more than 50 sources; SAAACA catchment climate daily data for 1950–1999 hosted on WRO | WRO-hosted records are tagged by access; catchment grids are not site stations; older SAAACA is climatology context, not validation for modern hourly operation |
| NOAA GHCN-Daily / Integrated Surface Database | Independent global station archives useful for cross-checking available variables | Check station, temporal coverage, missingness, instrument/siting changes and dataset-specific terms; daily GHCN cannot validate hourly transient response |
| CHIRPS / satellite precipitation | Spatial rainfall context and gap comparison | Rainfall product only; not air temperature, wind, humidity or sky long-wave truth |
| NASA POWER / MERRA-2 / ERA5-family reanalysis | Gridded historical and derived meteorological context | Grid cells average heterogeneous terrain and urban form; use for candidate years and gap filling only after local station checks. Never relabel a grid cell as a measured courtyard station |
| EPW/TMY file | Standardized building simulation input assembled from station/reanalysis sources | A model input product, not a raw observation source; keep station, year, gap-fill, typical-year method and transform provenance |

The Water Research Observatory says it indexes over 50 sources, lists SAWS/ARC measured networks as usually fee-based, describes selected WRO products as free, and recommends comparison with nearby gauges because gridded products are not uniformly validated across South Africa. SAWS describes hourly archive variables from 1950 onwards and restricts resale; ARC describes paid ad-hoc/subscription climate information from a national station archive. The current details must be reconfirmed at the date of acquisition.

## 3. Acquisition sequence

1. Define the project footprint at neighbourhood / site scale without assuming a street-level address. Freeze the intended building/courtyard coordinates and elevation only after the owner selects a study site.
2. Use the WRO catalogue to discover all candidate stations/grids by variable, temporal resolution, period and cost. Save the search query, date, catalogue revision and results digest.
3. Identify nearby SAWS and ARC stations with temperature, RH, wind, solar/sunshine and pressure coverage. Include multiple stations where possible to distinguish urban context from elevation / peri-urban siting effects.
4. Request commercial-use terms, fees, permitted derived outputs, retention and redistribution constraints from SAWS/ARC before acquisition. Obtain written permission/licence receipts and assign them to Mycelix instrument/source admission.
5. Add DWS daily rainfall/pan evaporation and relevant free WRO catchment products as independent contextual checks, not substitutes for hourly observations.
6. Acquire a TMY/EPW, a recent actual meteorological year, a predeclared hot-tail year and multi-day heat-wave/hot-night windows. Preserve raw provider files alongside normalized copies where the licence permits.
7. Compare overlapping variables across station, reanalysis and grid products using temporal alignment, bias, RMSE/MAE, quantile/extreme error, missingness, breakpoints, elevation/siting notes and seasonal stratification. Do not select the best matching station after looking at candidate cooling outcomes.
8. Install local reference instruments at the paired courtyard cell. The source package remains a boundary condition; it cannot stand in for courtyard wind, shade, cloud, sky-view, long-wave or water-surface measurements.
9. Freeze the source ensemble, source rights, spatial representativeness decision and uncertainty budget before model calibration. Keep held-out periods for VVUQ.

## 4. Required weather variables

### Core building / thermal boundary
- dry-bulb air temperature;
- relative humidity or dew point;
- barometric pressure;
- wind speed and direction at documented exposure/height;
- global horizontal solar irradiance and, where possible, diffuse/direct components;
- precipitation and rainfall capture input.

### Radiative-sink boundary
- cloud amount/type or measured long-wave downwelling radiation;
- sky-view factor and horizon obstruction from site geometry;
- sky-facing radiator surface temperature and up/down long-wave flux where installed;
- wind and humidity at the radiator exposure.

### Evaporative-sink boundary
- dry-bulb, RH/dew point and calculated wet-bulb method/version;
- wind/airflow and inlet/outlet state;
- water temperature, make-up, evaporation estimate and source class;
- aerosol/drift controls and humidity/condensation limits.

A weather package lacking long-wave radiation or cloud information can still support a declared subset of tests, but cannot silently qualify radiative-sink output. A temperature-only source cannot qualify evaporative or dry-air performance on its own.

## 5. Input package integrity and source admission

Every acquired dataset must carry:
- provider/source identifier, dataset and station ID (or grid cell identifier), provider contact and acquisition date;
- coordinates, elevation, siting/exposure metadata and spatial representativeness statement;
- variable names, units, aggregation statistic, measurement height and sampling interval;
- timezone, timestamp convention, daylight-saving handling and calendar;
- instrument IDs/types, maintenance/siting/breakpoint records where supplied;
- raw file digest, licence/terms digest, source URL and acquisition receipt;
- quality-control flags, missingness, duplicates, outliers and transformations;
- gap-fill method with source lineage; no hidden imputation;
- uncertainty and holdout split IDs;
- claim ceiling / permitted use, especially commercial/derived-data/redistribution constraints.

A digest proves which bytes were used; it does not prove calibration, rights or representativeness. Source admission must therefore be separate from content integrity.

## 6. Independence and train/validation/test design

Do not randomly shuffle hourly data across adjacent timestamps and call it independent validation. Use blocked, seasonal and event-held-out splits:
- training/calibration periods;
- validation periods for model and controller selection;
- final untouched holdout periods, including hot-tail or water-restriction conditions where possible.

Where station overlap exists, use a spatial holdout station. Retain a second independent product family as a cross-check (for example, station observations against reanalysis), while recognizing that products may share assimilated observations and therefore are not automatically statistically independent.

For extreme design events, predeclare the event selection rule before running candidate systems. Report missingness, station relocation/instrument changes and data gaps. Do not reconstruct missing radiation, wind or humidity from a convenient other variable without explicit model and uncertainty.

## 7. Acceptance and rejection gates

- 100% of model/weather outputs bind a versioned source package and transformation digest.
- 0 use of an acquired source beyond its licence or permission.
- 0 grid/reanalysis products relabelled as in-situ site measurements.
- 0 hidden interpolation/imputation in critical heat-sink boundary variables.
- 0 future weather information exposed to a causal controller.
- 100% of gaps and QC flags retained in the evidence packet.
- No radiative, evaporative or dry-air sink is qualified if the relevant boundary variables or local measurement evidence are absent.
- Station/grid discrepancies and uncertainty are reported by variable, season and extremes.
- Calibration and holdout data remain disjoint by predeclared time/event/station blocks.

No single universal correlation, RMSE or station-distance cutoff is asserted. Select reference stations and acceptance margins based on local siting, elevation, instrumentation, overlap, intended use and propagated thermal-model sensitivity.

## 8. Owners and evidence chain

- **Sol-Atlas:** site footprint, coordinates/elevation owner approval, sky-view/horizon/shading, courtyard geometry and weather-file/model revision binding.
- **Mycelix:** provider licence/rights, source and instrument admission, use restrictions, retention and permitted derived outputs.
- **Symtropy:** native acquisition/processing time, dataset lineage, matched causal traces, missingness chronology and replay receipts.
- **Thermal specialists:** map the meteorological inputs to appropriate boundary conditions and propagate uncertainty.
- **Independent reviewer:** station representativeness and train/validation/holdout separation.

The manifest beside this document is a source-acquisition registry. It contains no actual station selection or weather observations. It must remain not admitted until licences, actual station IDs, source files, digests, quality review and spatial representativeness are recorded.

## 9. References

- SAWS climate service archive and data-use policy: https://testing.weathersa.co.za/home/aboutclimateatsaws
- WRO South African climate/weather dataset finder and validation advice: https://www.waterresearchobservatory.org/data-and-resources/climate-and-weather-data
- ARC Agrometeorology data portal: https://www.agroclimate.agric.za/WP/WP/About.php
- WRO discover-data catalogue: https://www.waterresearchobservatory.org/discover-data
- Radiative cooling multi-scale review (Nov 2026): https://www.sciencedirect.com/science/article/pii/S2213138826006764

**Status:** \`WEATHER_SOURCE_REGISTER_ONLY_NOT_ADMITTED\`. No SAWS/ARC licence, station data, weather file, local sensor series or weather-model validation has been acquired by this artifact.
