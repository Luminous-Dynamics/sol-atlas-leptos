# Sol Atlas policy-store contract

This package is deliberately outside sol-atlas-core.

It defines the narrow persistence boundary for one-time RecoveryPolicyConsumptionSnapshotV1 state:

1. load the current state;
2. validate the transition against the exact decision, execution, and successor;
3. compare the expected snapshot digest with the stored version;
4. ask the backing store to atomically compare-and-set the exact successor;
5. report commit, conflict, replay, missing state, invalid transition, malformed semantic input/state, or store failure distinctly.

The package does not select a database, transport, locking implementation, authority-authentication scheme, OAuth/DPoP profile, or proof-of-possession mechanism.

The included MemoryStore test model is reference evidence for the concurrency contract only; it is not presented as production persistence.


## Commit acknowledgement semantics

The CAS adapter distinguishes three outcomes: known commit, known not-current conflict, and indeterminate commit. An indeterminate result is possible when the backing store may have committed but the acknowledgement was lost or otherwise became unknowable. The contract therefore forbids reporting success from that outcome and forbids blindly retrying a single-use authorization; reconciliation is a separate adapter concern.

The reference contract does not claim that a client can observe an exactly-once commit outcome across arbitrary failure modes. It claims the stronger and safer boundary: at most one contender can be accepted by a conforming atomic CAS implementation, and an uncertain acknowledgement can never be promoted to a successful-consumption report by this package.

## Reconciliation after an indeterminate acknowledgement

When the CAS adapter returns `Indeterminate`, callers must not retry the single-use action blindly. `reconcile_indeterminate_consumption` performs a read-only check using the exact expected pre-state digest and exact successor digest.

It can observe one of three useful states: the exact successor is present, the exact expected pre-state is present, or a different valid state is present. The helper deliberately reports these as observations rather than claiming a generic distributed-systems guarantee about what will happen after the read. An adapter may layer stronger database-specific guarantees on top.
