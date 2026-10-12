# Rust-native live-world integration

## Current vertical slice

Sol Atlas polls two allow-listed public data sources from Rust/WASM:

**USGS ComCat earthquake feed**
- Feed: https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/1.0_day.geojson
- Documentation: https://earthquake.usgs.gov/earthquakes/feed/v1.0/geojson.php

**NASA EONET v3 curated open events**
- Feed: https://eonet.gsfc.nasa.gov/api/v3/events?category=severeStorms,wildfires,volcanoes&status=open&days=7&limit=500
- Documentation: https://eonet.gsfc.nasa.gov/docs/v3

Pure Rust parsers live in `sol-atlas-core/src/usgs.rs` and
`sol-atlas-core/src/eonet.rs`. Transport is centralized in
`src/data/live_feed_http.rs`, with source-specific refresh lifecycles in
`src/data/live_earthquakes.rs` and `src/data/live_eonet.rs`.

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
- USGS refresh replaces only records validated and owned by USGS ComCat.
  EONET refresh replaces only records validated and owned by NASA EONET; bundled
  records from other sources remain untouched.
- Important bundled-data correction: `assets/data/nasa-firms.json` declares
  `source: "Demo Data"` and per-record `data_lineage: ["Demo", "Synthetic",
  "Generated"]`. It is a synthetic fixture, **not a NASA FIRMS observation**.
  The parser preserves that declared source and classifies each record as
  Synthetic rather than inferring NASA attribution from the filename. Its
  layer is marked mixed provenance because live EONET fire observations and
  synthetic bundled FIRMS examples can coexist in the same Fires view.
- Bundled USGS and EONET files are dated fallbacks, not a claim of currentness.
  Every natural-event record now separates two dimensions:
  `data_kind` (Observed, Curated, Synthetic, Mixed, or Scenario) and
  `record_temporal_origin` (LiveFeed, BundledSnapshot, or CuratedReference).
  An old bundled USGS event may be a genuine Observed record while still being
  a bundled snapshot, not a current live observation.
- Source metadata is retained per record; a file slot alone never establishes
  official provenance. Missing or conflicting provenance stays unclassified.
  Official event links are exposed only when source classification, source
  identity, record ID, and expected official URL path agree.
- EONET is a curated event catalog, not an exhaustive or uniformly real-time
  sensor feed. Point records retain their coordinates; polygons use an
  approximate spherical vertex-center, labeled in the event dossier.
  EONET measurements may have category-specific units, so they are not
  presented as a globally comparable earthquake-style magnitude.
- The selected-event dossier shows both provenance class and temporal
  origin, and shows numeric measures only for supported source-specific cases
  (USGS magnitude and EONET's own measure/unit). Synthetic, mixed, unclassified,
  or unsupported numeric values are never presented as verified magnitudes.
- The derived Confluence overlay accepts event records only when the individual
  record is (Observed + LiveFeed) or (Curated + CuratedReference). Bundled
  snapshots remain inspectable but cannot create a claim of current co-location;
  Synthetic, Mixed, and unclassified records fail closed.
- The selected-event dossier lists up to four nearest curated infrastructure
  records inside 100 km, using the Rust core's great-circle distance helper.
  This is geographic context only: the inventory is incomplete, and proximity
  is not an impact, damage, causal, or risk estimate.

## Deliberate limits

This slice provides recent earthquake observations and curated open-event
metadata for severe storms, wildfires, and volcanoes. It does not yet ingest
GEV's flight, ship, traffic, or camera sources, nor does it infer a risk score
from spatial overlap. The 100 km proximity view is deliberately not a model of
earthquake attenuation or infrastructure vulnerability. Add one source at a
time, with source-specific licensing,
schema, freshness, provenance, and deterministic parser tests before using it in
simulation or coordination decisions.

USGS records can be delayed or revised; EONET events are curated and may be
generalized or incomplete. “LIVE” means the latest retrieval passed source-specific
validation, not that every event is captured or final. Neither feed substitutes
for authoritative emergency guidance.
