# PowerIO MATPOWER Differential v0.2

A standalone research comparator that parses the same exact MATPOWER v2 source bytes with:

1. the Sol-Atlas strict importer and typed source-bound projection;
2. PowerIO 0.12.1, an independently maintained Rust-first power-system data compiler.

## Field coverage

The comparator checks:
- all 13 declared bus fields, including type, initial voltage, base kV, limits, area and zone;
- active/reactive demand aggregated from PowerIO's load table back to the source bus rows;
- bus conductance/susceptance aggregated from PowerIO's shunt table;
- all 10 generator fields, including active/reactive dispatch, Q/P limits, voltage setpoint, MBASE and status;
- aggregate online generation and Pmin/Pmax by bus;
- all 13 branch fields, including R/X/B, ratings, tap, phase shift, status and angle bounds;
- generator-cost model, startup/shutdown costs, NCOST and every retained coefficient.

The report binds the exact source bytes to the canonical Git blob SHA-1, records the exact PowerIO crate version and parser diagnostic count, and emits every field mismatch.

One explicit semantic representation mapping is applied: MATPOWER RateA/RateB/RateC equal to zero means unlimited; PowerIO represents an unlimited rating as positive infinity. The comparator compares those meanings, while the original raw zero remains preserved in the Sol-Atlas source projection. No other silent normalization is permitted.

## Run

    cargo test --manifest-path tools/powerio-matpower-differential/Cargo.toml
    cargo run --manifest-path tools/powerio-matpower-differential/Cargo.toml -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

This standalone nested workspace pins the PowerIO crate version to exactly 0.12.1 and disables optional SINCAL features. Cargo resolves the comparator's independent dependency graph and writes a local Cargo.lock on the first run; CI publishes that lockfile as an artifact for review and pinning.

## Interpretation

A zero-mismatch result means only that the two parsers agree on the enumerated fields for that exact case. It does not establish:
- complete MATPOWER format coverage or equivalence of every parser behavior;
- electrical validity, AC/DC power-flow correctness, OPF equivalence, or source authenticity;
- operational suitability, physical measurements or grid-control authority.

Differences are evidence. Do not add silent normalization to obtain parity. First classify whether each discrepancy is a source interpretation error, a declared profile boundary, a unit/representation difference, or a genuine unsupported field; then preserve that classification in a new test and qualification receipt.

## Reference

PowerIO repository: https://github.com/eigenergy/powerio  
Pinned published crate version: 0.12.1  
PowerIO project license: MIT OR Apache-2.0
