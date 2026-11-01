# IEAC SCITT witness-observation profile v0.1

**Status:** Structural observation contract and synthetic conformance fixture. No signed-tree-head, receipt, VDS proof or witness signature verifier is included.  
**Tracks:** [IEAC-TRANSPARENCY-003 / #385](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/385).  
**Composes with:** SCITT event profile #375 and event-history auditor #382.

## 1. The core distinction

There are three independent questions:

1. **Same-service consistency:** Are successive tree heads for the same Transparency Service (TS) append-only under the declared VDS consistency proof? Do independent observers see the same signed tree head for the same TS identity and tree size?
2. **Cross-service statement coverage:** Which statements are registered in each independently operated TS? Different TSs may register different subsets and have unrelated tree roots.
3. **Witness independence:** Are the observers controlled by genuinely distinct operators and trust domains, or do several endpoints share one administrative owner?

Do not collapse these questions into a single `transparent=true` flag.

RFC 9943 explicitly allows multiple independent TSs to register different subsets of statements and represent consistent or divergent opinions. It does not require their Merkle roots or tree sizes to match. An inter-service root mismatch is not a fork.

RFC 9162 defines Merkle inclusion and consistency proofs for a single log. A same-log, same-tree-size observation with two different signed roots is candidate equivocation evidence. A later tree head must also prove consistency with the earlier one.

## 2. Observation contract

The strict Draft 2020-12 schema is `schemas/industry-scitt-witness-observation-v0.1.schema.json`. The fixture is `schemas/examples/ieac-scitt-witness-observation-synthetic-v0.1.json`.

Each observation binds:
- exact TS identity and administrative operator identity;
- VDS type and log ID;
- receipt-keyset and registration-policy/configuration digests;
- tree size, root hash, signed-head timestamp and exact signed-head byte digest;
- optional earlier tree size/root and exact consistency-proof digest;
- observer identity, observer operator domain, build digest and observation method;
- digests of statements observed in the relevant service view.

The observation is a record of evidence inputs. It deliberately does not contain a boolean such as `consistency_verified=true`. A digest proves which bytes are being referenced, not that the signature or proof validates.

Runtime verification must retrieve the exact bytes, verify the TS signature under independently provisioned trust material, verify the VDS proof with a supported implementation, check freshness/key lifecycle, and bind its verdict to the verifier build and policy digests.

## 3. Correct comparison rules

### 3.1 Within one TS

Group by the exact tuple:

`(transparency_service_id, transparency_service_operator_id, vds_type, log_id)`

For observations with the same tree size:
- identical authenticated signed-head bytes/root are the same observed checkpoint;
- different authenticated roots for the same TS and same tree size are a split-view/fork candidate and must be preserved as evidence;
- unsigned or unverified roots are only conflicting claims, not proven TS equivocation.

For different tree sizes:
- verify the RFC 9942 VDS consistency proof from the old checkpoint to the new checkpoint;
- a missing, invalid or unsupported consistency proof is unknown or invalid according to the verifier result, never pass;
- an older head must not be silently discarded because a newer timestamp was observed.

### 3.2 Across different TSs

Different TSs maintain separate VDS histories. Compare:
- whether the same exact Signed Statement digest has a valid receipt in each required service;
- which statement subsets each service exposes;
- receipt registration time and policy scope;
- the distinct administrative operators and trust roots.

Do **not** compare root hashes, tree sizes or leaf indexes across unrelated TSs as a consistency test. A different root is expected when each service has different tree contents or ordering.

### 3.3 Witness diversity

The schema records both observer and operator IDs, but cannot prove organizational independence. Runtime must resolve these IDs through an independently provisioned registry or governance record.

Two DNS names, endpoints, regions, cloud accounts or signing keys under one controlling operator count as one administrative domain. Multiple observers with the same organization, root authority or shared control plane must not inflate a witness quorum.

A signed witness observation says what the witness claims to have seen. It does not make the witness honest. Preserve the underlying signed tree head and proof bytes for independent replay.

## 4. Synthetic fixture walkthrough

The fixture contains:
- an initial TS-A tree head at size 9;
- two independent witness operators observing the same TS-A size-10 signed head;
- a third witness observing a different TS-A root at the same size 10, a fork candidate;
- a TS-B observation with a different root but the same statement digest.

Expected structural conclusions:
- the first two TS-A size-10 observations refer to the same checkpoint;
- the third TS-A observation conflicts at the same service identity and size, pending verification of both signed heads;
- TS-B's root is not comparable with TS-A's root;
- TS-B independently has the same statement digest in its statement set;
- two witness operator IDs appear for the equal-head observation, but actual administrative independence still requires external identity authorization.

All hashes are synthetic labels; the fixture includes no signed head or proof bytes. It is not a cryptographic positive vector and does not establish a real log fork.

## 5. Threat and failure semantics

| Observation | Required treatment |
|---|---|
| Same TS, same size, same verified signed head | Same checkpoint observed by multiple witnesses |
| Same TS, same size, different verified signed roots | Preserve both; flag split-view/equivocation evidence |
| Same TS, later size, valid VDS consistency proof | Append-only extension under the supported VDS |
| Same TS, later size, missing/invalid proof | Unknown/invalid; no consistency pass |
| Different TSs, different roots | Not comparable; compare statement-level coverage |
| Different TSs, same statement digest and valid independent receipts | Corroborated registration in two services |
| Shared operator controls multiple endpoints | One administrative domain |
| Witness trust root or operator independence unknown | Witness quorum not established |
| Receipt keyset stale/unavailable/revoked | Cannot evaluate; no pass |
| Unsupported VDS | Explicit refusal; never approximate another VDS proof |
| Observation replayed or stale | Fail freshness policy; preserve audit record |

A second Receipt is not itself a consistency proof. A count of two TSs is not a Byzantine quorum. Any quorum claim needs a declared adversary model, independence evidence and a specified threshold.

## 6. Benchmark programme

The initial benchmark should include:
- valid same-TS tree growth across 2, 3 and 10 checkpoints;
- valid same-TS same-size identical head observations from independent witnesses;
- same-TS same-size conflicting signed roots;
- same-TS later size with invalid, absent, replayed or wrong-log consistency proof;
- two distinct TSs with different roots and one common statement digest;
- two TSs registering disjoint subsets;
- endpoints sharing one operator, one key root or one control plane;
- witness sets with 1, 2, 3, 4 and 7 distinct operators, plus dropouts and collusion;
- stale/future observations and key rotation;
- wrong log ID, wrong VDS type and receipt bound to a different statement.

Promotion targets:
- 100% detection of planted same-TS same-size root conflicts when both signed heads verify;
- 100% rejection of invalid same-TS consistency proofs;
- zero false fork alarms for legitimate independent TS roots;
- 100% correct statement-coverage classification on the frozen corpus;
- zero false independence counts for shared operators/trust roots;
- zero stale, unsupported or unavailable proof paths promoted to pass.

Measure p50/p95/p99 detection latency, proof bytes, observation age, CPU/memory, network egress, witness dropout sensitivity and service outage behavior. The 100% thresholds are synthetic conformance targets, not real-world risk bounds.

Report four separate metrics:
- `same_ts_consistency`;
- `cross_ts_statement_coverage`;
- `witness_operator_independence`;
- `issuer_history_completeness`.

An auditor must never collapse these to a single boolean.

## 7. Industry comparison

- RFC 9943 defines multiple TSs, Signed Statements and receipts; use that architecture rather than inventing a new log.
- RFC 9942 defines VDS proof structures and receipt binding.
- RFC 9162 supplies the established Merkle inclusion/consistency proof model for CT-style logs.
- Microsoft `scitt-verifier` provides a useful offline verification baseline for CCF receipts, but its published implementation documents a narrower receipt-VDS support scope and no revocation checking.

IEAC's contribution is the industrial relying-party policy, operator-diversity requirements, event-history completeness semantics and decision-outcome reconciliation—not a new Merkle algorithm.

## 8. Claim ceiling and next work

This tranche is a schema/fixture/profile specification. It does not:
- verify any TS signature or VDS consistency proof;
- authenticate witness identity or operator independence;
- prove that an issuer submitted every statement it created;
- authenticate physical evidence or model validity;
- grant action or actuation authority.

The implementation phase should compose an independently pinned SCITT verifier, a separate VDS consistency verifier and an external operator-identity registry. The structural auditor should classify their facts without reimplementing COSE or Merkle cryptography.
