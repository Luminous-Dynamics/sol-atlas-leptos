# EARTH-OBSERVABILITY-TEST-TRACEABILITY-001
## Frozen 18-case challenge to Rust contract mapping v0.1

**Parent:** Sol-Atlas #761  
**Branch:** `research/earth-observability-benchmark-v0-1`  
**Status:** source/test traceability; no claim that all benchmark scenarios have executed  
**As-of:** 2026-11-13

This matrix prevents a case-definition manifest from being mistaken for a completed benchmark. It distinguishes software contract guards, delegated domain owners, and work that remains pending. Root binary tests and core validators are committed but have not yet been compiled or executed in this environment.

## Status vocabulary

- **Contract guard added:** a deterministic Rust predicate/test covers a narrow data-contract property.
- **Partial contract guard:** a related semantic guard exists, but the full scenario/model/physical process is not implemented.
- **Delegated:** the underlying domain-specific qualification belongs to an existing workstream; Earth Observability composes its evidence.
- **Pending:** no executable implementation yet in this branch.

## Case traceability

| Case | Expected disposition | Branch implementation / test | Status | Remaining boundary |
|---|---|---|---|---|
| FRAME-AXIS-001 | reject qualification | `validate_spatial_observation`; `coordinate_axis_order_must_match_declared_frame` | Contract guard added | Does not implement CRS transformations or prove coordinates are correct in the named CRS |
| VERTICAL-DATUM-001 | reject qualification | vertical datum / height type / unit requirements; `three_dimensional_observation_requires_vertical_datum_and_height_type` | Contract guard added | Does not transform ellipsoidal to orthometric heights or validate geoid resources |
| TRANSFORM-GRID-001 | reject qualification | `validate_transform_receipt`; exact frame binding, 64-hex digest syntax; `coordinate_transform_must_bind_exact_frames_and_digests` | Partial contract guard | Does not fetch/hash grid bytes, verify operation mathematics, or prove operation validity in area/epoch; referenced resources must be checked by a real transform adapter |
| CHECKPOINT-LEAKAGE-001 | invalidate evaluation | `validate_evaluation_split`; `independent_checkpoints_cannot_leak_into_fit_partition` | Partial contract guard | Checks identifiers only; spatial overlap, derivative/near-duplicate scans and common-campaign leakage remain pending; compose with Symtropy #1607 |
| REGISTRATION-DRIFT-001 | change indeterminate | `classify_change`; `change_inside_uncertainty_detection_limit_is_not_confirmed` | Partial contract guard | Scalar threshold only; correlated covariance, frame drift, registration degeneracy and spatial detection surfaces remain delegated to Symtropy #1607 |
| GPR-CONDUCTIVE-GROUND-001 | unobserved | `classify_non_detection`; no-coverage and out-of-domain negative controls | Partial contract guard | Does not model GPR propagation, wet-clay attenuation, conductivity, frequency, target contrast or POD curves; modality qualification remains pending |
| ERT-NONUNIQUE-001 | preserve conflict | `compare_interpretations`; same-support differing claims return `Conflicted` | Partial contract guard | Preserves claims but does not invert ERT, characterize null spaces or test physical non-uniqueness |
| SPARSE-BOREHOLES-001 | abstain or widen uncertainty | No spatial random-field reconstruction in this branch | Pending | Compose sampling/EVSI with Sol-Atlas #134; reconstruction, covariance and uncertainty growth require a dedicated model and independent reference |
| STRATA-CONFLICT-001 | preserve conflict | `compare_interpretations`; different-support claims return `IncomparableSupport` and same-support disagreements remain conflicted | Contract guard added | No automatic source authority ranking or geological adjudication; support IDs must be correctly defined by the adapter |
| MODALITY-MISSING-001 | partial or abstain | `EvidenceState::Partial` / `Unknown` vocabulary; manifest shape guard | Partial contract guard | No fused-model input admission adapter yet; model-level required-modality handling remains pending |
| DETECTION-LIMIT-001 | below detection limit | `classify_non_detection`; `below_detection_limit_is_not_absence_evidence` | Contract guard added | POD supplied as an input; no empirical POD calibration or modality-specific detection curve is implemented |
| GEOLOGY-OOD-001 | out of domain | `classify_non_detection` returns OOD before considering high nominal POD | Partial contract guard | Tests declared observation-domain gating, not a geological model's OOD detection or uncertainty adaptation |
| EVSI-AUTHORITY-001 | ineligible action | `survey_action_is_eligible`; permit/custodial-review negative controls | Contract guard added | Does not calculate EVSI or verify authority declarations against external permit/land/custodial systems |
| EVSI-MECHANISM-001 | mechanism-specific plan | No spatial planner in this branch | Delegated / pending integration | Sol-Atlas #134 owns spatial sampling/EVSI; mechanism-specific loss models and Symthaea planner integration remain pending |
| EVIDENCE-REVALIDATION-001 | revalidation required | Existing Sol-Atlas evidence graph PR #138 owns dependency frontier; no new graph mutation code here | Delegated | Need an adapter from new Earth claims to #138's artifact and dependency graph; no solver rerun or new approval is authorized |
| EVIDENCE-HISTORY-001 | preserve history and mark stale | Existing Sol-Atlas #138 evidence/artifact revisions and revalidation work | Delegated | Branch adds no immutable event store or source revision mutation handler; integration and regression tests remain pending |
| DAS-CHAINAGE-001 | retain chainage; no geographic fix | `qualify_fibre_geolocation`; `das_chainage_without_route_calibration_is_not_a_geographic_fix` | Contract guard added | Qualification yields no geographic coordinate; a separate route-geometry projection must consume the exact calibration receipt |
| DAS-COUPLING-001 | stratify and calibrate | Existing Symtropy #4972 owns coupling and DAS observability profile | Delegated / pending | No acoustic signal simulator or DAS coupling model is implemented by Sol-Atlas |

## Additional rights and federation guards (supplemental to the 18 manifest cases)

The Rust core now contains a fail-closed owner-grant predicate. These are additional unit-test controls, not extra rows in the frozen 18-case JSON manifest:

| Supplemental control | Executable guard | Expected result | Coverage ceiling |
|---|---|---|---|
| RIGHTS-EXACT-SOURCE-001 | `missing_or_wrong_source_grant_fails_closed` | deny wrong-source or missing grant | Source identity only; grant signature/issuer authentication is upstream |
| RIGHTS-PURPOSE-001 | `exact_source_grant_allows_only_declared_local_processing` | permit declared read/process; deny ungranted redistribution | Checks declared booleans; does not inspect actual network egress or derivative content |
| RIGHTS-TIME-001 | `expired_and_future_grants_are_not_usable` | deny before not-before and at/after expiry | Wall-clock trust and revocation are not implemented here |
| RIGHTS-CONDITIONS-001 | `unmet_grant_conditions_block_processing` | deny when declared conditions are unsatisfied | Does not verify the underlying condition evidence |
| RIGHTS-RECEIPT-001 | `invalid_rights_receipt_digest_is_rejected` | reject malformed digest | Digest syntax only; Mycelix/custody must verify signature, issuer and revocation |

The grant-gate contract and execution boundary are documented in [EARTH-OBSERVABILITY-FEDERATED-EVIDENCE-001](EARTH_OBSERVABILITY_FEDERATED_EVIDENCE_V0_1.md). This is a policy-consistency predicate over an upstream-qualified grant, not a legal interpreter, signature verifier or privacy-preserving execution environment.

## Additional deterministic synthetic-profile fixture

The first data fixture now exists at `research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json`. It contains:

- 6 sparse training borehole contacts on an abstract transect;
- 22 noisy, deterministic surface-wave *interface proxy* records across two abstract transects;
- 5 explicit UNOBSERVED records in the held-out transect's blind zone, represented with null estimates and null uncertainty rather than zero;
- 11 independent validation truth points on a shifted held-out transect;
- fixed seed `20261113`, declared metre units, and an explicit no-field/no-physics claim boundary.

The corresponding Rust generator is in `sol-atlas-core/src/earth_observability_synthetic.rs`; a binary can write or verify the committed fixture. This is a deterministic benchmark fixture, not a physical forward model, sensor simulation, geological inference result, or field dataset.

| Additional challenge | Executable guard | Coverage ceiling |
|---|---|---|
| PROFILE-SPARSE-BOREHOLES-001 | `blind_truth_is_separate_from_training_boreholes` | Split/identity properties only; no geological interpolation benchmark yet |
| PROFILE-TRANSFER-001 | shifted held-out transect truth | Dataset supports a transfer test; no model or transfer score is implemented yet |
| PROFILE-BLIND-ZONE-001 | `blind_zone_remains_unobserved_and_is_not_zero_filled` | Checks missingness semantics; does not model GPR/ERT/wave propagation |
| PROFILE-NOISE-001 | `geophysical_proxy_noise_respects_declared_uncertainty_bound` | Noise is synthetic and bounded; uncertainty calibration against field sensors is not tested |
| PROFILE-REPLAY-001 | `generator_is_deterministic_for_a_frozen_seed`, committed fixture comparison | Deterministic generation only; no independent full-pipeline replay receipt yet |

## Executable entry points

The branch adds:

- `sol-atlas-core/src/earth_observability.rs`: renderer-independent frame, non-detection, split, transform, survey-authority, change, interpretation-conflict, fibre-chainage and exact-source data-use grant predicates.
- `sol-atlas-core/src/earth_observability_synthetic.rs`: deterministic abstract profile generator and synthetic-profile tests.
- `research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json`: pinned generated fixture.
- `src/earth_observability_tests.rs`: root native tests run by the package's binary test target.
- `src/bin/earth_observability_validate.rs`: offline challenge-manifest contract validator. Its receipt labels JSON Schema validation, scenario execution and physical validation as `NOT_RUN`.
- `src/bin/earth_observability_generate.rs`: writes the deterministic profile or verifies a committed fixture using `--check`.
- `research/earth-observability/EARTH_OBSERVABILITY_FEDERATED_EVIDENCE_V0_1.md`: rights-aware, source-local execution architecture and qualification gates.
- `.github/workflows/check.yml`: formats/checks the application, builds WASM, runs application and shared-core native tests, verifies the manifest, and verifies the pinned synthetic profile.

Commands from the repository root:

```sh
cargo fmt --all --check
cargo test --bin sol-atlas-leptos
cargo test --manifest-path sol-atlas-core/Cargo.toml
cargo test --bin earth_observability_validate
cargo run --bin earth_observability_validate -- research/earth-observability/earth-observability-case-manifest-v0.1.json
cargo test --bin earth_observability_generate
cargo run --bin earth_observability_generate -- --check research/earth-observability/fixtures/earth-observability-profile-synthetic-v0.1.json
```

The manifest validator checks authored manifest invariants over serde types; it is not a JSON Schema validator, geospatial transformation engine, GIS ingestion layer, geological inference model, or physical validation suite. The synthetic profile generator creates an abstract surface-wave interface *proxy*, not synthetic waveforms or a validated sensor forward model.

## Exit criteria for the next tranche

1. Run the new Rust binary tests on the exact PR head and retain the actual workflow receipt.
2. Add full JSON Schema validation only after a pinned, offline-capable validator and dependency policy are agreed; do not describe serde validation as JSON Schema validation.
3. Extend deterministic fixtures to frame/vertical-datum transformations, correlated registration drift, modality-specific probability-of-detection curves and spatial random fields.
4. Implement a first matched baseline (e.g. borehole-only piecewise/kriging baseline versus the synthetic geophysical proxy) with frozen split and a blind-scoring interface; do not treat the synthetic proxy as real geophysics.
5. Implement support-scale identity and conflict-aware adapters for borehole / geological / geophysical interpretations without silently fusing incompatible spatial supports.
6. Keep the EVSI planner, survey authority and graph revalidation with their current owners; add boundary adapters and integration cases rather than parallel owners.
7. Do not promote synthetic test success into geological, geotechnical, surveying or construction qualification.

