# Regenerative Nutrient Systems — Research and Engineering Plan

Status: **design and accounting-core stage** (2026-10-10)

This proposal extends Sol Atlas from a planetary infrastructure viewer toward an evidence-grounded nutrient-cycle planning layer. It does not claim to solve fertilizer scarcity, predict yields, certify amendments, or provide farm application recommendations. The first implementation is the pure-Rust accounting module at sol-atlas-core/src/nutrient.rs.

## Why work on this now?

FAO warned on 7 May 2026 that fertilizer scarcity associated with disruptions to the Strait of Hormuz could affect yields and food supplies in the second half of 2026 and into 2027. Its warning highlights a crucial systems detail: fertilizer has to arrive during specific crop-calendar windows; later supply may not recover missed application opportunities. IFPRI's 18 September 2026 market review adds nuance: nitrogen prices had eased from earlier peaks, but seasonal demand and continued disruptions could trigger fresh rises, while fertilizer supplies remained constrained in some trade flows. This should be modelled as a changing, region-specific risk scenario—not a claim that every nutrient is always scarce everywhere.

Sources:
- FAO, 7 May 2026: https://www.fao.org/newsroom/detail/strait-of-hormuz-crisis--fertilizer-scarcity-will-affect-next-harvests-and-food-supplies--fao-warns/
- IFPRI, 18 September 2026: https://www.ifpri.org/blog/how-are-fertilizer-markets-coping-with-the-continued-closure-of-the-strait-of-hormuz/

## Scientific guardrails: biochar and terra preta

Terra preta de Índio is a diverse family of anthropogenic Amazonian dark-earth soils and long-lived soil systems, not a single fertilizer recipe. Sol Atlas should use the term **terra preta-inspired** for modern amendments and practices unless a more specific evidence-based definition is supplied. We should not assert that modern biochar blends reproduce the historic soils or their timescales.

The evidence supports testing biochar, especially as part of context-specific nutrient-management systems; it does not support universal replacement of fertilizer:
- A 2024 systematic review of 92 studies (1,609 observations) on sandy-textured soils reported improvements in several nutrient-cycle and soil properties, but no average effect on soil mineral nitrogen or nutrient-use efficiency. Effects varied with soil pH, application rate and study context: https://doi.org/10.1186/s13750-024-00326-5
- UNEP says wastewater-derived nutrient recovery has a **theoretical potential** to offset around 13% of global agricultural fertilizer demand. Treat this as an upper-bound opportunity to investigate—not current production, immediately recoverable product, or an even distribution across farms: https://www.unep.org/facts-about-wastewater-and-nutrient-management
- The International Biochar Initiative describes quality assurance as necessary for safe use and points producers toward the World Biochar Certificate / Carbon Standards International pathway: https://biochar-international.org/biochar-standards/

The system must account for nitrogen, phosphorus and potassium separately. Composting and biochar do not create elemental P or K. Biological nitrogen fixation can add biologically available nitrogen to an agricultural system, but rates and net benefits depend on crop, ecology, management and losses. Every proposed intervention therefore needs nutrient balances, safety checks, and local field evidence.

## Architecture and responsibilities

### Sol Atlas — geographic discovery and scenario visibility

Add future nutrient-system layers for:
- crop areas, season calendars and regional demand;
- fertilizer production, trade, import exposure, prices and transport chokepoints;
- soil properties and their prediction uncertainty;
- organic-residue sources, wastewater treatment/nutrient recovery, composters and biochar producers;
- field-trial locations, measured results, and amendment quality records.

A marker is not evidence by itself. Each dataset should carry source, license, retrieval date, valid period, spatial resolution, units, method, uncertainty and a status such as **measured**, **estimated**, **scenario**, or **unknown**. Visual encodings must make those distinctions visible. Simulated sites and flows must never look like live or operating infrastructure.

### Symthaea — hypothesis generation and constrained decision support

Symthaea can help generate and compare scenarios, prioritize information gathering, and explore combinations of nutrient recovery, rotations, legumes, compost, biochar, water management, targeted mineral inputs and conventional fertilizer. It should propose *questions and candidate interventions*, not silently become the agronomic authority.

The numerical solver should be deterministic and separately testable. Hard constraints must include nutrient availability, crop/season fit, feedstock capacity, processing losses, transport, costs and amendment safety. Uncertainty and sensitivity must be exposed. An independent checker should validate balance arithmetic and constraint compliance; scenario outputs should never be represented as observed results.

### Mycelix — provenance and coordination between institutions

Potential records include amendment batches, laboratory results, feedstock chain-of-custody, offers and purchase commitments, deliveries, and field-trial observations. A signed/replicated record can help establish who asserted what and whether a record changed. It cannot by itself establish that the physical sample was representative, the sensor calibrated, the lab competent, or the material safe. Those facts need appropriate physical sampling, recognized test methods, independent audits where warranted, and accountable organizations.

Keep this layer behind explicit interfaces. Do not couple the core nutrient arithmetic to a particular Holochain version or make a distributed network a prerequisite for local offline use.

## First implementation: pure-Rust nutrient budget

sol-atlas-core/src/nutrient.rs currently provides:
- explicit site, crop, season and area context;
- N, elemental P, and elemental K values on a common kg/ha basis;
- per-value evidence class and mandatory source/method provenance for known values;
- unknown values that block deficit/surplus claims instead of being interpreted as zero;
- conversion helpers for fertilizer-label P₂O₅ and K₂O into elemental P and K;
- per-nutrient deficits/surpluses only where demand and all supply components are known;
- tests for arithmetic, unknown-data handling, provenance, non-finite/negative input, oxide conversions, overflow and serialization.

A **Complete** balance means only that the required numbers are present and arithmetically valid. It **does not** mean the estimate is accurate, the nutrients will all reach the crop, or the result is a safe application recommendation. Supply entries must represent the estimated amount available over the budget period, not simply total nutrient contained in a material. Avoid double-counting nutrients recycled through several steps.

## Data plan and caveats

Use these sources for discovery, then record immutable versioned ingests rather than relying on a single fragile online endpoint:

| Domain | Candidate source | How to use it and limitations |
|---|---|---|
| National fertilizer use / trade | [FAOSTAT Fertilizers by Nutrient](https://data.fao.org/catalog/dataset/4e50bf79-ddf5-41b1-8608-468eca2697e2) | National annual context; the catalog describes a 1961–2023 series, so verify the downloaded vintage and its latest year at ingest. The dataset reports P as P₂O₅ and K as K₂O, so normalize units, retain original values and respect its attribution/license. Annual national values cannot stand in for local seasonal availability. |
| Soil organic carbon | [FAO Global Soil Organic Carbon Map](https://www.fao.org/soils-portal/soil-survey/soil-maps-and-databases/global-soil-organic-carbon-map-gsocmap/en/) | Useful as a harmonized national/global baseline, not a crop-specific soil test or direct measurement of plant-available nutrients. CC BY source attribution applies. |
| Soil properties / uncertainty | [ISRIC SoilGrids](https://isric.org/explore/soilgrids) | Global 250 m predictions include uncertainty and multiple depths. As checked 10 October 2026, ISRIC reports the beta REST API is temporarily paused; do not build production ingestion on a live API assumption. Use official, licensed downloadable raster products/snapshots where available, pin checksums and versions, and keep model uncertainty. |
| Wastewater recovery | [UNEP nutrient recovery overview](https://www.unep.org/facts-about-wastewater-and-nutrient-management) | Use for opportunity sizing and policy context. Facility-level recoverable amounts, treatment capability, pathogen/chemical risks, and applicable approvals must be verified locally. |
| Biochar quality | [International Biochar Initiative standards page](https://biochar-international.org/biochar-standards/) | Use the current relevant standard/certification scheme and competent laboratory results; a feedstock name or a carbon percentage alone does not certify agricultural safety. |
| Crisis and trade scenarios | [FAO May 2026 warning](https://www.fao.org/newsroom/detail/strait-of-hormuz-crisis--fertilizer-scarcity-will-affect-next-harvests-and-food-supplies--fao-warns/) and [IFPRI September 2026 analysis](https://www.ifpri.org/blog/how-are-fertilizer-markets-coping-with-the-continued-closure-of-the-strait-of-hormuz/) | Use time-stamped alternative supply, price, route and timing scenarios. Do not extrapolate a single crisis snapshot into a permanent global forecast. |

Data collection must preserve original units and values alongside normalized values, record transformation code/version, retain licenses and attribution, and reject stale, conflicting or unit-ambiguous inputs. Modelled map layers should keep lower/central/upper estimates where available. Avoid interpolating a point soil test over large regions without stating the method and uncertainty.

## Pilot design and release gates

Start with **one crop, one bounded region and one growing season**, with access to qualified agronomy support, clean feedstocks, laboratory testing, participating farms and a design that does not unnecessarily put a harvest at risk.

1. Define a baseline for crop demand, soil test results, current fertilizer inputs, water and weather, yield, input prices, and local organic material flows.
2. Characterize every experimental amendment. Test nutrients on an appropriate basis, pH, salinity and relevant contaminants; sanitation-derived products also need suitable pathogen and chemical assessment. Unknown or unsafe material must be ineligible for recommendations.
3. Use pre-registered, replicated treatment comparisons and a conventional-practice reference. Keep a no-amendment control only where agronomically and ethically appropriate. Include multiple seasons where feasible.
4. Record yields and quality, nutrient use, farmer net return, application/labor/transport costs, soil change, water use and relevant nutrient losses. Report absolute values, uncertainty, missingness and negative results.
5. Compare predictions with observations on held-out data. Recalibrate only through versioned changes; do not overwrite adverse outcomes or weaken safety criteria to achieve a positive headline.

No region should be scaled based solely on a simulated forecast or one promising harvest. Expansion gates should require safety compliance, independently checked calculations, transparent uncertainty, acceptable farmer economics, and evidence that crop performance and environmental outcomes meet locally agreed criteria.

## Not in scope for this first increment

- No hard-coded global fertilizer shortage or universal biochar yield multiplier.
- No synthetic data represented as observed supply, soil measurements, or existing facilities.
- No automatic amendment dosing or fertilizer purchase decisions.
- No assumption that a ledger makes a physical product safe or a reported dataset true.
- No live SoilGrids REST dependency until the provider confirms stable availability.
- No requirement for network connectivity for local balance calculations.

The immediate objective is a small, auditable accounting primitive. Regional data ingestion, map visualization, constrained optimization, and distributed coordination should be separate subsequent increments, each with their own evidence and acceptance criteria.
