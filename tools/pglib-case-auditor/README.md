# PGLib Case Auditor v0.1

A small, offline Rust tool for admitting a narrowly defined subset of literal MATPOWER v2 case files into the Sol-Atlas network-benchmark intake path.

## What it checks

- A single literal `function mpc = ...` declaration, `mpc.version = '2'`, `mpc.baseMVA`, and literal `bus`, `gen`, `branch`, and `gencost` matrices, plus optional `areas` metadata.
- Unknown statements, MATLAB expressions, unsupported columns, duplicate bus identifiers, missing buses, invalid statuses/bounds, unsupported cost rows and non-finite values fail closed.
- The active branch graph is partitioned into energized islands. This profile requires exactly one type-3 reference bus in each energized island.
- The receipt binds the exact source bytes using Git's canonical blob SHA-1.

## What it does not establish

This is **structural import only**. It does not execute MATLAB, parse every MATPOWER extension, establish source authenticity, solve power flow, solve AC/DC OPF, validate an electrical model against measurements, certify grid-code compliance, or authorize dispatch or switching. A digest establishes byte identity, not truth.

This first profile is intentionally narrower than the complete MATPOWER format. Unsupported extensions are rejected rather than silently discarded. The parser is an intake guard, not yet the independent second-parser comparison required by Sol-Atlas issue #332.

## Run

```sh
cargo test --manifest-path tools/pglib-case-auditor/Cargo.toml --locked
cargo run --manifest-path tools/pglib-case-auditor/Cargo.toml --locked -- \
  tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m \
  --expect-git-blob-sha1 2c8260fff0311c902c3d36dcabcbe861576fc97c
```

The pinned fixtures are from PGLib-OPF v23.07 (IEEE 14-bus, IEEE RTS 24-bus, and IEEE 57-bus). Their upstream commit, paths, blob identifiers and attribution are recorded in NOTICE.md.
