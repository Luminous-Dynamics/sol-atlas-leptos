# Sol Atlas — Planetary Energy and Infrastructure Demonstrator

**Live application:** [atlas.luminousdynamics.io](https://atlas.luminousdynamics.io)  
**Application README:** [README.md](README.md)  
**Demo recording:** [demo.mp4](demo.mp4)

This guide describes the visualization layers and scenario concepts currently documented for Sol Atlas (the project was previously described as Terra Atlas in older material).

> **Interpretation boundary:** This is a visualization and research demonstrator, not a live global infrastructure feed, a calibrated energy-system forecast, or a utility operations system. The default app path loads bundled static data. Selected Holochain fetches are optional and feature-gated. Counts below describe the documented demonstration dataset, not a complete or current inventory of global assets. Planned infrastructure, future timelines, stress indicators, and animated exchanges are scenarios or visualizations unless the app explicitly presents a verified live source, its timestamp, and a provenance record.

## What the demonstration shows

Sol Atlas presents a navigable 3D globe with energy and infrastructure layers and scenario-oriented controls. The demonstration explores three ways of looking at energy assets:

### 1. Resource and asset context

The documented bundled scenario includes fossil-fuel deposit records positioned geographically, with marker size intended to represent reserve scale. The guide lists EIA, USGS, and BP publications as source families for parts of the dataset.

Before using any individual value analytically, verify the underlying source document, publication date, units, geographic definition, and transformation applied to create the bundled record.

### 2. Energy-return scenario bands

The demonstration assigns visualization bands to Energy Return on Investment (EROI) values:

- Green: above 12:1
- Amber: 5:1–12:1
- Red: 3:1–5:1
- Dark red: below 3:1

These are **scenario visualization categories**, not universal engineering, economic, or civilizational viability thresholds. EROI depends on system boundaries, energy quality, lifecycle assumptions, co-products, and the study methodology. Comparisons require a consistent and cited methodology.

### 3. Carbon-cost scenario

The demo includes carbon-cost calculations based on fuel-specific production and emission-factor assumptions. An EPA 2024 social-cost-of-carbon estimate is referenced in the historical scenario notes.

Treat that figure as a dated scenario input, not a current universal carbon price or a substitute for jurisdiction-specific policy, lifecycle analysis, or a current social-cost estimate. Revalidate source documents and assumptions before reusing the calculation.

## Documented data layers

The historical demonstration notes describe the following dataset entries:

| Layer | Documented entries | Notes |
|---|---:|---|
| Energy sites | 70 | EIA-related and clustered records |
| Geothermal nodes | 18 | Global survey-derived demonstration layer |
| Maglev corridors | 15 | Planned-route visualization |
| Terra Lumina sites | 12 | Conceptual renewable-site layer |
| Resontia vaults | 12 | Conceptual underground-infrastructure layer |
| Fossil deposits | 73 | EIA, USGS, and BP source families cited |
| Nuclear / SMR | 50 | Includes planned pipeline concepts |

These counts are the counts stated in the demonstration guide. They should be checked against the current bundled dataset before publication or analysis. A planned or conceptual layer must be labeled as such in the interface.

## Timeline and scenario narrative

The timeline explores a long-horizon transition scenario. The Year 0–300 sequence in the visualization is an illustrative model narrative, not a prediction.

Its assumptions include reserve depletion, changing EROI bands, renewable build-out, corridor construction, and the introduction of conceptual infrastructure. The scenario is useful for exploring relationships and communicating assumptions; it is not validated as a forecast of specific assets, markets, or dates.

For decision support, publish the scenario parameters, equations, calibration evidence, sensitivity ranges, and alternative assumptions alongside results.

## Grid-stress visualization

The demo describes a stress indicator for 16 population centers, motivated by Symthaea's Free Energy Principle vocabulary.

This is a **modeled scenario indicator**, not telemetry from grid operators and not an independently validated measurement of regional grid fragility. A real infrastructure deployment would need documented input data, clear units, calibration against observed events, uncertainty bounds, and independent validation.

## Peer-to-peer energy exchange visualization

Animated arcs in the demo represent peer-to-peer energy-exchange concepts. Do not interpret an arc as a real trade, settlement, or power flow merely because it is animated.

A live transaction claim requires an explicitly connected exchange implementation and inspectable evidence such as source and destination identities, transaction identifiers, timestamps, quantities, authorization, and settlement status. Otherwise label the visualization simulated.

## Technical architecture

The current web application is organized around:

~~~text
sol-atlas-core (Rust)
    Shared domain types, geospatial logic, and constants

sol-atlas-leptos (Rust / Leptos / WebAssembly)
    Browser application, routes, state, and WebGL2 globe renderer

Optional Holochain data adapter
    Feature-gated fetches for selected datasets when configured
~~~

The app loads bundled static data as its default fallback. Optional Holochain integration is compiled behind a feature flag; its presence in the source does not mean a live conductor is configured or that every layer is dynamically sourced.

## Experimental reactor digital-twin route

The application includes an experimental /labs/reactor-twin route. Treat this as a research demonstration. It is not a validated reactor design tool, a safety case, or a substitute for qualified engineering simulation and review.

## Economics calculations

The demonstration describes calculations for individual deposits, including:

- Annual revenue and extraction-cost assumptions.
- Carbon-cost scenarios.
- Break-even carbon-price calculations.
- Internal-rate-of-return estimation.
- Years-to-depletion estimates.

Every reported output depends on source data, units, boundaries, and assumptions. For analytical use, publish the input dataset revision, calculation version, parameter file, sensitivity analysis, and validation cases with the output.

## What is needed before operational use

Before Sol Atlas is used for planning, investment, or infrastructure operations, each relevant layer should have:

1. A traceable source and permission to reuse it.
2. A retrieval timestamp and source publication/effective date.
3. Explicit observed / estimated / modeled / planned / simulated status.
4. Units, coordinate reference assumptions, and transformation history.
5. Missing-data, uncertainty, and sensitivity reporting.
6. Validation against independent reference data or operational outcomes.
7. A reproducible export of the inputs, parameters, code revision, and result.

Sol Atlas is intended to make complex systems easier to explore. Trustworthy decisions require more than an attractive globe: they require transparent data, explicit assumptions, and independently checkable evidence.

---

*Luminous Dynamics — truth, dignity, flourishing, and sovereignty.*
