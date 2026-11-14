# EARTH-OBSERVABILITY-EVIDENCE-REVALIDATION-001
## Dependency closure, invalidation and controlled re-promotion v0.1

**Implementation:** `sol-atlas-core/src/evidence_revalidation.rs`  
**CLI:** `evidence_revalidation_eval`  
**Graph schema:** `EVIDENCE_DEPENDENCY_GRAPH_V0_1.schema.json`  
**Synthetic graph:** `fixtures/evidence-dependency-graph-synthetic-v0.1.json`  
**Pinned invalidation plan:** `baselines/revalidation-plan-source-change-synthetic-v0.1.json`  
**Parent:** Sol-Atlas #771; current implementation branch #779  
**Status:** Rust DAG validator, closure planner, fixture, schema, pinned baseline and SHA-256 receipt CLI authored; exact-head CI pending  
**Claim ceiling:** Structural dependency semantics only. No real asset, survey, sensor or owner-system update is represented.

---

## 1. Why evidence needs dependency-aware invalidation

A source correction can change many downstream claims without changing their stored bytes. For example:

`survey source -> coordinate transform -> georeferenced observation -> detected event -> candidate asset -> network hypothesis -> investigation decision -> partner export`

If the source survey, calibration, transform grid, sensor configuration or forward model changes, descendants cannot continue to be shown as current merely because their earlier receipts still exist.

ArcGIS Utility Network uses dirty areas to identify edits not yet reflected in its validated topology; downstream traces may be unexpected until validation clears those changes. Sol-Atlas should apply the same broad operational discipline to evidence claims, while using its own richer dependency graph and preserving the distinction between a candidate interpretation and an authoritative owner network.

The key rule is:

**A changed input makes dependent claims stale until each affected downstream operation has been re-executed and independently qualified.**

Historical receipts remain immutable. Invalidation does not erase the prior claim or pretend it was never produced.

## 2. Dependency DAG contract

The executable graph contains:
- immutable graph and snapshot identifiers;
- unique node IDs;
- a node kind (source evidence, spatial transform, sensor calibration, observation, event, asset hypothesis, geometry claim, network hypothesis, decision or export);
- an immutable revision identifier or digest;
- explicit direct upstream dependency IDs;
- a current/stale/superseded/conflicted/invalid status.

The graph must be acyclic, all dependencies must resolve, and duplicate/self dependencies are rejected.

The current module does not authenticate that the dependency list is complete. Owners of each transformation must declare all material inputs—including model/configuration, source revisions, coordinate transforms, calibration, environmental assumptions and policy versions. Missing edges are a provenance defect that structural graph validation cannot discover by itself.

## 3. Revalidation plan semantics

Given changed root node IDs:
1. Validate the graph and its DAG structure.
2. Verify each changed root exists.
3. Compute the union of direct and transitive descendants.
4. Keep changed roots separate from invalidated descendants.
5. Return a deterministic upstream-before-downstream revalidation order.
6. Return unaffected nodes so the scope of invalidation is inspectable.

The plan is a *work and invalidation declaration*. It is not a new validation receipt, does not mutate an evidence database and does not itself make descendants current.

After a source correction:
- the source owner must accept the new source revision;
- each dependent transform/observation/model must be re-executed;
- each dependent interpretation and candidate graph must be recomputed or explicitly withdrawn;
- each dependent decision and export must be re-evaluated;
- current status is restored only by new receipts bound to the new upstream revisions.

An unknown outcome must leave the affected descendants stale/blocked rather than silently reuse earlier approvals.

## 4. Synthetic fixture

The committed graph contains:
- one GPR survey source;
- one spatial transform dependent on that source;
- one independent sensor calibration;
- one observation dependent on both transform and calibration;
- event, candidate asset, network hypothesis, decision and export descendants;
- one unrelated source.

Changing the survey source invalidates seven descendants, in this order:

1. coordinate transform;
2. sensor observation;
3. detected event;
4. candidate utility asset;
5. joint network hypothesis;
6. investigation decision;
7. partner export.

The independent calibration and unrelated source remain unaffected.

Changing only the sensor calibration invalidates the observation and its descendants, but not the coordinate transform. This difference is intentional: invalidation follows declared dependency edges, not a global “everything changed” rule.

## 5. Adversarial controls

Rust tests cover:
- valid DAG read-back;
- transitive closure and topological order;
- selective invalidation from sensor calibration;
- multi-root closure deduplication;
- cycle rejection;
- self-dependency rejection;
- missing dependency and unknown changed-root rejection;
- stable plan under authored node-order permutation.

The CLI emits a receipt bound to the exact graph JSON bytes and compares a frozen synthetic source-change plan. The CI workflow runs the CLI tests and pinned baseline check.

## 6. Benchmark requirements

For physical data and real model runs, add evaluation of:
- completeness of declared dependency edges against a frozen transformation manifest;
- stale-descendant detection rate;
- false invalidation and missed invalidation rates;
- revalidation latency and compute cost;
- correct invalidation after source, calibration, transform-grid, model-weight, processing and policy changes;
- recovery correctness when a changed upstream revision is later rolled back or superseded;
- concurrent changes and unknown-outcome reconciliation;
- restricted exports that must be blocked when a supporting claim becomes stale.

Test common-cause cases as well as direct edges. For example, two separate sensor observations can share a hidden dependency on one coordinate epoch or one calibration certificate; both must become stale when that common input is revoked.

## 7. Connection to owner systems

The dependency graph is a Sol-Atlas evidence-layer control. It does not modify authoritative ArcGIS, Bentley, owner registries or as-built systems.

A later owner adapter may propose changes only after:
- the owner's canonical asset identity is resolved by an authorized process;
- the candidate geometry and attributes are independently accepted;
- all dependencies are current;
- data-use and disclosure grants permit the operation;
- the owner system itself validates the resulting network/topology.

The analogy to ArcGIS dirty areas is about disciplined downstream revalidation, not a claim that the Sol-Atlas graph is equivalent to Esri topology.

## 8. Nonclaims

The synthetic graph does not prove that all real dependencies have been captured, that a source is authentic, or that a dependent claim is physically correct. The plan does not run field surveys, update GIS registries, certify utility geometry or authorize excavation.

No real asset or sensitive infrastructure coordinate is included.
