# IEAC Runtime Admission v0.1

This crate implements the first executable runtime boundary for the Industry Engineering Assurance Contract (IEAC). It decides whether one exact request, under one exact input and authority snapshot, may proceed to a bounded descriptive or advisory step.

It is an admission gate, not a physical solver, model evaluator, optimizer, safety case, SCADA/MES controller, scheduling engine, or actuation service.

## Flow

```text
static IEAC profile + operator-provisioned profile pin + request + measurement snapshot + revocation/authorization snapshot
  -> profile contract validation
  -> profile-pin validation against a separate trusted input
  -> exact identity and digest binding
  -> validity and authority checks
  -> per-condition unit, interval, source and freshness checks
  -> candidate admission result
  -> separate report reconstruction
  -> sealed report
```

## Commands

Run the evaluator:

```sh
cargo run --manifest-path tools/industry-runtime-admission/Cargo.toml --locked -- \
  profile.json trusted-profile-pin.json request.json new-report.json
```

Run the independent checker as a separate process:

```sh
cargo run --manifest-path tools/industry-runtime-admission/Cargo.toml --locked --bin industry-runtime-check -- \
  profile.json trusted-profile-pin.json request.json sealed-report.json
```

The evaluator requires a new report destination and uses create-new semantics; it refuses to overwrite a file. Exit code 0 means the bounded request was admitted, 1 means abstain/reject, and 2 means malformed input, failed internal verification, or I/O failure. Abstain/reject reports are also written where the report contract can be produced, so callers can inspect stable reason codes.

## v0.1 policy

- D0 descriptive supports only view and annotate.
- D1 advisory supports only view, annotate, recommend, rank_options and prepare_plan; the profile must be qualified_for_profile, require human approval and have actuation disabled.
- D2 delegated scheduling and D3 autonomous execution are always rejected.
- The request binds the exact profile digest, model/evaluator/configuration/topology artifact IDs and digests, asset/instance, snapshot and revocation snapshot.
- An independent profile-pin file must be provisioned from a trusted local registry/control plane. It binds profile ID, expected profile SHA-256, registry ID and active/revoked status. The request cannot supply or widen this pin. The report also records the exact profile-pin digest.
- The model ID must be listed by the profile. Artifact bindings must be explicit, digest-matched and have rights_state=verified.
- Measurement unit equality is exact. No automatic unit conversion or inference is performed.
- Each operating-envelope variable needs a single measurement, value plus lower/upper uncertainty interval, observation timestamp, source artifact ID and source SHA-256.
- The profile must declare max_age_seconds for a measured condition. Missing freshness policy causes abstention; stale evidence causes abstention; future-dated or malformed evidence is rejected/abstained with a stable reason.
- A value outside a declared bound is rejected. A measurement interval crossing a bound while its central value remains inside is marginal and causes abstention.
- Unknown/unavailable revocation state or authorization state causes abstention. Denial, revocation, digest mismatch, identity mismatch, unit mismatch and authority escalation reject.
- The revocation snapshot must identify a profile-pinned authorization_source artifact and digest. Its observed_at is checked against max_authority_snapshot_age_seconds from the pinned profile; the request cannot widen the freshness window. Missing/unbound authority source rejects; stale or indeterminate authority-source time abstains.
- Admission never executes a recommendation or renews a profile. It is valid only for this request and time snapshot.

## Digests and determinism

Digest fields use SHA-256 over serde_json's compact UTF-8 serialization of the parsed JSON Value produced by the pinned serde_json 1.0.150 dependency. Without the preserve_order feature, object keys are serialized in map order. This is a versioned Sol-Atlas wire digest, not a claim of RFC 8785/JCS cross-language canonicalization. Interoperability work should migrate to a separately tested JCS implementation and publish a format version before cross-language digest compatibility is claimed.

The report digest is calculated after the independent-checker status is set and excludes only the report_sha256 field itself.

## Independent checker

The candidate evaluator and independent checker are separate decision paths. The checker reconstructs disposition, action ceiling, per-condition states and reason codes from the original profile, separately provisioned profile pin and request. It does not call the candidate evaluator. A second binary can replay a sealed report from the original inputs and verify the report digest and schema.

The profile-pin file is an out-of-band trust input and must be protected by the host/operator (for example, provisioned read-only from a trusted registry, not accepted from the request submitter). v0.1 validates pin contents and binding but does not cryptographically authenticate the pin itself. Signed pins, key rotation and a trusted registry transport remain required for distributed deployments.

The profile-pinned authorization_source binding and max_authority_snapshot_age_seconds policy verify that the request names the expected source identity and uses a bounded freshness window. They do **not** authenticate the supplied revocation payload or prove that it was signed by that source. v0.1 therefore must not be used as a production trust anchor; signed authority snapshots, key rotation, anti-replay sequence state and source-authentication verification remain required before any higher-risk deployment.

The checker shares low-level profile validation, JSON parsing, UTC parsing and SHA-256 primitives. It does not establish source-system truth or that two evidence artifacts are genuinely independent. Those require upstream provenance attestations and external source audits.

## Synthetic cross-vertical fixtures

The checked-in runtime fixture demonstrates a D0 telecom backup-energy annotation using synthetic identifiers, source URIs, digests and measurements. Rust tests generate additional water-food-energy, industrial-maintenance and building cases using the same evaluator and independent checker. All are contract tests, not field validation.

## Benchmark programme

- 100% agreement with hand-calculated positive and negative fixtures.
- Zero false admissions in the frozen negative-control corpus.
- 100% independent-checker agreement on all frozen fixtures.
- Boundary tests immediately below, at and above every numeric and time limit.
- Metamorphic digest checks: input key reordering is invariant under the pinned serializer; any decision-relevant change changes the appropriate input digest.
- Report admission/abstention/rejection by failure family, not one aggregate accuracy score.
- Measure false-confident admission, coverage, time-to-abstention, reason-code correctness, p50/p95/p99 latency, memory and CPU energy separately from physical solver runtime.

Do not claim physical validity, safety, service availability, regulatory conformity, SLA compliance, field validation, SOTA or production qualification from these synthetic fixtures. Extending D0/D1 across customer-controlled sources and a shadow pilot is a later gate; D2/D3 remain out of scope.
