# CTER Privacy-Preserving Coverage Profile v0.1

**Tracking:** [CTER-PRIVATE-COVERAGE-001 #962](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/962)  
**Parent:** [CTER-EGRESS-BINDING-001 #960](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/960)  
**Status:** research profile; no proof format or cryptographic implementation selected.  
**Research date:** 2026-11-20.

## 1. The problem: completeness without inventory disclosure

A CTER recipient may need assurance that an export is complete for a declared evidence frontier and request. The recipient does not automatically need to know all other objects held by the custodian.

Publishing the full inventory can reveal:
- claimant identities or the existence of a claimant;
- sensitive parcel, sacred-site, habitat, resource or infrastructure locations;
- which records were withheld or are under dispute;
- the size and shape of a community's evidence collection;
- the existence of records that are unavailable or restricted;
- low-entropy attributes recoverable from deterministic hashes.

The system therefore needs two complementary properties:

1. **Evidence completeness relative to a declared frontier:** every item in the committed query universe has exactly one accounted-for disposition.
2. **Minimal disclosure to the verifier:** a recipient learns only the facts necessary for its task.

This is not the same as hiding an object inventory behind an ordinary API. It must remain possible for an independent verifier to detect omission, substitution, stale frontiers and false absence claims.

## 2. Define the claim before choosing cryptography

Let:
- \(F_v\) be the custodian/source-owner's declared evidence frontier at version \(v\);
- \(Q\) be a versioned query and scope predicate bound to the request, project snapshot, purpose, recipient and jurisdiction;
- \(P_Q(F_v)\) be the subset of frontier items considered relevant by that exact predicate;
- \(I\) be items included in the materialized package;
- \(W\) be items withheld under a qualified restriction;
- \(U\) be items unavailable or unresolved;
- \(X\) be items explicitly excluded by the request's declared scope;
- \(R\) be items whose disposition remains unresolved.

The custodian-private coverage partition must satisfy:

\[
F_v = I \uplus W \uplus U \uplus X \uplus R
\]

where \(\uplus\) means disjoint union. Each committed frontier item appears once and receives exactly one disposition. The included portion must bind to the exact output manifest and package commitment from #960.

A verifier may receive a scoped statement such as:

> For frontier commitment \(F_v\), query profile \(Q\), request commitment \(A\), and package commitment \(M\), the declared coverage predicate was evaluated and the included set is bound to \(M\); withheld, unavailable, excluded and unresolved states have not been silently converted to absence.

This statement does **not** claim that \(F_v\) contains every relevant real-world object. It proves a property of the declared, committed frontier under the source and custodian assumptions that are explicitly recorded.

### Non-equivalences

- complete for a declared frontier != complete for the world;
- non-inclusion in frontier version \(v\) != proof that a right, claim, person, record or object does not exist;
- a proof that an exported item belongs to a frontier != proof that all eligible items were exported;
- a commitment to a custodian declaration != proof that source acquisition was complete or honest;
- proof verification != authorization to disclose;
- privacy of object values != privacy of object counts, metadata, timing or traffic patterns.

## 3. What RFC 9942 does and does not establish

RFC 9942, published in June 2026 as a Proposed Standard, defines COSE Receipts for proofs of properties of Verifiable Data Structures. It describes proof types for inclusion, consistency, disclosure (a minimal-disclosure special case of inclusion) and non-inclusion. It is an important existing primitive for portable verifier-facing evidence.

It does **not** automatically provide a complete proof that a query was correctly evaluated over every member of a committed frontier. For example, proving inclusion of every object in an export does not establish that the exporter did not omit another eligible object. That requires an additional completeness argument tied to \(Q\), \(F_v\), and the partition rule.

The initial research must compare:

| Candidate | Strength | Unresolved question for CTER |
|---|---|---|
| Mycelix-owned COSE Receipt / VDS proof | Interoperable proof envelopes; inclusion, consistency, disclosure and non-inclusion properties | Which actual VDS and proof types are implemented by Mycelix, and can the exact query/partition claim be verified without exposing inventory metadata? |
| Custodian-attested query evaluation | Operationally simple; can sign a versioned query result and frontier commitment | What independent execution, source-ingestion or audit evidence prevents a malicious custodian from omitting a record before commitment? |
| Transparent query-completeness proof (e.g. ZK circuit over a committed set) | Could prove a predicate over a committed frontier while hiding members | Circuit soundness, input availability, proving cost, verifier cost, update semantics, public parameter/trusted-setup assumptions and metadata leakage |
| Full inventory disclosure to a trusted auditor | Simple and independently inspectable | High privacy exposure; only acceptable to a verifier explicitly authorized to see the inventory |
| Redacted/hashed public manifest | Easy to implement and distribute | Hashing low-entropy values and publishing counts/structure may still reveal protected information; a hash is not an anonymity guarantee |

Do not select a new Merkle tree, accumulator, vector commitment or ZK scheme merely because it appears cryptographically attractive. First determine what the Mycelix transparency owner already issues and what exact property that receipt proves. CTER should consume the authoritative proof, not create a parallel log.

## 4. Proposed architecture: private partition, verifier-specific proof

Separate the following layers.

### 4.1 Custodian-private coverage state

The custodian retains the per-object inventory, source references, restriction reasons, evidence-classification context, and exact disposition records. This object is never public by default.

The private state records:
- frontier reference/version and source-owner provenance;
- exact request/query profile and its canonical commitment;
- per-item disposition and reason;
- exact included object references and payload commitments;
- withheld/unavailable/excluded/unresolved distinctions;
- supersession links if the frontier or a source assertion is corrected;
- evidence that the partition was computed against the named version.

The private state is not a second authority registry. Community mandate, restrictions, consent and publication authority continue to be resolved by their owners.

### 4.2 Verifier-specific proof envelope

A proof request declares the verifier's task and maximum disclosure level. Examples:

- **Recipient:** needs inclusion proofs for its package items and proof that the package commitment matches the authorization-bound manifest.
- **Independent auditor:** may need a proof that the exact request predicate was evaluated against frontier version \(v\), plus exception-state counts only when those counts are safe to disclose.
- **Public observer:** may only need a scoped receipt reference and consistency proof, without item identities, locations, withheld counts or the frontier inventory.
- **Community custodian:** may need the complete private partition and source references under its own authority.

Do not assume one proof is suitable for every verifier. Disclosure itself is an operation and must be qualified under the distinct authorization boundary in #951/#960.

### 4.3 Minimal-disclosure proof chain

The target chain is:

\[
\text{source evidence}
\rightarrow
\text{declared frontier}
\rightarrow
\text{query/partition result}
\rightarrow
\text{included package}
\rightarrow
\text{recipient-scoped proof}
\]

Each link is versioned and committed. The verifier checks the exact links necessary for its task. Mycelix supplies the authoritative VDS/receipt layer where implemented; CTER binds the returned proof references to its request, package, recipient and custody context.

### 4.4 Proof of coverage is separate from proof of authority

A coverage proof can establish that a declared set was partitioned according to a declared query. It cannot establish that the person who authored the query or restriction had authority to do so. Likewise, an authorization receipt does not prove the package is complete.

The two proofs must remain linked but independently qualified.

## 5. Privacy and side-channel threat model

Inventory contents are not the only leakage channel. The frozen synthetic threat model must include:

- deterministic hashes of coordinates, names, small categorical fields or short identifiers;
- object counts, withheld counts, package sizes and number of proof paths;
- ordered object references, path depth, tree shape and stable leaf positions;
- timestamps that reveal fieldwork, dispute, community meeting or site-visit activity;
- filenames, MIME types, thumbnails, map tiles, preview images and search-index fragments;
- API error differences that reveal whether a protected object exists;
- cache keys, temporary files, debug traces, analytics, crash dumps and log fields;
- transfer size and timing that reveal the size of a restricted inventory;
- proof requests that let one recipient correlate the same objects across unrelated purposes;
- reusable salts or opaque identifiers that make separate exports linkable.

The privacy suite must inspect every release channel, not only the main ZIP/BagIt payload. OPA-style decision-log masking is a useful operational baseline, but field masking alone is not a cryptographic completeness proof.

### Commitment rules to investigate

1. Never publish an unsalted deterministic hash of a low-entropy sensitive value.
2. Use explicit algorithm and domain identifiers; object, manifest, frontier, query and package commitments are distinct types.
3. Avoid cross-recipient or cross-purpose linkability by default.
4. Do not assume a random salt makes a commitment anonymous. If the salt is disclosed to the verifier, a small domain may still be enumerable; if it is not disclosed, public recomputation may not be possible.
5. Consider recipient-specific commitments or proof systems that avoid revealing object values; make key custody and verifier trust assumptions explicit.
6. Record proof-size, timing and count leakage as measurable output, not merely as a narrative privacy note.

No commitment, pseudonymization, encryption or ZK mechanism is selected in v0.1.

## 6. Adversarial state transitions

A minimum formal model should reject these transitions:

| Attack | Required result |
|---|---|
| In-scope item omitted from \(I\) but proof says complete | DENY / incomplete proof |
| Withheld item mapped to absent | DENY |
| Unavailable or unresolved item mapped to absent | UNKNOWN, never absent |
| Same item assigned to two dispositions | DENY |
| Item not in \(F_v\) inserted into package | DENY |
| Frontier version changes after query evaluation | UNKNOWN pending re-evaluation |
| Query, purpose, recipient or request commitment changes | DENY / re-authorize |
| Receipt stale, invalid, forked or mismatched to root | DENY / UNKNOWN according to the resolver's typed result |
| Non-inclusion proof interpreted outside the named frontier/version | DENY |
| Public count, path or hash leaks a protected inventory attribute | DENY public release |
| New source event arrives after the frontier was frozen | New frontier or explicit supersession; do not silently mutate the old proof |
| Independent restore preserves bytes but loses query/proof semantics | Custody-exit qualification failure |

A late source event is not retroactive proof that the earlier frontier was false; it requires a new version or a typed correction. The previous frontier stays reproducible.

## 7. Comparison with existing standards and systems

- **RFC 9942 COSE Receipts:** provides VDS proof machinery, including inclusion, consistency, disclosure and non-inclusion. Reuse the Mycelix owner-issued receipts; test the completeness predicate that sits above those primitives.
- **RFC 9943 SCITT:** provides a transparency architecture. Consume the existing owner-managed transparency service; do not launch a second CTER transparency log.
- **OPA decision logs:** already support traceable decision records and masking of sensitive fields. This is an important baseline for log minimization, but it is not a proof that a disclosed set exhausts a committed frontier.
- **W3C PROV-O / ODRL:** provide provenance and policy vocabularies. They do not solve confidential query completeness by themselves.
- **OCFL / BagIt:** help package and preserve objects with integrity and version history. They do not define a community-scoped semantic query or prove that its result is complete.
- **ODK / QField / Open Tenure / Cadasta:** provide practical field capture, offline collection and community workflows. They should remain capture-layer comparators, not be judged by a cryptographic proof benchmark they were not designed to implement.

## 8. Qualification targets

Targets, not achieved results:

| Metric | Target |
|---|---:|
| In-scope eligible items omitted while coverage proof says complete | 0 |
| Withheld/unavailable/unresolved states silently converted to absent | 0 |
| Duplicate or overlapping dispositions accepted | 0 |
| Proof accepted under a different request/query/recipient/frontier | 0 |
| Invalid, stale or forked receipt accepted | 0 |
| Protected object identities/counts/locations exposed in frozen public-channel tests | 0 |
| Global completeness claimed from a scoped frontier proof | 0 |
| Independent verification agreement on frozen proof fixtures | 100% |
| Independent restore of frontier, query, partition and proof metadata | 100% |

Benchmark proof generation and verification over 1k, 10k, 100k and 1M frontier entries. Report proof bytes, public metadata bytes, p50/p95/p99 generation and verification time, peak memory, CPU, storage amplification, update/reconciliation time and every side-channel leakage found. Report exact verifier disclosure level for every fixture. Do not combine security, privacy and performance into one weighted score.

## 9. First implementation sequence

1. Confirm Mycelix's existing RFC 9942/SCITT VDS proof types and owning APIs.
2. Define the formal partition semantics and query/frontier commitment boundary without choosing the proof primitive yet.
3. Create a synthetic fixture corpus with one valid partition and adversarial omitted/duplicated/withheld/unavailable/substitution cases.
4. Model verifier-specific disclosure profiles and public/private proof surfaces.
5. Compare a Mycelix receipt-only approach, an attested custodian evaluation and a ZK-completeness candidate on the same fixtures.
6. Add the chosen proof consumer to CTER only after Mycelix owner confirmation and independent privacy review.
7. Re-run the same fixtures during custody-exit restore without original-provider credentials.

No production privacy guarantee, global absence assertion or cryptographic completeness claim is made by this research profile.


## 10. Initial machine-readable artefacts

The first model artifacts are committed:

- [Private coverage invariants](../../benchmarks/territorial-integrity/egress/private-coverage-invariants-v0.1.json): 16 named invariants covering frontier identity, query binding, one-and-only-one partitioning, package leaf-set equality, proof freshness, privacy, verifier scoping and independent restore.
- [Private coverage adversarial corpus](../../benchmarks/territorial-integrity/egress/private-coverage-cases-v0.1.json): 17 synthetic cases comprising one valid partition baseline plus 16 adversarial specifications.

The baseline intentionally contains a private partition with included, withheld, unavailable, excluded and unresolved items, while its public verifier view exposes only synthetic commitments and an opaque proof reference. No real personal, community or territorial evidence is used.

These artifacts are a formal model candidate, not an executable proof verifier. The corpus states expected outcomes but is not yet run by a validator. Its proof primitive remains unselected. In particular, no RFC 9942 COSE receipt, SCITT receipt, Merkle proof, non-inclusion proof or zero-knowledge proof is fabricated or represented as valid.

The next step is to confirm Mycelix's implemented VDS/receipt types and test which of the required frontier/query/partition properties those receipts can actually prove. A proof that each included object exists in a committed set is insufficient to prove that all objects matching the query were included.
