# Engineering decision reliance receipt v0.1

**Status:** formal JSON Schema contract and synthetic refusal fixture implemented; Rust evaluator, independent temporal oracle and exact-head schema qualification are pending.
**Parent:** [#501 — proof-carrying, time-qualified decision reliance](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/501).
**Composes with:** [#486/#491 projector](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/491), [#492/#496 source descriptors](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [#500 payload byte verification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/500), [#488/#490 coverage](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/490), [#484 impact oracle](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/484), [#397 lifecycle invalidation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/397), and [#347 trust roots and statement transparency](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347).

## 1. User question

The receipt answers one narrowly defined question:

**At this exact evaluation time, can this exact engineering decision still be relied upon for this declared purpose, jurisdiction and scope, by this actor, given the evidence and changes currently known to the system?**

It is not a digital-twin snapshot, document approval, confidence score, or automatic action authorization.

The receipt is a deterministic evaluation of independently qualified evidence classes. It must preserve missing, stale, invalid, unsupported and not-comparable outcomes. A missing proof is not a negative fact.

## 2. Five outcomes

- `reliable_for_declared_scope`: every required proof obligation is qualified, the actor is authorized for the declared reliance capability, no mandatory revalidation remains, and the receipt is bound to exact current inputs.
- `revalidation_required`: a known change invalidates a result or introduces an outstanding blocking obligation.
- `insufficient_evidence`: a mandatory proof is absent, stale, unsupported, unverified or not comparable.
- `not_authorized`: the evidence may be technically qualified, but the actor lacks a current authority grant for the declared purpose/capability.
- `invalid_evidence`: identity/digest conflict, signature failure, broken temporal/event chain, contradictory receipt, or another integrity failure.

These are mutually exclusive top-level outcomes. A verifier must have an explicit deterministic precedence rule for cases with multiple defects; the schema only constrains each outcome's minimum evidence conditions.

## 3. Required proof profile

The receipt carries the required proof-class profile independently of the evidence rows. This is important: an evaluator must not be able to omit a difficult proof class and then report all of the remaining rows qualified.

The v0.1 proof vocabulary includes:
- decision identity;
- source payload bytes;
- source-owner authority;
- source snapshot currentness;
- source rights;
- source inventory coverage;
- dependency completeness;
- projection semantics;
- impact frontier correctness;
- lifecycle currentness;
- engineering verification;
- professional review;
- actor authorization;
- schema interoperability.

The profile is purpose- and domain-specific. A limited design-review query may require a different proof set from construction release, maintenance, or operation. Required classes must be bound to an immutable profile digest and reviewed before the evaluation. The v0.1 schema carries the declared class list; the future Rust evaluator must independently verify that there is exactly one obligation per required class, with no omissions, duplicates, or unexpected promotion.

## 4. Time model

The receipt separates:
- **decision time:** when the original decision event was made;
- **valid time:** when the underlying fact or approval is asserted to hold;
- **observation time:** when a source observed it;
- **system record time:** when the evidence platform recorded it;
- **trusted registration time:** when a transparency service registered the signed statement;
- **evaluation time:** when the reliance query ran.

A historical signature may remain authentic even after an authority key is revoked. A source correction may arrive after the original decision but have a valid time earlier than it. A calibration can be valid at decision time but expired at present evaluation time. The evaluator must determine the consequence according to declared policy, not overwrite history.

The schema carries a knowledge watermark with separate system-recorded and trusted-registration watermarks. If there is no authenticated transparency watermark, it remains null; a caller-supplied clock or timestamp must not be represented as trusted registration time.

## 5. Exact input binding

The receipt has separate digests for:
- IEAC graph snapshot;
- projection receipt;
- source descriptor catalogue;
- payload byte verification report;
- dependency coverage report;
- trust/issuer evaluation;
- lifecycle report;
- engineering verification report;
- independent impact report.

A digest is a binding to an artifact representation, not proof that the artifact is truthful or sufficient. The independent evaluator must verify that each referenced artifact exists, uses the expected schema/version, has the declared digest, and was produced by a qualified verifier under the current trust policy.

## 6. Authority separation

`actor_authorization` answers whether the actor may rely on or change the decision for the stated capability. It is not an action grant.

`action_authority` is fixed to `none` in v0.1. Construction release, operation, and actuation require separate authorization and safety/interlock systems. A receipt that says `reliable_for_declared_scope` never independently authorizes those actions.

Professional review is a separate proof class. A signature, VC, SCITT receipt, or validated graph cannot replace professional engineering judgment or statutory approval.

## 7. Schema claim boundary

The Draft 2020-12 schema:
- constrains the result vocabulary;
- requires an explicit proof profile and obligation rows;
- requires all obligations qualified, actor authorized and no revalidation outstanding for the `reliable_for_declared_scope` state;
- requires at least one blocking revalidation item for `revalidation_required`;
- prevents `not_authorized` from simultaneously claiming an authorized actor;
- requires an invalid proof row for `invalid_evidence`;
- requires at least one unresolved proof row for `insufficient_evidence`;
- fixes action authority to none.

JSON Schema does not independently authenticate digests, verify signatures, check current time, replay event histories, prove that the obligation rows match the required profile, or establish engineering correctness. Those are Rust evaluator and independent oracle responsibilities. The synthetic fixture deliberately demonstrates `insufficient_evidence`, not a successful reliance claim.

## 8. Qualification programme

P0:
- zero false `reliable_for_declared_scope` against an independently authored oracle;
- zero omitted required proof classes accepted;
- zero invalid/stale/revoked mandatory evidence treated as qualified;
- zero unauthorized reliance;
- zero `NoImpact` reliance with incomplete or unauthenticated source coverage;
- exact receipt replay from exact input bytes and verifier build;
- late-arriving corrections and revocations evaluated with bitemporal history;
- all non-claims and unknowns remain visible.

P1:
- 10, 100, 1,000, 10,000 and 100,000 graph/decision strata;
- at least 10,000 deterministic event mutations and 1,000 paired bitemporal histories;
- p50/p95/p99/worst-case query latency, memory, proof bytes, invalidation delay and revalidation work;
- compare full recomputation against selective revalidation with zero missed obligations;
- cross-case reuse of shared permits, certificates, utilities and metrology records.

P2:
- rights-cleared held-out project records;
- independent source-owner and professional review;
- prospective decision changes and operational outcomes;
- independent verification by a second implementation.

## 9. Standards composition and industry position

The evaluator should compose with NIST IR 8536 for manufacturing traceability, IETF RFC 9943 / SCITT for transparent signed-statement history, W3C Verifiable Credentials 2.0 for portable issuer/holder/verifier claims, and ISO 23247-6:2026 for digital-twin composition. Those standards provide important building blocks; none alone proves that a particular engineering decision remains justified under all current dependencies, changes, rights, authority and domain checks.

Benchmark actual integration surfaces against Bentley iTwin synchronization/exports and validation, Siemens Digital Twin Composer simulation/virtual commissioning, Palantir Ontology actions/security, and buildingSMART IFC/IDS conformance. Matched scope and exact versions are mandatory; inaccessible data is unassessed, not absent. See [#502](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/502) and the pre-registration requirements in [#338](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/338).

The differentiated claim is not "we have a digital twin" or "we have a trust graph." It is that a specific decision's continued reliance can be independently replayed, scoped, temporally qualified, and refused when any mandatory proof is missing.
