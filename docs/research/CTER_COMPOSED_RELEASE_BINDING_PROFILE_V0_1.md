# CTER Composed Release Binding Profile v0.1

**Tracking:** [CTER-PRIVATE-COVERAGE-001 #962](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/962) and [CTER-EGRESS-BINDING-001 #960](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/960).  
**Status:** structural composition and synthetic fixtures; no production cryptographic proof or authorization.  
**Research date:** 2026-11-20.

## 1. Paradigm: one release, one joined proof boundary

CTER now has separate reference evaluators for:

- request-to-package egress binding;
- private evidence-frontier coverage and its included/withheld/unavailable/excluded/unresolved partition;
- observable-channel and paired-world privacy fixtures.

The next error to prevent is treating their JSON records as independent assertions. A package could be structurally valid under #960 and a coverage partition could be structurally valid under #962 while the two describe different recipients, request scopes, package digests or evidence frontiers.

The composed release boundary therefore requires:

1. the egress manifest's \`coverage_record_ref\` equals the private coverage record's \`coverage_ref\`;
2. the egress package's exact payload-digest commitment equals the private coverage package commitment;
3. authorization request commitment, principal, subject, operation, purpose, recipient, resource, jurisdiction, project snapshot and data-class set agree across the egress authorization binding and coverage query;
4. the egress evidence-frontier reference equals the private coverage frontier reference;
5. the private coverage's asserted terminal source leaves equal the terminal leaves derived from the egress output-to-source transformation graph;
6. the coverage partition's included set equals that same terminal leaf set;
7. the upstream receipt binding/currentness projection and privacy-preserving verifier view remain structurally valid.

A caller-supplied list of terminal leaves is not accepted as its own proof. The reference implementation derives the leaves by traversing the exact source graph already checked by the egress evaluator.

## 2. Why the source-leaf equality matters

The lineage rule is transitive:

- if output A derives from source X, its terminal leaf is X;
- if output B derives from output A, B's terminal leaf is still X;
- if output C derives from A and source Y, its terminal leaves are X and Y;
- if a package contains multiple independent output roots, their terminal source leaves are unioned.

For a release with committed frontier \(F\), private partition \(P\), included set \(I\), and derived terminal source leaves \(L(G)\) from validated output graph \(G\), the structural requirement is:

\[
\operatorname{refs}(P)=F,\qquad I=L(G)=L_{\mathrm{asserted}}
\]

The equality is set-based; duplicate identities are separately rejected. A package object with a phantom source, hidden extra transform input, omitted source, or a caller-invented terminal leaf must not produce a valid composed result.

This establishes completeness relative to the exact declared frontier/query and the exact package graph. It does not establish that the custodian discovered every real-world source, right or object.

## 3. Exact cross-record bindings

| Egress manifest | Private coverage record | Requirement |
|---|---|---|
| \`coverage_record_ref\` | \`coverage_ref\` | Exact reference equality |
| \`package.package_payload_digest\` | \`package_binding.package_commitment\` | Same algorithm and digest value over the exact serialized package payload bytes |
| \`authorization_binding.request_commitment\` | \`query_binding.request_commitment\` | Exact equality |
| principal / subject | principal / subject | Exact equality |
| operation / purpose | operation / purpose | Exact equality |
| recipient / resource | recipient / resource | Exact equality |
| jurisdiction profile | jurisdiction profile | Exact equality |
| project snapshot SHA-256 | project snapshot SHA-256 | Exact equality |
| evidence frontier reference | \`frontier.frontier_ref\` | Exact equality |
| authorization data-class set | coverage query data-class set | Set equality |
| graph-derived terminal leaves | \`package_binding.terminal_source_leaf_refs\` | Set equality |
| graph-derived terminal leaves | private partition entries with disposition \`included\` | Set equality |

The private coverage receipt projection must also bind subject and data-class scope. A receipt for the right principal but wrong subject or classification is not reusable.

The package payload digest is the digest of the exact serialized package payload bytes, not the manifest commitment, source-object digest, transform-profile digest or Mycelix event identity. All digest fields remain algorithm-labelled and domain-specific.

## 4. Receipt and proof ownership

The coverage record contains a receipt reference and a host-resolved assessment projection. It does not authenticate the receipt itself.

Mycelix remains responsible for:
- verifying the exact COSE receipt bytes and signature;
- validating the registered VDS and proof type;
- resolving issuer trust, policy, freshness, revocation and status snapshot;
- checking inclusion, consistency, disclosure or non-inclusion proofs;
- preserving the authoritative typed receipt/registration semantics.

CTER consumes those qualified outputs and binds them to a local use. It MUST NOT mint a second receipt, transparency log, VDS, witness service or issuer trust system.

RFC 9942 defines COSE receipts that carry proofs about a VDS, including inclusion, consistency, disclosure and non-inclusion. It also warns that proof types are tied to their associated VDS and that implementers should not assume interoperability across structures; security analysis is required before migration. RFC 9943 defines SCITT's single-issuer signed-statement transparency architecture and requires transparency services to produce COSE receipts. CTER's implementation choice must follow the Mycelix owner-approved VDS and profile, not select an incompatible tree independently.

A valid inclusion proof means an item is included in a specific committed VDS state. A non-inclusion proof is bounded to that state/frontier. Neither proves global absence, source honesty, legal title, community consent or authorization for this particular disclosure.

## 5. Current executable scope

The public branch now contains:

- \`benchmarks/territorial-integrity/relay/reference/src/egress.rs\` — egress structure evaluator and graph-derived terminal-source-leaf helper;
- \`benchmarks/territorial-integrity/relay/reference/src/private_coverage.rs\` — partition, receipt-projection and public-view structural evaluator;
- \`benchmarks/territorial-integrity/relay/reference/src/release_binding.rs\` — joined cross-record evaluator;
- \`schemas/cter-private-coverage-v0.1.schema.json\` — subject/data-class receipt binding and required frontier-scoped non-inclusion label;
- \`benchmarks/territorial-integrity/egress/private-coverage-cases-v0.1.json\` — 32 private-coverage mutation cases;
- \`benchmarks/territorial-integrity/egress/composed-release-cases-v0.1.json\` — 17 joined-release fixtures;
- Rust conformance tests for private coverage and composed release;
- independent Node oracles for egress, private coverage and composed release;
- exact-head workflow steps for the schemas, corpora and Node oracles, plus Rust tests and strict Clippy.

The private-coverage evaluator now binds both subject and data-class set across query and receipt projections. The schema requires those fields and requires the public verifier view to explicitly scope non-inclusion to \`frontier_version_only\`.

The composed evaluator reports \`valid_composed_structure\`, not \`authorized\` or \`released\`. It is a structural assertion across two records. It still does not hash package bytes itself, authenticate receipt bytes, resolve authority or call a transport.

## 6. Qualification targets

Targets, not achieved results:

| Property | Target |
|---|---:|
| Coverage/egress record reference mismatches accepted | 0 |
| Package payload commitment mismatches accepted | 0 |
| Request/principal/subject/operation/purpose/recipient/resource/jurisdiction/snapshot drift accepted | 0 |
| Data-class set drift accepted | 0 |
| Evidence-frontier reference drift accepted | 0 |
| Caller-asserted terminal leaf set inconsistent with graph | 0 |
| Included partition inconsistent with derived graph leaves | 0 |
| Valid multi-stage provenance DAG rejected | 0 |
| Independent Rust/Node expected-disposition agreement on frozen fixtures | 100% |

Scale the joined evaluator at 1k/10k/100k/1M output objects and frontier entries. Measure p50/p95/p99 parse, graph validation, leaf derivation, coverage comparison and composed validation latency; peak memory; storage amplification; proof bytes; and cold restore time. Pin object-size and fan-in/fan-out distributions, hardware, toolchains, build flags and corpus digests. Report hashing time separately from graph-validation time. Energy claims require physical measurement.

## 7. Limits and next proof decision

The current reference slice uses a SHA-256 digest placeholder for the package and frontier commitments and a synthetic receipt-resolution status. It does not recompute commitments over bytes or generate/verify COSE receipts.

The next research decision is not "write a new zero-knowledge system." It is to compare the Mycelix-supported VDS/COSE receipt path and identify the minimum proof needed for:
- a recipient to verify the package's included items;
- an independent auditor to verify the named frontier/query/package relation without seeing withheld inventory;
- a public observer to learn no item-level inventory or revealing counts.

Proof size, verification cost, update cost, append-only/non-equivocation guarantees, non-inclusion semantics, privacy leakage, implementation maturity and post-quantum migration must be measured for the actual Mycelix-supported structure. No cryptographic primitive is selected by this profile.

### Long-term cryptographic agility

NIST IR 8547 (final, 27 October 2026) provides a transition roadmap from quantum-vulnerable digital signatures and key-establishment schemes to post-quantum standards. The archival design must therefore separate:

- historical verification under the policy/key/algorithm that existed at the time;
- present-day acceptance for new signing and transport operations;
- evidence renewal/migration and its own trusted-time/transparency context.

A later renewal can preserve evidence of an earlier verification result; it cannot retroactively make an invalid signature trusted. Mycelix owns the authoritative renewal and trust policy. CTER preserves and checks the continuity references needed for independent restore.

## 8. Non-goals

- no second Mycelix authorization, consent, identity, receipt, transparency or status system;
- no global real-world completeness assertion;
- no public disclosure of withheld/unavailable inventory or revealing counts;
- no generic anonymity guarantee;
- no assertion that a valid composed structure authorizes actual release;
- no claim of production cryptographic proof, field readiness or SOTA superiority.


The egress authorization projection includes `coverage_query_ref` and `coverage_query_commitment`. The composed evaluator requires exact equality with the private coverage record's query reference and commitment, preventing a request/package binding from being reused under a different selection predicate.


## Predicate profile is part of the release binding

The egress authorization projection now carries \`coverage_query_profile_ref\`. It must equal the private coverage query's \`query_profile_ref\`, which is also repeated in the receipt projection. This prevents a package/recipient/request binding from silently switching predicate semantics.

The private query uses \`urn:luminous:cter:predicate-json-v0.1\`, a restricted three-valued AST. Its per-item result must agree with the private partition's scope state before the composed release can be structurally valid. The 21-case corpus now contains two positive structural controls and 19 expected denials, including profile and predicate substitution controls.

A tool-side V8 mirror matched 21/21 composed fixtures (2 valid composed structures, 19 denials). This is not Node 22 CI or Rust execution. The predicate commitment is still cross-bound by reference/hash equality only; canonical-byte hash recomputation remains a separate requirement.
