# Highveld Keep — decision-reliance profile v0.1

**Purpose:** Map the existing Highveld Keep evidence owners to the v0.1 proof classes in the #513 decision-reliance reference implementation.
**Status:** research profile; not an executable project approval profile.
**Reference engine:** PR #513, `sol-atlas-core/src/decision_reliance.rs`.
**Claim ceiling:** profile mapping and synthetic integration planning only.

## 1. Why this profile exists

The habitat workflow now has separate owners for byte verification, source coverage, IEAC projection, native identity, invalidation, owner impact routing, restoration feasibility, temporal continuity, measured in-use outcomes, common-scope binding and statutory/professional review. A decision-reliance receipt should compose their exact outputs instead of treating a graph node or a set of green checks as sufficient.

The general reference engine in #513 has 11 typed proof classes. This profile maps those classes to the Highveld owner chain and preserves the evidence boundaries.

## 2. The v0.1 schema conflict

PR #505 and PR #513 both contain a file named `schemas/decision-reliance-receipt-v0.1.schema.json`, but the contracts differ. #513's Rust model has 11 proof classes. #505's alternative schema has a different receipt layout and 14 classes, including separate professional-review and schema-interoperability obligations.

These alternatives cannot both be canonical under the same v0.1 schema ID. This profile explicitly follows the #513 reference engine for its 11 core classes while recording the additional obligations that the engine cannot yet represent. It does not replace, rewrite or merge either v0.1 schema.

## 3. Owner-to-proof mapping

| v0.1 proof class | Habitat evidence owners and receipts |
|---|---|
| `decision_event_integrity` | Immutable decision event bytes plus the signed-statement/registration trust owners (#347) |
| `native_source_identity` | IEAC native identity and graph projection (#487/#491); v0.2 invalidation/event compatibility draft #670 |
| `source_payload_bytes` | #500/#512 byte verifier; #671 is the owner-payload adapter draft |
| `source_use_rights` | Source-specific rights, purpose, jurisdiction and resident-consent owners (#633/#635 where applicable) |
| `source_attestation_trust` | Issuer/key lifecycle and transparency receipts (#347); a receipt reference alone is not verification |
| `source_coverage` | Source inventory and coverage (#488/#490) |
| `projection_integrity` | Native IEAC projection (#491) and common-scope binding (#624) |
| `engineering_domain_validity` | Specialist solver, product, inspection, commissioning and accepted-output owners; #646/#617/#622 are relevant consumers |
| `lifecycle_currentness` | Temporal synchronization #638, invalidation #650, owner routing #651 and source lifecycle #397 |
| `impact_frontier_correctness` | IEAC impact-frontier owner #484, compatibility #670, city change-impact #644 and owner routing #651 |
| `actor_reliance_authority` | Actor capability and reliance-purpose authority; South African approval owner #604 is a separate jurisdictional input |

Every proof must bind the same exact decision, scope, subject set, functional unit, purpose, jurisdiction and evaluation time. A proof that covers a different population, phase, site, scenario or revision is not comparable.

## 4. Three additional blockers that cannot be collapsed into v0.1

### Professional review

A solver or owner result is not a professional sign-off. The current #513 vocabulary has no distinct `professional_review` proof class. Do not silently map it to `engineering_domain_validity` or `actor_reliance_authority`.

### Statutory and jurisdictional approval

Zoning, parcel-specific geotechnical/dolomite investigation where applicable, fire, structural, accessibility, water, environmental and other statutory approvals remain with the competent authority and relevant qualified professionals. A technically valid graph or actor capability does not prove these approvals.

### Schema interoperability

An import/export or projection pass is not proof that independently implemented consumers interpret the same claim identically. The current #513 vocabulary has no distinct `schema_interoperability` proof class. A v0.2 contract should require pinned schema profiles, exact-version support, transformation-loss reports and differential agreement across independent implementations.

These are mandatory external blockers in the JSON profile. Consequently `can_emit_reliable_for_declared_scope` is false under the present v0.1 engine. This is intentional.

## 5. Functional-unit and no-compensation rules

- `FU-COMP-01`, `FU-BLDG-01` and `FU-DIST-01` are not interchangeable. No component-level result is multiplied into a neighbourhood service claim without an explicit reviewed model/crosswalk.
- All mandatory proof classes must cover every requested subject and scope.
- `unknown`, `unsupported`, `stale`, `revoked`, `not_authorized` and `not_comparable` remain visible and cannot be averaged into a confidence score.
- Historical authenticity and current reliance are distinct.
- A panel acceptance receipt does not establish building commissioning, stable service continuity or lived outcomes.
- A `reliable_for_declared_scope` receipt never grants construction, operation or actuation authority. `action_authority` remains `none`.

## 6. Integration sequence

1. Complete #670's additive v0.2 event/report compatibility and qualify it independently.
2. Complete #671 payload-byte adapter qualification and bind its exact output to #500's verifier report.
3. Keep #624's common-scope binder as the only scope authority; consume its exact row bindings.
4. Complete #667's service restoration composition and preserve #646/#574/#635 owner axes.
5. Reconcile #505/#513 wire-contract divergence before any canonical decision-reliance v0.2 receipt is promoted.
6. Add independent proof classes for professional review, statutory approval and schema interoperability before this profile may emit reliable reliance.
7. Only then run an end-to-end Highveld synthetic case with exact source commits, graph/event/report digests and one decision-reliance receipt.

## 7. References

- [#501 decision-reliance objective](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/501)
- [#513 Rust reference implementation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/513)
- [#505 alternative receipt schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/505)
- [#487 native identity v0.2](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/487)
- [#670 invalidation/event/report compatibility](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/670)
- [#667 restoration outcome bridge](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/667)

**No Highveld project decision is qualified by this profile.** It defines how the existing evidence owners should be joined without crossing their authority boundaries.
