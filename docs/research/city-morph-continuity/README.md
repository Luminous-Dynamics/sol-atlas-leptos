# CityMorph continuity corpus v0.1

30 synthetic scenarios test time-dependent continuity and recovery using owner-native access-state receipts. The evaluator does not recompute routing or service capacities; it consumes outputs from CityMorph #555 and CityKit #3768.

## Contract

- Time is integer ticks and every phase uses a half-open interval [start_tick,end_tick).
- Phases must cover one contiguous horizon from tick 0 with no overlaps or gaps.
- Each declared requirement must have exactly one access-state receipt in every phase.
- Only current, subject-bound CityMorph receipts from the expected contract/revision count as known states.
- Adequate, Constrained, Inaccessible and Unknown remain distinct.
- Constrained and Inaccessible contribute to known degradation.
- Unknown/stale/wrong-subject evidence is counted separately; it never contributes to adequate duration.
- Recovery is confirmed only after the configured number of contiguous adequate ticks following a known degradation.
- Event IDs label the scenario only. No probabilities, independence assumptions or aggregate resilience score are invented.

## Outputs

Per requirement, report total horizon ticks, adequate/constrained/inaccessible/unknown duration, longest contiguous degradation, first degradation tick and recovery state/start/confirmation/time. Outcomes remain disaggregated by requirement and population group.

## Claim ceiling

This corpus validates timeline and receipt semantics only. It does not certify actual service resilience, hazard probability, engineering adequacy, municipal performance or community resilience.


## Exact owner-revision and scope binding

Every fixture receipt now carries the evaluation-scope digest and source revision. The input declares the expected owner revision, and the oracle requires exact equality for both revision and scope. Two additional hostile controls verify that a non-empty but incorrect source revision, and a valid-looking receipt from another scope, are counted as Unknown rather than Adequate. These are identity/contract checks; the synthetic strings are not cryptographic authentication or proof of a real upstream producer.


Unknown evidence before the first known degradation does not invalidate later recovery assessment. The evaluator counts unknown ticks separately before and after the first known degradation; only post-degradation unknown periods can leave recovery unresolved. Case CR-027 is the negative control.


## Global receipt identity and required revision

Each source receipt ID must be unique across the entire scenario timeline, not merely within one phase. The evaluation request must also declare an expected owner source revision. Case CR-028 reuses one receipt ID across two phases and must be Unsupported; CR-029 omits the expected revision and must be Unsupported. This prevents one owner receipt from being replayed as multiple observations and prevents a missing revision contract from being treated as a valid comparison.


## Recovery timing bounds (corpus v0.1.1)

The continuity result distinguishes the first observed adequate phase from the exact time service became adequate. When the last known degraded interval ends at tick L, an evidence gap follows, and the first sustained adequate observation begins at tick U, the recovery start is reported as `(L, U]`. The corresponding recovery duration from the first known degradation is reported with the same open/closed bounds. The point field `recovery_time_ticks` is null unless the interval is exact.

- CR-002 is the exact-boundary control: no evidence gap, so recovery time is exactly 2 ticks.
- CR-020 has unknown evidence after an outage but no confirmed return; recovery remains Unknown and no upper bound is claimed.
- CR-030 confirms adequate service after an evidence gap; recovery starts in `(5, 7]`, duration in `(2, 4]`, confirmed by tick 9. The result does not claim a false point estimate.


## CR-030 exact bounded recovery

CR-030 adds an evidence gap after a known degraded interval and before the first confirmed adequate observation. The oracle reports the recovery start in the half-open interval `(5, 7]` and duration in `(2, 4]` ticks. It leaves `recovery_time_ticks` null because the actual recovery instant is not observed. CR-002 remains the exact-timing negative control, and CR-020 remains unresolved because no adequate recovery is confirmed.

