# City Belonging Task Benchmark v0.1

**Status:** proposed synthetic task benchmark; not a usability study, user-outcome evaluation, or production matcher.  
**Companion data:** [JSONL cases](city-belonging-task-benchmark-v0.1.jsonl), [integrity manifest](city-belonging-task-benchmark-v0.1.manifest.json), and [strict case schema](city-belonging-task-benchmark-v0.1.schema.json).  
**Parent designs:** [City Belonging Navigator](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/123), [admission contract](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/125).

## Paradigm shift: constraints before ranking

The matcher should first determine which candidates satisfy **explicit hard requirements**, then explain the evidence and trade-offs. It should not blend must-haves, soft preferences, unknowns, and user identity into one opaque compatibility score.

A candidate cannot satisfy a requirement through missing data. A soft preference (for example, “smaller group preferred”) may guide order or presentation but must not silently remove otherwise eligible choices. If no candidate satisfies all hard constraints, return a truthful no-verified-fit result and ask before relaxing a requirement.

## Contents

The synthetic benchmark contains 14 scenarios covering cost, accessibility, language, expired events, service currentness, contactability, virtual/service-area discovery, explicitly selected worldview framing, no-fit handling, soft preferences, conflicting claims, live capacity, verified travel time, and privacy/opt-out.

- JSONL SHA-256: `90795bba044c4137e1a30c543b325a90d91be3be5e6692a07c82ff57a1982749`
- Case schema SHA-256: `0a72a5eaceddc3fb8f8fe92da726cad3a1ad6a7931c72cd796e38231037d12a7`
- The schema is pinned; a JSON Schema validator has not yet been executed against these 14 records.
- All candidates use synthetic IDs; no live listings, real contact details, coordinates, or participant data are included.
- Per-case expected eligible sets, exclusion reasons, forbidden behaviors and rationale are part of the frozen fixture.
- A valid task fixture is not a real-world service directory and does not establish that any listed synthetic service exists or is safe.

## Proposed qualification gates

| Metric | Target on frozen regression fixture |
|---|---:|
| Violation of a user-stated hard constraint | 0 |
| Unknown facts promoted to verified positives | 0 |
| Silent relaxation of a hard constraint | 0 |
| Cases with explicit eligible set and negative rationale | 100% |
| User opt-out from sensitive personalization honored | 100% |
| Soft preference incorrectly treated as hard exclusion | 0 |
| Currentness/capacity/contact success conflated | 0 |
| Case-level mismatch report completeness | 100% |
| One scalar match score used to hide constraint failures | Prohibited |

These are synthetic release gates, not validated estimates of live matcher precision or user satisfaction.

## Next qualification stages

1. Implement a separate, deterministic constraint evaluator that consumes this task schema and preserves the difference between `satisfies`, `does not satisfy`, and `unknown`.
2. Produce explicit per-constraint evaluation receipts. Do not implement a global match score as an authority.
3. Add mutation tests: unknown-to-positive, soft-to-hard promotion, hard-constraint relaxation, stale-to-current, conflict suppression, and opt-out leakage.
4. Validate fixture bytes and cases against a pinned task schema; freeze the rubric before evaluating candidate matchers.
5. Compare a baseline keyword search with the constraint-aware selector on the same representative task bank. Measure hard-constraint satisfaction, user comprehension, no-fit honesty and actual chosen next actions separately.
6. In a later compensated, consented usability study, report task-level success and failure, confidence intervals, subgroup/accessibility coverage and harm incidence before making any claim about social connection outcomes.

## Evaluation interpretation

- **Eligible is not “best.”** If several candidates satisfy all hard constraints, retain options and describe evidence/soft-preference trade-offs.
- **Unknown is not failure or success.** Unknown may be shown as an unresolved option if the user's constraints allow it, but it cannot be described as meeting a hard requirement that depends on that fact.
- **No-fit is useful output.** A truthful explanation and consented follow-up is better than inventing an answer or relaxing constraints.
- **No inferred identity.** A user may request a secular or faith-based filter explicitly. The system must not infer religion, loneliness, trauma, or mental-health status.
- **No false precision.** Never invent route duration, street pins, live capacity, or contact success.
- **No causal leap.** Matching correctly does not establish that a person formed a connection or experienced reduced loneliness.

The task fixture is currently a proposed benchmark. It must receive independent review before being treated as qualification evidence, and the Rust evaluator does not yet exist.


## Reference evaluator

The standalone Rust policy slice is [tools/city-belonging-task-matcher](../../../tools/city-belonging-task-matcher/README.md). It emits per-constraint states and separate eligible/preferred sets. A separate JavaScript policy mirror matched the current 14 cases' expected eligible sets, preferred sets, and exclusion-reason coverage. This is independent logic over the fixture, **not** execution of the Rust evaluator, and does not validate the JSON Schema with a schema engine. The Rust crate has 18 authored tests but remains uncompiled/unexecuted.


## Metamorphic tests: 7 invariants

The task matcher now has seven authored metamorphic tests, summarized in the [machine-readable contract](city-belonging-metamorphic-contract-v0.1.json):

1. Candidate ordering cannot change eligibility, preference, or receipt outcomes.
2. Reordering a conjunction of hard constraints cannot change the answer.
3. Changing soft preferences cannot change hard eligibility.
4. Adding a hard constraint cannot add eligible candidates.
5. Removing a hard constraint cannot remove eligible candidates.
6. Adding unrelated popularity/click-count fields cannot change a decision.
7. Satisfying one condition cannot override an unknown or mismatch in another hard constraint.

All seven are authored tests, not executed evidence. Target: 7/7 properties pass, with zero hard-constraint violations, zero unknown-to-positive conversions, and zero preference-only eligibility changes. The matcher has 25 authored tests at this revision; Rust execution is still pending.
