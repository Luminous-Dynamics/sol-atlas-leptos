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

An important ordering constraint remains: execution-fence takeover must be established before effect-resource takeover. The two stores are separate local transaction boundaries. The new fence-revalidated start helper improves the ordering boundary by rejecting already-stale ownership before the effect-store mutation, but it cannot make the two writes atomic. The design therefore does not claim cross-store atomicity or exactly-once external execution.
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

`reconcile_established_execution_fence` provides the corresponding read-only freshness check. It reports whether that established generation is currently observed, stale, missing, or malformed. `begin_execution_effect_after_fence_revalidation` composes that observation with an immediate effect-store start attempt and returns a typed result that distinguishes authoritative fence rejection from an effect mutation attempted after a positive point-in-time observation.

This remains deliberately advisory rather than transactional: the fence store and effect store are separate transaction boundaries, so a fence can advance after the freshness read and before the effect-store mutation. The composed helper therefore makes the race explicit instead of silently treating an established handle as permanently current; it does not claim cross-store atomicity or exactly-once execution.

The reference `FencedExecutionMemoryStore` demonstrates atomic acquisition and recovery, stale-owner rejection, fingerprint binding, and non-monotonic-successor rejection. It is test evidence only, not production persistence.

## Protected-resource fence boundary (#33)

`RecoveryExecutionProtectedMutationV1` carries the exact execution identity, input fingerprint, attempt, fence epoch, and optional stable external idempotency key across the adapter boundary.

`RecoveryExecutionFencedResource::mutate_if_fence_is_current` is the resource-local enforcement contract. A conforming implementation must validate the supplied fence and make the protected mutation as one resource-local atomic decision. It must not implement the boundary as a separate read of the current epoch followed by a later mutation.

The reference result distinguishes applied work, exact same-request replay, stale ownership, future/unestablished epochs, malformed fences, identity mismatch, different-attempt ownership, and a residual indeterminate outcome. When an idempotency key is supplied, it is part of replay identity and changing it is rejected. Fencing alone does not imply retry idempotency; adapters must use a stable idempotency key or another idempotent operation contract when replay can repeat an external effect.

The validation helper returns `Valid`/`Invalid` only for structural binding against an established fence. It deliberately does not report that a protected mutation occurred.

`RecoveryExecutionProtectedMutationReconciler` is the corresponding read-only boundary for an `Indeterminate` resource acknowledgement. It can report an exact request as observed applied, not currently observed, different, missing, or invalid. `ObservedNotApplied` is explicitly a point-in-time observation and is not proof that a prior indeterminate network request never took effect. When reliable reconciliation is unavailable, callers must remain fail-closed/manual unless the external system supplies an independent idempotency guarantee.

## External-effect safety profile (#35)

`RecoveryExecutionEffectSafetyProfileV1` makes adapter capabilities explicit instead of inferring recovery safety from the existence of a fence or receipt. Fencing, idempotency, and reconciliation are independent capability declarations, and unknown capabilities are deliberately non-permissive.

The derived modes are conservative: stable-key or transactionally coupled idempotency permits automatic retry; automatic takeover additionally requires fencing enforced at the protected-resource mutation boundary. A profile without both properties resolves to `ManualRecoveryOnly`. This profile describes adapter guarantees only; it does not establish authentication, proof-of-possession, cross-store atomicity, exactly-once execution, or truth of the external state.

The reference tests cover automatic takeover, retry-only operation without fencing, manual recovery without idempotency, unknown-capability fail-closed behavior, and malformed-profile fail-closed behavior.

The profile is also consumed by `RecoveryExecutionEffectSafetyProfileV1::next_action`, which maps observed protected-mutation states to permitted recovery actions. Thus a caller cannot obtain an automatic retry action from an adapter that has not declared idempotency, and an indeterminate mutation becomes manual recovery when the adapter has not declared read-back reconciliation.

Important claim ceiling: a fence protects only resources that actually enforce it. A lease or epoch record cannot retroactively cancel an arbitrary external API call. Where the external effect system supports fencing tokens, the token must cross the adapter boundary and be enforced there; where it does not, recovery remains fail-closed/manual rather than assuming takeover is safe.

## External-effect adapter conformance (#36)

The safety profile is only a capability declaration until an adapter-specific
conformance suite demonstrates the declared behavior. The conformance evidence
record is kept separate from the profile itself so a declaration cannot prove
its own truth.

For the StableKey capability, the concrete mutation must also carry a
non-empty stable idempotency key. `next_action_for_mutation()` is the
request-specific admission path; the generic profile action does not infer a
key that is not present on the request.

The reference resource now exercises the evidence path for:

- current-fence acceptance;
- stale/future-fence rejection;
- same-key exact replay;
- different-request rejection under the same key;
- changed-idempotency-key rejection;
- exact read-back of the applied request;
- explicit point-in-time reconciliation semantics;
- concurrent stale/current fencing behavior.

This still is not evidence about an arbitrary third-party service. A real
adapter must run the same conformance boundary against the actual protected
resource and preserve the evidence under CI/qualification.

External systems illustrate why this boundary belongs in the adapter. AWS
documents that retries can execute an operation more than once and recommends
stable idempotency keys for replay-safe work; conditional database writes can
also make an otherwise uncertain retry safe when the condition makes the
operation effectively idempotent. Google Cloud Storage exposes generation and
metageneration preconditions that let a resource reject stale writes. Chubby
uses a sequencer passed to the protected server, which validates the sequence
before accepting the operation. These are patterns for adapter design, not
claims of interoperability or exactly-once execution here.

#36 remains open until an actual external-effect adapter supplies
resource-specific conformance evidence.

## Conformance provenance (#37)

`RecoveryExecutionEffectConformanceReportV1` binds conformance evidence to
the exact safety-profile digest, adapter identity/revision, and harness
identity/revision that produced the report. The evidence payload itself is
content-addressed, and the report digest covers the complete provenance
binding.

`supports_profile()` therefore rejects:

- a report produced for a different safety profile;
- a modified evidence payload whose digest no longer matches;
- a report whose adapter or harness revision was changed without recomputing
  the report identity;
- missing or malformed provenance fields.

This is deliberately provenance, not authentication. A caller can still
forge a complete report unless the surrounding qualification system protects
the evidence source and verifies the claimed adapter revision. The contract
therefore treats the report as a tamper/drift detector and qualification
input, not as a cryptographic proof that an external service behaved honestly.

#37 remains open until the real adapter qualification path stores and reviews
these reports as part of its CI/evidence workflow.

## Ownership is not authentication

`execution_id` and `attempt_id` are durable correlation/ownership identifiers only. They are not credentials, proof of authorization, or proof-of-possession. A conforming store can establish which identifier owns a record, but it cannot establish that the caller presenting that identifier is entitled to act. Caller authentication, authority validation, and any sender-constraining or proof-of-possession mechanism belong to the external authorization adapter.
