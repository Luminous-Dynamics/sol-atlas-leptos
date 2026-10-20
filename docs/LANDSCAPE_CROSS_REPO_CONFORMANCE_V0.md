# Landscape Cross-Repository Conformance Profile v0

**Status:** integration contract and test plan; not a claim that all adapters are implemented.
**Scope:** Sol-Atlas, Symtropy, Symthaea, Mycelix.
**Principle:** a structurally valid artifact is not automatically geospatially valid, model-qualified, causally informative, consented for publication, or proof of an outcome.

## Paradigm shift: connect artifacts, not just applications

The shared unit of integration is an immutable, versioned evidence chain:

authored project snapshot → declared evaluation plan → pinned scenario manifest and inputs → separately versioned observation records → read-only review → consent-aware decision and stewardship record

Each edge must name the exact upstream digest and schema/profile version. A later edit creates a new artifact version; it must not silently retarget old analyses, observations, approvals, or published outputs. Hashes bind bytes, not meaning: a whitespace or serialization change changes an exact-byte digest. The repository supplying an artifact is responsible for defining the bytes that are hashed.

## Ownership and contract mapping

| Stage | Owner and artifact | Required binding | What validation means | What it does not mean |
|---|---|---|---|---|
| Project authoring | Sol-Atlas native landscape snapshot v0 | Exact snapshot bytes, schema URI, scope, coordinate-frame kind, geometry status, stable feature IDs | Payload matches the strict v0 schematic document contract | Not surveyed geometry, WGS84, physical dimensions, ecological suitability, or build-ready design |
| Evaluation plan | Sol-Atlas evaluation-plan v0 (draft PR #210) | project_snapshot_sha256, project scope, baseline/follow-up windows, declared methods/outcomes/privacy | Study protocol is structurally and cross-field consistent; causal intent has gated design requirements | Not proof that an intervention worked, nor authorization to publish sensitive data by itself |
| Scenario admission | Symtropy manifest v0 (draft PR #2322) | Exact design bytes and digest, boundary-condition bytes/digest, model ID/version, purpose, seed, scope, optional exact evaluation-plan bytes/digest | The scenario manifest is bound to the payload and claim class it declares | Not execution of an ecological/hydrological solver or proof of physical accuracy |
| Observation | Sol-Atlas observation envelope/batch v0 (draft PR #216) | Exact raw-source payload digest, method/source, phenomenon and result times, units, uncertainty, quality/calibration, privacy/location semantics; optional project snapshot digest and feature ID | The observation record satisfies structural and local consistency checks | Not ground truth by virtue of being well-formed; it may be synthetic, unreviewed, uncertain, or non-causal |
| Review | Symthaea LAND-REVIEW-001 #8196 | Exact project digest, exact scenario/result digests, exact observation/evidence references | A read-only review can reproduce support, conflicts, unknowns, and uncertainty | Not an edit to the design or an independent verification unless independent evidence is supplied |
| Decision/stewardship | Mycelix profile v0 (draft PR #5298) | Project snapshot digest; scenario manifest/result digest as applicable; linked observation/evidence references; actor role/authority basis; consent/access policy and append-only correction/withdrawal | A declared participant decision or commitment is linked to the version and policy it refers to | Not physical truth, legal title, statutory authority, causal impact, or proof a maintenance task was performed |

## Field mappings that must remain explicit

The schemas intentionally use owner-specific field names. The adapters must map them explicitly; do not rename or infer missing data at runtime.

| Semantic field | Sol-Atlas project | Evaluation plan | Symtropy manifest | Observation envelope | Mycelix profile |
|---|---|---|---|---|---|
| Project bytes commitment | Snapshot bytes hashed by caller/export receipt | project_snapshot_sha256 | design_snapshot.sha256 | project_ref.project_snapshot_sha256 (nullable for unattached records) | project_snapshot.sha256 |
| Declared scope | scope | project_scope | design_snapshot.scope | Optional link through exact project digest | project_snapshot.scope |
| Frame / georeferencing | coordinate_frame.kind and georeferenced=false in v0 | Referenced via project digest; never assume a transform occurred | design_snapshot.coordinate_frame_kind and georeferenced | location has an explicit geographic, project-local, or opaque-site form | project_snapshot.coordinate_frame_kind and georeferenced |
| Geometry / evidence status | geometry_status and per-feature provenance/representation | Plan status is separate from geometry status | design_snapshot.geometry_status and run purpose | evidence_class is observation evidence type, not project status | project_snapshot.geometry_status and evidence_class |
| Scenario identity | Not owned by editor | Can pin input project, outcomes, and analysis plan | manifest bytes/digest, model ID/version and purpose | May reference result / source payload as an upstream digest in profile-specific extensions | scenario_reference.manifest_sha256 and optional result_sha256 |
| Location privacy | No public-by-default precise land coordinates | privacy and site references | Scenario input does not grant publication rights | privacy classification, publication policy, location kind/precision | access_policy and consent records govern the record/public projection |

The project-local v0 frame is LOCAL_CARTESIAN_DRAWING_UNITS, not metres, not a CRS, and not georeferenced. It may be used for schema qualification and synthetic software benchmarks. It must not enter a scientifically interpreted landscape model until a future project profile supplies actual geometry, verified coordinate operation, units, source provenance/resolution, and the domain-specific qualification required for that model.

## Claim eligibility is a monotonic gate

Use stable eligibility states with separate meanings rather than one opaque “valid” boolean:

1. **Structurally valid:** the artifact matches its local schema and strict validator.
2. **Synthetic-only:** eligible for software, determinism, algorithm-runtime and harness tests only; results must retain synthetic labeling.
3. **Source-qualified:** each required source has pinned bytes/version, license/access and attribution review, CRS/units/resolution/time/no-data gates, plus required independent reference checks.
4. **Model-qualified:** a named domain model accepts the inputs under a declared applicability envelope, with boundary conditions, calibration/validation evidence, uncertainty/sensitivity and negative controls.
5. **Outcome-observed:** an actual observation links method, timing, units, quality/calibration, uncertainty, data digest and location precision/access constraints.
6. **Evaluation-qualified:** the outcome is interpreted against the declared baseline/comparison, sampling method, time windows and analysis plan. Causal claims require the evaluation contract’s stronger study-design gates.
7. **Publication-authorized:** a human/authority record and applicable consent/access policy permit the specific record or suitably aggregated projection to be disclosed.

A later state does not retroactively upgrade earlier evidence. For example, a consented observation does not establish a valid hydrology model; a model output does not become observed evidence; and an approved design is not proof of statutory permission.

## Required cross-repository conformance fixtures

Use small, synthetic, checked-in fixtures for each edge. Every test needs a positive example and mutations that must fail.

| Edge | Positive case | Negative controls |
|---|---|---|
| Snapshot → evaluation plan | Plan pins the exact snapshot digest and same project scope | Changed snapshot byte, wrong scope, baseline/follow-up inversion, unknown metric/method, causal intent without required comparison/preregistration fields |
| Snapshot → Symtropy scenario | Manifest digest matches exact snapshot and exact boundary-condition payload | One-byte snapshot change, one-byte boundary change, wrong frame, excessive feature count, synthetic data relabeled authored, unsupported purpose |
| Evaluation plan → Symtropy | Optional evaluation-plan payload digest, schema URI, ID, snapshot digest and scope all match | Stale plan digest, wrong evaluation ID/scope/snapshot digest, invalid plan schema, missing exact plan bytes |
| Observation → project | Observation references exact project snapshot digest and optional feature ID; location has explicit frame/precision/privacy | Missing units/time/source digest, unit conflict, axis/frame confusion, false calibration state, exact sensitive location with public policy, duplicate ID with changed content |
| Observation → evaluation | Metric ID/unit/method and time windows meet declared outcome contract | Wrong unit, missing method URI, observation outside allowed period, unknown evidence class, observation used as causal proof without study design |
| Decision → source artifacts | Mycelix record references exact project digest and scenario/observation digests, declares actor role/authority basis and policy | Wrong/stale digest, role-authority ambiguity, withdrawn consent, sensitive exact coordinates in public projection, synthetic result represented as observed |
| Full review chain | Read-only review references all exact upstream digest IDs and repeats on the same inputs | Any changed upstream artifact, omitted/conflicting evidence, unsupported model domain, hidden scalar score where metric vector is incomplete |

## Acceptance gates for the shared integration

- A checked-in crosswalk test covers the exact schema URI, current enum values, coordinate-frame labels, digest-field mapping, scope names, and evidence-class boundaries.
- Where a handoff hashes bytes, changing any single byte changes the relevant digest and invalidates the pinned reference. Do not hash reconstructed/pretty-printed JSON when the referenced artifact committed to different bytes.
- A scenario, observation, evaluation plan, recommendation or stewardship record cannot silently overwrite the authoritative design snapshot.
- Synthetic, assumed, missing, observed, modelled and curated evidence remain distinct through import, review, publication, and export.
- A structurally valid schema-qualification or synthetic benchmark is never displayed as a physically qualified result.
- Unknown/stale sources, missing units, unsupported CRS/frame, wrong-scale inputs, invalid consent/access status, and digests that cannot be verified fail closed with actionable errors.
- Any public projection checks both sensitive-location rules and the actual consent/authority reference semantics; schema shape alone does not prove the external authorization is valid.
- Every integration run records source commits, schema versions, fixture digests, toolchain, raw results, and negative-control outcomes.
- No performance superiority claim is inferred from CI pass; browser rendering, native document kernels, GIS interoperability, model validation, field outcomes, and human task studies remain separate benchmark families.

## Current implementation boundaries

- Sol-Atlas PR #191: visual editor, strict v0 snapshot validator and synthetic scene harness.
- Sol-Atlas PR #210: evaluation-plan contract; structural validation does not establish causality.
- Sol-Atlas PR #216: observation/batch envelope and read-only SensorThings mapper; schema validation does not establish ground truth or legally sufficient consent.
- Symtropy PR #2322: exact-byte scenario admission and claim-eligibility gate; no physical landscape solver is qualified by this contract alone.
- Mycelix PR #5298: consent-aware stewardship profile schema/specification; signed runtime event adapter and authorization enforcement remain separate work.
- Symthaea issue #8196: read-only evidence-bound review.

All referenced PRs are drafts. This matrix is an integration/test plan, not a claim that all adapters or runtime authority checks are already implemented.
