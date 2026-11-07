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
