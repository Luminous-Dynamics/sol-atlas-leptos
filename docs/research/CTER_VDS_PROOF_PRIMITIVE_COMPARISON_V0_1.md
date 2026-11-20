# CTER VDS Proof Primitive Comparison v0.1

**Tracking:** [CTER-VDS-PROOF-QUAL-001 #967](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/967)  
**Composes:** #960 exact egress binding, #962 private coverage, #964 cross-channel privacy; Mycelix #5942/#5944 witnessed SCITT checkpoint research and #1215 typed receipts.  
**Status:** proof-selection study; no proof primitive selected or implemented.  
**Research date:** 2026-11-20.

## 1. The exact proof question

The private-coverage problem is not simply "is an item in a Merkle tree?" It is whether a verifier can establish a relation across a named source frontier, an exact query, a private disposition partition and a package derived from some of those frontier items.

Let:

- \(F_v\) be the exact item set committed by source owner \(S\) at frontier version \(v\);
- \(Q\) be the canonical query/scope predicate;
- \(A\) be the upstream authorization binding (principal, subject, operation, purpose, recipient, resource, jurisdiction, project snapshot and data classes);
- \(P\) be the custodian's private partition;
- \(I\) be the items classified as included;
- \(G\) be the validated egress transformation graph;
- \(L(G)\) be the terminal source leaves derived from the exact package graph;
- \(D\) be the exact serialized package payload commitment.

The relation to qualify is:

\[
\operatorname{refs}(P)=F_v
\]
\[
F_v=I\;\dot\cup\;W\;\dot\cup\;U\;\dot\cup\;X\;\dot\cup\;R
\]
\[
I=L(G)
\]

where \(W\), \(U\), \(X\), and \(R\) denote withheld, unavailable, excluded, and unresolved items respectively. The disjoint-union symbol means every frontier item has exactly one disposition.

The proof is bound to \((F_v,Q,A,D)\). A verifier must not be able to substitute another recipient, query, package, evidence frontier or project snapshot while retaining the same apparent result.

**What the proof does not say:** the frontier is not necessarily complete in the real world; non-inclusion in \(F_v\) does not prove global absence; a source owner may have failed to discover evidence; a receipt does not establish truth, consent, authority, title or permission to disclose.

## 2. Standards: separate the envelope, VDS and proof relation

### RFC 9942 COSE Receipts

[RFC 9942](https://www.rfc-editor.org/rfc/rfc9942.html) defines a COSE envelope for carrying VDS-specific receipts. Its general introduction describes inclusion, consistency, disclosure and non-inclusion proof concepts. Crucially, proof encodings and semantics are specific to the selected VDS; the RFC warns against assuming proof interoperability across different structures.

The initial concrete \`RFC9162_SHA256\` profile defines Merkle inclusion and consistency proof encodings. It is not a generic proof of arbitrary key non-membership, disjoint partitioning, query completeness or set equality. A CTER implementation must name the exact VDS and proof type it consumes, not just say "COSE receipt verified."

### RFC 9162 Certificate Transparency

[RFC 9162](https://www.rfc-editor.org/rfc/rfc9162.html) specifies Merkle inclusion and consistency proofs for Certificate Transparency. RFC 9162 is published as an **Experimental** RFC, not an Internet Standards Track specification. It is a useful, widely studied transparency structure, but a generic CTER completeness claim cannot be inferred from it.

### RFC 9943 SCITT

[RFC 9943](https://www.rfc-editor.org/rfc/rfc9943.html) defines an architecture for transparency of single-issuer signed statements through transparency services and VDSs. It addresses statement registration and auditable/consistent transparency history. The Mycelix #5942/#5944 work studies witnessed checkpoints and split-view resistance around that boundary.

SCITT registration, inclusion and checkpoint consistency are distinct from proving that an arbitrary private partition exhausts the items in a CTER query and that its included subset equals the source leaves of one package. A witnessed checkpoint is necessary evidence for the transparency state; it is not the coverage theorem.

### W3C selective disclosure and status

W3C Verifiable Credentials and Bitstring Status List are relevant to minimizing disclosed claims and reducing one-credential/one-URL status-query correlation. They do not prove private source-frontier set equality. Mycelix should own issuer/status/currentness resolution and CTER should consume typed results.

## 3. Candidate construction families

No construction is selected in v0.1. Each candidate must be evaluated on identical source sets, frontier updates, query predicates, private partitions, package DAGs, proof audiences and fault schedules.

| Candidate | What it can plausibly prove | Main trade-offs / open questions | CTER position |
|---|---|---|---|
| RFC 9162-style append-only Merkle log via SCITT | Statement inclusion in a committed log and consistency between tree sizes | Inclusion of a statement is not proof that a private query result is exhaustive; CT tree ordering and size are observable; non-inclusion/set equality requires more than the initial RFC9162_SHA256 profile | Reuse Mycelix's transparency service for source-frontier and receipt history; do not claim it alone proves coverage |
| Sparse Merkle tree / authenticated map | Membership and, with a defined default-leaf/key-space rule, non-membership for a keyed item | Key derivation and collision semantics; proof domain size; default-leaf and empty-key rules; update cost; tree versioning; proof size; leakage through key reuse; VDS/COSE interoperability | Candidate for opaque frontier item membership/non-membership, subject to Mycelix proof-type support and privacy analysis |
| Ordered authenticated dictionary / range proof | Membership and absence between authenticated neighboring keys; potentially range completeness | Ordering is observable; insertion/rebalancing can affect proofs; range boundaries and count may leak; stable sorting/canonicalization and version binding required | Candidate when query can be expressed as a stable range or ordered predicate |
| Set-commitment plus zero-knowledge relation proof | Can potentially prove disjoint partitioning, full frontier coverage and \(I=L(G)\) without revealing item identities | Circuit/constraint size; prover CPU/RAM; verifier cost; trusted setup (if any); soundness assumptions; key/salt management; proof update invalidation; mature implementation availability; long-term verification and migration | Research-only candidate for privacy-sensitive partition relation, not a presumed default |
| Hybrid | Mycelix SCITT/VDS proves append-only source-frontier history; a second narrowly scoped authenticated-map or ZK proof proves the private query/partition/package relation | More components and versioned interfaces; must prove that both layers bind the same frontier, query and package; avoid duplicating transparency ownership | Most plausible architecture to evaluate first, not a chosen design |

### Important implementation distinction

A sparse authenticated map might prove that each known key is included or absent from the committed frontier. That is not automatically enough to prove the *enumeration is exhaustive* for a query. The construction also needs to bind the exact query predicate, input universe and response set, or provide a proof that no qualifying key was skipped.

A zero-knowledge proof may express that relation, but its cost and security assumptions must be measured. Do not reach for ZK merely because a proof must be private. First evaluate whether a scoped auditor, a keyed authenticated map, or a smaller selective-disclosure proof can answer the operational question with less complexity.

## 4. Proof audiences and minimum disclosure

| Audience | Possible minimum statement | Must not learn by default |
|---|---|---|
| Authorized recipient | Exact package commitment, inclusion relation for items received, and proof that those items are within the authorized query/frontier | Other recipients' objects, unrelated frontier items, withheld inventory, community-wide counts |
| Independent auditor | Frontier/query/package binding and evidence that the private partition predicate was checked under an authorized audit role | Raw protected inventory, claimant identity, sensitive coordinates, revealing withheld/unavailable counts |
| Public observer | Opaque scoped reference and a consistency/registration fact only when separately authorized | Item identities, exact set size, withheld count, low-entropy hashes, location or query details |
| Community custodian | Full private partition and lineage needed for the authorized stewardship role | Unrelated projects, unrelated recipients/purposes, or another community's restricted inventory |

Proof bytes, metadata, tree size, query timing, stable key derivation and cache behavior are all observable channels. A proof can be cryptographically sound and still disclose more than its audience may receive.

## 5. Adversarial protocol and benchmark

Use identical synthetic inputs and run each candidate at 1k, 10k, 100k and 1M frontier entries. Include:
- multiple included/withheld/unavailable/excluded/unresolved ratios;
- late source events and frontier supersession;
- package DAG depth, fan-in/fan-out and multiple output roots;
- selective and broad query predicates;
- online/offline and partitioned verifiers;
- stale, revoked, unavailable and conflicting VDS checkpoints;
- split-view witnesses and unknown witness-domain independence;
- recipient and purpose substitution;
- repeat release and replay after an unknown transport outcome.

### Metrics (report separately)

- proof generation and verification p50/p95/p99 and max;
- proof bytes, request bytes, public metadata and leakage surface;
- prover peak memory/CPU and verifier memory/CPU;
- frontier append/update cost and proof invalidation cost;
- package/partition reconciliation time;
- checkpoint/witness quorum availability and split-view detection rate;
- false acceptance, false rejection, REVIEW and UNKNOWN rates;
- independent restore bytes, time and dependency count;
- privacy under dictionary, linkage, snapshot-differencing and status-query correlation tests;
- cryptographic renewal and proof-version migration cost.

No single weighted score. A small, fast proof that leaks withheld counts does not beat a larger private proof. A proof that cannot be independently restored is not a custody-exit success. An UNKNOWN result during unavailable status resolution is not a false positive; it is the designed safe state.

### Target invariants (not achieved results)

| Invariant | Target |
|---|---:|
| Omitted frontier item accepted while claiming complete | 0 |
| Phantom package source accepted | 0 |
| Duplicate/overlapping disposition accepted | 0 |
| Query/request/recipient/frontier/package substitution accepted | 0 |
| Stale/forked/unverified checkpoint promoted to current | 0 |
| Withheld/unavailable/unresolved disclosed to unauthorized audience | 0 |
| Global absence inferred from one frontier's non-inclusion | 0 |
| Independent verifier disagreement on frozen valid/invalid fixtures | 0 |
| Valid multi-stage provenance graph rejected | 0 |

## 6. Decision gate

Before selecting a proof primitive:
1. Mycelix must identify the VDS, receipt types and resolver output it can actually issue and verify.
2. The exact relation above must be expressed independently of the proof implementation.
3. At least two candidates must run against identical frozen fixtures, or a candidate must have a documented, evidence-backed exclusion.
4. The public leakage surface must be measured, including frontier size, counts, stable IDs and proof metadata.
5. The proof must survive provider exit and preserve historical VDS/profile/algorithm semantics.
6. A cryptographic/privacy reviewer must inspect the construction and its implementation; model checking or fixture agreement alone is not a proof of cryptographic soundness.

## 7. Ownership and claim ceiling

- **Mycelix** owns identity, issuer trust, signed receipt verification, SCITT/VDS, witness/checkpoint policies, currentness/revocation and cryptographic renewal.
- **CTER** owns the consumer-side query/package/partition relation, exact cross-record bindings, privacy audience projection and custody-exit qualification.
- **Communities and custodians** define the relevant source frontier, disclosure audience and applicable restrictions under their own legitimate processes.
- **Symthaea** may discover anomalies or generate counterexamples read-only; it does not establish proof validity or authority.
- **Symtropy** may generate synthetic spatial/network scenarios; those simulations do not establish field or legal facts.

A successful benchmark establishes only the exact statement, VDS, frontier version, query, package, policy, audiences, implementation and tested schedules. It does not prove global real-world completeness, truth, consent, legal title, community legitimacy, or permission to release.


## 8. Machine-readable candidate workload

The matched workload matrix is committed at \`benchmarks/territorial-integrity/egress/vds-proof-workloads-v0.1.json\`. It records five candidate families, frontier sizes of 1k/10k/100k/1M, three private-partition mixes, three source-DAG profiles, four update/churn schedules, four verifier audiences and 22 fault injections.

The file records proof generation and verification separately, checkpoint/receipt bytes, public metadata bytes, frontier update and invalidation cost, source-leaf derivation, quorum availability, split-view detection, false accept/reject/unknown counts, privacy findings and independent restore. It explicitly forbids a single weighted score and requires candidates to receive the exact same frontier bytes, query, package graph, partition, audience, schedules, host and resource limits.

The workload file is a benchmark specification only; no candidate proof has been generated, measured or selected.


## 9. Query-input provenance is a separate relation

[CTER-QUERY-INPUT-ATTESTATION-001 #973](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/973) identifies a further condition for any private completeness proof: the exact per-item attributes evaluated by the committed predicate must themselves be frozen and bound to the frontier. A predicate commitment proves which rule was used; it does not prove which values were evaluated.

The proof statement must therefore include a separate source-input snapshot relation:
- exact frontier reference/version/commitment;
- exact query predicate profile and commitment;
- exact attribute-projection schema/profile;
- sorted per-item query-input projections, availability states and source object/digest bindings;
- a domain-separated snapshot commitment;
- an owner/resolver attestation of that commitment under a typed receipt, distinct from the custodian's own hash.

The snapshot hash alone proves only that a particular projection was frozen. It does not prove that the source values are true, owner-authoritative, or globally complete. Mycelix must confirm the receipt type and owner/resolver semantics before CTER describes the snapshot as source-attested.

The extended proof relation is now at least:
\`frontier + query predicate + source-input snapshot + private partition + package transform graph\`.
The verifier must establish that predicate evaluation follows from the exact attested input snapshot, that the partition accounts for the exact frontier, and that the included set equals the graph-derived package leaves. The five candidate proof families must be compared against this stronger statement, not the earlier partition-only abstraction.
