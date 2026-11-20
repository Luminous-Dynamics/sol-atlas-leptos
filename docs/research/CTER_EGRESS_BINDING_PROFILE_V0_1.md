# CTER Egress Binding and Proof-Carrying Disclosure Profile v0.1

**Tracking:** [CTER-EGRESS-BINDING-001 #960](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/960)  
**Parent:** [CTER-AUTH-CONSUMPTION-001 #951](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/951)  
**Status:** research contract; synthetic qualification only; no production egress adapter is claimed.  
**Owner:** Sol-Atlas CTER consumer/enforcement boundary.  
**Research date:** 2026-11-20.

## 1. Core shift: authorize a scope, then bind the exact use

The authorization consumer in #951 asks whether upstream decisions match one exact request: principal, subject, operation, resource, project snapshot, purpose, recipient, jurisdiction, data classes and evaluation context.

That is necessary, but it is not the final disclosure proof.

A request can be eligible and the package subsequently changed. An additional object can be inserted; a recipient endpoint can rotate; a transform can be swapped; the project/evidence frontier can advance; a withdrawal or revocation can arrive between evaluation and release. The egress boundary must therefore prove that the bytes leaving are the bytes described by the authorized scope and the approved transformation lineage.

The target is **proof-carrying disclosure**:

1. An upstream owner makes and qualifies the decision.
2. CTER materializes a recipient-specific disclosure package.
3. CTER binds the package manifest to the exact request and upstream decision references.
4. The host re-resolves currentness/revocation and recipient binding immediately before release.
5. The sender records the attempt and reconciles any uncertain outcome without unsafe replay.
6. A recipient or independent verifier can check the package, its lineage and the declared limits without being given unrelated protected data.

This is an enforcement/continuity layer around existing decisions, not another authority system.

## 2. Ownership and non-equivalences

| Fact | Owner / source | What it does not establish |
|---|---|---|
| Identity and issuer trust | Mycelix identity/trust resolvers | Authority for a particular action |
| Participant consent, site access, publication consent and community mandate | Mycelix LAND-STEWARD and the community's own applicable process | Statutory title or a generic permission for every use |
| Typed AuthorizationReceipt and its currentness/revocation result | Mycelix LEX-NET / relevant authority owner | Truth of source claims or proof that a later package is unchanged |
| Source claim, restriction, derivation, objection and dispute lineage | Mycelix evidence owners and source custodians | Automatic permission to disclose |
| Spatial/project/evidence snapshot semantics | Sol-Atlas and source owners | Legal registration or adjudication |
| Output manifest and exact package commitment | CTER egress consumer | Authority to release the package |
| Transport delivery | Transport provider / Mycelix transport stage where applicable | Recipient acknowledgement, comprehension or external finality |
| External legal effect | Competent external registry, court or authority | A fact that CTER or Mycelix may mint |

Core non-equivalences:

- eligible request != eligible package;
- package hash != source truth, consent, authority or legality;
- transformation receipt != permission to declassify;
- transport delivery != recipient acknowledgement;
- recipient acknowledgement != understanding or acceptance;
- withdrawal after release != deletion of every downstream copy;
- hash equality != confidentiality;
- missing source/receipt != proof that no underlying right or decision exists.

CTER MUST NOT create an AuthorizationReceipt, resolve community representation, invent a global data-class hierarchy, mint a transparency receipt, or assert legal title.

## 3. The disclosure package contract

A future machine profile should separate three objects, rather than one overloaded JSON record.

### 3.1 Authorization binding

The binding references, without reinterpreting:

- exact normalized authorization request and request commitment;
- Mycelix AuthorizationReceipt reference and exact receipt-type/profile;
- upstream resolver/currentness/revocation result references;
- policy and input-receipt lineage;
- exact principal, subject, operation, resource, project snapshot, purpose, recipient, jurisdiction and data-class scope;
- host evaluation context and trusted-time basis.

The public synthetic evaluator may model these values, but production values are accepted only from the canonical Mycelix resolver boundary. A JSON field that says signature verified, issuer trusted or authority resolved is not itself proof of that fact.

### 3.2 Disclosure manifest

The immutable manifest identifies every output object using opaque references and records, at minimum:

- output object reference;
- exact payload digest and explicit digest algorithm;
- byte length and media type;
- output data-class labels;
- source object references and source digests;
- every transformation reference, transform-profile identifier and exact version/commitment;
- project snapshot and evidence-frontier references;
- recipient reference, destination/key epoch and transport profile;
- the exact authorization request/decision references under which this output was prepared.

No raw names, claimant identities, exact sensitive coordinates, sacred-site descriptions, private object paths or protected payloads belong in a public manifest. Even a hash of a low-entropy location or identity can leak information through guessing. Manifest visibility is itself governed and must be recipient/custodian scoped.

A disclosure manifest is not a public inventory. The complete manifest may be private to the authorized recipient and custodian; an independent verifier receives only the minimum proof necessary for its task.

### 3.3 Custody-controlled coverage record

The package must preserve the disposition of the requested evidence frontier: included, withheld, unavailable, excluded by scope, or unresolved. These categories must not be silently collapsed into absence.

The coverage record may contain sensitive counts and object references. It is therefore separately access-controlled. A public response should not reveal that a protected object exists merely by publishing a count, identifier, digest or gap description.

The export profile must define the coverage universe precisely. "Complete" means complete for a named source frontier and declared query/scope, not complete for the world.

## 4. Lineage closure and transformation discipline

For every disclosed output object, the manifest must support a closed, acyclic path to one or more source objects and the transformations applied.

- An unchanged copy is still an explicit identity-copy transformation.
- A redacted, generalized, aggregated, translated, converted or summarized output records its transform profile and version.
- Every transform records exact input and output commitments.
- Missing inputs, missing transform ancestors, cycles, unresolved versions or unqualified transformations return INCOMPLETE/UNKNOWN.
- An AI-generated summary is a derived artifact, not a source fact or authority receipt.
- A transformed output cannot silently inherit the original source's evidence class or authority.
- A data-class change is not automatically safe because a field was removed or a coordinate was generalized.

Data classes such as public, community, restricted and culturally protected are not assumed to form one total order. A permitted transformation from one class to another requires a named, separately qualified policy and the relevant authority. CTER does not define a universal declassification algorithm.

The first implementation should prefer explicit allowlists and exact transformation profiles. It should not attempt to prove that every possible inference channel has been eliminated.

## 5. Canonicalization and digest rules

A future v0.1 manifest must specify one canonical serialization profile before any digest is treated as interoperable. For JSON, evaluate RFC 8785 JCS and its I-JSON constraints; for binary package containers, preserve exact byte-level commitments and use a pinned package profile. Canonicalization test vectors are mandatory across Rust and the independent implementation.

Keep digest domains explicit:

- Sol-Atlas source/output byte references currently use SHA-256.
- Mycelix signed scientific-event identity uses BLAKE3.
- A Mycelix BLAKE3 event identity is not a SHA-256 digest of an exported file.
- A payload digest, a manifest digest, a package/container digest and an event identity are distinct fields and MUST NOT be substituted.
- Include algorithm identifiers with every digest; never infer an algorithm from length or field name.
- Define domain separation for commitments and version the canonicalization profile.
- Do not publish a digest of sensitive low-entropy values without a privacy analysis and an approved disclosure mechanism.

RFC 8785 is Informational, not an Internet Standards Track specification. It is a candidate canonicalization profile, not a claim of standards-level mandate. Its exact constraints and test vectors must be adopted explicitly if selected.

## 6. Last-responsible-moment authorization and TOCTOU

Authorization must be checked twice at distinct stages:

**Preparation:** bind the frozen request and exact output manifest to the qualified upstream decision references.

**Release:** immediately before bytes leave the controlled boundary, the trusted host rechecks:
- request and manifest commitments still match;
- recipient and destination key epoch still match;
- project snapshot and evidence frontier have not changed outside the authorized scope;
- upstream receipt validity, revocation and currentness remain acceptable;
- offline lease remains valid under the operation/data-class policy;
- required community or custodian restrictions have not changed;
- no unresolved challenge or new prohibition blocks this use.

The host's time and resolver context must be independently qualified. The caller's evaluated_at value or a disconnected relay's wall clock is not trusted time merely because it is syntactically valid. The synthetic consumer can test time comparisons; it cannot authenticate the time source.

A revocation or withdrawal race between preparation and release must block release until the host re-resolves. If the operation has already been released, the system records what it knows and does not claim remote deletion.

A stale relay cannot extend its own lease or convert unavailable revocation state into a positive decision.

## 7. Release state and unknown outcomes

Use the existing typed Mycelix receipt stages where they exist. Do not invent a new CTER receipt hierarchy.

The implementation must distinguish at least these operational facts:

- prepared: package and manifest were materialized and locally validated;
- release-attempted: the host began the exact one-shot release attempt;
- transport-accepted: the transport reports custody/delivery at its own layer;
- recipient-acknowledged: the intended recipient confirms receipt of the exact package commitment;
- reconciled: an uncertain attempt has been resolved with supporting evidence;
- externally-final: a separately qualified external process establishes finality, where applicable.

These are not interchangeable and must not be collapsed into success=true.

An unknown outcome is a safety state. A non-idempotent export/publication must not be repeated with a new attempt merely because the acknowledgement was lost. Reconcile the original attempt first. A retry is permissible only when the operation's idempotency semantics are explicit and the exact package, recipient, key epoch and attempt identity remain bound.

A transport receipt proves only the transport-stage fact specified by its type. It does not prove recipient understanding, data correctness, legal effect or erasure of other copies.

## 8. Standards and industry positioning

| Existing mechanism | What it already provides | CTER use / remaining question |
|---|---|---|
| Mycelix LEX-NET typed receipts | Explicit stage separation, including authorization, transport, execution and external-effect concepts | Consume exact types; never promote a weaker receipt |
| ODRL 2.2 | Interoperable vocabulary for permissions, prohibitions, duties, assets, parties and constraints | Map CTER's narrower operation/scope and community policy references; do not create a new general policy language |
| W3C PROV-O | Interchange model for entities, activities, agents and provenance relations | Map source-to-output derivation lineage |
| W3C DCAT 3 | Dataset and distribution catalog metadata | Use only for intentionally publishable catalog metadata |
| IETF RFC 8493 BagIt | File package structure, payload manifests and reliable storage/transfer conventions | Candidate archive container; integrity is not permission or truth |
| IETF RFC 9942 and RFC 9943 | COSE proofs for VDS properties and SCITT transparency architecture | Consume Mycelix-owned proofs/receipts; no CTER transparency ledger |
| OPA decision logs | Decision IDs, query input, policy/bundle metadata and audit/debug records | A useful policy-decision baseline; test whether the combined package/lineage/release binding adds independently verifiable guarantees |
| FAO Open Tenure and Cadasta | Existing participatory/offline capture, governance, access controls, mapping, dispute workflows and institutional integrations | Fair workflow comparators; do not compare their field workflows to a synthetic CTER package validator as if they were matched products |

The proposed differentiation is not "CTER has policy checks" or "CTER hashes exports." Existing policy engines and package formats already do parts of this. The research claim is narrower: a portable and independently checkable binding from the exact upstream authorization decision, through a complete output-to-source transformation graph, to a specific recipient/package/release attempt, while preserving withheld/unavailable and dispute context across provider exit.

### References

- [W3C ODRL Information Model 2.2](https://www.w3.org/TR/odrl-model/)
- [W3C PROV-O](https://www.w3.org/TR/prov-o/)
- [W3C DCAT 3](https://www.w3.org/TR/vocab-dcat/)
- [IETF RFC 8493 BagIt](https://www.rfc-editor.org/rfc/rfc8493.html)
- [IETF RFC 8785 JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html)
- [IETF RFC 9942 COSE Receipts](https://www.rfc-editor.org/rfc/rfc9942.html)
- [IETF RFC 9943 SCITT Architecture](https://www.rfc-editor.org/rfc/rfc9943.html)
- [Open Policy Agent decision logs](https://www.openpolicyagent.org/docs/management-decision-logs)
- [FAO Open Tenure / Community Tenure Recording](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en)

## 9. Qualification programme

The first corpus is synthetic and should cover at least the following independent fault families:

1. an output object is added after authorization;
2. an output object is removed but the export is still marked complete;
3. payload bytes change after manifest commitment;
4. object reference remains the same while bytes change;
5. BLAKE3 event identity is relabelled as a SHA-256 file digest;
6. one source ancestor is missing or unavailable;
7. a transformation ancestor is missing, cyclic or version-mismatched;
8. identity-copy is falsely labelled as redaction;
9. a transformation silently downgrades a protected data class;
10. a recipient reference is correct but destination key epoch changes;
11. the transport channel changes after authorization;
12. the evidence frontier/project snapshot changes before release;
13. revocation arrives between prepare and release;
14. host time is unavailable or untrusted;
15. local clock skew makes an expired offline lease appear current;
16. offline lease is extended locally by a disconnected relay;
17. duplicate non-idempotent release is attempted after an unknown outcome;
18. the same attempt reference is reused with changed package bytes;
19. metadata, filenames, thumbnails, tiles, logs, caches or model prompts expose a protected value;
20. public low-entropy commitments leak sensitive values by guessing;
21. a withheld source is incorrectly reported as globally absent;
22. recipient acknowledgement is incorrectly treated as external finality;
23. post-release withdrawal is represented as universal deletion;
24. two independent implementations serialize the same manifest differently.

### Targets, not achieved results

| Metric | Target |
|---|---:|
| Unauthorized releases in frozen negative controls | 0 |
| Package/request/recipient/transform/frontier substitution detection | 100% |
| Output objects with complete lineage closure | 100% |
| Unauthorized data-class downgrade accepted | 0 |
| Changed bytes accepted under unchanged commitment | 0 |
| Stale/unknown time or authority promoted to release | 0 |
| Unsafe duplicate non-idempotent release after unknown outcome | 0 |
| Tested side-channel disclosures in frozen channels | 0 |
| Deterministic cross-language manifest agreement | 100% |
| Coverage accounting that silently converts withheld/unavailable to absent | 0 |

These are conformance targets, not proof of zero risk in the world. Publish the attack families, corpus digest, versions, all failures and exclusions.

Scale with 1k, 10k, 100k and 1M output objects after pinning size distribution, number of sources per output, transform depth, conflict density, network schedule and hardware. Report p50/p95/p99 manifest construction, canonicalization, validation and pre-release recheck latency; throughput; memory; storage amplification; CPU; and energy only when physically measured. Keep unauthorized-release and privacy failures separate from performance scores.

## 10. Maturity gates

Do not collapse these into one PASS:

1. schema validity;
2. deterministic manifest construction and cross-language canonicalization;
3. structural lineage-closure checks;
4. actual source-byte digest verification;
5. cryptographic receipt/signature verification by Mycelix;
6. issuer trust and decision-authority resolution by Mycelix;
7. currentness/revocation and trusted-time resolution;
8. pre-release TOCTOU recheck;
9. actual transport invocation and transport-level evidence;
10. recipient acknowledgement for the exact package commitment;
11. recovery/reconciliation after an unknown outcome;
12. independent restore and verification without original provider credentials;
13. field/community review and jurisdiction-specific legal mapping.

A queued or skipped CI job is not a pass. A structural fixture pass is not cryptographic, integration, field, community or legal evidence.

## 11. First executable slice

The first code increment should be a standalone Rust consumer validator plus an independent second-language oracle. It should consume synthetic host-resolved references, validate canonical manifest shape, compare scope bindings, verify object/manifest digest commitments over exact supplied bytes where available, enforce lineage closure and classify unknown outcomes conservatively.

It must explicitly state which checks remain external:
- signature verification and trust-anchor validation;
- Mycelix issuer and decision-authority resolution;
- community-governance determination;
- trusted time and current revocation;
- actual transport/recipient confirmation;
- inference-risk review and legal effect.

Do not block this standalone reference work on production Mycelix APIs, but do not describe its synthetic status inputs as authenticated upstream facts. When the Mycelix runtime adapter exists, feed the same frozen corpus through it and add cross-repository conformance tests.


## 12. Initial machine-readable artefacts committed

The initial profile artefacts now exist on the CTER research branch:

- Schema: \`schemas/cter-disclosure-manifest-v0.1.schema.json\`.
- Synthetic suite: \`benchmarks/territorial-integrity/egress/cases-v0.1.json\`.
- Frozen suite: one structurally valid synthetic baseline plus 31 adversarial mutation cases.

The corpus is specified but its reference validator has not yet been implemented or run. Expected dispositions are test requirements, not results. The schema validates the manifest's shape; it does not resolve cross-field equality, source/transform graph closure, exact byte digests, canonicalization, resolver trust, trusted time or release safety. Those checks belong in the Rust reference validator and its independently implemented oracle.

A package's SHA-256 digest is distinct from the manifest commitment and from Mycelix's BLAKE3 event identity. The package payload digest excludes the CTER manifest; the manifest commitment hashes the RFC 8785 canonicalized manifest with the manifest_commitment property omitted. Canonicalization must be pinned and tested before the digest is used across implementations.


## 13. Initial implementation slice committed

The first standalone reference implementation now exists on the CTER research branch:

- Rust structural evaluator: \`benchmarks/territorial-integrity/relay/reference/src/egress.rs\`.
- Rust conformance tests: \`benchmarks/territorial-integrity/relay/reference/tests/egress_conformance.rs\`.
- Independent no-dependency Node oracle: \`benchmarks/territorial-integrity/relay/reference/egress-oracle.mjs\`.
- Source-side data-class binding is required in the manifest schema.
- Scale workload specification: \`benchmarks/territorial-integrity/egress/scale-workloads-v0.1.json\`.

The evaluator checks the internal manifest shape and links between request, recipient, output, source and transformation references. It rejects output scope overreach, divergent identity-copy commitments, duplicate source/output/transform identities, unresolved lineage links and provenance cycles. It uses iterative topological elimination for cycle detection so a deeply nested graph does not rely on recursive call-stack depth.

Classification is deliberately not a total order. A source/output class-set change produces REVIEW when no authority reference is present and UNKNOWN when a reference exists but the host has not resolved it. A reference string is not proof of permission to declassify.

The Rust test module currently specifies 14 focused tests. The independent JavaScript oracle specifies 13 structural checks, including a multi-stage provenance DAG and duplicate-source rejection. The 32-entry corpus contains one valid baseline case and 31 adversarial specifications. The JavaScript evaluator source and its 13 checks were executed in a V8 harness with the committed corpus injected: **13/13 matched**. This is not a Node 22 workflow result. The 14 Rust tests remain unexecuted, and exact-head CI must pass before any Rust or Node workflow claim.

### Strict limits of this slice

- SHA-256 digest *shapes* and equality relationships are checked; the validator is not supplied with package/source bytes and therefore does not recompute payload hashes.
- The RFC 8785 canonicalization identifier is required, but canonicalization and manifest commitment recomputation are not implemented in this slice.
- The Mycelix AuthorizationReceipt reference is bound but not cryptographically authenticated or resolved.
- Issuer trust, decision-maker authority, currentness, revocation, trusted time and community governance remain upstream host responsibilities.
- Source availability, transformation correctness, declassification safety, inference-risk review, transport invocation, recipient acknowledgement and external finality are not established.
- \`valid_structure\` is not \`eligible\`, \`authorized\` or permission to release.

The scale workload file specifies 1k/10k/100k/1M output-object profiles, source fan-in, transform depth/fan-out and payload-size distributions. It records required hardware/software metadata and metrics, but contains no performance claims or arbitrary latency target.


## 14. Long-term cryptographic migration is part of custody, not just hashing

NIST IR 8547 reached final publication in October 2026 and describes the transition from quantum-vulnerable to post-quantum digital-signature and key-establishment standards. This affects CTER's long-lived evidence exports and independent restores.

The CTER manifest must not imply that SHA-256, BLAKE3, an Ed25519 signature or a COSE/SCITT receipt is timelessly secure. The current profile binds explicit algorithm identifiers and versioned profiles, but this is only the start of crypto agility.

Compose the egress binding with CTER custody exit #947 and Mycelix cryptographic renewal #1354. A future archive/renewal record should preserve:
- original digest/signature algorithm and exact object/manifest scope;
- original key/trust-anchor and verification context;
- trusted-time or transparency evidence that existed at the time;
- later revalidation and renewal events, with their exact inputs and outputs;
- migrated digest/signature algorithms and profiles;
- algorithm deprecation/compromise status at each assessment frontier;
- proof that renewal did not alter the original bytes or rewrite historical verification results.

Historical validation and present-day cryptographic acceptability are separate. A signature may have been valid under a past trust policy while its algorithm is no longer accepted for new actions. A later renewal may preserve evidence of prior validity but cannot retroactively make a previously untrusted signature authoritative.

No post-quantum algorithm has been selected or implemented in this profile. The workstream should consume NIST transition guidance and Mycelix's canonical cryptographic renewal owner rather than define a second renewal ledger.


The lineage check has an additional exact-set invariant: for each output node, the set of immediate transform inputs must equal the output's declared source-reference set. It is not sufficient for every declared source to appear among the transform inputs; an additional undeclared input is also an incomplete provenance record. The Rust suite and independent Node oracle now include a synthetic hidden-transform-input mutation.
