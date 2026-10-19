# Model credibility → service assurance handoff v0.1

## Purpose

This contract connects Symtropy's MINE-MCRED-001 model-credibility gate to Sol Atlas's evidence-aware water/power and protected-service-floor evaluators.

It prevents a dangerous semantic collapse: a model abstention must not become zero capacity, a normal value, or a claim that service is safe. It also prevents a model decision from becoming an operational command.

## Data contract

A handoff carries:
- stable decision, model and physical asset identifiers;
- canonical UTC decision time;
- explicit capacity interval and a unit string, with no implicit unit conversion;
- the upstream disposition;
- exact source commit, calibration-set digest and generated-case-set digest;
- supporting evidence IDs.

The adapter validates the schema version, identifiers, timestamp syntax, nonnegative finite ordered capacity bounds, unit presence, lowercase hexadecimal digest syntax, and unique nonempty evidence IDs.

## Disposition mapping

| Symtropy disposition | Sol Atlas disposition | Usable capacity interval |
|---|---|---|
| use_model | eligible_for_assessment | Present, preserving the declared bounds and unit |
| reject_unsafe_interval | rejected | Absent |
| abstain_missing_observation | indeterminate | Absent |
| abstain_stale_telemetry | indeterminate | Absent |
| abstain_residual_out_of_envelope | indeterminate | Absent |
| abstain_recovery_holdoff | indeterminate | Absent |

A null eligible-capacity field means not available for service assessment, **not zero capacity**. Downstream service-floor code should retain indeterminate weight and must not count it as guaranteed adequate or guaranteed inadequate.

## Verification boundary

This adapter validates shape and propagates disposition; it is not an identity, signature, source-commit existence, evidence-truth, or authority verifier. SHA-256 digests bind bytes, not truth. Before a real cross-system deployment, the consumer must authenticate the producer/artifact channel, resolve referenced evidence under the applicable policy, and independently enforce schema, unit and freshness rules.

The adapter also does not independently prove the upstream model was correctly calibrated or that the interval is physically accurate. That responsibility remains with the upstream model-credibility and execution-feasibility evaluation, and eventually with site-calibrated validation.

## Tests and fixtures

Unit tests cover admitted bounds, all four abstentions, explicit unsafe rejection, malformed schema/time/unit/interval/provenance, missing/duplicate evidence IDs and deterministic serialization. JSON Schema files and a stale-telemetry input/output pair define a language-neutral wire contract. The Rust module test does not, by itself, establish full external JSON Schema validator conformance.

## Cross-project references

- Symtropy [MINE-MCRED-001 / PR #2180](https://github.com/Luminous-Dynamics/symtropy/pull/2180): model acceptance, drift/staleness abstention and recovery holdoff.
- Sol Atlas [joint water–power resilience #147](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/147): service trajectories and protected service floors.
- Symthaea [PIE-002 / #1610](https://github.com/Luminous-Dynamics/symthaea/issues/1610): typed resource accounting; unknown utility inputs must not default to zero.
- Mycelix [decision-to-outcome lineage #2953](https://github.com/Luminous-Dynamics/mycelix/issues/2953): decision, authority, command attempt, acknowledgement, observation and outcome remain distinct.

This is an interface contract and synthetic test slice only. It does not authorize operations or prove field performance.
