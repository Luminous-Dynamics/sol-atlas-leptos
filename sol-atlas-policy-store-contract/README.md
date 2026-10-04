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

The storage contract is defense-in-depth rather than wrapper-only validation: a conforming claim store must also reject a malformed claim without mutating its state, so direct adapter calls cannot bypass the semantic boundary.

The current IETF Idempotency-Key Internet-Draft follows the same fundamental rule: an idempotency key must not be reused with a different request payload, and retries/concurrent requests require explicit handling. The draft is currently expired rather than an RFC, so it is treated here as design guidance rather than a normative standard.

## External-effect receipt boundary (#30)

The package models the durable receipt around an external effect as `InProgress`, `Succeeded`, or `Failed` in `RecoveryExecutionEffectReceiptV2`. Each receipt also carries the exact monotonic `fence_epoch` of the execution owner. Starting an effect is an atomic idempotency operation keyed by the stable execution identity, exact input fingerprint, and current fence generation. Only the attempt that owns the `InProgress` receipt at the same fence epoch may advance it to a terminal receipt. Fence epochs protect live mutable ownership; once a receipt is `Succeeded` or `Failed`, that terminal record is immutable history and remains replayable even after a later fence generation takes ownership.

A repeated request with the same execution identity and fingerprint is therefore classified instead of starting a second effect. A different fingerprint is rejected. An uncertain store acknowledgement is reconciled by reading the receipt; the receipt state itself is never treated as proof that the external action completed.

The effect-store contract also requires defense-in-depth validation at the mutation boundary: malformed starts, malformed or argument-mismatched terminal receipts, and non-monotonic recovery successors must be rejected without mutation even when the low-level store method is invoked directly rather than through the package helpers.

This deliberately leaves one hard systems boundary explicit: the effect must itself be idempotent or transactionally coupled to the receipt if retries after an uncertain outcome are expected to be safe. AWS guidance recommends stable idempotency tokens and persisted operation state, and durable-execution replay returns checkpointed results instead of re-running completed work. These references guide the contract shape; they are not claims that Sol Atlas interoperates with any particular API. The canonical execution-result binding above likewise establishes result identity, not external-effect truth.

### Effect-fence propagation

The effect receipt is now explicitly fence-aware. A receipt must carry a non-zero `fence_epoch`, and `complete_execution_effect` supplies that epoch to the storage boundary. A stale owner whose execution fence has been superseded is therefore rejected rather than being allowed to terminalize the durable effect receipt.

For a stronger outcome-identity path, `RecoveryExecutionEffectReceiptV2::succeeded_from_execution` and `failed_from_execution` derive the terminal `outcome_digest` from the canonical `RecoveryExecutionResultSnapshotV1` content identity. `complete_execution_effect_from_execution` uses that binding after requiring the core execution to satisfy its success/failure predicate and to match the exact fence execution ID and input fingerprint. This does not prove the external side effect happened; it makes the durable receipt's recorded result identity agree with the canonical execution-result identity instead of relying on an unrelated caller-supplied digest.

The strongest completion convenience, `complete_execution_effect_for_established_fence`, composes the provenance-bearing established fence with the canonical result identity, keeping the normal path from accepting a caller-minted fence as authoritative at completion time. The lower-level raw-fence helpers remain available for adapter/reconciliation code where that provenance has already been established externally.

`RecoveryExecutionEffectReceiptV2::in_progress_for_fence` derives the receipt identity directly from an established `RecoveryExecutionFenceV1`, avoiding a second caller-supplied copy of execution ID, fingerprint, attempt, and epoch. `begin_execution_effect_for_fence` uses that same derivation for effect admission, so the normal start path has one fence-bound source of truth. The helper still requires the caller to supply a fence that was actually established by the execution-fence store; a raw structurally valid fence is not itself proof of current ownership.

The stronger `begin_execution_effect_for_established_fence` and `recover_execution_effect_for_established_fence` paths instead accept the provenance-bearing established-fence capability, reducing the normal-path risk of accidentally treating caller-minted fence-shaped data as authoritative ownership.

This closes the durable-receipt stale-owner gap, but it does not magically fence an arbitrary external API. The concrete effect adapter must propagate the same epoch/token to the protected resource and have that resource reject stale epochs. Google Chubby's sequencer design uses this same division: the client passes the sequencer to the server and the receiving server validates it before allowing the protected operation.

Receipt recovery is also an explicit compare-and-set operation: `recover_execution_effect(expected, successor_fence)` derives the successor receipt directly from the established execution fence and only accepts an `InProgress` receipt whose successor preserves the exact execution identity/fingerprint, changes the attempt, and advances the fence epoch by exactly one. Concurrent recovery therefore has a single durable winner, and an indeterminate recovery acknowledgement is reconciled by observing the resulting receipt rather than blindly repeating the transfer.

An important ordering constraint remains: execution-fence takeover must be established before effect-resource takeover. The two stores are separate local transaction boundaries. The design therefore does not claim cross-store atomicity or exactly-once external execution.
Where the external effect system cannot enforce such a token, automatic takeover remains unsafe; the adapter must use an idempotent effect contract or fail closed/manual recovery. Saga participants still require idempotency because saga orchestration does not provide distributed transaction isolation.

## Orchestration recovery matrix (#29)

The matrix is now encoded as pure, non-mutating decision functions in the contract package:
- `RecoveryAuthorizationOrchestrationStateV1::next_action()` models the safe claim/authorization progression;
- `RecoveryEffectOrchestrationStateV1::next_action()` models the safe effect replay/recovery progression.

The functions classify observations; they do not perform the next action and therefore do not imply a cross-store transaction.


The policy, claim, and effect stores are separate local transaction boundaries. The contract therefore treats orchestration as a saga-like sequence rather than a distributed transaction. The pure decision matrix keeps the permitted recovery behavior machine-checkable while preserving that boundary. Current recommended ordering is: admit policy context → atomically acquire execution claim → atomically consume authorization → begin effect → perform effect → terminalize receipt. Saga participants must be idempotent when retries are possible. See AWS Prescriptive Guidance, “Saga patterns” and “Saga orchestration pattern.”

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

The matrix intentionally leaves abandoned claims and in-progress effects without automatic expiry. Automatic expiry can transfer ownership while the original external side effect is still live; a concrete adapter needs a stronger heartbeat/fencing protocol before making that safe. AWS guidance similarly calls out idempotency and transaction-isolation concerns in saga orchestration; its Durable Execution guidance also distinguishes retry semantics from exactly-once side-effect claims.

## Fenced recovery for abandoned execution ownership (#31)

`RecoveryExecutionFenceV1` adds an explicit recovery-capable ownership generation. The first fenced claim starts at epoch `1`; recovery is an atomic compare-and-set from one exact current fence to a successor with the same execution identity and fingerprint and exactly `epoch + 1`. The store trait itself carries these invariants as defense in depth: a direct initial acquire at another epoch, or a direct recovery with a non-monotonic successor, must be rejected without mutating durable fence state.

`recover_execution_fence` never guesses ownership from elapsed time. A stale recovery attempt is rejected when its expected fence no longer matches the durable current fence. `reconcile_execution_fence` is read-only and distinguishes current ownership, another owner at the same generation, a stale generation, an older observed generation, fingerprint drift, missing state, and malformed state.

The fence epoch is a fencing capability, not authentication. After takeover, the old process may still be alive, so any correctness-sensitive protected resource must actively compare the supplied epoch with its current epoch and reject stale epochs. `check_execution_fence` makes that resource-side predicate explicit: only `Current` is admissible; `Stale`, `Future`, and `Invalid` are rejected.

For a stronger caller-side provenance boundary, `establish_execution_fence` returns an `EstablishedRecoveryExecutionFenceV1` only after the authoritative store returns `Acquired` or an exact same-attempt replay. An indeterminate acknowledgement is never promoted into that type. `recover_established_execution_fence` derives the next generation from the established handle and returns another established handle only after a positive recovery acknowledgement. This does not prove that the handle remains current after another actor recovers it; the store/resource fencing checks still establish current ownership.

The reference `FencedExecutionMemoryStore` demonstrates atomic acquisition and recovery, stale-owner rejection, fingerprint binding, and non-monotonic-successor rejection. It is test evidence only, not production persistence.

Important claim ceiling: a fence protects only resources that actually enforce it. A lease or epoch record cannot retroactively cancel an arbitrary external API call. Where the external effect system supports fencing tokens, the token must cross the adapter boundary and be enforced there; where it does not, recovery remains fail-closed/manual rather than assuming takeover is safe.

## Ownership is not authentication

`execution_id` and `attempt_id` are durable correlation/ownership identifiers only. They are not credentials, proof of authorization, or proof-of-possession. A conforming store can establish which identifier owns a record, but it cannot establish that the caller presenting that identifier is entitled to act. Caller authentication, authority validation, and any sender-constraining or proof-of-possession mechanism belong to the external authorization adapter.
