# CityMaker Temporal Query Adapter v0.1

**Status:** synthetic-first bitemporal query/report adapter; exact-head qualification pending  
**Parent work:** [CITY-TEMPORAL-REPLAY-001 #653](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/653)  
**Router:** [CITY-IMPACT-ROUTER-001 #649 / PR #651](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/651)  
**Branch:** `research/city-temporal-query-adapter-v0.1`

## Purpose

The CityMaker router binds event valid-time/system-time envelopes and owner-report evaluation time. A single `evaluation_at` cannot answer four different questions:

| Dimension | Field | Question |
|---|---|---|
| World time | `world_time_as_of` (W) | When did the asserted fact apply in the physical/design world? |
| Knowledge time | `knowledge_time_as_of` (K) | Which owner-recorded facts had arrived by the chosen cutoff? |
| Trust time | `trust_time_as_of` (T) | Which evidence had a trust/registration timestamp by the chosen cutoff? |
| Evaluation time | `evaluated_at` (E) | When was this query/report actually evaluated? |

The query also declares one of two perspectives:

- `historically_known_then`: evaluate W using the historical K/T cutoffs. A correction recorded or trusted after those cutoffs is not silently inserted into the old knowledge state.
- `currently_reconstructed_history`: evaluate the same earlier W using later K/T cutoffs. Later-arriving corrections may affect today's reconstruction of the earlier world state, and are marked as late arrivals.

The adapter never chooses W/K/T for a consumer. The decision/reliance owner supplies them, and every owner report must bind the exact tuple and perspective. A historical projection and a current reconstruction are separate, immutable outputs.

## Temporal semantics

- Valid intervals are half-open: `[effective_at, effective_until)`. Null `effective_until` means no declared end; a missing field is incomplete.
- Effective-time confidence must be `verified` for temporal eligibility. `authority_asserted`, `unknown` and `disputed` remain insufficient.
- A fact recorded after K is `recorded_after_cutoff`; a trust timestamp after T is `trusted_after_cutoff`. These are explicit exclusions, not proof that the event never existed.
- An explicit null pair (`trusted_at=null`, `trusted_timestamp_receipt_sha256=null`) is untrusted. A missing field is incomplete. A malformed or contradictory pair is invalid.
- `late_arrival=true` when the valid-time start precedes the system recorded time. This is an informational temporal property, not a legal retroactivity decision.
- The report's event identity, subject/revision, valid/system time, query tuple, report evaluation time and content-address reference must bind exactly.

Timestamps are restricted to canonical UTC RFC 3339 whole-second form under `UTC_RFC3339_SECONDS`. Fixed-width UTC timestamps allow deterministic lexicographic ordering after syntax/date validation.

## Result dispositions

- `eligible_for_declared_temporal_projection`: all declared temporal and report-binding conditions match.
- `excluded_from_temporal_projection`: event is not world-valid at W, was recorded after K, or was trusted after T.
- `insufficient_evidence`: time confidence is not verified, trust is absent, or required query/report fields are missing.
- `invalid_input`: malformed timestamps/intervals, contradictory ordering, report/query mismatch, event/subject mismatch, invalid report reference or action-authority escalation.

The output preserves separate world, knowledge, trust and report-binding states. It is not a scalar confidence score.

## Corpus

`CITY_TEMPORAL_QUERY_CORPUS_V0_1.json` contains 28 synthetic known-answer cases:

- 2 eligible temporal projections;
- 7 excluded projections;
- 5 insufficient-evidence cases;
- 16 invalid-input cases.

The controls include historical vs current reconstruction, late evidence, trust arriving after cutoff, future/expired valid intervals, disputed and authority-asserted time, null vs missing trust receipt, reversed trust/recorded ordering, invalid intervals, mismatched W/K/T/E/perspective, subject revision mismatch, noncanonical timestamps, owner-report event/time mismatches, future knowledge/trust cutoffs, and events recorded or trusted after query evaluation.

Expected outcomes are stored in the corpus and compared against the Rust evaluator. This is a synthetic oracle, not a field trial.

## Run

From the repository root:

    cargo fmt --manifest-path tools/city-temporal-query-adapter/Cargo.toml --all -- --check
    cargo test --locked --manifest-path tools/city-temporal-query-adapter/Cargo.toml
    cargo clippy --locked --manifest-path tools/city-temporal-query-adapter/Cargo.toml --all-targets -- -D warnings
    cargo run --locked --quiet --manifest-path tools/city-temporal-query-adapter/Cargo.toml -- \
      docs/research/city-temporal-replay/CITY_TEMPORAL_QUERY_CORPUS_V0_1.json \
      --source-commit <40-lowercase-hex-sha>

The CLI receipt binds the exact corpus bytes and source commit.

## Benchmark targets

P0:
- zero historical false inclusion from evidence recorded after K;
- zero trust false inclusion from trust timestamps after T;
- zero report reuse across W/K/T/E or perspective mismatch;
- zero silently overwritten historical results after a late correction;
- 100% exact agreement with independently authored frozen expected projections;
- zero timestamp/order contradictions treated as eligible.

P1:
- 1,000, 10,000 and 100,000 event histories with late-arrival ratios 0%, 1%, 10% and 50%;
- replay and current-reconstruction p50/p95/p99 latency;
- event-to-current-reconstruction correction latency;
- receipt growth for append-only supersession;
- operator review burden for disputed effective time, missing trust and retroactive corrections;
- report-binding and invalidation precision by evidence owner and event class.

Do not reduce Unknown/Insufficient rates by weakening cutoffs. A missing timestamp or unverified effective time is not a performance optimization opportunity.

## Claim boundary

This crate validates temporal query syntax, declared event-time eligibility and owner-report binding. It does not authenticate source timestamps, validate trust receipts, determine legal/professional retroactivity, prove source inventory/dependency completeness, establish engineering correctness, or grant approval/action authority. A passing synthetic receipt proves only the stated temporal adapter semantics over this corpus.
