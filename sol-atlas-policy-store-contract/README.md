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

## Execution-side-effect boundary

Observing `ObservedCommitted` proves that the exact consumed successor is present at reconciliation time; it does not prove that the observing caller was the CAS winner. Therefore the policy-store contract establishes at-most-one persisted consumption transition, not exactly-once execution side effects. A concrete execution adapter must either make the execution effect idempotent under the stable execution identity or introduce its own atomic execution-claim/ownership primitive before performing irreversible work.

## Execution claim contract (#28)

The package also defines a storage-neutral execution claim boundary. A claim binds a stable `execution_id` to the exact `execution_input_snapshot` fingerprint and an opaque `attempt_id`. A conforming `claim_if_absent` implementation atomically permits one attempt to acquire that execution identity, treats an exact same-attempt replay as an idempotent duplicate, rejects reuse of the identity for a different fingerprint, and distinguishes competing attempts.

An `Indeterminate` claim acknowledgement is reconciled by a read-only lookup. Seeing the same attempt's claim proves durable claim ownership at observation time, not completion of an external side effect. The external effect therefore still needs either idempotent semantics keyed by the same execution identity/fingerprint or its own transactional ownership boundary.

The current IETF Idempotency-Key Internet-Draft follows the same fundamental rule: an idempotency key must not be reused with a different request payload, and retries/concurrent requests require explicit handling. The draft is currently expired rather than an RFC, so it is treated here as design guidance rather than a normative standard.
