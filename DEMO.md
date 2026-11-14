# Sol Atlas — application walkthrough

Sol Atlas is an interactive planetary data globe built with Rust, Leptos, WebAssembly and WebGL2. The application is a layered visualization and planning prototype; it should not be interpreted as a continuously refreshed digital twin or an authoritative infrastructure registry.

## Main globe

The root route (`/`) displays a navigable 3D globe with:
- a WebGL2 globe canvas;
- a selectable layer dock with source/provenance flyouts;
- a HUD, item dossier, tooltips and planet navigation;
- a 0–777-year planning timeline.

The timeline is a speculative scenario control. Its five epochs—Foundation, Growth, Maturation, Hardening and Deep Time—animate the planning layers. They are not forecasts of actual infrastructure construction, resource depletion or energy-system outcomes.

## Data layers and provenance

The application bundles JSON data under `assets/data/` at build time. Layer metadata separates three different relationships to reality:

| Provenance | Interpretation | Examples |
|---|---|---|
| Observed | Source observations or dated source snapshots; not necessarily live | USGS earthquakes, NASA FIRMS active fires, NASA EONET storms/events, energy-site inventory |
| Curated | Aggregated, estimated, schematic or illustrative content | Earth regions, supply chains, climate projects, fossil-deposit EROI, nuclear sites, critical infrastructure, cities and maritime chokepoints |
| Scenario | Speculative planning content | Geothermal nodes, maglev corridors, Resontia vaults, Terra Lumina sites and robotics dispatch |

The layer flyout is the authoritative in-app provenance cue. Snapshot dates, where supplied, are dates of bundled source material—not refresh guarantees.

The **Confluence** overlay is derived from spatial co-location among observed/curated layers. It is not a risk score, causal inference, proof of connection or statement of ownership.

## Reactor Twin demo

The separate route `/labs/reactor-twin` replays a recorded run of Symthaea's FissionTwin simulator.

It is:
- a static, pre-recorded simulation replay;
- not connected to a real nuclear plant;
- not live telemetry or a plant monitoring system;
- explicitly labelled where a synthetic capstone value appears in the recorded scenario.

Keep this demo separate from the globe's Nuclear layer, which contains a different curated operating-plant dataset.

## Current limitations

- Most content is bundled JSON and may become stale until refreshed and reviewed.
- Some layers are estimates or schematic representations.
- Scenario layers and the timeline are speculative, not operational plans.
- Marker co-location is not proof of physical connection.
- The application does not provide certified underground utility location, PAS 128 survey quality, owner-accepted network records or excavation clearance.

The Earth Observability programme is active research in draft public PRs. Its sensor qualification, utility inference, asset scoring and field validation capabilities are not yet part of the default-branch application.

## Development

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
```

The development server uses port 8091, as configured in `Trunk.toml`.

For architecture, checks, research links and contribution guidance, see [README.md](README.md).
