# IEAC deterministic verification capsule packer v0.1

**Status:** Offline Rust packer with strict build template, pack receipt and fail-closed input tests.  
**Issue:** [IEAC-EVIDENCE-002 / #456](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/456).  
**Composes with:** capsule contract #445 / PR #447; offline replay auditor PR #455; in-process TUF verifier #441 / PR #444.

## 1. Construction is not verification

The packer reads a staging tree and a build template, checks the root and policy bytes against an out-of-band trust pin, computes exact file commitments and writes a deterministic capsule directory.

It does **not** verify TUF signatures, thresholds, metadata delegation, expiry or target authorization. Its receipt is always `status = packed_unverified` and `verification_performed = false`. A separate offline verifier must replay the capsule.

The packer does not access the network and contains no HTTP client.

## 2. Inputs

CLI:

```sh
cargo fmt --manifest-path tools/industry-verification-capsule-builder/Cargo.toml -- --check
cargo clippy --manifest-path tools/industry-verification-capsule-builder/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tools/industry-verification-capsule-builder/Cargo.toml --locked

cargo run --manifest-path tools/industry-verification-capsule-builder/Cargo.toml --locked -- \
  ./staging \
  ./capsule-build-template.json \
  ./operator-trust-pin.json \
  ./new-capsule \
  ./pack-receipt.json \
  [./local-cas]
```

- **Staging root:** only the exact files named by the template are permitted. Symlinks, non-regular files, unsafe paths, duplicate source paths and extra files are rejected.
- **Build template:** maps the root, policy, TUF metadata artifacts and four target paths to their source and capsule paths. Target content mode is explicit: embedded, referenced or withheld.
- **Trust pin:** must be provisioned separately. It fixes repository origin/identity, root and policy digests, policy scope/kind, clock-verifier identity, evaluation-time interval, allowed capsule modes, size/count ceilings and exact target paths.
- **Local CAS:** required for referenced targets. Each CAS object must be named by its SHA-256 digest and match the target length/hash. It is read locally; no network fallback exists.

## 3. Deterministic output

The packer:
1. Reads the template and trust pin once.
2. Reads exact source bytes into memory, checking the bootstrap root and policy digests against the external pin.
3. Computes raw lengths and SHA-256 for every metadata file and embedded target.
4. Verifies referenced targets against local CAS blobs; withheld targets contribute no payload bytes.
5. Requires exactly one of each four target IDs, unique target paths, unique metadata repository names/capsule destinations, and sequential root-history versions.
6. Requires the staging file set to equal the template source set exactly.
7. Sorts metadata artifacts and targets by stable keys and file inventory by ASCII path.
8. Computes RFC 8785 JCS for the sorted file-inventory array only.
9. Writes payload files and `capsule.json` with create-new semantics, fsyncs each file and refuses an existing output directory.
10. Writes a separate pack receipt with hashes of the template, trust pin, manifest and inventory.

Equivalent inputs with the same template, pin and source bytes produce the same manifest and inventory commitments. The output directory and receipt are never overwritten. A failed write removes its incomplete output directory.

TUF metadata remains in its original signed bytes. TUF's securesystemslib/OLPC canonical JSON is handled by `sigstore-tuf` in the separate offline auditor; the packer does not parse or reserialize TUF signatures using JCS.

## 4. Privacy modes

- **Self-contained:** all four target files are embedded; the template must declare fully replayable.
- **Metadata-complete / target-referenced:** all target commitments are present; each referenced blob must exist in the local CAS during packing. It is not embedded in the capsule, so replay still requires the local CAS.
- **Commitment-only:** all four target bytes are withheld. The packer copies no target bytes and generates no reference locator.
- **Mixed:** mode consistency is checked against the per-target modes. Any withheld target cannot claim stronger than commitment-only replayability.

The packer never silently downgrades an embedded target to a reference or a commitment. Missing/corrupted referenced blobs fail packaging rather than being silently treated as verified.

## 5. Pack receipt and independent audit

The pack receipt is an unsigned, non-authoritative packaging record. It includes:
- template and trust-pin digests;
- output capsule path and mode;
- capsule manifest and inventory digests;
- per-file length/hash;
- per-target content mode and embedded indicator;
- explicit `verification_performed = false`.

After packaging, run the separate offline auditor:

```sh
cargo run --manifest-path tools/industry-verification-capsule-offline/Cargo.toml --locked -- \
  ./new-capsule \
  ./operator-trust-pin.json \
  ./offline-audit-report.json \
  [./local-cas]
```

The offline auditor recomputes TUF metadata and target integrity. Its report remains unsigned audit output and does not create a protected monotonic high-water state or evaluate IEAC domain policy. The clock-verification identity is pinned but is not itself verified by either this packer or the current TUF-only replay binary.

## 6. Promotion benchmarks

The packer should reach:
- 100% rejection of frozen path/symlink/substitution/omission/duplicate-role/root-pin/policy-pin negatives.
- 100% deterministic manifest and inventory digest across repeated builds with identical inputs.
- Zero verified/admitted status from packer-only execution.
- Zero replayable label for missing embedded/referenced bytes.
- 100% agreement between packer output and the separate offline auditor for supported signed TUF fixture corpora.

Measure packing throughput, peak RSS, source bytes read, capsule size by privacy mode, p50/p95/p99 pack time, output collision behavior and failure-cleanup success. Report packaging, TUF replay, target-byte availability and IEAC domain policy evaluation separately.

Synthetic template hashes are fixture data only. They are not cryptographic corpus results.

## 7. Claim ceiling

A successful pack means only that the input bytes were assembled into a deterministic content-addressed capsule consistent with the supplied out-of-band root/policy pins and template. It does not verify TUF signatures or thresholds, prove completeness of the evidence channel, verify clock evidence, evaluate industrial policy, establish source truth or decision correctness, or grant action authority.
