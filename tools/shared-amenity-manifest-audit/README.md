# Shared amenity manifest audit

A small, standalone Rust tool that checks the frozen v0.1 shared-amenity registry's semantic invariants.

## Run

From the repository root:

```bash
cargo test --manifest-path tools/shared-amenity-manifest-audit/Cargo.toml
cargo run --manifest-path tools/shared-amenity-manifest-audit/Cargo.toml -- docs/research/highveld-neighborhood-commons/SHARED_AMENITY_COMMONS_MANIFEST_V0.1.json
```

## What it checks

- exact v0.1 study identity and claim ceiling;
- the five frozen comparison arms and ten scenario identifiers;
- a complete 5 × 10 case matrix with no duplicate/missing pair;
- all 50 cases remaining NOT_EXECUTED / NOT_SUPPLIED / NO_RESULT;
- one functional unit for each resident-facing amenity class;
- the non-essential plant remaining an enabling asset, never a standalone service;
- explicit separation from existing household-resilience, multifamily, capacity, scope-binding, allocation and stewardship owners;
- unique source IDs and the proposed target invariants.

The report includes SHA-256 of the exact manifest bytes. That digest establishes byte identity only.

## Deliberate non-goals

This tool is not:
- a JSON Schema Draft 2020-12 implementation;
- a booking, queueing or fairness allocator;
- a service-capacity, physics or building-performance solver;
- a whole-life cost or stock-flow ledger;
- a title, governance, accessibility, safety or regulatory approval authority.

Those functions remain with the owner-native systems referenced by the benchmark. A semantic audit pass means only that the frozen registry is internally consistent.


## Cross-project composition checks

The auditor also requires the exact owner-native composition records for:
- Highveld Keep #607 and Home Expression Library #904;
- Highveld food/climate readiness #107 and food-network projection #105;
- Symtropy CEA accounting #3957, climate/airflow #4065 and harvest control #4074;
- Mycelix food cooperative #5001;
- Symtropy room-scale service test cell #4531, WaterLoop demonstrator #5335 and building-service cassettes #4181.

This confirms that the frozen registry names the intended owners and responsibilities. It does not validate those systems or read their payloads; downstream composition still requires exact, source-addressable owner receipts.
