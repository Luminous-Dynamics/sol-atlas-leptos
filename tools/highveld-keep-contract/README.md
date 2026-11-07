# Highveld Keep Contract v0.1

Standalone offline Rust checker for the internal consistency of the two E0 Highveld Keep design-definition JSON documents.

## Exact claim ceiling

A valid result means only:

`definition_valid_synthetic_only`

It means the tracked scenario and parameter-space documents satisfy this checker's declared internal invariants. It does **not** mean that any candidate is geometrically valid, buildable, safe, code-compliant, structurally adequate, thermally qualified, approved, optimal or costed.

The checker does not:
- generate or inspect geometry;
- authenticate or inspect climate archives;
- calculate weather, energy or engineering results;
- validate zoning, title, survey, geotechnical/dolomite, fire, structural, material or product approvals;
- determine aesthetic, resident or community outcomes;
- authorize construction or robot tasks.

Those remain with the appropriate Symthaea, Symtropy, Sol-Atlas, Mycelix and qualified professional owners.

## Exact inputs

The crate includes the tracked source files at compile time:

- `docs/research/highveld-keep/HIGHVELD_KEEP_SCENARIO_V0_1.json`
- `docs/research/highveld-keep/HIGHVELD_KEEP_PARAMETER_SPACE_V0_1.json`

Cargo recompiles when included source contents change. The CLI validates both documents as a package; it does not emit a durable certificate or reuse a stale validation receipt.

## Run

From repository root:

```sh
cargo fmt --manifest-path tools/highveld-keep-contract/Cargo.toml --all -- --check
cargo test --locked --offline --manifest-path tools/highveld-keep-contract/Cargo.toml
cargo run --locked --offline --manifest-path tools/highveld-keep-contract/Cargo.toml
```

The nested `[workspace]` isolates the package from the parent Leptos crate. `serde_json` is the only third-party dependency and is pinned in the manifest. The package requires no network, GIS, rendering, database, specialist solver or model call.

## Checked invariants

- Shared scenario identity and schema revisions.
- E0/synthetic claim ceiling, empty real parcel/site fields and synthetic envelope labelling.
- Exact four spatial alternatives and explicit alternative-ID-to-family bindings.
- All typed variable collections: finite ordered numeric ranges with units, non-empty categorical sets, unique variable IDs, and unresolved product/material evidence references.
- Synthetic parameter-bound status and explicit prohibition against treating bounds as engineering/legal/code limits.
- Five Gauteng TMYx files (Tambo, Botanical Gardens, Grand Central, Rand Airport and Lanseria) remain distinct catalogued candidates; the source page says All Rights Reserved and file-specific reuse terms remain unknown, so no redistribution rights are presumed and no TMY/heatwave substitution is allowed.
- Geometry, thermal, structural and fire specialist gates remain unqualified/owned by their existing authorities.
- Hard requirements remain separate from soft objective vectors.
- E0 evidence tier and unresolved site/resident/solver/approval inputs remain visible.
- The exact 11-edge functional relationship graph and functional-programme nodes have unique IDs; endpoints and required excluded intermediate functions exist; relation types map strictly to hard versus preference classes; unversioned geometry/distance fields are rejected.
- Every functional relationship ID is bound to its exact source, target, relation type, hard/preference class and exclusion set; IDs cannot be repurposed while preserving a green result.
- The EPW.tools alternative provider-method record remains research-only/unqualified, and its Johannesburg 0.5-degree MERRA-2 grid trend cannot be promoted to parcel/station design weather.
- Four ASHRAE / NOAA GHCND / SAWS-WMO station metadata cross-checks remain metadata-only, with no observation-series download or identity/history qualification; daily GHCND records cannot be promoted to hourly series.
- ARC operational-network (~500 station) summaries remain separate from its wider ~1,900-station portal archive (1,100 manual / 763 AWS / 460 active AWS); SAWS WIS product topics retain licence gates; NOAA ISD global archive metadata is not promoted to a local Johannesburg series.

## Tests

Unit tests mutate parsed in-memory copies to ensure the checker rejects populated real parcel fields, removed synthetic labels, missing/duplicate alternatives, wrong family binding, bad topology and envelope bounds, empty categorical sets, promoted TMY evidence, TMY-as-heatwave, solver/geometry promotion, E0 promotion, unqualified product references and hard life-safety objectives moved into the soft objective vector.

This is E1 package-integrity evidence only. It does not qualify the content of a weather file, material certificate, design, solver, site or building.
