# Sol Atlas

**An interactive planetary data globe and coordination-layer prototype, built with Rust, Leptos, WebAssembly and WebGL2.**

Sol Atlas brings energy, environmental, infrastructure, supply-chain and planning layers into one navigable 3D view of Earth. The current application is a visualization prototype: it helps people explore relationships among datasets and scenarios; it is not an authoritative utility registry, a continuously refreshed digital twin, or an operational decision system.

> **Project status:** The globe and its layered visualization are on the default branch. The more ambitious **Earth Observability** programme—sensor qualification, underground utility inference, evidence-qualified network reconstruction and investigation planning—is active research in draft pull requests. Those research capabilities are not yet part of the default-branch application and have not been field-qualified.

## What is in the application

- **Interactive 3D globe:** a WebGL2 renderer with selectable locations, layer controls, an information dossier, tooltips and navigation.
- **Provenance-aware layers:** the layer dock distinguishes source-based observations, curated/estimated content and speculative planning scenarios.
- **Confluence overlay:** highlights spatial overlap among observed/curated systems. It is a derived co-location signal, **not** a risk score or proof of causal interaction.
- **Planning timeline:** a 0–777-year scenario timeline that animates the planning layers. It is speculative world-building, not a forecast of actual infrastructure deployment or resource depletion.
- **Reactor twin demo:** a separate `/labs/reactor-twin` route that replays a recorded Symthaea FissionTwin simulation. It is not live reactor telemetry and is not connected to any operating plant.

## Data provenance and interpretation

The application bundles JSON datasets at build time. A source-labelled layer does not imply that its data are live, complete, independently verified or suitable for engineering use.

| Provenance class | Meaning in Sol Atlas | Examples in the current application |
|---|---|---|
| **Observed** | Directly sourced observations or source snapshots. The snapshot date is shown where available. | USGS earthquake feed; NASA FIRMS active-fire feed; NASA EONET event feed; energy-site source inventory |
| **Curated** | Hand-selected, aggregated, estimated, schematic or illustrative real-world content. | Regional aggregates, supply-chain flows, climate projects, fossil-deposit EROI estimates, nuclear-plant and infrastructure layers |
| **Scenario** | Speculative planning content; it does not describe an existing asset unless independently established elsewhere. | Geothermal nodes, maglev corridors, Resontia vaults, Terra Lumina sites and robotics dispatch |

Examples of dated source snapshots encoded in the current layer metadata include the energy-site inventory (2025-11), USGS earthquakes (2026-04), NASA FIRMS fires (2026-04) and NASA EONET storms/events (2026-04). Those dates describe bundled snapshots, not a promise of currentness.

Use the layer's provenance flyout in the UI before interpreting a marker. For research, cite the source dataset and its version rather than citing a rendered globe marker.

### Important limitations

- Bundled JSON is not a live feed. Refresh cadence and source coverage vary by layer.
- Curated estimates and schematic datasets must not be treated as measurements.
- Scenario geometry and the 777-year timeline are planning fiction, not forecasts.
- The Confluence overlay indicates co-location only; it does not infer risk, causality, ownership or network connectivity.
- The Reactor Twin page is a recorded simulator replay, not live operational monitoring.
- The current default-branch app does not provide certified subsurface utility location, excavation clearance, PAS 128 quality certification or an authoritative owner network.

## Technology

| Area | Implementation |
|---|---|
| Application UI | Rust + Leptos 0.8 |
| Rendering | WebAssembly + WebGL2 |
| Shared logic | `sol-atlas-core` Rust crate |
| Data loading | Compile-time bundled JSON under `assets/data/` |
| Build and development server | Trunk |
| Hosting workflow | GitHub Pages deployment workflow |

The core crate contains shared data types, provenance labels, geospatial and geometry utilities, rendering support, timeline logic, economics and energy-trading helpers. The application layer contains the Leptos components, renderer, state and data-loading code.

## Quick start

### Prerequisites

- Stable Rust toolchain.
- The `wasm32-unknown-unknown` target.
- Trunk.

### Run locally

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
```

Trunk uses the repository's `Trunk.toml`; the development server is configured for port **8091**. Open [http://localhost:8091](http://localhost:8091).

### Build a release bundle

```bash
trunk build --release --no-sri
```

The GitHub Pages deployment workflow uses this release build and publishes the generated `dist/` directory.

### Run checks

```bash
cargo fmt --all --check
cargo check --target wasm32-unknown-unknown
cargo test --bin sol-atlas-leptos
cargo test --manifest-path sol-atlas-core/Cargo.toml
```

The repository's `.github/workflows/check.yml` is the authoritative source for the current CI matrix. A local pass does not replace exact-head CI evidence.

## Repository layout

| Path | Responsibility |
|---|---|
| `src/app.rs` | Routes and application composition |
| `src/components/` | Globe canvas, HUD, layer dock, timeline, dossier and demo page |
| `src/renderer/` | WebGL2 rendering |
| `src/state/` | Reactive application and globe state |
| `src/data/` | Bundled data loading and application data adapters |
| `assets/data/` | Bundled source, curated and scenario datasets |
| `sol-atlas-core/` | Shared Rust types and reusable core logic |
| `research/` | Research protocols, schemas, fixtures and benchmark material |

## Earth Observability: the research frontier

The longer-term research direction is to move from displaying layers to reconstructing defensible physical-world knowledge: what may exist underground, which evidence supports each claim, what remains unobserved or out of domain, and which additional investigation is worth acquiring.

The active public programme is deliberately separated from the shipped globe:

- [Issue #771 — Decision-grade Earth observability](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/771): parent research and implementation programme.
- [PR #779 — Joint utility-network hypotheses](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/779): draft work on competing network worlds, asset-association scoring, project-level split validation and sensor-to-decision reasoning.
- [PR #793 — Constrained investigation tournament scorer](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/793): draft evaluator for equal-budget policy submissions, hard authority constraints and realized decision regret; its campaign layer preserves each project result, macro-average and worst-project outcome rather than pooling all events. Planner-reported information gain stays separate from decision value.
- [Issue #783 — UT-Gold-D retrospective field benchmark](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/783): dataset admission and leakage-resistant physical-data evaluation.
- [Issue #785 — Forward-model validation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/785): analytical, numerical-convergence, inter-code and physical validation ladder.
- [PR #780 — AGS4 quoted-descriptor compatibility](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/780) and [PR #781 — sensor test corrections](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/781): separate draft compatibility/test fixes.

**Research claim boundary:** synthetic fixtures and structural contract tests are not physical sensor performance. UT-Gold-D has been identified as a candidate retrospective dataset, but dataset admission, field evaluation and independent truth review remain separate steps. No current Sol Atlas result should be interpreted as a survey certification, utility-owner approval or permission to excavate.

The research programme aims to compare complete asset/network reconstruction and investigation decisions—not just detector accuracy—using project/site holdouts, proper probabilistic scores, explicit overmerge/oversplit and false-connection metrics, calibrated uncertainty, and equal-budget decision loss. Any SOTA comparison must use matched data, truth, split, metric and cost boundaries.

## Contributing

Useful contributions improve either the application or the evidence behind it.

1. Keep each dataset's source, snapshot/version, licence, units, coordinate reference and known limitations explicit.
2. Preserve the distinction between observed, curated and scenario content.
3. Add tests for parser failures, missing metadata, provenance leakage and invalid assumptions—not only happy paths.
4. Keep physical truth, synthetic fixtures, simulation outputs and expert interpretation in separate evidence classes.
5. For subsurface benchmarks, split by independent project/site and keep blind truth inaccessible during model fitting and calibration.

Please open an issue before introducing new data sources, authoritative-looking utility claims or changes to qualification thresholds.

## Licensing and data rights

Rust source files carry SPDX headers identifying **AGPL-3.0-or-later**. At the time this README was prepared, the inspected default-branch root did not contain a repository-level `LICENSE` file, and some source headers refer to a `COMMERCIAL_LICENSE.md` that was not present in that root listing. The repository should resolve that metadata before external redistribution or commercial reuse; this README is not legal advice.

Bundled datasets may have separate attribution, licence, privacy and redistribution requirements. Do not assume the source-code licence grants rights to every data asset.
