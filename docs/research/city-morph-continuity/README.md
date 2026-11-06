# CityMorph continuity corpus v0.1

26 synthetic scenarios test time-dependent continuity and recovery using owner-native access-state receipts. The evaluator does not recompute routing or service capacities; it consumes outputs from CityMorph #555 and CityKit #3768.

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
