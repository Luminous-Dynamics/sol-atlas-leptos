# Semantic transformation receipt v0.1

**Status:** Rust receipt model, consistency validator, Draft 2020-12 schema, frozen synthetic fixture and exact-head workflow are implemented on a research branch. Exact-head Rust and schema qualification remains pending.
**Parent:** [#518 — ITIP semantic-equivalence qualification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/518).
**Composes with:** [#366 ITIP federation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/366), [#486/#491 projector](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/491), [#492/#496 source descriptors](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [#500/#517 byte verification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/517), [#501/#513 decision reliance](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/513), and [#338 IBAP run manifest](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/338).

## Why this receipt exists

Byte-preserving, schema-valid and deterministic transformation does not prove engineering-semantic preservation. A converter can change millimetres to metres incorrectly, alter coordinate reference systems or axis order, drop tolerance/uncertainty, erase a native relationship, or replace an unknown enumeration with a convenient default while still returning success.

The receipt therefore records the source and target artifact identities, exact payload/schema digests, transformation implementation/build/configuration, source and target semantic inventories, explicit per-item mapping dispositions, typed conversion metadata, and any external equivalence-oracle reference.

It is a **claim-accounting interface**, not a converter-success certificate.

## The five separate claims

| Layer | What it establishes | What it does not establish |
|---|---|---|
| Byte preservation | Exact byte digest of a specified artifact | Correct interpretation |
| Schema conformance | Payload conforms to a declared schema version | Semantic equivalence |
| Structural/model equivalence | Declared objects and relationships correspond | Decision-relevant engineering equivalence |
| Semantic equivalence | Required values, units, frames, uncertainty and relationships remain equivalent under an independent oracle | Domain correctness or professional acceptance |
| Domain/professional acceptance | A qualified discipline-specific review accepts the result | Automatic action authority |

A pass at one layer never silently promotes the next.

## Receipt contents

The Rust model binds:
- source and target native artifact IDs, payload digests, media types, schema IDs/versions and schema digests;
- transformation implementation ID/version/build digest, mapping profile ID/digest, configuration digest and parameters digest;
- a source semantic inventory, including which items are required for the declared purposes;
- a target semantic inventory, with source-mapped, derived or generated origin;
- exactly one mapping disposition per source semantic item;
- transformation rule identity/version and rationale;
- optional independent oracle identity/build/corpus/result digests and evaluation time;
- explicit action authority ('none') and limitations.

Mappings distinguish 'preserved', 'converted', 'opaque', 'dropped', 'unsupported', 'split' and 'merged'. Opaque is not equivalent: it says that content is retained without a validated interpretation. Dropped and unsupported items require an explicit reason and cannot have invented target IDs. Split and merge mappings require a group ID and reciprocal target/source links.

## Typed conversion metadata

- **Units:** source/target units and dimensions plus exact rational scale and offset. Floating-point “close enough” is not a substitute for a reproducible conversion rule. A dimensional mismatch is not equivalent.
- **Coordinate frames:** source/target CRS IDs, frame IDs, axis order, handedness, vertical datum, coordinate operation ID and exact operation-parameter digest.
- **Uncertainty:** source/target uncertainty models, behavior, propagation method, assumptions digest, optional covariance digest and target-bound digest. Reduced or unknown uncertainty is a declared loss; propagation without its method and assumptions is indeterminate.

The v0.1 validator does not implement a general unit ontology, execute CRS transformations, propagate statistical models, compare geometry or validate solver mathematics. It checks that the receipt contains consistent, typed metadata and does not let unsupported claims pass silently.

## Validator outcomes

- 'receipt_consistent_for_independent_review': inventory and mapping structure reconcile and no known loss or incompleteness is declared. This is **not** a semantic-equivalence pass.
- 'declared_semantic_loss': one or more explicit semantic losses/opaque items exist. Decision-required loss is separately counted.
- 'not_equivalent': a required contradiction or a producer-declared/oracle-declared failure makes the equivalence claim false.
- 'indeterminate': inventory, conversion evidence or oracle evidence is missing/unverified.
- 'invalid_receipt': malformed digests, duplicate identities/mappings, unknown source/target references, invalid mapping cardinality or authority escalation.

Every validation result fixes:
- 'oracle_evidence_authenticated = not_performed';
- 'semantic_equivalence = not_evaluated_by_receipt_validator';
- 'action_authority = none';
- 'claim_ceiling = mapping_integrity_only'.

An embedded oracle 'pass' remains a producer-supplied reference until a separately built and trusted oracle verifier authenticates the oracle build, corpus and result. A full-equivalence claim without an oracle reference is rejected. Even with an oracle pass reference, this module returns indeterminate for no-loss receipts; a separate verifier must execute/replay the oracle.

## P0 qualification

- 100% source inventory closure: every source semantic item is mapped exactly once or explicitly remains unmapped/indeterminate.
- 0 decision-required opaque, dropped, unsupported or unknown-uncertainty items silently promoted as equivalent.
- 0 unknown target references or non-reciprocal source/target relationships accepted.
- 0 unit dimensional mismatches accepted.
- 0 invalid rational scale/offset denominators accepted.
- 0 coordinate transformations without CRS, frames, axis order, handedness, vertical datum and parameter digest.
- 0 uncertainty propagation claims without method and assumptions digest.
- 0 producer self-declared equivalence accepted as an authenticated result.
- 0 authority escalation.
- Deterministic receipt representation digest and independent oracle replay for every supported fixture.

## P1 benchmark programme

Use frozen, independently authored known-answer corpora:
- unit scale/offset/compound units and dimensional mismatch;
- geographic/projected CRS, vertical datums, local/global frames, axis order and handedness;
- uncertainty intervals, covariance, tolerances and propagation;
- IFC/IDS object/property/relationship semantics;
- QIF feature/characteristic/measurement and uncertainty semantics;
- OPC UA/UAFX node identity, engineering units, status, source sequence and time;
- AAS semantic IDs, qualifiers and submodels;
- split/merge mappings, opaque extensions, enumerations, null-vs-zero, missing-vs-false.

Add at least 1,000 deterministic mutations per semantic family and at least two independently implemented adapters/checkers per format family. Report semantic precision/recall, mapping closure, false equivalence, false refusal, loss rates, unit/CRS conversion error, uncertainty-propagation error, p50/p95/p99/worst-case latency, peak RSS, allocations, input/output bytes and energy across 10, 100, 1,000, 10,000 and 100,000 source/target items.

Do not combine these into one compensating score. A low latency cannot offset one false-equivalence result.

## Competitive position

Build upon, rather than duplicate, existing standards and tools:
- ISO 23247-6:2026 for integrated/unified/federated twin composition;
- IFC 4.3.2.0 and buildingSMART IDS for built-environment exchange and information requirements;
- QIF / ISO 23952 for quality information;
- OPC UA/UAFX and IEC 63278 Asset Administration Shell for industrial information;
- NIST IR 8536 for cross-organization traceability.

Compare on matched exports, versions, scope and hardware. Report vendor-private or inaccessible source fields as unassessed, never absent. The differentiator is independently replayable semantic preservation and explicit loss accounting across heterogeneous systems—not a larger supported-format list.

## Nonclaims

This receipt does not authenticate source-owner identity, retrieve or re-hash source bytes, authenticate rights/signatures/trust roots, prove full source inventory coverage, execute the transformation, establish professional engineering approval, verify all geometry/solver/structural mathematics, or authorize construction, operation or actuation. Those remain separate evidence obligations in the Engineering Assurance Fabric.
