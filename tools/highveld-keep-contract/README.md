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
- The two Johannesburg TMYx files remain catalogued but unqualified; no TMY/heatwave substitution.
- Geometry, thermal, structural and fire specialist gates remain unqualified/owned by their existing authorities.
- Hard requirements remain separate from soft objective vectors.
- E0 evidence tier and unresolved site/resident/solver/approval inputs remain visible.
- The exact 11-edge functional relationship graph and functional-programme nodes have unique IDs; endpoints and required excluded intermediate functions exist; relation types map strictly to hard versus preference classes; unversioned geometry/distance fields are rejected.

## Tests

Unit tests mutate parsed in-memory copies to ensure the checker rejects populated real parcel fields, removed synthetic labels, missing/duplicate alternatives, wrong family binding, bad topology and envelope bounds, empty categorical sets, promoted TMY evidence, TMY-as-heatwave, solver/geometry promotion, E0 promotion, unqualified product references and hard life-safety objectives moved into the soft objective vector.

This is E1 package-integrity evidence only. It does not qualify the content of a weather file, material certificate, design, solver, site or building.
