# CityMaker owner-payload adapter v0.1

**Status:** synthetic-first read-only Rust adapter; exact-head qualification pending  
**Issue:** Sol-Atlas #637  
**Owner verifier:** Sol-Atlas #500 / #512  
**Scope binder:** Sol-Atlas #623 / PR #624  
**Branch:** research/city-owner-payload-adapter-v0.1

## Purpose

Join a CityMaker owner-artifact envelope to one exact result row from the independently implemented source-payload verifier.

The scope binder and byte verifier are separate owners:
- #624 binds the owner/contract/schema, source revision, receipt, subject and common district scope. It deliberately does not verify payload bytes.
- #500/#512 owns the caller-supplied resolver, streaming, byte counting, payload SHA-256, byte-length comparison, resource limits and typed per-record outcomes.
- This adapter maps an exact owner artifact to an exact descriptor record key, validates the report and preserves the separate states.

## Hard boundary: no duplicate verifier

This crate does not open payload objects, make network requests or hash source payload bytes. It consumes the #500/#512 report as input. It computes SHA-256 only over the exact raw JSON report bytes supplied to it, to pin the report artifact reference and declared report digest. It never hashes a consumer-reserialized owner JSON object and treats that as the original source payload.

The expected mapping is explicit:

**owner artifact ID → descriptor record kind + case record ID → exact record key**

For example, an artifact descriptor with record kind artifact and case record ID A-17 must map to artifact:A-17. A matching payload digest string is not a substitute for this mapping.

## Required cross-checks

For every payload-required owner artifact:
- owner repository, contract, version, schema digest algorithm and schema artifact digest;
- exact source revision, receipt ID, subject ID and subject digest;
- common scope digest;
- descriptor record kind, case-record ID and exact record key;
- exact descriptor case ID/representation digest and catalog ID/representation digest;
- digest basis and algorithm;
- exact object reference, expected digest and expected byte length;
- one unique matching verifier result row;
- verifier resolver-open state, expected/observed digest, expected/observed byte length, byte count, and typed disposition;
- report raw-byte digest, object reference, report/catalog identity, verifier build identity and report denominators.

A report-level success cannot promote a missing, unattempted, resource-limited, unavailable or failed record. A duplicate record key is rejected. Reusing one object reference with conflicting declared digests rejects every affected mapping; there is no last-write-wins behavior.

## Keep the evidence axes separate

The adapter reports at least three distinct things:

1. **Raw byte equality:** whether the exact verifier result row establishes that bytes read by the owner verifier match the descriptor's expected SHA-256 and byte length.
2. **Owner/scope binding:** the existing binder's Bound, Unknown, NotComparable or Unsupported result, plus the raw owner disposition.
3. **Rights/use declaration:** the owner-declared rights state and whether it declares the required use permitted. Rights authentication remains not_performed.

Byte equality can be Verified while the owner disposition is Revoked or Blocked, scope is NotComparable, rights are restricted, or issuer trust is unknown. Those facts must remain independently visible. Byte equality grants no legal rights, issuer authentication, currentness, source completeness, technical correctness, professional approval or action authority.

Report integrity and report disposition are also distinct: a raw-byte-intact report can legitimately contain an owner verifier invalid_input result. Do not confuse “the report bytes are intact” with “the report's verification operation was valid or complete.”

## Corpus

The machine-readable corpus CITY_OWNER_PAYLOAD_ADAPTER_CORPUS_V0_1.json contains 24 synthetic cases and exact raw report fixtures, including:
- exact match, empty payload and partial-read success;
- digest mutation, truncation, extension, wrong object reference and owner-envelope digest disagreement;
- missing object, restricted rights, stream errors and hard limits;
- unsupported digest basis/algorithm and invalid report disposition;
- scope mismatch, stale/non-comparable owner binding, revoked/disputed/blocked owner dispositions;
- duplicate object references with conflicting digests;
- separate schema-artifact and instance-payload outcomes.

All payload bytes, reports, owners, IDs, timestamps, revisions, scopes and object references are synthetic fixtures. No private or restricted source material is included.

## CLI and receipt

The CLI reads the frozen corpus, evaluates each case from its input only, compares against independently authored expected outputs and emits a JSON receipt bound to:
- exact source commit;
- exact corpus bytes SHA-256;
- schema and corpus IDs;
- per-case result and failure denominator;
- a synthetic-only claim ceiling and no-action-authority declaration.

Run the following commands from the repository root:

    cargo fmt --manifest-path tools/city-owner-payload-adapter/Cargo.toml --all -- --check
    cargo test --locked --manifest-path tools/city-owner-payload-adapter/Cargo.toml
    cargo clippy --locked --manifest-path tools/city-owner-payload-adapter/Cargo.toml --all-targets -- -D warnings
    cargo run --locked --quiet --manifest-path tools/city-owner-payload-adapter/Cargo.toml -- docs/research/city-owner-payload-adapter/CITY_OWNER_PAYLOAD_ADAPTER_CORPUS_V0_1.json --source-commit 0000000000000000000000000000000000000000

Replace the final example commit with the exact 40-character source commit being evaluated.

An E1 receipt proves only the declared adapter semantics on these frozen synthetic reports. It does not qualify a real source, legal rights, the completeness of a district's owner evidence, a physical asset or any city-service outcome.

## Exact source-descriptor/report compatibility

The input fixture now uses the #500 source-descriptor record shape rather than a simplified payload object:

- `SourceRecordDescriptor`: record kind, case-record ID, owner namespace, native identity components, source payload, source snapshot, observation time, rights and attestation.
- The adapter-only `descriptor_binding` explicitly pins record key, case/catalogue IDs and representation digests, owner namespace, owner receipt, source revision, subject and schema pin.
- The report fixture carries the actual verifier's `catalog_validation` shape, including descriptor count, missing/extra counts, `identity_sha256_by_case_record`, `native_identity_namespaces`, `native_identity_components`, validation disposition, findings and independent rights/attestation/action-authority axes.
- Each `payload_results` row uses the verifier's optional digest/object/length fields, bytes hashed, match predicates, resolver-open flag and snake-case disposition.

The adapter cross-checks the catalogue's per-record native namespace and component map against the exact descriptor binding. This does not independently authenticate the catalogue producer; it proves only internal consistency of the supplied report bytes and the caller-declared report reference.

### Optional expected byte length

The verifier's expected byte length is optional for some digest bases. When it is absent, a successful SHA-256 comparison can establish raw byte equality, but the adapter reports `byte_length_comparison_state=not_declared`. It never claims a length comparison occurred. Where a length is declared, expected/observed length and bytes-hashed must agree.

### Report integrity versus verification disposition

An intact raw report can have disposition `incomplete_or_indeterminate` or `invalid_input`. A single verified record can remain individually Verified in an incomplete catalogue (for example, when owner rights remain Restricted); an exact per-record mismatch remains Mismatch even when the report overall is `invalid_input`. These axes are reported independently.

## Owner contract versus payload schema

The CityMaker owner-envelope `contract_id`/`contract_version` and the source descriptor's `source_payload.schema_id`/`schema_version` are different identity axes. The adapter pins both independently in `descriptor_binding`:
- `owner_contract_id` and `owner_contract_version` must match the owner envelope;
- `schema_id` and `schema_version` must match the exact source payload descriptor;
- `schema_artifact_digest` pins the owner contract/schema artifact reference;
- `source_payload.schema_sha256` pins the source payload's declared data schema when present.

A contract ID is not silently accepted as the payload schema ID. Mutation controls prove that either cross-wire is rejected while the underlying raw byte equality remains visible as a separate fact.
