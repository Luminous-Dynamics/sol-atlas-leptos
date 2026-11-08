# CityMaker reference district composition manifest v0.1

This is a machine-readable cross-owner integration map for the parameterized district profile. It records known owner boundaries, current schema readiness and explicit gates. It intentionally does not define a universal receipt or guess missing source schemas.

## Current posture

- The reference district profile is uninstantiated: 10 service contracts, five asset archetypes, 15 required parameters remain null/Unknown.
- The integration scope is unbound. No shared district digest or scenario exists until geography, population frame, service horizon, scenario family, functional units, cost base, design/BOM revisions and evidence policy are declared.
- Mycelix Living-City `living-city-metabolism/0.1` is a versioned physical-flow/model-evidence contract; it is not qualified inventory, batch custody or supplier availability.
- Mycelix #4814 and #5149 own the production network and custody/provenance facts; schema-native projections must be registered before field mapping.
- Symthaea CIV-BOOT owns productive capability closure; CityKit consumes and consistency-checks owner receipts, but does not re-derive closure.
- Symtropy #3605 owns construction accepted-output denominators and outcomes; #3773 specializes the panel family.
- Symthaea #6036/#6040 own the openBIM and benign workcell digital-thread projections.
- Sol-Atlas #564 and #574 synthetic oracles are independently qualified on finite corpora, but not on real Johannesburg data.
- All PRs referenced here are draft/unmerged as of the manifest date.

## Integration gates

1. Bind one district scope and one scenario; do not compose independent results from different scopes.
2. Register exact owner schema/version/revision/digest and state mappings in issue #606.
3. Create one shared synthetic district instance with fixture receipts, known answers and negative controls.
4. Run a cross-owner replay: missing/stale/revoked/mismatched evidence must remain Unknown/Blocked.
5. Only after rights-cleared local snapshots and verified local service evidence should a field pilot be considered.

See `CITY_REFERENCE_DISTRICT_COMPOSITION_MANIFEST_V0_1.json` for the machine-readable owner map.


## Owner schema artifact pinning update — 7 November 2026

The composition manifest now records exact consumer/schema artifacts rather than only issue-level descriptions:

- **Mycelix Living-City:** draft JSON Schema 2020-12, ID `https://luminousdynamics.io/schemas/living-city-metabolism-v0.1.schema.json`, version `living-city-metabolism/0.1`, Git blob `4f7f3eeee4e91964a6422d2aba9f59e3c3aff57b`, source PR head `0aae2f0a7625304c9a7eab0c3528e02fe5d57567`. It is a draft, unmerged owner contract for system-boundary/functional-unit flow and model evidence—not a lot/inventory schema.
- **Symtropy CityKit:** exact consumer implementation Git blob `32217062db71d63785caad7cdc614af3b0530e39`, source PR head `678f2c496486096cda311337e98e6c9f336101ff`, plus 18-case synthetic corpus Git blob `944a0b2ea73f9424be72ff26dca1d1c830fcada0`. This proves the bounded consumer's field use, not a canonical published CIV-BOOT schema.
- **Symthaea CIV-BOOT:** consumer-observed fields are enumerated, but no owner-published canonical schema/version/revision has been pinned. Do not infer one from CityKit code.
- **Mycelix CIV-ECON and #5149**, **Symtropy #3605**, and **Symthaea #6040** remain issue/contract-described until the owner-published schema artifacts are pinned.
- **IFC 4.3.2.0** and **IDS 1.0** are published external exchange standards; they validate model/information exchange, not physical product qualification, custody or commissioning.

The manifest adds gate `G1A_OWNER_SCHEMA_ARTIFACT_PINNING`. Its unresolved entries remain blocking/Unknown; no shared district scope has been bound.


## Four-layer urban digital-twin synchronization architecture (7 November 2026)

The district manifest now separates four responsibilities based on a 2026 review of 114 urban digital-twin publications:

1. Reference/scope — #623 common scope binder.
2. Entity/semantics — #606 exact owner schema and persistent entity registration.
3. State synchronization — #625 valid time, observation time, record time, source-specific freshness, corrections, revocations and disputes.
4. Feedback synchronization — #603 product/service lifecycle and owner-native approval, execution, inspection, commissioning and measured outcome links.

A visual 3D model or a recent timestamp does not qualify a digital twin. Each source has its own update cadence and claim ceiling. No universal TTL or latest-timestamp-wins is admitted. See the manifest gate G1B and https://www.sciencedirect.com/science/article/abs/pii/S2210670726009455.


## Shared synthetic evaluator scope binding — 8 November 2026

New machine-readable fixture manifest:

- `CITY_REFERENCE_DISTRICT_SHARED_FIXTURE_SCOPE_V0_1.json`
- Scope ID: `SYNTHETIC-DISTRICT-01`
- Scope SHA-256: `dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd`

This exact synthetic digest is present in:
- state-sync PR #638, on the owner event and required-subject scope fields;
- service-restoration PR #646, on the reference scope;
- change-impact PR #644, on query, owner report, every claim and revalidation receipt.

Those three individual evaluator workflows passed their exact-head synthetic corpora. The change-impact adapter additionally rejects a report whose digest differs even when its display scope ID is the same.

**This is only a common scope-identity fixture. It is not yet a common district scenario or integrated district PASS.** The three corpora test different events, case semantics, horizons and functional units. They have not been cross-joined to one dwelling, panel asset, material lot, service requirement, intervention or outcome.

Next integration gate remains: bind one explicitly declared synthetic district scenario with common asset/design/BOM revisions and a shared time horizon; run the owner scope binder and schema registry; verify exact owner payload bytes; then run cross-owner positive and negative controls. The real district evaluation scope remains Unbound.
