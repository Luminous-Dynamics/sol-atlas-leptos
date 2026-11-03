# Built-environment dependency coverage v0.1

**Status:** Strict manifest contract and synthetic fixture. No real-world dependency-completeness result is claimed.
**Parent:** [BE-COVERAGE-001 / #488](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488).
**Composes with:** built-environment graph [PR #138](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/138), frontier benchmark [#484](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/484), IEAC bridge [#486](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/486), native identity [PR #487](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/487), and temporal lifecycle [PR #397](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/397).

## 1. Why this contract exists

A dependency traversal can be perfectly correct over the graph it receives and still miss a dependency that was never extracted or represented.

Three distinct statements must not be conflated:

1. **Graph validity:** the supplied nodes, edges, digests and references are structurally consistent.
2. **Traversal correctness:** the computed impact set agrees with an independent oracle over that graph.
3. **Dependency coverage:** the declared source universe and mandatory relationship families for a stated scope have actually been inventoried and reconciled.

Even all three do not prove that a real project has no unknown physical, legal, geotechnical, structural or operational dependencies. Real-world adequacy requires qualified domain review and the responsible source owners.

The central rule is:

> A graph digest proves which graph was evaluated; it does not prove that the graph contains every dependency that should have been evaluated.

## 2. Result semantics

A consumer must keep structural validity, traversal result and coverage disposition separate.

| Graph / traversal | Coverage disposition | Permitted interpretation |
|---|---|---|
| Valid / exact oracle agreement | `complete_for_declared_scope` | `NoImpact` or `RevalidationRequired`, limited to the declared coverage scope |
| Valid / exact oracle agreement | `incomplete_or_indeterminate` | Never claim `NoImpact`; report the missing/unknown coverage |
| Invalid / inconsistent | Any | `GraphInvalid`; no partial impact claim |

An empty result from an unavailable, inaccessible, unsupported, unsearched or stale source is not evidence of absence.

Likewise, a “no open hold / no recall / no permit conflict” assertion is admissible only when it binds:
- the authoritative source and exact snapshot;
- the query and bounded subject/domain/time scope;
- complete enumeration and pagination;
- the source's completeness evidence;
- a validity interval and review basis.

The manifest's `disposition` is a declared input. A runtime verifier must independently derive whether the manifest is internally consistent with its sources, family coverage, edge provenance, counts, rights, time bounds and review evidence. It must not trust the declared disposition by itself.

## 3. Manifest structure

The Draft 2020-12 schema is `schemas/built-environment-dependency-coverage-v0.1.schema.json`.

### Case and profile scope

`case_scope` fixes the project, design revisions, jurisdiction, intended review purpose, valid time and spatial boundary. `coverage_profile` lists mandatory dependency families and explicit exclusions, together with an exact applicability-basis digest.

A profile is a bounded declaration, not a universal list of dependencies for every building or infrastructure project. Applicability is project-, discipline-, jurisdiction- and decision-specific.

### Source inventory

Every source entry pins:
- owner namespace and source-native snapshot identity;
- exact snapshot and schema digests/version;
- capture and retrieval times;
- access and rights states;
- allowed purposes;
- adapter identity/version/build, configuration and exact query digests;
- source enumeration method, boundary, expected/returned counts, pagination state and completeness-proof digest.

Unknown rights, restricted access, unsupported schema, source outage and unsearched records remain visible. They cannot be converted into a complete inventory by an adapter's local success status.

### Relationship coverage

Each mandatory dependency family declares:
- contributing source IDs;
- expected and observed relationship counts where available;
- unresolved and unsupported counts;
- exact mapping-rule and coverage-evidence digests;
- an explicit status: complete, partial, unavailable, unsupported, not searched or out of scope.

Runtime must check that every required family in the profile appears exactly once, that every referenced source exists, and that counts and proof artifacts reconcile. JSON Schema alone cannot compare two arrays or authenticate the completeness proof. The manifest's inventory/profile/proof SHA-256 fields refer to exact source-owned or separately retained evidence artifact bytes; they are not hashes of a mutable display projection. The v0.1 schema deliberately does not choose a new signed-payload canonicalization format: any canonical serialization used for signatures or cross-language canonical digests must follow [IEAC-TRUST-001 / #347](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347).

### Edge provenance

Every generated dependency edge carries its originating source relationship ID, source snapshot, extractor build and mapping-rule digest. Exact native relationships, deterministic mappings, ambiguous mappings, unsupported relationships and manually asserted edges are distinct.

Ambiguous or unsupported mandatory relationships make coverage incomplete. A manually asserted relationship requires separate review; it must not be presented as an exact native source edge.

### Independent review

The review record binds the declared scope and evidence. A complete-for-scope result requires a reviewer decision of `accepted_for_declared_scope`. The reviewer must independently inspect the inventory boundary and relevant exclusions—not merely re-run the candidate extractor.

The current fixture's reviewer and all signatures/digests are synthetic placeholders. They do not establish a real review, identity, signature or source authority.

## 4. Initial mandatory dependency families

The profile vocabulary is intentionally broader than the first synthetic fixture:

- artifact lineage and revisions;
- source evidence and measurements;
- solver inputs and assumption evidence;
- decision authority;
- requirement/specification/code/profile revisions;
- material lot, serial and custody;
- geometry, datum and survey control;
- utility/network interfaces;
- permit and external constraints;
- verifier, policy, schema and trust-bundle revisions.

The first fixture covers only six families. It explicitly excludes material custody, survey control, utilities, permits, and verifier/policy/trust dependencies. The fixture must therefore never be cited as comprehensive coverage of a real construction release.

## 5. Frozen benchmark plan

Create a gold dependency inventory independently of the production extractor and the frontier traversal. The gold inventory must not call production mapping helpers.

### Positive cases

- Fully represented linear/branching artifact lineage.
- Multiple exact solver inputs and assumption-evidence edges.
- Decisions consuming multiple receipts and exact authority evidence.
- Correctly bounded source export with complete pagination and matching source counts.
- Proven absence with a bounded complete source snapshot and exact query digest.
- A truly disconnected control with complete source coverage.

### Negative mutations

Delete one mandatory edge at a time while leaving all remaining graph digests internally valid:
- artifact parent;
- source evidence;
- solver input;
- assumption evidence;
- decision authority;
- governing specification;
- material lot/custody;
- survey datum/control;
- utility interface;
- verifier/policy/trust dependency.

Also mutate:
- source inventory omitted or duplicate;
- one page/partition absent;
- stale source snapshot or stale retrieval;
- source unavailable, access denied, unsupported schema or not searched;
- unknown/restricted rights;
- wrong adapter/build/config/query digest;
- expected/observed count mismatch;
- unresolved/ambiguous edge reported as complete;
- relationship family absent, duplicated or incorrectly marked optional;
- reviewer scope digest mismatch or disputed review;
- “no open hold” query with partial pagination, unknown source universe or expired validity;
- equal payload digest but distinct native identity (compose with #486 / #487);
- a newly introduced dependency family not supported by the selected profile.

Each omission must produce an incomplete/indeterminate coverage result or be detected by the independent gold comparison. A valid graph digest must not make a missing edge look safe.

## 6. Metrics and proposed gates

P0 targets:
- 100% recall of mandatory gold dependencies in the declared supported profile.
- Zero `NoImpact` claims when coverage is partial, unavailable, unknown, unsupported, unsearched or invalid.
- 100% of omitted mandatory edge mutations detected or downgraded to incomplete/indeterminate.
- Zero empty-result/absence assertions accepted without a bounded, complete, fresh source enumeration proof.
- 100% edge provenance binding to source snapshots, extractor builds and mapping rules.
- 100% deterministic output under input ordering permutations.

P1 reporting:
- coverage precision/recall by dependency family, source system and extraction method;
- mandatory-family coverage and unsupported-family rate;
- source inventory completeness, extraction failure and timeout rates;
- expected/observed/unresolved/unsupported edge counts;
- reviewer disagreement and resolution time;
- extraction and evaluation p50/p95/p99, peak memory and graph expansion;
- held-out source-system combinations and project families.

P2:
- rights-cleared, de-identified source exports from at least two independent source-system families;
- reviewer-curated gold inventories on held-out projects;
- cross-vendor comparisons only where equivalent source relationships and export scope can be represented.

No universal speed-up threshold should be set before baseline runs. Coverage recall and unsupported coverage are hard gates, not a performance trade-off.

## 7. Industry comparison

Use the same source snapshot and declared decision scope where access and licensing allow. Compare:
- native PLM/BIM/asset graph dependency exports;
- information-container and relationship logs;
- source connector/inventory reports;
- independent gold dependencies and their change impact.

An inaccessible vendor relation is `unassessed`, not absent. A checker that exports fewer dependency families should not be rewarded for a smaller graph.

The relevant performance measure is **correctly supported impact conclusions per declared scope**, with unknown coverage reported explicitly—not merely the speed of graph traversal.

## 8. Nonclaims

This contract does not prove that every physical, legal, geotechnical, structural, fire, environmental or operational dependency has been modeled. It does not authenticate source owner statements, certify data rights, validate solver correctness, or authorize construction, operation or actuation. Passing the synthetic fixture proves only that the fixture conforms to the declared schema.
