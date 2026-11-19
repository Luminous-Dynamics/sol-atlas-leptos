# Shared amenity spatial-comparison manifest audit

A standalone Rust tool that checks the frozen v0.1 **Sol-Atlas spatial provisioning comparison registry**.

## Run

From the repository root:

```bash
cargo test --manifest-path tools/shared-amenity-manifest-audit/Cargo.toml
cargo run --manifest-path tools/shared-amenity-manifest-audit/Cargo.toml -- docs/research/highveld-neighborhood-commons/SHARED_AMENITY_COMMONS_MANIFEST_V0.1.json
```

## Scope

Sol-Atlas #940 compares spatial provision options for resident-facing amenities:
- private/on-plot or accessory provision;
- one central shared facility;
- distributed shared nodes;
- a resident-selected private/shared hybrid;
- external commercial providers.

The 50 registered cases cover five provision models across ten spatial contexts: high/low density, mixed-use/transit access, site-edge travel, accessible-route discontinuity, population distribution, phased occupancy, land competition, external provider distance/closure and catchment rebalancing.

## What the auditor checks

- exact v0.1 study identity and claim ceiling;
- the five frozen provision arms and ten spatial scenario identifiers;
- a complete 5 × 10 matrix with no duplicate/missing pair;
- all 50 cases remaining NOT_EXECUTED / NOT_SUPPLIED / NO_RESULT;
- ten amenity archetypes and nine candidate service functional units;
- every candidate functional unit is explicitly marked CANDIDATE_NOT_OWNER_APPROVED and bound to Mycelix #6328 for owner review;
- non-essential plant remains an enabling asset, never a standalone amenity service;
- required excluded scopes and 14 owner-native dependencies remain present with exact owner/role/reference bindings;
- at least 34 unique source URLs and unique source IDs;
- proposed target invariants.

The report includes SHA-256 of the exact manifest bytes. That digest establishes byte identity only.

## Canonical owner boundaries

- **Mycelix #6328 / draft PR #6329** owns the canonical shared-amenity asset/service/beneficiary/access/finance/reserve/maintenance/governance/handback contract and its 33-case synthetic conformance corpus. Sol-Atlas #940 must consume its records; it must not create a second asset/service ledger.
- **Sol-Atlas #816** owns whole-household cost-of-place affordability. #940 consumes its cost/access outputs instead of recalculating household affordability.
- **Mycelix #912** owns capital-to-commons financing and stewardship transition semantics.
- **Symtropy #3768** owns generic service/capacity/resource feasibility.
- **Mycelix #5901** owns matching/reservation and constrained allocation policy.
- Sol-Atlas #607/#904 own the reference home and resident expression/experience packages.
- Sol-Atlas #107/#105, Symtropy #3957/#4065/#4074 and Mycelix #5001 own the food/CEA evidence chain.

The auditor checks that these owner references remain in the registry. It does not validate their payloads, schemas, or current execution state.

## Deliberate non-goals

This tool is not:
- a JSON Schema Draft 2020-12 implementation;
- a GIS, route engine, catchment optimizer or site-selection solver;
- an amenity asset/service registry;
- a capacity, queueing, booking or fairness allocator;
- a whole-household affordability, investment or financial ledger;
- a physical building/CEA/pool/greenhouse solver;
- a title, governance, accessibility, safety, pool, water-quality or regulatory approval authority.

A semantic audit pass means only that the Sol-Atlas comparison registry is internally consistent and its owner boundaries are preserved.
