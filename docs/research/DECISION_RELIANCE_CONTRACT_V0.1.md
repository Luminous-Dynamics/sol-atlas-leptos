# Proof-carrying engineering decision reliance v0.1

**Status:** formal receipt schema, typed Rust reference oracle, and frozen synthetic temporal corpus implemented on a research branch. Exact-head formatting, strict Clippy, unit tests and schema qualification remain pending.
**Parent:** [#501 — proof-carrying, time-qualified engineering decision reliance](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/501).
**Composes with:** [#496 source descriptor sidecar](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [#500 payload byte verification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/500), [#488 dependency coverage](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488), [#486 projector](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/491), [#397 lifecycle invalidation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/397), [#484 impact-frontier oracle](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/484), and [#347 trust/canonicalization](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347).

## The query is a decision, not a confidence score

The engineering question is:

> Can this exact decision still be relied upon, at this evaluation time, for this actor, purpose, jurisdiction and subject scope—and which qualified receipts justify that answer?

A historical approval event is not current permission. A matching byte digest is not source authorship. A source graph that validates is not necessarily complete. A correct impact traversal is not sufficient if the extractor omitted a required dependency. A current engineering report does not automatically authorize the actor making the request.

The reference oracle therefore treats reliance as a conjunction of proof obligations, with explicit refusal outcomes. It does not average these obligations into one score.

## Mandatory proof classes

A request declares its required classes. The v0.1 benchmark baseline exercises all eleven:

1. `decision_event_integrity` — exact decision event identity and immutable event receipt;
2. `native_source_identity` — owner namespace and source-native record identity;
3. `source_payload_bytes` — actual byte equality against the declared source digest;
4. `source_use_rights` — rights evidence valid for the requested purpose and time;
5. `source_attestation_trust` — issuer, key, signature/transparency and revocation evidence;
6. `source_coverage` — bounded source universe and mandatory dependency-family completeness;
7. `projection_integrity` — exact source-to-IEAC mapping, graph and loss-accounting receipt;
8. `engineering_domain_validity` — domain-specific verification results and assumptions;
9. `lifecycle_currentness` — current temporal state of all relied-upon evidence;
10. `impact_frontier_correctness` — independent correctness of the affected/revalidation set;
11. `actor_reliance_authority` — actor capability/authority for this exact purpose, jurisdiction and scope.

The receipt schema deliberately distinguishes each proof result. A missing or not-comparable proof is not a negative fact; it is insufficient evidence. A historical record may remain authentic while no longer being usable for current reliance.

## Exact purpose, jurisdiction and subject applicability

Every proof is bound to:
- one or more supported purposes;
- an exact jurisdiction;
- one or more scope subject identifiers.

The oracle checks that each required proof class covers every requested subject for the exact purpose and jurisdiction. A proof scoped to design review cannot satisfy a construction-release request. A certificate from another jurisdiction cannot silently qualify the current request. One segment's inspection receipt cannot support a corridor-wide decision unless the set of proofs covers the complete requested subject set.

Optional/contextual proofs do not contaminate a query merely because they are included in the receipt. Likewise, a change event that affects only an optional proof outside the exact required proof set is retained as contextual information but does not invalidate the current request. Events with references that do not resolve to any proof in the receipt remain insufficient evidence.

## Five result states

| Result | Meaning | Typical causes |
|---|---|---|
| `reliable_for_declared_scope` | All mandatory proof classes qualify and are current for the exact request | Complete current proof set, no trusted invalidating event |
| `revalidation_required` | A known change makes at least one relied-upon proof stale or downstream work outstanding | Source correction, new dependency, expired calibration, schema change, solver build revocation, inspection hold |
| `insufficient_evidence` | A proof is missing, unsupported, unverified, stale in its registration history, not comparable, or fails purpose/scope coverage | Missing source family, untrusted change, incomplete subject scope |
| `not_authorized` | The evidence may exist, but rights or actor capability do not authorize this exact reliance request | Rights revoked/expired, wrong actor role, purpose mismatch or expired authority proof |
| `invalid_evidence` | A contradiction or integrity failure prevents reliance | Digest/identity contradiction, malformed time/hash, invalid event trust, key revoked before the decision, malformed event-to-proof type binding |

Outcome precedence is explicit: invalid evidence, then not authorized, then revalidation required, then insufficient evidence, and only then reliable. This prevents a missing proof from hiding a known integrity failure, or a stale report from masking an authority withdrawal.

No result grants construction, operational or actuation authority. The receipt's `action_authority` is always `none`.

## Bitemporal and trust semantics

The reference fold preserves:
- **valid time** — when the fact/change is asserted to apply;
- **observation time** — when it was observed;
- **record time** — when the evidence system learned/recorded it;
- **trusted registration time** — when the statement/event was qualified by the separate trust provider;
- **decision time** — when the historical decision was made;
- **evaluation time** — when current reliance is queried.

A backdated source correction recorded after the decision but before the evaluation is treated as a late-arriving retroactive change and requires revalidation. A correction registered after the evaluation is not leaked into the historical query. A key revoked before the decision is invalid evidence for the historical decision; a key revoked after the decision requires current trust re-evaluation. A rights or actor-authority revocation maps to `not_authorized`, not simply stale.

Change events are bound to explicit affected proof IDs. Rights revocation must affect a rights proof; actor-authority expiry must affect an authority proof; key revocation must affect an attestation-trust proof. An incorrectly typed event is invalid. Events with unverified trust are not used as trusted invalidations and instead leave the result insufficient.

## Industry baseline and where to differentiate

Do not claim that digital-twin federation, traceability, or operational ontologies are new. The standards and commercial baseline is already substantive:

- **ISO 23247-6:2026** explicitly defines integrated, unified and federated digital-twin composition, including interoperability among twins produced by different vendors and organizations. Sol-Atlas should be composition-compatible and make the evidence and reliance state independently checkable across the composed view.
- **NIST IR 8536 (final, September 2026)** supplies an industry-neutral manufacturing traceability meta-framework for linking and querying cross-sector provenance, cryptographically verifiable links and selective disclosure while allowing existing sector standards to remain in use. This is a foundation to compose with, not a novelty claim.
- **buildingSMART IDS** remains a concrete standards-conformance baseline. Its 2026 survey received 186 responses; 131 respondents (70%) wanted some form of IDS software certification, and respondents reported cases where the same IDS file produced different checker results. A stronger comparison must measure cross-implementation semantic agreement, unsupported-feature refusal and reason-code determinism—not simply whether an IFC file imports.
- **Bentley iTwin** exposes synchronization/export, changed-element review, schedules, reality data, validation/clash detection and workflow automation. The assurance-layer comparison is whether the exact source exports, mappings, assumptions and change conclusions can be independently replayed and scope-qualified.
- **Palantir Ontology** already combines objects, links, actions, functions and dynamic security over integrated digital assets. Luminous should not claim to invent an operational ontology; it should show portable proof receipts and independently reviewable reliance without forcing all counterparties to move into one centralized operational repository.
- **Siemens Digital Twin Composer**, unveiled at CES 2026, targets real-time product/plant/factory contexts, large-scale simulation, virtual commissioning and autonomous robot training. The decision-reliance layer should consume and qualify the outputs of such specialized systems rather than pretending to replace their simulators.

The fair competitive experiment is matched-scope and multi-dimensional. Freeze the same source snapshot, exportable relationships, purpose, jurisdiction, evaluator time, hardware and compute budget. Count inaccessible vendor-native relationships as `unassessed`, not as missing. Compare:
1. native identity and schema fidelity;
2. source/relationship coverage against an independent gold inventory;
3. source byte and attestation verification;
4. decision reliance classification against an independent bitemporal oracle;
5. missed invalidations and false reliable outcomes;
6. revalidation latency and avoidable recomputation;
7. proof/report byte size, memory, throughput and operational burden;
8. refusal quality for unknown, unsupported and untrusted evidence.

A credible P0 advantage is zero false-reliance outcomes and zero missed required invalidations on frozen corpora. A P1 advantage is lower end-to-end revalidation cost while retaining those gates on 10–100,000 node/decision strata and held-out project/source-system combinations. We must not advertise a commercial head-to-head until exact vendor versions, comparable export scope, configurations and raw artifacts have actually been run.

## Ownership and non-claims

This is a reference fold, not a replacement for upstream proof providers:
- #492/#496 owns the descriptor contract and native identity metadata;
- #500 owns source payload byte re-hashing;
- #489/#490 owns bounded dependency/source coverage;
- #486/#491 owns projection integrity and mapping receipts;
- #347 owns canonicalization, signatures, transparency, trust roots, key status and revocation;
- #397 owns lifecycle invalidation and temporal evidence state;
- #484 owns independent impact-frontier correctness;
- domain verifiers and qualified professionals own engineering-domain correctness and professional approval;
- the authorization provider owns actor capabilities and grants.

The reference oracle consumes these proof statuses. It does not authenticate their signatures, rehash source bytes, evaluate legal rights, run engineering solvers, recompute the complete dependency inventory, or verify actor credentials. A `qualified` input status is a claim made by a separate provider and must be backed by its own retained receipt and exact verifier build.

The receipt's `canonicalization_profile` and `receipt_sha256` are explicit inputs to #347. This module does not choose JCS versus deterministic CBOR/COSE, and it does not issue or sign the final canonical receipt.

## Independent temporal corpus

`schemas/examples/decision-reliance-temporal-corpus-synthetic-v0.1.json` contains eight frozen adversarial histories, now each with all eleven proof classes and explicit purpose/jurisdiction/scope bindings:

- complete current proof set;
- backdated correction discovered after the decision;
- key revoked before the historical decision;
- rights revoked after the historical decision;
- missing coverage evidence;
- future correction not yet registered at evaluation;
- unverified change event;
- expired actor-authority proof.

The Rust reference oracle test loads the frozen corpus, checks each expected outcome, and checks the late-arriving event denominator. Additional hand-built controls cover purpose mismatch, jurisdiction mismatch, incomplete subject coverage, optional proof non-contamination, and event/proof-class mismatch.

This corpus is synthetic. It qualifies reference folding semantics only; it does not authenticate source or engineering evidence.

## Qualification targets

P0, non-compensable:
- zero false `reliable_for_declared_scope` outcomes against an independently authored oracle;
- zero missed trusted invalidations for any required proof class;
- zero authority/rights escalation;
- zero future-information leakage into historical queries;
- zero incomplete or unsupported required scope accepted as reliable;
- exact replay of receipt inputs, event ordering, proof IDs and findings;
- all denominators preserved, including missing, unsupported, not-comparable, stale, not-authorized and untrusted events.

P1:
- 10, 100, 1,000, 10,000 and 100,000 subject/proof/decision strata;
- 10,000+ deterministic temporal mutations and at least 1,000 paired event histories;
- p50/p95/p99/worst-case reliance query latency, peak RSS, proof bytes and event-fold throughput;
- false reliance, false refusal, incomplete rate, authority refusal, invalidation detection latency and selective revalidation work avoided;
- full recomputation versus selective invalidation, with zero missed obligations;
- cross-case decisions relying on shared permits, utility constraints, survey records and material certificates.

P2:
- rights-cleared held-out project/source-system histories;
- independent professional and source-owner review;
- production trust provider, revocation, lifecycle and coverage receipts;
- prospective evaluation on real change histories before any market-leading claim.

No aggregate score may allow speed to compensate for false reliance, missed invalidation or authority escalation.


Implementation note (2026-11-04): the Rust reference evaluator and temporal mutation corpus are formatted under the repository's stable Rust toolchain. Exact-head strict Clippy, unit tests, and schema validation remain the qualification gates; no reliance accuracy or production assurance claim is made until those complete.
