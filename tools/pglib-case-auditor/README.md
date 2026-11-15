# PGLib Case Auditor v0.1

A small, offline Rust tool for admitting a narrowly defined subset of literal MATPOWER v2 case files into the Sol-Atlas network-benchmark intake path.

## What it checks

- A single literal `function mpc = ...` declaration, `mpc.version = '2'`, `mpc.baseMVA`, and literal `bus`, `gen`, `branch`, and `gencost` matrices, plus optional `areas` metadata.
- Unknown statements, MATLAB expressions, unsupported columns, duplicate bus identifiers, missing buses, invalid statuses/bounds, unsupported cost rows and non-finite values fail closed.
- The active branch graph is partitioned into energized islands. This profile requires exactly one type-3 reference bus in each energized island.
- The receipt binds the exact source bytes using Git's canonical blob SHA-1.

## Fixed-injection DC projection

The library also exposes `project_matpower_dc_case(source)`, which returns a typed, source-bound projection for the narrow DC reference model.

The projection:
- binds the exact UTF-8 source bytes by Git blob SHA-1;
- preserves every field in the supported 13-column bus, 10-column generator and 13-column branch rows;
- aggregates in-service generator active power by bus and computes non-reference fixed injection as `sum(Pg_online) - Pd`;
- leaves type-3 reference-bus injection unspecified for the downstream DC solver to balance;
- retains generator P/Q limits, voltage setpoints, shunts, branch ratings, phase shift, tap and angle limits as metadata;
- checks each in-service generator's supplied base Pg against Pmin/Pmax, rejects negative branch ratings, non-positive in-service reactance, inverted branch angle bounds and type-4 isolated buses;
- reports which input fields are intentionally not enforced by the DC model.

This is a projection contract, not full MATPOWER semantics. The downstream fixed-injection DC model does not enforce generator redispatch/reserve/ramping, slack-generator post-solve capability, resistance losses, branch charging, reactive power, voltage magnitude, bus shunts, branch angle-difference bounds, Rate B/Rate C, or gencost objectives. Rate A is the only branch rating passed to the DC feasibility profile, where it is compared to active MW flow as a linear approximation. Type-4 buses are rejected rather than silently dropped.

The structural import audit and projection currently parse the same source independently in two passes inside the same crate. This does not count as independent parser parity; the second-parser comparison against MATPOWER/PowerModels remains a separate gate.

## Fixed-injection DC projection

The library also exposes `project_matpower_dc_case(source)`, a typed projection bound to the exact source bytes.

It retains:
- all 13 supported bus fields;
- all 10 supported generator fields;
- all 13 supported branch fields;
- each supported `gencost` row's model, startup/shutdown cost, NCOST and exact coefficient vector;
- the presence of optional legacy `areas` metadata (its operational semantics remain unused in MATPOWER v2).

Non-reference bus injections are the sum of in-service generator Pg minus Pd. Type-3 buses remain unspecified for the downstream reference solver's balancing injection. Type-4 isolated buses are rejected because the current DC solver has no isolated-bus state.

Generator costs are retained for independent comparison and future optimization work, but are not used by the current fixed-injection DC power-flow solver. The projection also explicitly lists model fields that are retained but not enforced by the downstream DC equations.

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
