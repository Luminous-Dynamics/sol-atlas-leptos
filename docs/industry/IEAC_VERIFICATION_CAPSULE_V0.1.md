# IEAC verification capsule v0.1

**Status:** Versioned manifest contract and three synthetic privacy-mode vectors. The bundle builder and offline replay verifier are the next implementation step in issue #445.  
**Composes with:** in-process TUF verifier #441 / PR #444; channel completeness #419 / PR #422; decision-time frontier #413 / PR #415; TUF high-water #427 / PR #432; common-cause independence #423 / PR #407.

## 1. Purpose

A verification report is an output of a process. Detached from that process, its status string is only another claim. A portable capsule carries the minimum signed metadata, policy and target bytes/commitments needed for a second verifier to reproduce the checks.

The capsule manifest does not say that a decision is safe or admitted. It names the inputs, exact byte commitments, TUF metadata closure, policy and fixed replay context. The independent verifier computes the result again.

## 2. Privacy and replayability modes

The schema deliberately distinguishes three modes:

| Mode | TUF metadata | Target bytes | Replayability |
|---|---|---|---|
| self-contained | Embedded | All four targets embedded | Fully replayable, subject to a trusted root/policy |
| metadata-complete target-referenced | Embedded | Local content-addressed references | Conditional on target bytes being available locally |
| commitment-only | Embedded or referenced by manifest | Target bytes withheld | Commitment only; no target-byte replay |

A fourth `mixed` mode supports per-target combinations. It must be classified no stronger than its least replayable required target.

A digest is not a substitute for a withheld target. If a target byte is absent, the independent verifier must return `not_replayable` for that target, not `verified`. Metadata completeness and target-byte availability are separate conclusions.

## 3. Manifest structure

The manifest binds:
- out-of-band repository identity, HTTPS origin and root metadata path/version/SHA-256;
- fixed evaluation time, uncertainty interval and pinned clock verifier/policy/trust-bundle identities;
- exact policy identity/digest/scope and verifier implementation/version/build;
- raw metadata artifacts: root, timestamp, snapshot, top-level targets and all delegated roles needed to reach the four targets;
- the four IEAC targets (manifest, checkpoint state, completeness attestation and audit report), each with authorized role, path, byte length, SHA-256 and content mode;
- an inventory of every file included in the capsule, with path, length, SHA-256 and type.

The file-inventory digest is computed from the RFC 8785 JCS serialization of the normalized, path-sorted file inventory. JCS applies only to the capsule manifest/inventory. **TUF signed metadata must continue to be verified with TUF's securesystemslib/OLPC canonical JSON rules** by the selected TUF implementation. Never reserialize TUF metadata with JCS to test a TUF signature.

## 4. Required verifier behavior

The future capsule verifier must:
1. Validate the strict manifest schema and reject duplicate paths/IDs, traversal, absolute paths, ambiguous encodings and unexpected files.
2. Verify the inventory digest and every file's raw length/SHA-256 before parsing.
3. Compare root bytes to the out-of-band root digest, then reconstruct the TUF metadata state with the pinned TUF verifier.
4. Reverify root chain, timestamp, snapshot, top-level and delegated targets at the manifest's fixed evaluation interval.
5. Resolve each of the four IEAC targets from authenticated metadata and compare exact path/role/length/hash.
6. Verify bytes for embedded or locally referenced targets; return `not_replayable` when bytes are withheld or unavailable.
7. Recompute downstream decision-time/channel-completeness checks only when their required raw inputs are present.
8. Emit per-check `passed`, `failed`, `unknown`, `not_replayable` or `unsupported` results. Do not collapse these to a weighted assurance score.

The first schema fixtures are synthetic placeholders, not cryptographic evidence. A fixture's digest strings must never be presented as measured or production hashes.

## 5. Adversarial benchmark and measurable performance

Freeze attacks that mutate each metadata link, remove delegated metadata, replace targets, change lengths/digests, downgrade privacy mode, omit files, alter evaluation time, replay expired metadata, reorder entries, duplicate paths, corrupt cache data and remove high-water state.

Promotion gates:
- 100% rejection of frozen metadata/signature/delegation/target substitutions.
- 100% independent agreement for supported self-contained vectors.
- zero verified status for absent target bytes.
- 100% deterministic file-inventory digest for equivalent inputs after path sorting.
- zero silent downgrade from self-contained to target-referenced or commitment-only.
- byte-identical replay output for fixed capsule, policy, verifier build and evaluation time.

Measure bundle size by mode, metadata/target bytes, pack and unpack duration, p50/p95/p99 offline replay latency, memory ceiling, cache hit rate, and unavailable-target rates. Report pass/fail/unknown/not-replayable/unsupported separately. Synthetic mutation detection is a conformance metric, not a production compromise probability.

## 6. Claim ceiling

A self-contained capsule can support independent replay that its included signed metadata and target bytes pass the pinned TUF checks at the fixed evaluation time. It does not prove source truth, access-channel completeness outside the declared scope, common-cause independence, physical safety or decision correctness.

Target-referenced capsules require local availability of the referenced bytes. Commitment-only capsules prove only that recorded commitments were preserved; they do not prove byte-level replay. No capsule grants action authority.
