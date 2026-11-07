# City Morph Safe Oracle v0.1

Standalone, dependency-free Rust reference evaluator for the synthetic evidence gates defined by [CITY-MORPH-SAFE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/591).

## Claim ceiling

The evaluator returns exactly one of:

- `blocked` — a hard gate failed, prohibited/restricted data use was attempted, an invalid denominator was supplied, or a duplicate identity was detected.
- `unknown` — required evidence is missing, stale, unsupported, conflicting, scope-mismatched, or bound to the wrong candidate/profile/corpus revision.
- `eligible_for_professional_review` — the synthetic review packet satisfies the declared prerequisites and can be handed to qualified reviewers.

**This does not mean safe.** The evaluator cannot issue crime-reduction, fire-safety, structural, code-compliance, building approval, or professional certification findings. It does not model crime, fire, evacuation, structures, lighting, accessibility geometry, or physical security. Those require domain-specific data, validated tools, professionals and field evidence.

## Synthetic scope

The v0.1 profile requires six distinct evidence classes:

1. Recorded crime.
2. Victimisation survey.
3. Route audit.
4. Site context.
5. Adjacent-area impact / spillover.
6. Lifecycle maintenance baseline.

It also enforces seven non-compensatory gates: fire egress, responder access, accessible journey, lawful public access, privacy/consent, emergency manual fallback, and no unmitigated spillover.

This is a deliberately conservative profile for comparing neighbourhood interventions. A narrower dwelling-only profile must be separately versioned with an explicit requirement matrix; do not silently omit evidence.

## Run

From this directory:

```sh
cargo fmt --check
cargo test --offline
cargo run --offline
```

There are no third-party Rust dependencies and no network access is needed. The nested `[workspace]` declaration isolates this crate from the parent Leptos package.

The CLI exits non-zero if any fixture differs from its expected disposition. It prints a deterministic FNV-1a 64-bit fixture fingerprint to detect accidental corpus drift. **FNV-1a is non-cryptographic:** it is not an integrity signature, provenance proof or substitute for a cryptographic digest in a signed evidence receipt.

## Corpus and tests

The frozen suite has 30 explicitly synthetic cases. Unit tests cover exact expected dispositions, ID uniqueness, hard-gate dominance, missing/stale/restricted evidence, invalid denominators, scope/revision mismatch, forbidden automatic certification requests and deterministic output.

Expected outputs live next to the fixtures, while `evaluate()` does not read the expected field. These tests are a first internal E1 harness, not an independently audited benchmark. Independent review of fixture labels and evaluator logic is still required.

## What this is not

- Not a crime prediction or location risk score.
- Not a CPTED effectiveness estimator.
- Not a security-system design or autonomous enforcement tool.
- Not a structural, geotechnical, fire, evacuation, thermal, acoustic or energy solver.
- Not an IFC/IDS validator.
- Not a building-plan approval or statutory compliance engine.
- Not evidence that any real South African property or neighbourhood is safe.
