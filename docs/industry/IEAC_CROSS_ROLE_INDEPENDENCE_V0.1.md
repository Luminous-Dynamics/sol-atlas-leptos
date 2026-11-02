# IEAC cross-role verifier and witness independence v0.1

**Status:** Role-to-CCF binding contract and synthetic common-control-plane fixtures. This is a composition layer over the existing CCF auditor, not a new dependency-graph engine.  
**Issue:** [IEAC-IND-002 / #423](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/423).  
**Composes with:** common-cause auditor #406 / PR #407; channel completeness #419 / PR #422; decision-time evidence frontier #413 / PR #415; cross-layer false-assurance benchmark #391 / #392.

## 1. Paradigm shift: independence is about control

Distinct keys, endpoints, organizations and cloud regions do not automatically provide independent evidence.

A query signer and an access-ledger operator may use different companies and different keys while sharing one cloud account, KMS administrator, CI release pipeline, break-glass operator or control plane. That is one effective failure/control domain for the relevant claim.

The claim we need is not "there are N signatures". It is:

> The exact query, log, checkpoint, verifier, witness and relying-party policy roles have dependency closures that satisfy the separation requirements under one pinned common-cause registry and one independently checked CCF report.

This profile must not convert missing dependency information into independence credit.

## 2. Reuse #407, do not rebuild it

The cross-role request binds:
- exact channel-manifest and availability-attestation digests;
- exact CCF registry ID/version/authority and registry digest;
- the CCF request and report digests;
- CCF auditor build identity;
- verifier build, verification policy and trust-bundle digests used to accept the CCF report.

Each role assignment points to one CCF source ID and domain ID. Every typed dependency token includes the corresponding CCF dependency-node or common-cause-group reference. The future adapter consumes #407's report and its source_resolutions, conflict_edges and control_results. It must not independently recompute transitive closure or a maximum independent set.

The role binding adds the missing application semantics: which role each CCF source represents, and which pairs must be separated for the channel-completeness claim.

## 3. Roles

The v0.1 role vocabulary is:

1. channel policy owner;
2. access-ledger operator;
3. zero-match query signer;
4. transparency-service operator;
5. checkpoint/witness operator (repeatable);
6. verifier build/release authority;
7. verifier deployment operator;
8. trust-root/key-management administrator;
9. relying-party policy owner;
10. independent reviewer.

Each binding records a stable assignment ID, principal, organization, CCF source/domain, source identity digest, identity-evidence digest, dependency-closure status and typed dependency references.

Dependency tokens include cloud account, control plane, cluster, shared database, KMS/HSM administrator, CI/release pipeline, trust-root administrator, break-glass authority, DNS control plane and identity provider. The token's CCF reference must resolve against the exact CCF registry/report closure. A free-form label without a CCF reference gives no independence credit.

## 4. Non-compensating separation requirements

The request names exact assignment pairs and the required distinctions for each pair. Typical requirements include:
- different principal and organization;
- different cloud account and control plane;
- different KMS and break-glass administrators;
- different CI/release pipeline and trust-root administrators;
- disjoint CCF dependency closure and common-cause groups.

The future auditor returns one result per role and one per separation requirement:
- separate_under_pinned_ccf_model;
- conflict_under_pinned_ccf_model;
- unknown.

A mandatory conflict prevents independence qualification even if all signatures verify. Unknown or disputed dependency closure remains unknown; it is never treated as an empty dependency set. Keep source-level cryptographic validity, channel completeness and organizational independence as separate gates.

## 5. Synthetic fixtures

The synthetic corpus contains two cases:

- **Shared cloud/KMS/CI/control plane:** distinct principals and organizations intentionally share operational dependency IDs. Ledger/query, query/verifier-build, verifier-build/deployer and witness A/B separation must fail under the declared model. The reviewer pairs remain separate.
- **Disjoint control/failure domains:** every mapped dependency token and principal/org identity is distinct, so the declared pair requirements pass under the synthetic model.

All hashes, identities, dependency evidence and CCF registry/report digests are synthetic placeholders. The cases test contract shape and common-cause reasoning only. They do not contain an actual #407 auditor output and do not prove a real operator's dependency closure.

## 6. Benchmarks

### Safety / false-independence

- 100% detection of same-control-plane, same-cloud-account, shared-KMS, shared-CI, shared-root-admin and shared-break-glass controls in the frozen corpus.
- Zero independence credit for missing, stale, disputed or incomplete CCF dependency closure.
- 100% binding between the role request and exact channel manifest, availability attestation, CCF registry, CCF request, CCF report and auditor build.
- 100% role/source/domain identity consistency with #407 report resolutions.
- 100% detection of CCF conflict edges between required-separated roles.
- Unknown remains unknown; no fallback to distinct-organization or distinct-key counting.

### Utility

- Report per-role status and per-pair status; do not collapse to one weighted score.
- Measure false-independence credit, conservative over-rejection, role coverage, unknown rate, conflict-reason precision/recall and worst-group performance.
- Compare distinct-key count, distinct-organization count, endpoint count and registry-backed dependency separation on identical cases.

### Scale / cost

- CCF dependency graphs at 10², 10³, 10⁴ and 10⁵ nodes.
- Report dependency closure size, conflict edges, per-role resolution time, p50/p95/p99, memory and report bytes.
- Stress transitive fan-in/fan-out, shared providers, rotations and registry updates.
- Preserve #407's exact maximum-independent-set versus greedy-lower-bound status; never present a lower bound as proven optimal.

Synthetic success is conformance to this frozen model, not a field probability that two operators will fail independently.

## 7. Next implementation

The next Rust adapter should take three separately supplied inputs:
1. role-binding request;
2. exact common-cause report from #407;
3. independently verified channel manifest/availability attestation facts.

It should verify raw-byte digests and version identities, map every role's CCF source/domain to the report's source resolutions, consume the report's dependency closures and conflict edges, and emit per-role/per-pair status. It must not recalculate the CCF graph or its maximum independent set. The outer trust layer remains responsible for authenticating the CCF report, registry authority and identity evidence.

## 8. Claim ceiling

The output establishes separation only under the exact pinned CCF registry/report and the declared role-binding model. It does not prove that organizations disclosed every dependency, that no unmodelled common cause exists, that a source is truthful, or that the channel is universally complete. profile_qualification_attested = false; action_authority = none.
