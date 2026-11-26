# Sol Atlas

**An interactive planetary visualization of energy, infrastructure, climate, and related systems, built with Rust, Leptos, WebAssembly, and WebGL2.**

Sol Atlas is a browser-based visualization application. It combines a WebGL globe renderer, timeline and layer controls, contextual panels, and typed data structures shared through \`sol-atlas-core\`.

## What the application contains

- **Interactive globe:** custom WebGL2 rendering, camera controls, geometry, picking, shaders, and texture support.
- **Layers and panels:** energy sites, infrastructure, corridors, vaults, regions, climate projects, fossil deposits, nuclear sites, and other bundled datasets.
- **Timeline and scenario controls:** explore modeled changes and illustrative future scenarios.
- **Reactor twin demonstration:** a UI surface for an experimental reactor-twin visualization.
- **Shared data model:** \`sol-atlas-core\` contains shared types and geographic/data helpers.

## Data and interpretation

The default frontend loads JSON files bundled into the application at build time (via \`include_str!\` in \`src/data/static_data.rs\`). It is therefore **not a continuously refreshed live-data service**. Dataset labels and source references are carried by repository assets and the demo documentation; users should inspect those files for provenance, date, coverage, units, and licensing before using them for analysis.

Timeline changes and future-state narratives are visualized scenarios, not validated forecasts. Modeled EROI, emissions, grid stress, infrastructure plans, and similar values must not be interpreted as authoritative operational measurements or investment, energy-policy, or safety advice without independent source verification.

The source tree also contains feature-gated Holochain call wrappers under \`src/data/holochain.rs\`. The current root manifest does not declare the corresponding \`holochain\` feature or its sibling integration dependency, so this should be treated as an integration surface under development rather than a supported enabled data path in the default application.

## Architecture

\`\`\`text
src/
  app.rs             Leptos routes and application composition
  components/        Globe canvas, HUD, panels, layers, timeline, tooltips
  data/              Static JSON loading, domain types, Holochain adapters
  renderer/          WebGL2 globe renderer, geometry, shaders, picking
  state/             Globe and data state
  vitality/          Application vitality and status presentation

sol-atlas-core/      Shared Rust types and geographic/data helpers
assets/data/         Bundled JSON data assets
\`\`\`

## Run locally

Requirements: Rust, the \`wasm32-unknown-unknown\` target, and Trunk.

\`\`\`sh
rustup target add wasm32-unknown-unknown
cargo install trunk

# Development server (Trunk.toml configures port 8091)
trunk serve

# Optimized WebAssembly build
trunk build --release
\`\`\`

The deployment workflow and hosted demo can change independently of the local build. Verify the Actions status and deployed revision before relying on a hosted instance.

## Related material

- [Demo walkthrough and scenario descriptions](DEMO.md)
- [Source code](src/)
- [Shared core crate](sol-atlas-core/)
- [Bundled datasets](assets/data/)

## License

The application source and shared core declare AGPL-3.0-or-later. Check file headers and individual asset metadata for applicable licensing and redistribution restrictions.
