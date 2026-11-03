# Built-environment coverage runtime verifier v0.1

**Status:** Rust implementation and exact-head workflow added; no compiler, Clippy, or test result is claimed until the workflow reports against this exact commit.
**Parent contract:** [BE-COVERAGE-001 / #488](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488) and [manifest PR #489](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/489).
**Implementation:** `tools/built-environment-coverage-auditor/`.

## What the runtime verifier adds

JSON Schema validates the shape of a coverage manifest. It cannot by itself reconcile cross-object relationships. The Rust auditor independently checks:

- uniqueness of source, family, edge, source-relationship, and absence-assertion identities;
- source snapshot, schema, extractor, configuration, query, mapping-rule, and proof digest syntax;
- source access, rights, and permitted-use state;
- source capture/retrieval/evaluation chronology and complete pagination;
- required/excluded family symmetry, exactly-one family rows, source membership, and relationship counts;
- edge-to-source snapshot and extractor-build bindings, plus edge-to-family mapping-rule bindings;
- exact/deterministic versus ambiguous, unsupported, or manually asserted relationship mappings;
- bounded absence assertions, query/source snapshot binding, complete enumeration and pagination, and validity at evaluation time;
- declared incomplete/invalid states, which cannot be promoted.

Duplicate JSON object member names are rejected before semantic evaluation. The report hashes the exact input bytes; it does not invent a JSON canonicalization rule.

## Two distinct outputs

The report separates:

1. **Internal declared-scope consistency:** `complete_for_declared_scope`, `incomplete_or_indeterminate`, or `invalid`.
2. **Decision reliance:** always blocked in v0.1.

A complete internal result means only that the manifest's declared source and relationship assertions reconciled under this verifier. It does not authenticate the source owner, completeness proof, reviewer, or reviewer credentials. It also does not establish that the profile includes every dependency required by the actual project or decision.

Accordingly, `no_impact_eligible` is always `false` and the report carries `blocked_unverified_external_attestations_and_profile_adequacy`. The next trust tranche must use #347's authenticated trust-root, signature, scope, freshness, sequence, and revocation design to authenticate source and reviewer proof artifacts. Profile adequacy still requires an independently scoped domain review; a valid signature cannot make an incomplete scope complete.

The synthetic fixture's previous `eligible_to_support_no_impact_within_declared_scope: true` expectation was corrected to `false`. Its reviewer and proof digests are placeholders, not authenticated attestations.

## Failure semantics

- Structurally contradictory or rebound evidence is `invalid`.
- Missing, unavailable, unsupported, stale, incomplete, ambiguous, or unreviewed mandatory coverage is `incomplete_or_indeterminate`.
- A zero-result query is not absence unless the exact source snapshot and query are bounded, enumeration and pagination are complete, a proof digest is present, and its validity interval includes evaluation time.
- Neither an empty result nor a graph digest independently proves source completeness.
- No failure path authorizes construction, release, actuation, or a professional/legal decision.

## Qualification protocol

The dedicated workflow keeps the existing Draft 2020-12 schema/fixture lane and adds:

- `cargo fmt --check`;
- locked Clippy with warnings denied;
- locked Rust unit tests;
- positive replay of the frozen synthetic fixture, explicitly asserting the external reliance gate remains blocked;
- negative runtime controls deleting a mandatory relationship while reconciling the observed count, and evaluating an expired absence assertion.

The authored test corpus also covers duplicate JSON member rejection, vector-order invariance, unavailable sources, unknown rights, missing required families, source-snapshot rebinding, and declared-incomplete non-promotion.

## Explicitly not checked

The auditor does not authenticate signatures or trust roots, fetch original source bytes, prove source-owner identity, verify reviewer credentials, adjudicate scope adequacy, compare against a gold dependency inventory, execute the impact traversal, verify engineering solvers, or establish physical or legal sufficiency. These are separate assurance obligations, not hidden assumptions.

## Next composition

- #489 owns the coverage manifest contract.
- This Rust tool owns cross-object consistency checks for that manifest.
- #484 owns independent traversal correctness against a full-scan impact oracle.
- #486 owns the read-only identity-preserving projection into IEAC.
- #397 owns temporal invalidation and current-reliance classification.
- #347 owns trust-root, signature/canonicalization, anti-replay, and revocation foundations.

Do not collapse these into a single boolean or duplicate graph. A consumer may combine their independently versioned receipts only after checking their exact input digests, scope, freshness, and verifier identities.
