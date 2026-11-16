# Monotone Invariant Kernel
## Scaling and exactness protocol v0.1

**Owner:** [DEEP-TIME-VIABILITY-SCALING-001 #901](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/901).  
**Exact oracle:** finite stationary controlled-invariant solver in [sol-atlas-core/src/viability_kernel.rs](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/sol-atlas-core/src/viability_kernel.rs).  
**Structured prototype:** [sol-atlas-core/src/monotone_invariant.rs](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/sol-atlas-core/src/monotone_invariant.rs).  
**Status:** source implementation and test fixtures committed; tests have not yet been executed on the current exact PR head. No runtime or superiority claims.

## 1. State of the art and the actual assumptions

Sönmez, Khaled, Zamani and Arcak (2026), *Real-Time Synthesis of Robust Controlled Invariant Sets for Monotone Systems*, arXiv:2609.14115, represent a lower-closed safety set on a finite product grid by one threshold per projected column. They prove that, under state-monotone abstract transitions, the robust controlled predecessor of a lower set is lower-closed; within each column, predecessor membership is a prefix and its threshold can be found by binary search. They prove one-step equivalence with an antichain/lazy-basis formulation. The paper gives O(N^(d-1)) threshold storage and O(C log N) per-column predecessor work; columns are independent and parallelizable. It reports 10^9 grid cells in under 50 ms and 10^14 in under two minutes in its experiments.

Primary source: https://arxiv.org/abs/2609.14115 .

Critical assumptions from the paper:
- a finite product-grid abstraction and a declared componentwise partial order;
- a lower-closed safety set under that same order;
- a state-monotone abstract transition map for every admissible controller action and disturbance;
- a finite disturbance alphabet that soundly represents the abstracted continuous exogenous effects.

The paper explicitly warns that finite disturbance modes are not automatically a sound replacement for a continuous disturbance set: an extremal-mode reduction needs an additional dominance property, such as disturbance-state monotonicity.

These are not generic infrastructure assumptions. Rights, authority, discrete consent, common-cause hyperedges, non-monotone ecology, categorical configuration changes and model-form disagreement do not automatically form a lower-closed product order.

## 2. What this implementation does—and does not do

The current prototype restricts transitions to saturating affine integer maps:

`x'_j = clamp(sum_i(a_{ji} x_i) + b_j, 0, N_j-1)`

with all `a_ji >= 0`. This is a machine-checkable sufficient condition for componentwise state monotonicity. Negative offsets are allowed; negative coefficients are not.

A lower-closed initial safe set is encoded by one signed height threshold per projected column. The sentinel −1 denotes an empty column; all other values are 0-based maximum safe heights. The validator checks the antitone threshold relation on every cover edge. The implementation:
- stores O(number of projected columns) thresholds, not a full boolean state universe;
- computes the greatest stationary invariant subset by repeatedly applying per-column binary search;
- reports explicit state-universe, column, iteration and predecessor-query limits;
- returns BOUNDED without partial thresholds when a limit is exceeded;
- computes a witness action on demand and binds the report to a non-cryptographic problem fingerprint to reject accidental stale-model queries.

**Current performance boundary:** the columns are processed sequentially. The paper's parallel column update is a future implementation target. No speedup, real-time property or SOTA parity is claimed.

## 3. Exactness and generated test panel

The machine-readable benchmark panel is [MONOTONE_INVARIANT_THRESHOLD_BENCHMARK_V0_1.json](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/research/deep-time-infrastructure/MONOTONE_INVARIANT_THRESHOLD_BENCHMARK_V0_1.json), with its Draft 2020-12 schema.

The current Rust module has eight tests written:

1. Small 2D threshold result compared with explicit state-set greatest-fixed-point iteration.
2. 160 generated differential cases: 16 cross-coupling/offset rules × 10 lower-closed threshold sets on a 2×3 grid. Non-negative coefficients guarantee transition monotonicity; each compressed result is compared with a separate explicit-set fixed-point representation.
3. Reject an initial threshold set that is not lower-closed.
4. Reject malformed affine transition dimensions.
5. Return BOUNDED without partial thresholds when a column cap is exceeded.
6. Reject a witness query when its problem fingerprint differs from the one used to compute the report.
7. 8,000,000-state / 40,000-column identity fixture, checking representation count and invariant cardinality.
8. Maximum-i64 height-axis boundary control, checking that signed casting does not overflow.

These are test definitions, not execution evidence. The exhaustive 160-case fixture checks finite algorithm agreement only; the 8-million-state fixture is a structured smoke test and does not provide measured runtime or memory.

## 4. Matched comparison matrix

Every method must receive the same finite grid, initial safe set, action alphabet, disturbance rules, axis order and terminal/invariant semantics.

| Comparator | What it represents | Required measure |
|---|---|---|
| Explicit state-set greatest fixed point | Exact small-instance oracle | Exact set equality, time, memory |
| Threshold table (current sequential prototype) | O(columns) representation | Thresholds, predecessor probes, iterations, time, peak memory |
| Antichain/lazy basis | Existing monotone-system representation | Basis size, dominance/membership checks, time, peak memory |
| Threshold table (future parallel implementation) | Same algorithm with parallel column execution | Scaling by cores, deterministic exact equality, scheduling overhead |
| BDD/symbolic encoding | Structured Boolean representation | Node count, reorder cost, time, memory, exact equality |
| CEGAR / sound abstraction | Large heterogeneous model approximation | Abstract state count, refinements, counterexamples, false-viable rate |

Report product-grid dimensions, total state count, projected column count, max axis size, actions, disturbance branches, number of fixed-point rounds, predecessor-query count, hardware, compiler, build profile, thread count and memory measurement method. No cross-paper speedup comparison is valid without equivalent transition models, hardware and implementation conditions.

## 5. Proposed acceptance ladder

- **M0 — shape and order:** reject malformed rule dimensions, invalid axes and non-lower-closed safe sets.
- **M1 — exact finite:** 100% agreement with the explicit oracle on the 160 generated maps and additional randomized seeded finite maps.
- **M2 — metamorphic:** invariance under input-order permutations; monotonicity preserved under non-negative coefficient changes; safe-set shrinkage cannot enlarge the invariant subset when transition/action assumptions stay fixed.
- **M3 — scale:** publish time and peak memory for increasing grid sizes; include the 8-million-state case only after it executes.
- **M4 — fair algorithm comparison:** explicit enumeration, antichain, threshold sequential, BDD and threshold parallel where applicable, all on identical mathematical problems.
- **M5 — infrastructure integration:** only after typed dependency/hard-floor structures are proven to admit a sound partial order or are separated into a compositional exact/symbolic method.

Targets:
- zero mismatches against the independent explicit oracle on frozen small instances;
- zero acceptance of invalid lower-set or malformed-rule inputs;
- 100% deterministic replay for pinned finite cases;
- BOUNDED rather than partial exact-looking results when resource limits are reached;
- publish p50/p95 time, peak memory and all problem dimensions; do not optimize only one headline grid size.

## 6. Extension beyond affine monotonicity

The next scientific task is a proof-carrying abstract transition interface that can admit a broader monotone function only when state monotonicity is established by a checkable certificate or external proof. If the proof is absent, the solver must reject the structured fast path and use the exact finite kernel or return BOUNDED/UNRESOLVED.

For physical disturbances, a small finite list is insufficient unless it soundly over-approximates the continuous disturbance set or a dominance theorem justifies extremal modes. The infrastructure owner must provide this evidence; the threshold kernel must not infer it.

For mixed infrastructure systems, use a product order only for state variables whose order semantics are defensible and stable. Categorical authority/consent and non-monotone ecological constraints remain separate hard gates or are handled by the explicit game/assume-guarantee layer. Do not make every asset graph “monotone” by inventing coordinate transforms after seeing the desired result.

## 7. Claim ceiling

A successful implementation would establish exact stationary invariance for the restricted synthetic monotone grid class. It would not establish that any real canal, port, city, water allocation, ecosystem or railway system satisfies those assumptions. No industry or SOTA superiority is claimable until matched benchmarks are run and independently reviewed.
