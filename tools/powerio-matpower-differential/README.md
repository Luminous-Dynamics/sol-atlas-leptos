# PowerIO MATPOWER Differential v0.1

A standalone research comparator that parses the same exact MATPOWER v2 source bytes with:

1. the Sol-Atlas strict importer and typed fixed-injection projection;
2. PowerIO 0.12.1, an independently maintained Rust-first power-system data compiler.

It compares the supported bus, generator, branch and cost-row fields, including aggregated loads and online generation by bus. Each receipt binds the exact input bytes to the canonical Git blob SHA-1, records the PowerIO version and diagnostic count, and emits every field mismatch rather than silently coercing differences.

## Run

    cargo test --manifest-path tools/powerio-matpower-differential/Cargo.toml
    cargo run --manifest-path tools/powerio-matpower-differential/Cargo.toml -- tools/pglib-case-auditor/fixtures/pglib_opf_case14_ieee.m

This standalone nested workspace pins the PowerIO crate version to exactly 0.12.1 and disables optional SINCAL features. Cargo resolves the comparator's independent dependency graph and writes a local Cargo.lock on the first run; CI stores that lockfile as an artifact for subsequent review and pinning.

## Interpretation

A zero-mismatch result means only that the two parsers agree on the enumerated supported fields for that exact case. It does not establish:
- complete MATPOWER format coverage or equivalence of every parser behavior;
- electrical validity, AC/DC power-flow correctness, OPF equivalence, or source authenticity;
- operational suitability, physical measurements or grid-control authority.

Differences are evidence. Do not add silent normalization to obtain parity. First classify whether each discrepancy is a source interpretation error, a declared profile boundary, a unit/representation difference, or a genuine unsupported field; then preserve that classification in a new test and the qualification receipt.

## Reference

PowerIO repository: https://github.com/eigenergy/powerio  
Pinned published crate version: 0.12.1  
PowerIO project license: MIT OR Apache-2.0
