# City State Synchronization v0.1

Offline Rust reference oracle for the synthetic temporal state and feedback semantics in CityMaker issue #625.

## Why this exists

The common scope binder (#623) can establish that owner artifacts refer to a common declared scope. The owner-adapter registry (#606) binds exact owner schemas and revisions. Neither answers what state was valid at a past decision point, whether an event arrived late, whether a correction/revocation changed a previous assertion, or whether a proposed intervention was actually executed, inspected and observed.

This evaluator defines a deterministic, read-only projection over owner event envelopes. It is not a new event ledger, sensor platform, digital-twin engine, identity authority, scheduling system, or causal-inference engine. Each owner remains authoritative for its own source records and correction/revocation semantics.

## Temporal model

A projection is evaluated at three explicit cutoffs:

- **Valid time**: the physical-world interval during which the assertion is stated to apply.
- **Record time**: when the owner recorded or published the assertion.
- **Ingestion time**: when the synchronizer received the owner record.

The v0.1 synthetic profile normalizes comparable instants to UTC Unix seconds while retaining the original source clock identity, source IANA timezone (for example, `Africa/Johannesburg`) and uncertainty. The source timezone is metadata about the originating clock, not a demand that the source itself use UTC. Unresolved local wall-clock timestamps without a qualified offset/timezone conversion are not silently coerced.

The validity interval is half-open: `[valid_from, valid_until)`. A state is not reconstructed by sorting timestamps and taking the last one. Within one owner-native subject, the owner sequence is the ordering authority. Records with later publication times but lower owner sequence cannot overwrite a newer owner assertion.

A record is visible to an as-of replay only when both its record time and ingestion time are at or before the requested cutoffs. This makes late arrivals and historical replay explicit.

## Owner-declared freshness

There is no universal city-data TTL. Each owner/contract policy declares:
- expected refresh cadence;
- maximum observation age;
- maximum source record delay;
- maximum consumer ingestion lag;
- maximum clock uncertainty.

Missing or incomplete freshness policies produce `Unknown`, not `Current`. A partial source outage preserves the last-known state as `Stale`. The synchronizer never invents an update cadence or favourable default.

## State and relation semantics

The evaluator preserves these distinct dispositions:

- `Current`: admitted evidence is in its valid-time interval and within the exact owner freshness policy.
- `Stale`: validity/freshness expired or the owner source is in a declared partial outage.
- `Superseded`: represented by a new owner event and its explicit predecessor link; the current projection selects the successor while preserving historical event IDs.
- `Corrected`: represented by a correction event targeting the exact previous event; the new assertion may become current while the prior event remains in history.
- `Revoked`: an owner revocation targets the selected event; consumer replay cannot restore the revoked assertion.
- `Disputed`: an owner dispute remains visible and blocks favorable composition.
- `Delayed`: source-record or consumer-ingestion delay exceeds its declared policy.
- `Unknown`: source, time basis, freshness policy, validation or evidence admission is incomplete.
- `NotComparable`: scope digest differs from the requested subject scope.
- `Unsupported`: duplicate/conflicting IDs, owner-sequence conflicts, unsupported time basis, invalid relation target or malformed identity.

Owner raw state remains separate from synchronizer state. A `Current` binding does not imply that the owner state is favorable; a raw `Blocked`/ `Rejected` state remains owner-owned.

## Identity and feedback

Identity continuity is never inferred from a name or similar geometry. An explicit owner-declared successor link is preserved as a declared mapping, but it does not automatically merge the two subject records.

Intervention stages are linked in order:

`proposal -> approval -> execution -> inspection/commissioning -> post-intervention outcome observation`

An approval without execution is not completed work. Execution without inspection is unverified. A passed inspection without post-intervention observation is not evidence of the intended outcome. Even a complete chain produces `OutcomeObserved`, not a causal effect estimate. Causal evaluation and physical measurement remain with their qualified owners.

## Standards alignment (mapping, not replacement)

The profile reuses established temporal/provenance concepts rather than inventing new data semantics:

- [OGC SensorThings API 1.1](https://docs.ogc.org/is/18-088/18-088.html) distinguishes `phenomenonTime`, `resultTime` and `validTime` for observations. The owner adapter should preserve the native meaning of those fields rather than equating device observation time with publication time.
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) provides provenance relationships such as generation, derivation, revision and invalidation. The synchronizer retains owner-specific correction, supersession and revocation references without replacing PROV or owner-native ledgers.
- [SQL:2011 temporal concepts](https://wiki.postgresql.org/wiki/SQL2011Temporal) distinguish application/valid time from system/transaction time and commonly use half-open intervals. The third ingestion cutoff here captures the separate time at which the projection obtained the owner record.

These are interoperability mappings. v0.1 is not a complete SensorThings server, PROV implementation, or temporal database.

## Corpus and benchmark

`CITY_STATE_SYNC_CORPUS_V0_1.json` contains 36 synthetic known-answer cases:
- late arrival and owner-sequence ordering;
- validity expiry and owner-declared freshness;
- correction, supersession, revocation and dispute;
- duplicate/conflicting event and receipt identities;
- missing/unsupported clock semantics;
- refresh-policy absence, stale feeds, source outage and pending validation;
- scope mismatch and explicit identity succession;
- intervention approval, execution, inspection and outcome chain gaps;
- source record delay, ingestion delay, clock uncertainty and as-of replay.

The JSON Schema is `CITY_STATE_SYNC_CORPUS_V0_1.schema.json`. The Rust oracle computes results from `input` only and compares them to exact expected objects. The expected known-answer records are never consulted by the evaluator.

### Reproduction

From the repository root:

```sh
cargo fmt --manifest-path tools/city-state-sync-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/city-state-sync-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/city-state-sync-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/city-state-sync-bench/Cargo.toml -- \
  docs/research/city-state-sync/CITY_STATE_SYNC_CORPUS_V0_1.json \
  --source-commit <40-hex-commit>
```

## Metrics and targets

Report the state counts and reasons by owner and entity class, plus valid-time age, source record delay, consumer ingestion lag, correction/supersession frequency, duplicate/conflict frequency, identity-link count, unresolved identity count, and feedback-chain completion stage.

P0 targets:
- 100% agreement with the 36-case known-answer corpus;
- deterministic identical as-of replay for identical pinned owner snapshots;
- zero latest-timestamp overwrite where owner sequence / valid time says otherwise;
- zero correction/revocation erasure of historical records;
- zero `Current` state without a complete owner freshness policy;
- zero completed-intervention claim without the required approval, execution, independent inspection and outcome-observation references;
- no inferred identity merge and no causal claim.

Do not combine these into a composite “digital twin fidelity” percentage. A single stale critical asset or missing intervention receipt must remain visible.

## Claim ceiling

A pass demonstrates only the declared temporal state-reduction and feedback-chain semantics on this frozen synthetic corpus. It does not authenticate live owner payload bytes, sensor clocks, physical assets, building condition, municipal data, intervention effectiveness, causal impact, district resilience or professional approval.

Next integration steps remain #606 (owner-native adapters), #623 (common scope), #637 (exact payload-byte verification), Mycelix #5149 (custody/correction/revocation), and the outcome owners identified by #633.
