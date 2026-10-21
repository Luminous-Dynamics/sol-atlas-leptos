# Landscape Evaluation Protocol v0

**Status:** schema + strict Rust validator + synthetic fixture; no field measurements are included.  
**Module:** `src/landscape_evaluation.rs`.  
**Purpose:** make landscape interventions measurable and comparable over time without confusing a validated plan with evidence that a project worked.

## Paradigm: design → intervention → observation → measured outcome

A rendered concept is only a hypothesis. This protocol captures the exact design snapshot and defines baseline/follow-up windows, intervention/comparison arms, metric methods, units, sample minima, uncertainty plans, analysis intent, and publication constraints before outcome claims are made.

The critical boundary is epistemic:
- A **design snapshot** says what was proposed.
- An **evaluation plan** says how the proposal will be tested.
- A **model output** says what a declared model estimates under its assumptions.
- An **observation** says what a defined method measured.
- An **evaluation result** says how the measured evidence compares with a baseline or comparator under a stated analysis.
None of these artifact types may silently substitute for another.

## Implementation

- JSON Schema: [landscape-evaluation-plan-v0.schema.json](../schemas/landscape-evaluation-plan-v0.schema.json).
- Synthetic example: [evaluation-plan-v0.json](../fixtures/landscape/evaluation-plan-v0.json).
- Rust validator and tests: [src/landscape_evaluation.rs](../src/landscape_evaluation.rs).
- Parent work item: [LAND-EXPERIMENT-001 #209](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/209).

The Rust validator is a structural and policy-contract boundary. It checks the snapshot digest shape, time ordering, unique arms/sites/outcome IDs, methods/units/sample minima, supported study-design/analysis combinations, preregistration shape for causal-estimation intent, and basic publication/consent rules. It intentionally does **not** prove the digests resolve to trusted records, the method is scientifically adequate, the sites were actually matched, consent is legally valid, or the estimated effects are causal.

## Study designs and claim ceiling

| Study design | Supported use | Not automatically established |
|---|---|---|
| Descriptive monitoring | Track observed conditions over declared windows | Change caused by an intervention |
| Before–after | Compare a site's measurements across time | Causal effect; season, weather and co-interventions can explain change |
| Matched comparison | Compare intervention and matched site(s) across time | No residual confounding; matching quality still needs evidence |
| Before–after control–impact (BACI) | Predeclared difference-in-differences style analysis when assumptions hold | Causality by naming the design; parallel trends and other assumptions still need validation |

Causal-estimation intent therefore requires a matched-comparison or BACI design, an intervention and comparison arm, a preregistered/approved plan, declared confounders, sensitivity checks, and independent review. The preregistration timestamp must be strictly earlier than the follow-up window start; a timestamp at or after that start is rejected. This gate only qualifies the *plan shape*; results and inference require a later analysis/review artifact.

## Native checker

A small feature-gated native CLI validates the same plan contract outside the browser:

```sh
cargo run --features landscape-eval-check --bin landscape-evaluation-check -- fixtures/landscape/evaluation-plan-v0.json
```

It emits a deterministic JSON report with input SHA-256, byte length, evaluation ID, validity and errors. Exit code 0 means the structure passed; exit code 2 means validation failed. The report conforms to [landscape-evaluation-validation-report-v0.schema.json](../schemas/landscape-evaluation-validation-report-v0.schema.json). This reports structural qualification only.

## Evaluation-plan contents

- A SHA-256 reference to one immutable design snapshot.
- Project scope and versioned time windows with baseline ending before follow-up begins.
- Unique intervention/reference/comparison arms with opaque site-reference digests rather than inline coordinates.
- Each outcome's metric ID, unit, method ID/URI, sampling design, minimum observations per arm/window, uncertainty method, target direction and optional target band.
- Privacy classification, publication mode, consent-record reference and aggregation policy.
- An explicit interpretation-limit marker: `validation-does-not-prove-impact-or-causality`.

### Privacy boundary

Public publication is rejected by the validator unless location sensitivity is explicitly public and consent is approved or not required. An approved or withdrawn consent state must carry an opaque authoritative-record reference; the status string alone is not accepted as proof. Restricted/sensitive site references should stay opaque; an aggregate-only plan must declare how aggregation/redaction is to be performed. A withdrawn consent status blocks public and aggregate publication in this v0 contract. This structural rule is not a replacement for Mycelix's authoritative consent/permissions enforcement.

## Interoperability and repository responsibilities

- **Sol-Atlas:** owns the evaluation-plan schema, links to an exact design snapshot, displays the plan, and maintains spatial/project artifact integrity.
- **Symtropy:** produces versioned model-result artifacts tied to the same design digest and named model/solver version. Model predictions stay distinct from observations.
- **Symthaea:** reviews protocol completeness, evidence gaps, uncertainty and inference limits; it must not infer causation just because a plan declares a causal-estimation intent.
- **Mycelix:** supplies the authoritative consent/authority record and contributor-owned observation/provenance path. This protocol only references opaque record identifiers. Follow-up mapping is tracked in [Mycelix Observation V2/evidence graph #419](https://github.com/Luminous-Dynamics/mycelix/issues/419).

Use existing open standards rather than creating a parallel sensor network API:
- OGC SensorThings API Part 1 models observations, observed properties, sensors, procedures, units and times: https://www.ogc.org/standards/sensorthings/
- OGC STAplus extends SensorThings with contributor ownership, data licensing, relations and grouped observations useful in citizen-science settings: https://www.ogc.org/standards/sensor-things-api-extension/
- OGC JSON-FG provides richer typed features, temporal characteristics and alternate CRS support beyond GeoJSON's constraints: https://www.ogc.org/standards/json-fg/

Adapters are future work; the current module does not implement these network protocols. It should map to them rather than redefining their semantics.

## Acceptance gates

- JSON Schema is valid JSON and its stable constants match the Rust module.
- The canonical plan fixture passes; negative tests cover preregistration after follow-up starts, overlapping windows, reversed windows, unsupported causal design, absent preregistration, retroactive registration, duplicate arms/site references/outcome IDs, blank units, target-band inversion, sensitive/public location conflict, unapproved or withdrawn consent, and unknown fields.
- Identical plan bytes yield identical validation results.
- Schema breaking changes require a new version; changing a formula or study design never rewrites previous records.
- Native tests, `cargo fmt --all --check` and the WASM target build pass on the exact commit.
- No field-performance claim is inferred from a valid evaluation plan.
