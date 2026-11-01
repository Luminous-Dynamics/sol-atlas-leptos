# IEAC cross-layer adversarial benchmark v0.1

**Status:** Preregistered synthetic expected-gate matrix. No verifier execution result, real attack outcome or field-risk claim is included.  
**Tracks:** [IEAC-CROSS-LAYER-BENCH-001 / #391](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/391).  
**Reuses:** IBAP manifest #338; stratified runtime panel #348 / PR #364; dependence-aware sampling #367; SCITT profile #375; event-history auditor #382; selective disclosure #384; witness observations #387; outcome reconciliation #390.

## 1. Why composition is the benchmark

The industrial assurance problem is not whether each cryptographic primitive passes its own unit tests. It is whether individually valid artefacts can be combined into a false overall assurance claim.

Examples:
- a genuine SCITT Receipt can cover a statement signed by an issuer that IEAC does not authorize;
- an internally consistent TS history can still omit the issuer's unsubmitted events;
- a valid SD-JWT can be replayed if the relying-party policy did not require holder Key Binding;
- a valid holder signature can bind the presentation to the wrong decision case or audience;
- a domain outcome receipt can be valid but refer to an intervention with no authorized execution;
- an observed improvement can be real but not causally attributable to the intervention.

The cross-layer benchmark measures the composed system under these cases. It does not replace the component conformance corpora.

## 2. Non-compensating decision vector

Each case preserves independent gate states. A passing gate cannot offset a failed mandatory gate in another layer.

| Gate layer | Gate identities |
|---|---|
| Signed statement | statement signature; statement issuer authorization |
| Transparency receipt | receipt issuer policy; receipt-to-statement binding; VDS inclusion |
| Transparency service histories | same-TS consistency; cross-TS statement coverage |
| Witnesses | operator independence |
| Issuer history | sequence/predecessor/checkpoint completeness |
| Source evidence | identity/lineage; rights; freshness; semantics/units |
| Selective disclosure | holder Key Binding; case and audience binding |
| IEAC runtime | admission; action authorization |
| Execution | execution receipt |
| Outcome | receipt binding; missingness; causal claim ceiling |
| Software supply chain | verifier build provenance |

The expected overall disposition is:

`reject` if any mandatory gate fails; otherwise `abstain` if any mandatory gate is unknown/unavailable; otherwise the case may be `admit_descriptive` within its declared claim ceiling.

This hierarchy is intentionally not a weighted score. A failed rights, authorization or signature gate cannot be compensated by excellent latency, confidence, service restoration or unrelated evidence quality.

The machine-readable case contract is `schemas/industry-cross-layer-adversarial-case-v0.1.schema.json`. The 18-case synthetic expected matrix is `schemas/examples/ieac-cross-layer-adversarial-matrix-synthetic-v0.1.json`.

## 3. Frozen initial matrix

The initial 18 cases cover:

| Cases | Adversarial composition | Expected disposition |
|---|---|---|
| 001 | All applicable required gates pass; descriptive/record-only claim | Admit descriptive |
| 002–003 | Valid receipt with unauthorized statement issuer; valid statement with untrusted TS receipt issuer | Reject |
| 004 | Consistent supplied TS history but issuer history has an unanchored omission/gap | Abstain |
| 005 | Independent TSs have different roots but common statement coverage | Admit descriptive; not a fork |
| 006 | Same TS identity/log/tree size has conflicting authenticated roots | Reject affected TS view |
| 007–008 | SD-JWT lacks mandatory holder Key Binding; valid Key Binding is bound to wrong audience/nonce/case | Reject |
| 009 | Source rights revoked despite cryptographically valid records | Reject |
| 010 | Current authority snapshot unavailable | Abstain |
| 011 | Outcome exists but execution receipt does not | Abstain |
| 012 | Missing/censored outcome is labelled successful | Reject |
| 013–014 | Before/after difference or synthetic evidence is upgraded to causal impact | Reject |
| 015 | Same metric label but incompatible unit/population/window | Reject |
| 016 | Verifier report has valid signature but wrong build provenance | Reject |
| 017 | Receipt advertises an unsupported VDS proof type | Abstain |
| 018 | Source snapshot is stale under its IEAC profile | Reject |

The fixture contains 2 descriptive admissions, 4 abstentions and 12 rejections across 91 gate observations. Every digest is a synthetic placeholder; these are expected vectors, not executed verifier facts.

## 4. Required independent component inputs

The benchmark should compose facts from independently versioned components:
- SCITT verifier: exact Signed Statement and Receipt signature/binding/inclusion facts, plus its explicit unsupported-VDS/refusal status;
- same-TS consistency verifier: exact tree-head signatures and RFC 9942/RFC 9162 consistency-proof result;
- cross-TS coverage auditor: statement-level coverage by service, not Merkle root equality across services;
- witness identity registry: operator domain and independently provisioned trust-root mapping;
- IEAC event-history auditor: issuer sequence/predecessor graph, authenticated checkpoint and completeness result;
- ITIP source adapter: native source identity, transformation lineage, rights, freshness, units and uncertainty;
- SD-JWT+KB verifier: issuer signature, disclosure digests, holder key proof, audience, nonce and `sd_hash`;
- IEAC admission/authorization verifier: fresh authority, scope, nonce/replay and permitted action;
- domain outcome validators: exact domain receipt and metric-contract facts, missingness, interpretation and causal limitations;
- build-provenance verifier: expected repository, builder, workflow, commit and exact executable digest.

A component that cannot evaluate an input returns an explicit `unknown` or refusal, not a fabricated pass.

## 5. Benchmark protocol

### 5.1 Preregistration

Reuse the Industrial Benchmark / IBAP run manifest rather than inventing a second run format. The manifest should bind:
- exact cross-layer fixture corpus digest;
- each component corpus digest and schema version;
- each verifier executable/source/build provenance digest;
- trust-root and keyset digests;
- expected policy, action and claim ceiling;
- seeds and generator versions for randomized robustness trials;
- hardware/software environment and resource budget.

Keep authored exhaustive controls separate from stochastic robustness sampling. A deterministic PRNG gives replayability; it does not establish IID independence. Apply #367's dependence-aware bounds and family-wise correction when many strata are tested.

### 5.2 Cross-product coverage

Start with the 18 hand-authored controls. Extend to:
- pairwise and selected 3-/4-way combinations of independent gate faults;
- multiple issuer/subject streams and event graph shapes;
- one, two and many TSs with distinct operators and statement subsets;
- observer dropout, collusion and shared administrative roots;
- one-time nonce replay under concurrent presentations;
- domain receipts with mixed measurement completeness and claim ceilings;
- verifier/key rotation and partially unavailable trust services.

Do not report only a single aggregate mutation count. Publish coverage per interaction family and the exact generated corpus.

### 5.3 Expected-vector adjudication

The expected gate vector is authored independently from the candidate evaluator. A separate adjudicator checks:
- every gate's expected and observed status;
- mandatory versus optional gate handling;
- overall disposition precedence;
- blocking gate IDs and exact artifact digest references;
- no unsupported input promoted to a pass.

A single final boolean is insufficient: the report must preserve gate vectors, blockers, unknowns and claim ceiling.

## 6. Promotion metrics

**False assurance**
- 100% of frozen positive controls produce the expected gate vector.
- 100% of frozen mandatory hard negatives block the corresponding gate.
- Zero overall false admissions in the frozen synthetic matrix.
- Zero unknown/unavailable required gates promoted to pass.
- 100% detection of the planted cross-layer combinations in the bounded corpus.

**Coverage and utility**
- 100% of preregistered pairwise and selected higher-order combinations executed.
- Report positive admission coverage and abstention separately; rejecting every case is not success.
- Report exact expected/observed reason-code precision and recall.
- Report minimal counterexample and interaction attribution for every mismatch.

**Statistical confidence**
- Per-stratum and worst-group false-admission bounds.
- Multi-seed replication and dependence-aware intervals from #367.
- Explicit sampling model, correlation assumptions and family-wise adjustment.
- Synthetic conformance percentages are not field-risk estimates.

**Performance and cost**
- p50/p95/p99 end-to-end evaluation latency and per-gate latency;
- CPU, memory, bytes read/written, proof size, storage and network egress;
- resource cost per correctly adjudicated case;
- trust-key rotation, witness dropout, service outage and recovery behavior;
- time-to-resolution for unknown/abstained cases.

Compare performance only at matched information content, hardware, policy strictness, VDS support, verifier budget and security gates.

## 7. Industry baselines

The benchmark should compare:
1. isolated component conformance only (schema/signature/receipt checks with no cross-layer composition);
2. an established SCITT reference verifier with a fixed relying-party policy;
3. a conventional digital-twin workflow that consumes vendor/application status as supplied;
4. IEAC rule-based composition with independently sourced verification facts;
5. the full cross-layer system including event history, witness observations, holder binding, rights and outcome constraints.

The objective is not to beat a specialized receipt verifier at COSE signature speed. It is to reduce false assurance when independently valid records are combined into a high-consequence industrial decision.

Use the same frozen input corpus, VDS support subset, trust material, hardware and resource budget. Report capability limitations as refusals/unknowns, not hidden exclusions.

## 8. Current limits and claim ceiling

This initial matrix is an expected-result fixture. It does not execute the component verifiers or demonstrate measured attack-detection performance. The benchmark cannot prove that issuers are honest, TS operators are globally honest, source measurements are physically true, models are valid, an outcome is causally attributable, or an intervention is safe.

Any production risk statement requires independently reviewed deployment evidence, real-world validation, domain-specific context of use and a declared adversary model.
