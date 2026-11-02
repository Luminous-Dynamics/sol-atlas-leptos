# IEAC in-process TUF verifier v0.1

**Status:** First in-process TUF cryptographic-verification increment, using pinned `sigstore-tuf` 0.15.1.  
**Parent work:** [IEAC-TUF-003 / #441](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/441).  
**Composes with:** TUF structural policy/high-water prototype #427 / PR #432; bounded channel completeness #419 / PR #422; decision-time frontier #413 / PR #415; common-cause independence #423 / PR #407.

## 1. Security boundary

The earlier structural adapter accepted an external JSON report containing self-asserted verifier flags and build/policy/trust-bundle digests. Those fields do not authenticate the process that produced them. This package removes that report boundary for TUF cryptographic verification.

It directly invokes `sigstore-tuf` 0.15.1 in process:

1. Read an operator-provisioned bootstrap `root.json` and verify its exact raw bytes against a separately configured SHA-256 pin.
2. Construct the TUF repository transport.
3. Create an `Updater` from the pinned root.
4. Run the full root → timestamp → snapshot → targets refresh at one fixed update-start time.
5. Resolve each of the four required target paths through TUF's top-level and delegated-role search.
6. Verify/download target bytes or re-verify cached target bytes against authenticated target length and hashes.
7. Emit a report with exact repository, metadata versions and four target hashes/lengths.

There are no report-supplied `verified=true` fields, verifier-build strings or threshold booleans used as proof. The cryptographic checks happen inside the same process that emits the result. The root file is rejected unless it is a regular file no larger than 512,000 bytes and its SHA-256 exactly matches the pin in the operator-provisioned trust policy.

## 2. Why this Rust implementation

`sigstore-tuf` 0.15.1 is a pure-Rust implementation aligned with TUF 1.0 and interoperable with the Python reference implementation and TUF-on-CI style repositories. It has a pluggable `Repository` transport, root-chain refresh, threshold verification, metadata expiry and rollback checks, delegation resolution, target integrity validation and cache re-verification.

The direct adapter uses the crate's:
- `Updater::new(repository, bootstrap_root_bytes)`;
- `Updater::with_config` to bound target byte length;
- `Updater::with_store(FileStore)` to persist metadata/target cache;
- `Updater::refresh(fixed_update_start)` to verify the metadata chain;
- `get_target_info(path, fixed_update_start)` to resolve a top-level or delegated target;
- `find_cached_target` / `download_target` to verify bytes against the authenticated target metadata.

The HTTP client is configured with a 10-second connection timeout, a 45-second overall timeout and redirects disabled. The URL must be HTTPS and cannot contain userinfo, query parameters or a fragment. The bootstrap root must be a regular file no larger than 512,000 bytes. Target paths are restricted to normalized ASCII repository-relative segments; absolute paths, traversal, percent-encoded separators/dot segments, whitespace, ambiguous separators and duplicate target paths are rejected. The TUF transport receives the configured maximum target length, so oversized target downloads are refused before target bytes are buffered.

References: [TUF 1.0.36 specification](https://theupdateframework.github.io/specification/v1.0.36/) · [sigstore-tuf 0.15.1](https://docs.rs/crate/sigstore-tuf/0.15.1) · [Updater API](https://docs.rs/sigstore-tuf/0.15.1/sigstore_tuf/client/struct.Updater.html).

## 3. Canonicalization boundary

TUF metadata signatures use TUF's securesystemslib/OLPC canonical JSON serialization rules. They do **not** use RFC 8785 JCS. IEAC must not reuse its restricted JCS implementation to canonicalize TUF signed metadata.

This package deliberately delegates canonicalization and signature checks to `sigstore-tuf`; it does not parse and re-serialize signed metadata to verify a signature itself.

## 4. Four required IEAC targets

Every request must declare exactly these distinct target identifiers and repository-relative paths:

1. `channel_manifest`
2. `checkpoint_state`
3. `completeness_attestation`
4. `availability_audit_report`

For each target, the verifier resolves a matching authorized target entry from verified TUF top-level or delegated targets metadata. It checks the configured maximum byte length and optional operator-pinned digest/length, then returns the authenticated digest and length. Target data is fetched only after its TUF metadata has authorized the path.

The channel-completeness semantics remain the responsibility of #419/#422. A TUF target hash proves that the target bytes match the bytes named by authenticated metadata. It does not prove the completeness attestation's zero-match query is truthful or that every access route was instrumented; the independent #422 auditor must check that contract.

## 5. Offline cache and high-water limitation

The metadata cache uses the TUF library's `FileStore`. The library re-verifies cached metadata from the out-of-band pinned bootstrap root and verifies cached target bytes against the authenticated target metadata. The offline unit test refreshes once using the reference repository and then reopens the same cache through `StoreRepository`, with no network transport.

Important limitation: the cache is not, by itself, an authenticated monotonic high-water store. If an attacker can delete or replace the entire cache and an operator bootstraps with the same root, cache rollback protection can be lost. FileStore gives atomic per-file cache writes, not a cross-file transaction or a TPM-backed anti-rollback anchor.

This first increment therefore reports `persistent_high_water = not_managed_by_this_binary`. It does not write or advance a trusted high-water state. The separate #427/#432 layer remains structural-only until the persistence/authentication design in #441 is completed. Do not interpret this package as end-to-end freeze resistance for a relying party that does not protect its bootstrap root and persistent state.

## 6. Independent reference corpus

The tests vendor unmodified reference metadata generated by `python-tuf` from the `tough` project's fixture set, with the original attribution and MIT OR Apache-2.0 licensing notice preserved under `tests/fixtures/ATTRIBUTION.md`.

The frozen corpus includes:
- a signed root, timestamp, snapshot and top-level targets metadata;
- a delegated `role1` and `role2` chain;
- top-level file1/file2 targets and a delegated file3 target;
- negative roots with no signatures, a corrupted signature and duplicate signatures from one key against a threshold of two.

Tests:
- reference root self-verification plus end-to-end refresh;
- successful resolution of a target absent from top-level targets but authorized through delegation;
- tampered target bytes rejected even though metadata is valid;
- offline cache re-open and re-verification;
- malformed roots fail bootstrap closed;
- duplicate known request fields are rejected by deserialization.

This is a cross-implementation conformance baseline, not yet IEAC's complete threat corpus. Additional coverage must include root rotation chains, expiry, rollback/freeze, delegation path matching and terminating-role behavior, repository/cache rollback, four-target substitution and common-cause failure modes.

## 7. Run

```sh
cargo fmt --manifest-path tools/industry-tuf-inprocess-verifier/Cargo.toml -- --check
cargo clippy --manifest-path tools/industry-tuf-inprocess-verifier/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tools/industry-tuf-inprocess-verifier/Cargo.toml --locked

cargo run --manifest-path tools/industry-tuf-inprocess-verifier/Cargo.toml --locked -- \
  request.json report.json
```

The report output is created with create-new semantics; an existing output file is not overwritten.

## 8. Report authenticity boundary

The emitted report is an **unsigned audit output**. It records what this invocation verified, but it is not a portable proof of that verification if copied to another system or detached from the process/binary that produced it. A downstream admission service must either rerun this verifier against the protected policy/root/cache or consume a separately authenticated verification bundle. Do not accept the report's `cryptographically_verified` string alone as authority.

The report explicitly states:
- `report_authentication = unsigned_audit_output`;
- `standalone_admission_proof = no_requires_reverification_or_authenticated_binding_before_offline_admission`.

A signed evidence bundle / independent offline verifier and an authenticated monotonic high-water store remain follow-on work under #441.

## 9. Claim ceiling

A `cryptographically_verified` report from this package means the pinned `sigstore-tuf` implementation verified the metadata chain, signatures/thresholds, expiry/version relationships and delegation path under the supplied out-of-band root, and that the returned target bytes match authenticated length/hash metadata.

It does not mean:
- the local bootstrap root or FileStore directory is protected against administrator/host compromise;
- the relying party has persisted a rollback-resistant high-water state;
- a channel captured every relevant access event;
- source measurements or outcome data are true;
- independent witnesses are actually independent (#423);
- the original decision was correct or safe.

`action_authority = none`.
