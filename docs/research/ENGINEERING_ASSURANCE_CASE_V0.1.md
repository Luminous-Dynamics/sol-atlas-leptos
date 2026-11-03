# Engineering assurance case v0.1 — design note

**Status:** draft representation and synthetic structural fixture. This is not an ISO/IEC/IEEE 15026-2 conformance claim, a safety certificate, an engineering approval, or an authorization mechanism.

## Why this layer exists

An IEAC graph tells us which records and relationships have been represented. A projection receipt explains how a typed source case became that graph. Neither alone explains why a specific engineering proposition is justified, what assumptions it depends upon, which defeaters could invalidate it, or which independent appraisal is required before a relying party may act.

The assurance case adds that decision-centred argument layer while preserving separation between:
- source and evidence custody;
- argument construction;
- verifier appraisal;
- relying-party policy and authorization.

This matches the separation of claim/argument/evidence structure in assurance-case practice and the distinct Attester, Verifier and Relying Party roles in the RATS architecture. The representation is a Luminous-specific draft; standards mapping and conformity assessment are separate work.

## Model

A case has one top-level claim and a set of uniquely identified claims, arguments, evidence bindings, assumptions, defeaters and verifier attestations.

- **Claim:** a bounded proposition about a subject, purpose, jurisdiction and scope.
- **Subclaim:** an obligation whose result contributes to a parent claim.
- **Argument:** an explicit inference strategy and warrant connecting premises/evidence to a conclusion.
- **Evidence binding:** a reference to an owner-native record or IEAC node, with exact payload/snapshot digest where available and a status that can remain unresolved or conflicted.
- **Assumption:** a proposition being relied upon, its owner, consequence if false and review interval.
- **Defeater:** a condition that could undermine an argument, plus detection method, severity and appraisal status.
- **Verifier attestation:** a result bound to exact input set, policy, verifier build, trust bundle and validity period.
- **Relying-party appraisal:** a separate policy decision. v0.1 is intentionally fixed at `not_evaluated`; the case has `action_authority=none`.

The schema deliberately allows unresolved evidence and proposed arguments. It does not let a schema pass masquerade as a claim appraisal.

## Synthetic example

`schemas/examples/engineering-assurance-case-tunnel-lining-synthetic-v0.1.json` models a tunnel lining dimensional-conformance claim. The top-level claim decomposes into (1) the conservative measurement interval being inside tolerance and (2) the source measurement being identifiable. It includes a calibration-expiry defeater and an unresolved inspection evidence binding. It is a draft and has no verifier result or relying-party decision.

The scalar conservative interval rule is:

`[x-u, x+u] ⊆ [L, U]`

where (x) is the measured value, (u) the expanded uncertainty, and ([L,U]) the permitted interval. Overlap with a hard boundary is not accepted by this profile. Complex GD&T, weld, surface, NDT and structural decisions require specialist obligations and independently reviewed methods.

## Cross-object validation required beyond JSON Schema

A future Rust runtime verifier must check:
1. unique IDs and all claim/argument/evidence/assumption/defeater references resolve;
2. top-level claim exists and is not its own descendant;
3. claim and argument graph is acyclic;
4. child claim scope is no broader than parent claim scope under explicit scope policy;
5. every appraised claim/argument points to a valid verifier attestation;
6. attestation input digest equals the exact evidence + argument + dependency set;
7. attestation policy/build/trust-bundle identities and valid-time are current;
8. source bindings resolve to current IEAC nodes and exact payload digests;
9. active, unknown or unevaluated critical defeaters block support;
10. no unsupported evidence binding can be counted as supporting evidence;
11. no top-level claim is promoted from proposed solely by a graph/schema pass;
12. relying-party appraisal remains a separate decision with its own policy digest, actor authority, decision record and revocation lifecycle.

## Integration

- Symthaea can propose claim decompositions, arguments and candidate defeaters. These remain proposals.
- Sol-Atlas supplies identity-preserving graphs, transformation receipts, source coverage and change-impact obligations.
- Symtropy supplies physical, metrology, inspection and accepted-work results; each result is bound to exact input digests and its information horizon.
- Mycelix supplies owner/issuer/verifier identity, custody, authority, corrections, dispute and revocation history.
- #347 authenticates trust roots and signed payloads.
- #397 evaluates temporal freshness and invalidation.
- #484 supplies the independent dependency/impact oracle.
- #489/#490 qualify source coverage and internal manifest consistency.
- #491 projects typed BuiltEnvironmentCase input into IEAC v0.2 with a loss receipt.
- #492 makes source-native descriptors durable rather than caller-supplied only.

## Benchmark

Do not score the case as one scalar. Compare against independently authored, professionally adjudicated gold cases. Mutate one assumption, evidence record, source namespace, verifier build, policy, calibration interval, jurisdiction, load case, defeater and dependency at a time while preserving the rest of the case.

Report:
- claim and proof-obligation coverage;
- evidence-to-claim binding precision/recall;
- warrant/premise structural integrity;
- defeater detection recall and false activation;
- stale/revoked attestation acceptance;
- false-supported top-level claims (hard target: zero);
- missed mandatory invalidations (hard target: zero);
- over-invalidation and rework;
- unresolved/unsupported/refused cases and full denominators;
- replay equality, p50/p95/p99 latency and memory.

Professional adjudication is required for semantic claim validity. JSON Schema, cryptographic validity and internal graph consistency are necessary controls, not proof that the engineering proposition is true.
