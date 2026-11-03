# IEAC incremental impact deltas v0.1

**Status:** Synthetic reference implementation and contracts; not a production trust store or authenticated revocation processor.  
**Parent:** [IEAC temporal assurance lifecycle / #396](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/396), [full-report auditor PR #397](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/397).  
**Track:** [IEAC-IMPACT-DELTA-001 / #475](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/475).  
**Owner:** Sol-Atlas owns the built-environment/assurance dependency graph and its impact computation. Mycelix owns identity, authority and evidence governance. This component consumes the already-resolved event and does not create authority.

## 1. Paradigm shift

The full-report auditor remains the reconciliation and audit representation. The delta path adds a different execution mode:

`revalidate graph + rebuild adjacency + materialize every node` for every event

becomes:

`validate exact base once + reuse immutable graph index + traverse exact affected closure + emit changed rows only`.

The delta is a *proof-carrying state transition proposal*, not a decision or a permission. It is valid only for its exact graph snapshot, full base report, event-chain head/sequence, policy and evaluator identity. A consumer must reject a delta whose base state differs from its own.

Reachability answers **what may need review**. It does not grant the event issuer authority over every descendant. Only exact event targets can be directly invalidated; descendants can be classified for re-verification or quarantine.

## 2. Artifacts and contracts

- `industry-assurance-impact-delta-input-v0.1.schema.json`: frozen event batch plus evaluation time, policy and engine digests.
- `industry-assurance-impact-state-pin-v0.1.schema.json`: exact graph/report digest, event-chain high-water sequence/head, and latest recorded/trusted times.
- `industry-assurance-impact-delta-v0.1.schema.json`: sparse changed-row transition, direct targets, independently checked closure, base and result digests.
- `industry-assurance-impact-delta-batch-v0.1.schema.json`: one output envelope carrying all deltas, final full report and next state pin.
- `ieac-impact-delta-source-correction-synthetic-v0.1.json`: schema-valid synthetic event fixture.

The batch runner validates and indexes the graph once, initializes one mutable impact-row map, applies each event's sparse changes in place, and materializes the final full report once at the batch boundary. It enforces a cumulative limit of 2,000,000 affected-node closure memberships per batch. The tool reuses the main lifecycle auditor's strict duplicate-member parser, graph/event validators and exact impact-row classifier. It does not create a second event vocabulary or signature format.

## 3. State and event binding

The state pin binds:
- exact graph snapshot SHA-256;
- exact canonical full impact-report SHA-256;
- incremental `impact_state_sha256` root, initialized from the genesis report digest and advanced by each accepted event;
- last authority sequence and event-chain head digest;
- last `recorded_at` and `trusted_at` times;
- exact impact policy and engine build digests.

For the next event, sequence must be exactly prior + 1, predecessor digest must match the state pin, and neither recorded nor trusted time may move backward. The new trusted time cannot precede its recorded time. A policy or evaluator identity change requires full reconciliation before a delta can be applied.

The state root is advanced as a canonical digest over the previous state root, event digest, sequence and digest of the sorted sparse update rows. This lets a consumer verify the transition chain without requiring the producer to rebuild the full report after every event. The final state pin binds both the final full-report digest and the final state root.

The state pin explicitly says `caller_managed_monotonic_external_store_required`. This package does not implement persistent atomic high-water storage, distributed consensus, pin authentication, key custody or crash-safe compare-and-swap. A caller that reloads an older pin is vulnerable to rollback unless it stores the high-water in a separately protected monotonic system.

## 4. Graph index and closure

For one batch, the tool:
1. Validates exact graph digest, typed node identities, endpoints, unique edges and acyclicity, with the parent auditor's 100,000-node / 500,000-edge limits.
2. Builds deterministic forward and reverse adjacency, checks their consistency and computes an index identity digest bound to the graph snapshot.
3. Reuses that index for every event in the batch.
4. Traverses forward from exact authorized targets to calculate the impact closure.
5. Computes the lifecycle engine's breadth-first paths and compares its affected-node set against a separate iterative depth-first closure.
6. Rejects any claimed direct invalidation on a node that is only reachable and not an event target.

The closure checks are deliberately distinct in traversal strategy. The classification implementation is shared with the full-report auditor, not a second independent implementation of policy semantics. The full report remains the policy/reconciliation reference; the DFS only independently checks closure membership.

## 5. Sparse transition semantics

The delta contains:
- direct target node IDs, separately from the full affected node IDs;
- only rows whose classification/reasons/path change under the monotonic merge policy;
- exact immutable batch base-report SHA-256, graph and index SHA-256;
- base and resulting incremental state-root SHA-256;
- event ID, sequence, digest and predecessor;
- exact policy/evaluator digests;
- resulting report digest and event-chain head;
- closure verifier name and completion marker;
- no action authority.

The v0.1 state root commits each transition as `H(previous_state_root, event_digest, event_sequence, sparse_updates_digest)`. The batch path does not rebuild a full report per event; it updates the in-memory row map and materializes one final full report after all events are processed.

The v0.1 merge is deliberately conservative and monotonic:
- a new correction may raise a row from eligible/no-impact to reverify/quarantine;
- an existing quarantine or invalidation is not silently cleared by a later event's absence from its closure;
- reason codes are retained and unioned deterministically;
- the full report remains one row per graph node.

A reversal, correction that reinstates reliance, or expiry of a quarantine requires a later, explicit policy and a full reconciliation event. It is not inferred from an empty sparse update.

## 6. Single-file result envelope

The CLI exposes both a producer batch mode and a consumer `apply` mode. The consumer verifies the complete received delta against the exact base state pin before producing a new result. The CLI writes one `create_new` output file containing:
- the ordered delta batch;
- final full impact report;
- next state pin;
- resulting report digest, event-chain sequence and head.

This avoids exposing a three-file inconsistent tuple at the application boundary. The file writer uses create-new semantics and synchronizes the file, but it is not a crash-atomic rename or a durable distributed transaction. The caller must persist the envelope and advance its protected high-water state in one external transaction or recover through full reconciliation.

## 7. Reconciliation and negative controls

The synthetic tests exercise:
- sparse transition equality with a full graph recomputation + monotonic merge;
- independent DFS closure versus lifecycle BFS closure;
- exact base report and graph pinning;
- tampered delta even after its own digest is recomputed;
- exact replay idempotence and replay after high-water advance;
- event sequence gaps and broken predecessors;
- recorded/trusted time rollback;
- multi-event batch reusing one graph index;
- no direct invalidation of a descendant merely because it is reachable.

CI additionally validates each schema and runs the frozen source-correction event through the CLI. It must reject stale-base and backdated-time mutants.

## 8. Benchmark programme

Use the same frozen graph and event corpus to compare:

| Lane | Strategy | Purpose |
|---|---|---|
| B0 | Full report recomputation for every event | Reference/audit baseline |
| B1 | Validate graph + build index + sparse impact closure | Separate index cost from event cost |
| B2 | Reuse a validated immutable index across an event batch | Incremental operational candidate |
| B3 | Differential Dataflow adapter | Research comparator only; not a production dependency decision |
| B4 | Independent full-recompute verifier | Reconciliation oracle |

Graph sizes: 100, 1,000, 10,000 and 100,000 nodes; vary edge count, fan-out/fan-in, depth and disconnected components. Impact fractions: 0.1%, 1%, 10%, 50% and 80%. Run one event and repeated events on an unchanged graph; separately measure graph revision/index rebuild.

Report:
- closure false-positive and false-negative counts;
- exact full-report digest agreement after delta application;
- direct-target over-revocation (target: zero);
- index build latency and bytes;
- event-to-delta p50/p95/p99;
- visited nodes/edges and output bytes / full report bytes;
- peak RSS and allocations;
- fraction of graph rows touched and rows actually changed;
- replay determinism, stale-base/rollback detection and recovery outcomes;
- re-verification amplification (impacted rows per true dependent row).

Do not set an arbitrary speed-up target before baseline measurements. Publish raw per-case results, build and graph digests, environment, thread count, memory budget and repeated-run uncertainty. Use the full-report oracle to establish correctness before promoting performance.

## 9. Acceptance targets

**P0 — correctness and binding**
- zero false-positive and zero false-negative closure nodes on the frozen exact-expected corpus;
- zero direct invalidations of untargeted descendants;
- 100% rejection of stale base, wrong graph, broken predecessor, sequence gap, tampered delta and time rollback;
- delta application reconstructs the exact expected full-report digest;
- one graph-index construction and one mutable row-map initialization per batch, reused for all accepted events;
- exactly one full-report materialization at the batch boundary;
- cumulative affected-node closure memberships at or below the v0.1 budget of 2,000,000;
- unknown or unauthenticated trust state never silently becomes accepted.

**P1 — incremental benefit**
- sparse updates contain no `no_impact` rows;
- report update count and affected closure count remain separately measured;
- publish B0/B1/B2/B3 accuracy, latency, memory and output-size Pareto curves at all graph sizes;
- show measured savings at low impact fractions without security or correctness regression.

**P2 — production readiness**
- durable authenticated high-water storage;
- crash-safe state/result transaction with recovery and concurrent-writer semantics;
- independently implemented impact-policy evaluator;
- rights-cleared, independently adjudicated real dependency graphs and held-out event classes.

## 10. Claim ceiling

This is a deterministic synthetic dependency-closure and sparse state-transition prototype. It does not authenticate COSE signatures, SCITT receipts, issuer/revoker authority, trust roots, timestamps or incident facts; does not persist a protected high-water mark; and does not prove source correction truth, legal authority or physical consequence. It cannot issue permits, qualify a physical design, authorize operations, or undo a physical intervention. A passing synthetic benchmark is not production reliability, regulatory conformity or SOTA superiority.
