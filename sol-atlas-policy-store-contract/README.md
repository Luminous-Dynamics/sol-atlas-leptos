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

## External-effect receipt boundary (#30)

The package now models the durable receipt around an external effect as `InProgress`, `Succeeded`, or `Failed`. Starting an effect is an atomic idempotency operation keyed by the stable execution identity and exact input fingerprint. Only the attempt that owns the `InProgress` receipt may advance it to a terminal receipt.

A repeated request with the same execution identity and fingerprint is therefore classified instead of starting a second effect. A different fingerprint is rejected. An uncertain store acknowledgement is reconciled by reading the receipt; the receipt state itself is never treated as proof that the external action completed.

This deliberately leaves one hard systems boundary explicit: the effect must itself be idempotent or transactionally coupled to the receipt if retries after an uncertain outcome are expected to be safe. AWS guidance recommends unique idempotency tokens and persisted operation state, while Stripe documents replaying the stored first result for a repeated idempotency key. These references guide the contract shape; they are not claims that Sol Atlas interoperates with either API.

## Orchestration recovery matrix (#29)

The policy, claim, and effect stores are separate local transaction boundaries. The contract therefore treats orchestration as a saga-like sequence rather than a distributed transaction. Current recommended ordering is: admit policy context → atomically acquire execution claim → atomically consume authorization → begin effect → perform effect → terminalize receipt. Saga participants must be idempotent when retries are possible. citeturn751713search0turn751713search2

| Observed state | Safe next action | What must not be inferred |
| --- | --- | --- |
| Authorization unconsumed + no execution claim | Acquire claim, then consume authorization | No claim means nobody can race later |
| Authorization unconsumed + same execution claim | Consume authorization if admission remains valid | Claim implies authorization validity |
| Authorization consumed + same execution claim | Reconcile/continue the same execution | Consumption implies external effect occurred |
| Authorization consumed + no claim | Reconcile execution/effect identity before any irreversible work | Missing claim proves that nothing happened |
| Effect `InProgress` + same attempt | Reconcile effect; do not start a second effect | Process ownership implies effect completion |
| Effect `InProgress` + other attempt | Fail closed or use adapter-specific recovery | Stale ownership can be safely stolen |
| Effect `Succeeded` | Return/reuse the recorded result | A replay should re-run the external effect |
| Effect `Failed` | Apply an explicit retry policy, normally with a new execution identity | A failure receipt is permission to repeat blindly |

The matrix intentionally leaves abandoned claims and in-progress effects without automatic expiry. Automatic expiry can transfer ownership while the original external side effect is still live; a concrete adapter needs a stronger heartbeat/fencing protocol before making that safe. AWS guidance similarly calls out idempotency and transaction-isolation concerns in saga orchestration. citeturn751713search2turn751713search7
