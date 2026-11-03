# Temporal Decision Freshness and Revocation — Research Contract v0.1

**Status:** research contract / not yet implemented  
**Depends on:** descriptor catalogue composition, federated inventory manifests, IEAC dependency graph and built-environment revalidation frontier  
**Primary claim:** a historical engineering decision remains an immutable record, while its present reliance status is recomputed from later evidence, source, trust, rights and dependency events.

## 1. The paradigm shift

A conventional digital twin often answers what the model looks like now. An engineering assurance system must additionally answer:

- what exact evidence was available when this decision was made?
- what snapshot/watermark did each contributing source represent?
- which connector, schema, solver, policy and verifier versions were used?
- what new information has arrived since then?
- has an input been revised, deleted, corrected, disputed, expired, or revoked?
- which downstream conclusions must be revalidated?
- is the decision still current for this declared reliance purpose and scope?

This is not a mutable status field on an approval. It is a temporal evidence ledger plus a reproducible invalidation computation.

## 2. Separate the clocks

Do not collapse these times:

1. **Source valid time:** when the source says the represented fact was true.
2. **Source transaction/recorded time:** when the source committed that fact.
3. **Observation/retrieval time:** when the connector observed or retrieved it.
4. **Trust evaluation time:** when issuer identity, signature, rights and policy were evaluated.
5. **Decision time:** when the authorized actor recorded the decision.
6. **Reliance evaluation time:** when a downstream actor asks whether the decision may still be relied upon.

Preserve source clock uncertainty and known skew. An event arriving late can have an earlier valid time than the decision but a later observation time. A system must not treat ingestion order as truth order.

## 3. Decision evidence vector

A decision receipt should bind a canonical as-of vector with at least:

- case ID and exact typed-case representation digest;
- IEAC graph ID and graph snapshot digest;
- descriptor catalogue digest(s);
- source inventory receipt digest(s) and bounded completeness disposition(s);
- per-source owner namespace, source snapshot ID/digest and source watermark/sequence;
- exact native record identity and payload digest basis for each relied-upon record;
- connector and mapping-profile digests;
- solver/replay artifact digests and declared assumptions;
- reviewer/verifier build digests;
- policy, trust bundle and rights evaluation digests;
- decision event ID, actor/authority evidence, decision timestamp and scope;
- freshness policy ID and version;
- explicit limitations and unresolved dependencies.

The vector is not merely a list of latest records. It identifies the precise version of each evidence stream at decision time.

## 4. Event classes and reliance effects

Represent change events as immutable append-only records. At minimum distinguish:

| Event | Example | Default reliance effect |
|---|---|---|
| Source revision | IFC/PLM/permit record changed | Invalidate dependent conclusions |
| Source correction | Prior observation corrected with earlier valid time | Re-evaluate every decision whose as-of window is affected |
| Retraction/tombstone | Certificate/permit/inspection withdrawn | Invalidate or suspend dependent conclusions |
| Trust revocation | Signing key, issuer or trust path revoked | Re-evaluate affected attestations and decisions |
| Rights change | License/contract no longer permits use | Block prohibited future reliance/use; retain historic record |
| Policy update | Acceptance criteria or regulatory policy changes | Re-evaluate only under explicitly versioned transition policy |
| Schema/connector change | Mapping semantics or parser changes | Re-run affected normalization and compare semantic deltas |
| Solver/verifier defect | A prior build is found unsound | Invalidate outputs produced by the affected build/profile |
| Freshness expiration | Source evidence exceeds its reliance TTL | Mark stale/unknown until refreshed; do not silently promote to false |

A change event does not rewrite an old decision. It creates a new reliance evaluation with a reasoned disposition.

## 5. Revalidation certificate

The revalidation result should be a portable, content-addressed certificate binding:

- decision ID and original decision receipt digest;
- exact prior as-of vector digest;
- current source inventory/descriptor/graph snapshot digests;
- changed source-native identities and change-event digests;
- dependency graph and mapping profile digests;
- invalidation algorithm/version/build digest;
- full-scan oracle version and digest, when used;
- impacted node/receipt/decision IDs, sorted canonically;
- missing/unknown dependency sets;
- resulting disposition and claim ceiling;
- timestamp, trust evaluation and verifier receipt.

The certificate should report one of these explicit outcomes: revalidation_required, revalidated_within_declared_scope, stale_or_expired, revoked, unknown, incomplete_or_indeterminate, or invalid_input. Do not compress those outcomes into one risk score.

## 6. Safe NoImpact semantics

An empty impact frontier is not sufficient to conclude NoImpact.

A bounded NoImpact claim requires all of the following:

- valid dependency graph and successful graph integrity validation;
- complete inventory within the declared source query scope;
- all expected source owners for the declared reliance scope accounted for;
- no unresolved or unsupported mandatory relationship family;
- current source watermarks and tombstones;
- no applicable late-arriving correction, revocation or supersession event;
- freshness policy satisfied for every relied-upon evidence class;
- algorithm version qualified against the independent full-scan oracle;
- exact decision scope and source snapshot vector bound into the certificate.

If any prerequisite is absent, the result is NoImpactNotEstablished / incomplete_or_indeterminate, not NoImpact. A correct traversal over an incomplete graph can still produce a false negative.

## 7. Benchmark protocol

### P0 correctness gates

| Metric | P0 target |
|---|---:|
| False-negative invalidation versus independent full-scan oracle | **0** |
| Stale/revoked decision presented as current | **0** |
| Historical decision overwritten or silently rewritten | **0** |
| Late-arriving event with earlier valid time silently ignored | **0** |
| Revoked trust key accepted for new reliance | **0** |
| Rights-restricted evidence reused for a forbidden purpose | **0** |
| Incomplete inventory or graph used to assert NoImpact | **0** |
| Deterministic replay of a frozen decision/evidence vector | **100%** |

Run at least 1,000 deterministic mutations per event family in CI and a larger held-out mutation corpus for release qualification.

### Required negative controls

- source event arrives after decision but has a valid time before decision;
- source transaction time is later than source valid time;
- clock skew exceeds the declared bound;
- key revoked before decision, after decision, and with retroactive compromise time;
- rights revoked for one purpose but not another;
- event stream gap, cursor discontinuity or missing tombstone;
- connector/schema change that maps one native identity into a different field;
- solver build found defective after approval;
- policy superseded with and without an explicit transition/grandfathering rule;
- dependency graph is cyclic, stale, incomplete or missing a source family;
- two owners provide conflicting payload digests for one owner-native identity;
- a prior decision has been superseded by a newer decision;
- revalidation frontier is compared against an independently authored full-scan oracle.

### Performance and operational metrics

Measure invalidation latency, event-to-decision-staleness detection latency, revalidation compute saved relative to full re-run, false-positive revalidation rate, replay cost, certificate storage overhead, change-event backlog age, recovery after event-stream outage, and operator time to resolve an unknown/incomplete state.

Efficiency is reported only after zero false-negative and zero false-NoImpact gates pass.

## 8. Industry comparisons

The problem is not new in isolation: engineering platforms already expose incremental synchronization, source changesets, review workflows and operational security. The gap to evaluate is whether an independent system can qualify downstream reliance across multiple owners and tools.

- Bentley iTwin exposes synchronization changesets, changed-element review, webhooks and engineering validation. Test whether its exported changes and source snapshots can drive a portable, independently replayable reliance certificate.
- Palantir Ontology integrates operational data, logic, actions and dynamic security. Test whether decision evidence and invalidation can be independently replayed outside a centralized operational application.
- NIST IR 8536 provides cross-sector, temporally ordered traceability links and selective disclosure. Extend the benchmark from pedigree linkage to the validity of downstream engineering decisions under later corrections, revocations and source gaps.
- ISO 23247-6:2026 defines integrated, unified and federated digital-twin composition. The decision-freshness contract should be transportable across those composition patterns, not coupled to one centralized twin.

## 9. Implementation sequence

1. Implement immutable change-event types and canonical as-of vectors.
2. Bind source inventory, descriptor, graph and trust/rights receipts into a decision evidence vector.
3. Add freshness policy evaluation with explicit time semantics and source-specific watermarks.
4. Emit content-addressed revalidation certificates.
5. Compare selective frontier results against an independent full-scan oracle.
6. Add event-stream gap detection, tombstones, revocation and late-arrival mutation suites.
7. Only then allow bounded NoImpact claims for scopes with qualified inventory closure.

**Non-goal:** the system does not decide whether an engineering design is safe merely because its data is current. Engineering correctness, professional review, legal authority and physical validation remain separate claims.
