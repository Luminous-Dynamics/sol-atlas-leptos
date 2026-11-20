# CTER Query Input Snapshot and Source Attestation Profile v0.1

**Tracking:** [CTER-QUERY-INPUT-ATTESTATION-001 #973](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/973)  
**Composes:** #960 egress binding, #962 private coverage, #967 VDS proof selection and #972 deterministic query semantics.  
**Status:** profile/schema/workload and initial commitment/evaluator implementation; no Mycelix owner-attestation adapter is claimed.  
**Research date:** 2026-11-20.

## 1. The distinction that must not collapse

The predicate commitment authenticates the rule. It does not authenticate the values on which the rule was evaluated.

\`predicate_hash\` != \`query_input_snapshot_hash\` != \`source_owner_attestation\` != \`authorization\`

A custodian could otherwise alter a private item's jurisdiction/dataset/status values, recompute its TRUE/FALSE/UNKNOWN scope, and retain a correct predicate hash. The partition would remain internally consistent while using different source attributes.

The new snapshot profile freezes the exact private query-input projection and binds it to:
- the exact source frontier reference, version and commitment;
- the exact query profile and predicate commitment;
- a versioned attribute-projection profile;
- per-item object references, availability states and private attributes;
- per-attribute source object references and content digests.

The snapshot commitment establishes integrity of the declared projection after construction. It does **not** prove that the values are true, that the source owner supplied them, or that the source owner discovered every relevant fact. A distinct owner/resolver receipt must attest the commitment before it may be described as source-attested.

## 2. Canonical snapshot commitment

Profile: \`urn:luminous:cter:query-input-json-v0.1\`.

The commitment is:

\`SHA-256("luminous.cter.query-input-snapshot.v0.1\\0" || JCS(preimage))\`

The canonical preimage contains:
- \`snapshot_ref\` and \`profile_ref\`;
- \`frontier_ref\`, \`frontier_version\`, \`frontier_commitment\`;
- \`query_profile_ref\`, \`query_commitment\`;
- \`attribute_projection_profile_ref\`;
- the exact item list sorted by \`object_ref\`;
- each item's exact availability state, private \`query_input\` attribute projection (when available), and attribute-source bindings sorted by \`path\`, then source reference.

Source-binding order is canonicalized before JCS so the same semantic set produces the same digest. The item list must be unique and exactly equal to the named frontier's item-reference set. Each source reference must resolve to an object in that same frontier in v0.1.

## 3. Three-valued input availability

Each item has an input availability state:
- \`available\`: the private attribute projection is present; each predicate-referenced attribute that is present has a source binding.
- \`unavailable\`: the attribute projection is not available; predicate evaluation is UNKNOWN.
- \`unknown\`: the custodian cannot establish whether the input projection is available; predicate evaluation is UNKNOWN.

A missing attribute in an available projection is also UNKNOWN for that predicate path. It is not silently converted to FALSE.

The scope result remains the #972 rule:
- TRUE -> \`in_scope\`
- FALSE -> \`out_of_scope\`
- UNKNOWN -> \`unknown\`

This prevents an unavailable or incomplete source attribute from becoming a false non-match.

## 4. Attribute-source bindings

For each predicate-referenced attribute present in the private projection, the snapshot includes:
- the exact one-level predicate path, e.g. \`/jurisdiction\`;
- a source object reference that must exist in the committed frontier;
- the source object's SHA-256 content commitment.

These references are not self-authenticating. They are a provenance map that a source-owner/resolver receipt must cover. A custodian-provided \`source_verified=true\` flag is not accepted as authority.

The first evaluator slice checks the source-reference set, digest format, availability semantics and exact snapshot commitment. It does not retrieve source bytes, recompute their content digests, interpret the semantic extraction profile or verify a Mycelix signature.

## 5. Cross-record bindings

The same \`snapshot_ref\` and \`snapshot_commitment\` must appear in:
- the private query binding;
- the host-resolved receipt projection;
- the egress authorization binding.

The snapshot itself binds the frontier, query profile and predicate commitment. The composed release evaluator requires exact equality of the snapshot ref/digest across these projections.

The public verifier view does not expose raw per-item attributes, per-item source references, item counts or the snapshot digest by default. Those may be released only under an explicitly qualified proof/audience profile. Deterministic low-entropy hashes can leak information even when their input text is hidden.

Mycelix must confirm the precise typed receipt and authority semantics for source-owner attestation. CTER does not mint a new source-owner receipt, identity system, source registry or transparency service.

## 6. Immutable revision and requalification

A snapshot is immutable. A changed attribute, source object digest, availability state, frontier version, query predicate or attribute-projection profile produces a new snapshot reference and commitment. An old receipt is never mutated to refer to the new snapshot.

If the authoritative resolver reports the snapshot receipt stale, revoked or unavailable, the current action remains UNKNOWN/INCOMPLETE and must be requalified before a new disclosure or mutation. Missing source data is not proof of non-existence.

## 7. Adversarial cases

Required controls include:
- change an attribute after snapshot commitment;
- substitute a source reference while retaining the same attribute value;
- change source content digest;
- omit source binding for a predicate-used attribute;
- provide a source reference outside the committed frontier;
- duplicate item or attribute-source references;
- change availability and promote UNKNOWN to TRUE/FALSE;
- substitute frontier, predicate, query profile, subject, recipient or purpose;
- restore the predicate but not the input snapshot/profile;
- use a custodian hash without owner/resolver attestation.

## 8. Benchmarks

The machine-readable workload is \`benchmarks/territorial-integrity/egress/query-input-snapshot-workloads-v0.1.json\`. It specifies 1k/10k/100k/1M frontiers, varied attribute counts, source fan-in, missing/unavailable proportions, frontier churn and query complexity.

Report separately:
- canonical snapshot generation and verification p50/p95/p99;
- raw item count and private snapshot bytes;
- receipt/proof bytes and public metadata leakage;
- source-resolution and stale-requalification latency;
- per-item and whole-snapshot memory/allocations;
- UNKNOWN rate and causes;
- Rust/Node commitment agreement;
- owner-attestation availability and verification cost;
- false acceptance/rejection and independent-restore outcomes.

No single weighted score. Integrity, privacy, owner-attestation, availability and performance are separate gates.

## 9. Claim ceiling

A matching snapshot commitment proves only that the exact declared private attribute projection is bound to the named frontier/query/profile. Source bindings establish declared provenance references, not source truth. A custodian-computed hash is not owner attestation. Owner attestation does not itself authorize disclosure. Authorization remains a separate Mycelix-owned decision, and global real-world completeness is never inferred from one frontier.
