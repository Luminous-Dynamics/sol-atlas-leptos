# Rust-native live-world integration

## Current vertical slice

Sol Atlas polls the official USGS ComCat GeoJSON summary feed for magnitude 1.0+
earthquakes from the previous day:

- Feed: https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/1.0_day.geojson
- Documentation: https://earthquake.usgs.gov/earthquakes/feed/v1.0/geojson.php
- Pure Rust parser/validator: `sol-atlas-core/src/usgs.rs`
- Rust/WASM transport and refresh lifecycle: `src/data/live_earthquakes.rs`

No GEV runtime, JavaScript package, embedded frame, or JavaScript production module
is introduced. Browser network transport uses Rust `web-sys`; decoding,
validation, state updates, and UI remain in Rust.

## Safety and freshness behavior

- The request target is a compile-time constant, not a caller-controlled URL.
- Requests abort after 12 seconds; refreshes are spaced at least 75 seconds apart.
- The response body is streamed into a Rust buffer capped at 8 MiB (the
  Content-Length is checked early when the server supplies it); decoded feed
  records are separately capped at 20,000 features.
- Feed generation time is checked against a 10-minute freshness window and a
  5-minute future-clock-skew allowance.
- Coordinates, magnitudes, event IDs, event types, and event links are validated.
  Invalid features are skipped individually; a nonempty feed with no valid
  feature is rejected as schema drift.
- A link is populated only for the official USGS event-page path.
- A failed, invalid, oversized, or stale refresh does not clear the last accepted
  earthquake records. The HUD distinguishes LIVE, LAST KNOWN, STALE, OFFLINE,
  and CONNECTING states.
- The live refresh replaces only earthquake records. Fire, storm, and volcano
  records from their other configured sources remain intact.
- The selected-event dossier lists up to four nearest curated infrastructure
  records inside 100 km, using the Rust core's great-circle distance helper.
  This is geographic context only: the inventory is incomplete, and proximity
  is not an impact, damage, causal, or risk estimate.

## Deliberate limits

This first slice provides recent earthquake observations. It does not yet ingest
GEV's flight, ship, traffic, or camera sources, nor does it infer a risk score
from spatial overlap. The 100 km proximity view is deliberately not a model of
earthquake attenuation or infrastructure vulnerability. Add one source at a
time, with source-specific licensing,
schema, freshness, provenance, and deterministic parser tests before using it in
simulation or coordination decisions.

USGS records can be delayed or revised. “LIVE” means a recent feed passed the
parser's source-time and data-quality gates; it does not mean the feed is complete
or final, and it is not a substitute for authoritative emergency guidance.
