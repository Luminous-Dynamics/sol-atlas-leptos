# Sol Atlas

**An interactive planetary visualization for energy and infrastructure systems.**

Sol Atlas is a Rust and WebAssembly application built with Leptos and WebGL2. It presents a navigable planetary globe with energy and infrastructure layers, timeline-based scenario exploration, and an experimental reactor digital-twin route.

- **Live application:** [atlas.luminousdynamics.io](https://atlas.luminousdynamics.io)
- **Demonstration guide:** [DEMO.md](DEMO.md)
- **Short demo recording:** [demo.mp4](demo.mp4)
- **Application source:** src/
- **Shared domain and geospatial types:** sol-atlas-core/

> **Status and data boundary:** Sol Atlas is an active visualization and research application, not a production utility-control system or authoritative live geospatial data service. The default application loads bundled static datasets. Selected Holochain-backed fetches are optional and feature-gated. Timeline projections, scenario stress indicators, planned infrastructure, and animated flows must not be interpreted as live measurements, transactions, or forecasts unless the active data source, timestamp, provenance, and verification evidence are explicitly shown.

## What is in the application

The current application includes:

- A WebGL2 planetary globe with interactive navigation.
- Selectable energy and infrastructure visualization layers.
- A timeline and scenario-oriented views.
- An experimental /labs/reactor-twin route.
- A shared Rust core for domain types and geospatial calculations.
- An optional, feature-gated Holochain integration for selected datasets.

The demonstration guide describes the bundled layers, scenario assumptions, and visualization concepts in more detail. Its counts and numerical examples describe the demo's documented dataset and assumptions; they are not a substitute for a dated, independently validated operational dataset.

## Data semantics and responsible use

The current application initializes from local static data so the visualization has a deterministic fallback. When the Holochain feature is enabled, the application attempts to fetch selected records from a configured Holochain environment and replaces corresponding static collections when non-empty results are returned.

For analytical or operational use, each layer should expose:

1. **Source and provenance** — originating organization, dataset, license, and retrieval method.
2. **Time semantics** — observation time, publication time, retrieval time, and effective period.
3. **Status** — observed, estimated, modeled, planned, synthetic, or simulated.
4. **Uncertainty and coverage** — confidence, missingness, spatial/temporal resolution, and known limitations.
5. **Reproducibility** — dataset revision or content digest, transformation version, and scenario parameters.

Do not use the bundled demonstration data to make grid-dispatch, investment, safety, regulatory, or infrastructure-planning decisions without independently validating the underlying sources and models.

## Technology

| Layer | Implementation |
|---|---|
| Language | Rust (edition 2024) |
| UI | Leptos 0.8 |
| Rendering | WebGL2 |
| Browser target | WebAssembly (wasm32-unknown-unknown) |
| Domain core | sol-atlas-core |
| Development server and build | Trunk |

## Build and run locally

Prerequisites:

- A current stable Rust toolchain.
- The wasm32-unknown-unknown target.
- Trunk.

Install the target and Trunk if needed:

~~~bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
~~~

Run the development server:

~~~bash
trunk serve
~~~

The repository's Trunk.toml configures the development server on port 8091 and uses index.html as the build target.

Useful checks matching the repository workflow:

~~~bash
cargo fmt --all --check
cargo check --target wasm32-unknown-unknown
cargo test --bin sol-atlas-leptos
~~~

Build the release site:

~~~bash
trunk build --release --no-sri
~~~

The GitHub Pages workflow builds the release site and copies index.html to 404.html for client-side route fallback. A successful local build is not a validation of dataset quality or operational suitability.

## Project structure

~~~text
.
├── src/
│   ├── app.rs             # Routes and application composition
│   ├── components/        # Globe, layers, timeline, panels, demos
│   ├── data/              # Bundled data and optional data adapters
│   ├── renderer/          # WebGL2 rendering
│   ├── state/             # Application and globe state
│   └── vitality/          # Browser/device responsiveness
├── sol-atlas-core/        # Shared Rust domain and geospatial logic
├── assets/                # Static assets
├── docs/                  # Research and industry notes
├── DEMO.md                # Demonstration layers and scenario notes
├── index.html             # Browser entry point and page metadata
└── Trunk.toml             # Trunk build and development-server settings
~~~

## Relationship to the Luminous Dynamics portfolio

Sol Atlas is the spatial visualization and scenario-exploration project within [Luminous Dynamics](https://github.com/Luminous-Dynamics).

Potential integrations with [Symthaea](https://github.com/Luminous-Dynamics/symthaea), [Symtropy](https://github.com/Luminous-Dynamics/symtropy), and [Mycelix](https://github.com/Luminous-Dynamics/mycelix) are developed where they provide tested value. The existence of related projects does not imply that every integration is enabled or available in the current application.

## Contributing and licensing

Small, reproducible contributions are welcome. Changes to data should include provenance, licenses, units, timestamps, and tests for parsing and transformation behavior. Changes to scenario calculations should document assumptions and include reference cases or sensitivity tests.

Source files carry AGPL-3.0-or-later SPDX notices; the shared core package also declares that license in its Cargo metadata. Review the applicable file and dependency licenses before redistribution or commercial integration.
