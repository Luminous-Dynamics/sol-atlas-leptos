# CityMaker reference-district evidence readiness matrix v0.1

**As of:** 8 November 2026  
**Parent:** CITY-REFERENCE-DISTRICT-001 #615  
**Purpose:** keep individual synthetic owner qualification separate from common-scope district composition.

## Interpretation rules

- A green synthetic oracle qualifies only its own frozen input contract and bounded evaluator.
- A draft PR is not merged production functionality.
- Separate receipts do not compose merely because each reports PASS.
- A shared scope digest, exact owner schema/version, source revision, subject identity and payload digest are required before a cross-owner claim can be bound.
- The current scope binder does not verify owner payload bytes; its outputs explicitly retain `payload_bytes_verified=false`.
- Unknown, unregistered, stale, revoked, disputed, schema-mismatched or cross-scope inputs cannot be promoted to a favourable integrated result.
- No row in this matrix certifies a real district, building, product, service, supplier, robot or municipal process.

## Current E1 synthetic owner slices

| Domain | Exact owner artefact | Latest bounded evidence | What it proves | What it does not prove / next gate |
|---|---|---|---|---|
| Effective access | Sol-Atlas [PR #564](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/564) | 24-case access corpus and standalone Rust oracle; exact-head receipt recorded in PR | The declared synthetic network/service/affordability/eligibility/step-free semantics match the frozen known answers | No Johannesburg access result, live GTFS parity, or owner-native municipal service payload |
| Continuity and recovery | Sol-Atlas [PR #574](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/574) | 30 synthetic cases, 11 tests; [workflow #42453574999](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/42453574999); artifact `13606836535` | Time accounting separates adequate, constrained, inaccessible and unknown; recovery time is interval-bounded after evidence gaps | No real hazard probability, field continuity or service-resilience claim |
| Urban constraints and Pareto reference | Sol-Atlas [PR #566](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/566) | 24 synthetic cases, 10 tests; exact-head receipt described in PR | Hard gates, robust interval comparison and uncertainty abstention on its finite reference cases | Production comparison must use Symtropy #2529/#1874 and Sol-Atlas #131 owner-native certificates, not this local reference oracle |
| Service-to-product lifecycle | Sol-Atlas [PR #605](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/605) | 24 synthetic cases, 9 tests; exact-head receipt described in PR | The bounded synthetic product/service chain catches missing steps and wrong identity/revision | The fixture envelope is not a universal canonical owner schema; exact source adapters remain under #606 |
| Production geography | Sol-Atlas [PR #614](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/614) | 24 synthetic cases; exact-head receipt described in PR | Spatial projection and owner evidence checks on finite synthetic supply/worker/freight fixtures | No actual supplier/factory capacity or Johannesburg production geography |
| Construction delivery | Sol-Atlas [PR #627](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/627) | 33 synthetic cases; [workflow #42453626880](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/42453626880); 10 tests; artifact `13606816754` | Source lot → qualification/BOM → transport → site → receiving/inspection/custody → delivery-state semantics; half-open deadline and shared-resource overbooking | No actual inventory, route permission, vehicle/lifting certification, site permission or real accepted delivery |
| Panel manufacturing | Symtropy [#3773](https://github.com/Luminous-Dynamics/symtropy/issues/3773) and CityKit [#3768](https://github.com/Luminous-Dynamics/symtropy/issues/3768) | CityKit reference oracle 18/18 synthetic cases; panel-cell protocol/benchmark is separately specified | Bounded service/resource/capability reference semantics and a matched panel production experiment design | No physical productivity, worker-safety or building-component qualification from synthetic results |
| Panel disassembly and repair | Symtropy [PR #4153](https://github.com/Luminous-Dynamics/symtropy/pull/4153) | 24 synthetic cases, 8 tests; [workflow #42455141898](https://github.com/Luminous-Dynamics/symtropy/actions/runs/42455141898); artifact `13606954373` | BOM identity, connection/tool prerequisites, condition, repair/requalification, reassembly-function and shared-fixture semantics | No physical removability, robot task capability, structural/fire fitness, material qualification or real productivity |
| Urban metabolism | Mycelix [Living-City PR #5035](https://github.com/Luminous-Dynamics/mycelix/pull/5035) | Draft owner-native `living-city-metabolism/0.1` schema and synthetic flow/model evidence | Physical flow boundary, functional-unit, scenario, model-run and distributional-impact contract surface | Draft only; not product inventory, batch custody, real material flow or calibrated Johannesburg model |
| Supply / production network | Mycelix [#4814](https://github.com/Luminous-Dynamics/mycelix/issues/4814) | Canonical issue contract for suppliers, inventories, capability, substitution, lead times and logistics | Defines which owner supplies production/supply dependency facts | Requires exact versioned schema/receipt registration and rights/evidence; proximity does not prove deliverable capacity |
| Custody / inspection / revocation | Mycelix [#5149](https://github.com/Luminous-Dynamics/mycelix/issues/5149) | Canonical provenance/custody authority | Defines lot/batch identity, custody, correction, revocation, dispute and inspection ownership | Requires exact owner schema registration and owner-native artefacts |
| Construction accepted output | Symtropy [#3605](https://github.com/Luminous-Dynamics/symtropy/issues/3605) | Canonical accepted functional-unit and complete-workflow denominator owner | Makes first-pass/final acceptance, defect/rework and whole-workflow labour comparable | Do not infer accepted output from robot cycle time or simulation |
| Asset digital thread | Symthaea [#6040](https://github.com/Luminous-Dynamics/symthaea/issues/6040) | Canonical design/configuration/as-built/metrology/commissioning integration owner | Defines design-to-as-built and commissioning evidence handoff | Requires owner-native schema/revision and physical inspection evidence |
| Productive capability closure | Symthaea [#5774](https://github.com/Luminous-Dynamics/symthaea/issues/5774) | CIV-BOOT owns AND/OR closure; CityKit consumes its assessment | Defines the capability closure theorem owner | CityKit/CityMorph must not recalculate closure; exact source receipt schema remains an adapter requirement |

## Common-scope and schema readiness

| Gate | Status | Evidence / limitation |
|---|---|---|
| Parameterized district service and asset template | <badge>Defined</badge> | #616 contains ten service contracts, five initial asset archetypes and explicit Unknown parameters; no population/site/demand values are guessed |
| Owner-specific adapter registry | <badge>In progress</badge> | #606 has documented owner contracts; only some exact schema artefacts are available |
| Common scope binder | <badge>Synthetic pass</badge> | Sol-Atlas [PR #624](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/624): 25/25 synthetic metadata cases; hardens duplicate registrations; owner payload bytes remain unverified |
| Canonical owner schemas registered | <badge>Partial</badge> | Mycelix Living-City `living-city-metabolism/0.1` is pinned in the binder; CIV-BOOT, CIV-ECON, custody, accepted-output and commissioning schemas remain Unknown/unregistered |
| One common synthetic district scope | <badge>Not started</badge> | Boundary, population frame, horizon, scenarios, functional units, cost basis, design/BOM revisions and evidence policy still need one explicit scope instance |
| Cross-owner payload verification | <badge>Not started</badge> | Requires fetching exact artifact bytes and verifying exact schema, revision, subject, scope, digest and owner status; a hash string alone is not verification |
| Integrated district E1 receipt | <badge>Not started</badge> | No current claim that the separate green PRs qualify one and the same reference district |
| Rights-cleared real Johannesburg pilot | <badge>Not started</badge> | Source register identifies candidate sources and rights states; no local GIS/population/service/transit layer is ingested/qualified by this manifest |
| Physical panel / building / service validation | <badge>Not started</badge> | Requires E3 laboratory and later supervised site evidence with professional and worker/community review |

## Recommended next integration sequence

1. **Finish #606 registrations:** pin exact owner schema ID/version, schema digest, source-code revision, subject fields, receipt IDs, valid-time/currentness, revocation and claim-ceiling semantics. Unregistered owners remain Unknown.
2. **Bind one synthetic G2 scope through #623:** fixed synthetic grid boundary, population frame, service-requirement set, horizon, scenario set, functional-unit definitions, cost base and design/BOM revisions. Do not call it Johannesburg.
3. **Prepare owner artifacts for that same scope:** rerun or adapt only the required owner fixtures against the common scope. Do not reuse a green receipt from a different scope.
4. **Verify bytes, not only envelopes:** retrieve each exact owner artifact, validate schema and digest, and set payload verification true only after the bytes are actually checked.
5. **Run cross-owner negative controls:** wrong scope, stale revision, revoked batch, unsupported schema, wrong functional unit, imported-vs-local production, duplicate material/resource, uncommissioned asset, access failure and maintenance dependency absent.
6. **Issue a district composition receipt only if every required claim is Bound and every hard service floor is satisfied.** Otherwise produce Unknown/Blocked/NotComparable with explicit reasons and an evidence gap list.
7. **Move to E2/E3 only after E1 composition:** source-backed pilot data, then controlled physical panel and service commissioning evidence.

## What "better than SOTA" means here

Do not claim CityMaker routes better than R5/OTP, performs structural engineering better than specialist solvers, or outperforms certification tools based on synthetic test counts. The differentiator to test is the **cross-owner, lifecycle-complete, evidence-qualified decision process**: a district alternative should be traceable from resident/service requirements through product/process/capability, qualified supply, delivery, accepted assembly, commissioning, operation, repair and measured outcome, with uncertainty and decision rights preserved.

No composite district score; report service, equity, lifecycle, productivity, resilience, cost and environmental vectors separately and expose all non-compensatory gates.

## Claim ceiling

This matrix documents E1 owner slices and integration readiness. It does not certify a district or prove that the listed owner outputs describe the same scenario, location, population, time horizon or physical state. Integration is **not qualified** until the common scope and exact owner payloads are bound and verified.
