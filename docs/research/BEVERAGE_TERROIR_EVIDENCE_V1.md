# Beverage Terroir and Resource Evidence V1

Status: proposed spatial-data contract and synthetic controls. No crop suitability map, water-risk result, vineyard recommendation, ingredient-quality prediction or causal claim is qualified by this document.

Tracking issue: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/226  
Scientific benchmark: https://github.com/Luminous-Dynamics/symthaea/issues/8232  
Process twin: https://github.com/Luminous-Dynamics/symtropy/issues/2387  
Batch provenance: https://github.com/Luminous-Dynamics/mycelix/issues/5330

## Objective

Represent geographically grounded evidence relevant to beverage inputs: grapes and vineyards; barley, hops and brewing water; grains and botanicals for spirits; weather and climate history; soil/site properties; water availability and scarcity; energy/logistics context; and source-region resilience.

The paradigm shift is to make regional context a versioned, uncertainty-bearing input into beverage process planning—not a map-colored score that pretends to establish taste or terroir. Site conditions, crop response, ingredient chemistry, finished-product sensory outcomes, economics and legal origin designation are separate claim families.

## Required data contract

For every source/derived layer, preserve:
- source URI/identifier, publisher, dataset/version/edition, retrieval time and license/attribution terms;
- physical meaning, parameter name, units, evidence class, valid range and known method limitations;
- coordinate reference system, axis order, geometry/support type and spatial resolution or footprint;
- temporal coverage, time standard/time zone, cadence, aggregation, climatology period and forecast vs observation label;
- transformations/interpolations/resampling, code/version/digest and source artifact hashes;
- missingness, uncertainty, confidence limitations and coverage denominator;
- intended use, prohibited extrapolations and claim ceiling.

Never silently convert missing data to zero, ordinary conditions or evidence of absence. A coarse gridded climate estimate is not point weather at the root zone. A national water statistic is not the licensed abstraction, local aquifer state or plot-level irrigation budget.

## First research adapters

- NASA POWER: candidate point/grid meteorological and solar context. Honor its source product resolution, UTC/LST semantics, parameter units and request limitations; do not upsample its coarse grids and claim new detail.
- OIV viticulture water guidance: a domain reference for locally adapted vineyard water stewardship, not a universal irrigation schedule or a computed compliance rule.
- BIER beverage-facility benchmarks: downstream producer water/energy/emissions context, not field-level crop water demand.
- Local measured soil moisture, farm weather stations, irrigation meters, crop yield and grape chemistry should be higher-priority for plot-specific validation when access, calibration and rights are established.

For South African and other water-stressed regions, use regional water scarcity and seasonal variation as explicit dimensions rather than optimize average liters alone. Compare water consumption and water scarcity together, and report energy and crop/product quality trade-offs.

## Benchmark tasks and metrics

1. **Source integrity** — hash source bytes where locally captured; pin versions, rights and exact parameters.
2. **Spatial/temporal compatibility** — reject incompatible CRS, unit, date window, aggregation or geometry support; never imply false precision through resampling.
3. **Coverage** — report covered area/time and valid-observation fraction beside every result.
4. **Prediction** — evaluate yield, crop-stress or ingredient-property estimates only against suitable measured targets with farm/vintage/site holdouts.
5. **Decision quality** — evaluate water/energy allocation under explicit ecological and operational constraints with fixed candidate sets and a matched baseline.
6. **Transferability** — leave out sites, seasons, vineyards, water districts or source products when sample size supports it.
7. **Multi-objective comparison** — preserve yield stability, crop/ingredient quality proxies, water scarcity, irrigation volume, energy, logistics and ecological constraints as a vector; do not hide trade-offs in one weighted score.

The quality model must not infer wine taste or beer flavor directly from broad climate maps unless a separate dataset links the specific source, harvest, chemistry and blind sensory target, with held-out-region validation.

## Acceptance and negative controls

- CRS, units, source version, resolution and temporal coverage are required and validated before comparing layers.
- Missing or stale inputs make affected claims incomplete or indeterminate; they do not become zeros or normals.
- Grid-to-plot resampling records method and does not claim to improve source resolution.
- Extrapolation beyond the observed climate/soil/crop domain yields explicit out-of-domain status and uncertainty.
- Sensor values require instrument, calibration state, measurement timestamp and supported spatial footprint.
- All outputs include source digest, algorithm/model version and claim limits.
- No water-use optimization result can violate an explicitly supplied ecological, allocation or safety constraint.
- A crop/site ranking is not an origin-authenticity determination or a sensory-quality verdict.
- Rights unknown means ineligible for reuse; source attribution stays attached to derived outputs where required.

## Industry and stretch target

BIER's 2025 beverage-facility study, released in April 2026, reports 3% lower water-use ratio, 4% lower energy-use ratio and 11% lower emissions ratio across participating facilities' 2020–2024 reporting years; more than 60% were in locations with at least medium water risk. These are facility-sector trends, not plot-level agricultural baselines.

The proposed first regional research milestone is not a universal crop-performance percentage. It is a source-audited regional assessment in which every map/estimate is accompanied by native resolution, coverage, uncertainty and exact source lineage, and a held-out-site/season benchmark precedes actionable recommendations. A farm/producer pilot may later preregister a ≥10% normalized resource improvement target only where comparable metered baselines and non-inferior quality/yield constraints exist.

## Claim ceiling

These documents specify data contracts and benchmark gates. They do not establish crop yield, irrigation prescriptions, terroir, ingredient quality, water rights, geographical-indication compliance or causal environmental improvement. Field recommendations require local data, domain review, applicable legal constraints and prospective validation.

## Research anchors

- NASA POWER API source documentation: https://power.larc.nasa.gov/api/pages/
- NASA POWER daily API, native spatial resolution and UTC/local-solar-time details: https://power.larc.nasa.gov/docs/services/api/temporal/daily/
- OIV sustainable use of water in winegrape vineyards: https://oiv.int/index.php/the-sustainable-use-of-water-in-winegrape-vineyards
- OIV protocol for sustainable use of water in viticulture: https://www.oiv.int/node/3132
- BIER 2025 Water, Energy, and Emissions Benchmarking: https://www.bieroundtable.com/2025-water-energy-and-emissions-benchmarking-summary-bier
