# IEAC TUF channel freshness and anti-rollback v0.1

**Status:** Consumer-side TUF freshness adapter, Rust auditor and synthetic high-water corpus. Not a TUF client or signature verifier.  
**Issue:** [IEAC-AVAIL-002 / #427](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/427).  
**Composes with:** #419 / channel completeness #422; #413 / decision-time frontier #415; #347 / #396 trust lifecycle; #406 / #407 common-cause independence.

## 1. Security boundary

A valid, internally consistent checkpoint can still be stale. Monotonic sequence numbers inside one observed channel chain do not protect a relying party from being shown an older, internally consistent chain.

TUF already addresses metadata rollback and freeze attacks. This package does not implement TUF signatures, canonicalization, root rotation cryptography, delegation verification or metadata parsing. It consumes an external TUF verifier report, checks its exact verifier build/policy/trust-bundle pins, then compares the verified state to a separately provisioned policy and persisted high-water state.

The trusted high-water file is a separate CLI argument; it is not embedded in the verifier report or caller request. Its storage/authenticity/atomic update are still operator responsibilities. The report binds the exact raw bytes of the verifier report, policy and state files. Strict JSON parsing rejects duplicate object members.

## 2. Why TUF, not another freshness service

TUF has four required top-level roles: root, targets, snapshot and timestamp. Root establishes trust in other roles; timestamp metadata points to snapshot metadata; snapshot binds the current targets/delegation metadata; targets metadata binds paths, hashes and lengths of target artifacts. Delegated target roles let channel-specific state be authorized separately. [TUF roles and metadata](https://theupdateframework.io/docs/metadata/).

The TUF client workflow fixes an update-start time, requires incremental N+1 root updates verified by predecessor and successor thresholds, rejects metadata rollback, and checks timestamp/snapshot/targets expiration and hash/version consistency. The adapter therefore consumes the verifier's result rather than trying to reproduce these cryptographic rules. [TUF v1.0.36 specification](https://theupdateframework.github.io/specification/v1.0.36/).

Rust implementation selection matters:
- `sigstore-tuf` 0.14.0 describes a pure-Rust TUF client with root chaining, timestamp/snapshot/targets verification, delegated target discovery and offline cache support.
- `tough` 0.24.0 describes TUF 1.0 support but explicitly lists delegated roles as not yet supported.

The actual integration must select a compatible, pinned implementation and prove compatibility against the chosen repository profile. Do not infer support from the crate name alone. [sigstore-tuf](https://docs.rs/sigstore-tuf/latest/sigstore_tuf/) · [tough](https://docs.rs/tough/latest/tough/).

## 3. Inputs

The CLI consumes three independently managed files:

1. **TUF verifier report** — external verifier output: repository identity, current root/timestamp/snapshot metadata, delegated target-role metadata, exact target path/length/hash, channel checkpoint and manifest digests, evaluation-time evidence, verifier provenance and explicit check results.
2. **Operator policy pin** — repository, relying party, channel, tenant, purpose, ledger, sequence namespace, delegated roles, target paths, maximum timestamp expiry horizon, pinned TUF verifier build/policy/trust-bundle and pinned clock verifier build/policy/trust-bundle.
3. **Persisted high-water state** — last accepted root/timestamp/snapshot versions and digests, delegated target-role versions/digests, exact manifest/checkpoint target digests, ledger epoch and last accepted sequence.

The TUF report is not trusted just because it says `verified`. Its verifier build, policy and trust bundle must match the out-of-band policy. Each evaluation-time fact is separately checked against the pinned clock verifier, policy and trust bundle.

## 4. Checks performed by this adapter

- Exact repository, relying-party, channel, tenant, purpose, ledger and sequence-namespace binding.
- Exact manifest and checkpoint target-role/path/length/SHA-256 binding between the verifier report, policy and target metadata.
- Monotonic root, timestamp, snapshot and delegated-target metadata versions compared with persisted high-water state.
- Same-version metadata equivocation detection: equal version with a different metadata digest is rejected.
- Incremental root-chain structure: each root transition is N → N+1 and carries externally verified old-root and new-root threshold results; the chain must end at the reported root digest.
- Metadata expiry and policy-bounded timestamp expiry horizon using a fixed evaluation-time interval. If the interval overlaps expiry, the result is unknown rather than current.
- Channel sequence rollback and same-sequence checkpoint equivocation checks.
- Ledger epoch changes require an externally verified transition whose authorization target is the exact channel manifest digest.
- Strict JSON duplicate-member rejection, unknown-field rejection and exact raw-input SHA-256 binding.

The report's `current` status means the external TUF verifier facts satisfy the pinned policy and no local high-water rollback/expiry conflict was found. It does not mean the channel is complete: that remains #419/#422. It does not mean the underlying industrial evidence is true.

## 5. What this tool does not do

- It does not fetch repositories or implement TUF metadata/signature verification.
- It does not implement TUF's full trusted metadata update state machine.
- It does not establish that the local high-water file cannot be replaced, deleted or rolled back. Protect that file with an appropriate operator-controlled monotonic persistence mechanism.
- It does not prove the access channel captured every access event; see #422.
- It does not prove a source measurement or physical outcome is true.
- It does not grant decision or actuation authority.

Missing/lost high-water state is a hard refusal. It must not be treated as a fresh bootstrap. Recovery requires an explicit operator-controlled bootstrap/rotation ceremony.

## 6. Run

```sh
cargo fmt --manifest-path tools/industry-tuf-freshness-auditor/Cargo.toml -- --check
cargo clippy --manifest-path tools/industry-tuf-freshness-auditor/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tools/industry-tuf-freshness-auditor/Cargo.toml --locked
cargo run --manifest-path tools/industry-tuf-freshness-auditor/Cargo.toml --locked -- \
  schemas/examples/ieac-tuf-verifier-report-synthetic-v0.1.json \
  schemas/examples/ieac-tuf-freshness-policy-synthetic-v0.1.json \
  schemas/examples/ieac-tuf-high-water-state-synthetic-v0.1.json \
  /tmp/ieac-tuf-freshness-report.json
```

Exit codes:
- `0`: current under pinned TUF policy and persisted high-water state;
- `3`: valid inputs but freshness/evidence status unknown (e.g., expired or unverified metadata);
- `2`: invalid inputs, policy mismatch, rollback, equivocation, target substitution, broken root chain or failed state binding.

## 7. Frozen synthetic benchmark

The starter fixture advances:
- root version 3 (unchanged);
- timestamp 13 → 14;
- snapshot 8 → 9;
- manifest delegated role 4 → 5;
- checkpoint delegated role 7 → 8;
- channel sequence 118 → 120 in the same ledger epoch.

The test suite covers timestamp rollback, same-version digest equivocation, expired metadata, excessive expiry horizon, verifier trust-bundle substitution, delegated target rollback, missing root-rotation chain, a valid N+1 root rotation, target-path substitution, channel sequence rollback, same-sequence checkpoint equivocation, unsupported verification checks, duplicate JSON members and unapproved epoch reset.

Promotion targets:
- 100% rejection of the frozen rollback, same-version equivocation, target substitution, root-chain discontinuity and unauthorized epoch-reset controls.
- 100% unknown for expiry/freshness boundaries or unavailable external verifier facts.
- 100% binding of exact repository, root, timestamp, snapshot, delegated role, target path/length/hash, channel identity, epoch and high-water state.
- Zero current status when the persisted high-water state is absent.
- Report p50/p95/p99 metadata verification and adapter latency, target count, root-chain length, offline window, state bytes and recovery time.
- Compare sequence-only, signed-checkpoint-only, wall-clock-expiry-only and TUF-backed persisted-high-water baselines.

The 100% values are frozen synthetic conformance targets, not estimates of real-world compromise probability.

## 8. Claim ceiling

This tranche establishes a freshness/anti-rollback classification under the exact external TUF verifier report, policy pins and persisted high-water state. It does not prove that those inputs were honestly provisioned, that every target role is independent, that channel capture is complete or that any industrial decision is correct. `action_authority = none`.
