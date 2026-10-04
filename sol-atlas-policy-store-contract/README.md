# Sol Atlas policy-store contract

This package is deliberately outside sol-atlas-core.

It defines the narrow persistence boundary for one-time RecoveryPolicyConsumptionSnapshotV1 state:

1. load the current state;
2. validate the transition against the exact decision, execution, and successor;
3. compare the expected snapshot digest with the stored version;
4. ask the backing store to atomically compare-and-set the exact successor;
5. report commit, conflict, replay, missing state, invalid transition, or store failure distinctly.

The package does not select a database, transport, locking implementation, authority-authentication scheme, OAuth/DPoP profile, or proof-of-possession mechanism.

The included MemoryStore test model is reference evidence for the concurrency contract only; it is not presented as production persistence.
