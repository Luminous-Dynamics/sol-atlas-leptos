# CityMaker owner impact lane router v0.1

**Status:** synthetic-first routing registry and conformance harness; exact-head qualification pending  
**Issue:** [CITY-IMPACT-ROUTER-001 #649](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/649)  
**IEAC schema blocker:** [IEAC-INVALIDATION-V2-001 #650](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/650)  
**Parent projection:** [CITY-CHANGE-IMPACT-001 #643 / PR #644](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/644)  
**Branch:** `research/city-impact-router-v0.1`

## Purpose

Route each CityMaker change event to the correct specialist owner impact family. This prevents a structurally valid report from one domain being substituted for another domain's authority.

This crate is a registry and contract-binding adapter only. It does not compute impact, traverse a graph, verify report bytes, authenticate owner signatures, qualify rights, or authorize work.

## Owner capability matrix

The registry records the status visible in the pinned upstream work on 8 November 2026:

| Lane | Current contract readiness | Owner |
|---|---|---|
| `artifact_revision` | Typed Rust function pinned to open PR #138 / benchmark #484; no standalone report schema yet | Sol-Atlas |
| `dependency_coverage` | Versioned schema v0.1 and Rust auditor pinned to open PRs #489/#490 | Sol-Atlas |
| `source_evidence_lifecycle` | **Schema-incompatible**: current event/report v0.1 use digest-based node IDs and omit v0.2 graph node types | Sol-Atlas #396/#397/#487; blocked by #650 |
| `source_inventory` | Design/issue only; no versioned report contract registered here | Sol-Atlas #497 |
| `supply_chain_provenance` | Owner contract not versioned in this adapter | Mycelix #4814/#5149 |
| `engineering_qualification` | Benchmark/design owner work; no CityMaker report schema pinned | Symthaea #10543/#6117 |
| `policy_applicability` | No policy-owner contract registered; #501 is decision reliance, not policy authority | Policy owner required |
| `decision_reliance` | Owner concept exists, but not yet pinned as a CityMaker adapter contract | Sol-Atlas #501 |
| `asset_operation` | Service/outcome/inspection owners exist; adapter contract not versioned here | Sol-Atlas #574/#633; Symtropy #4043 |

Only the first two lanes can currently be contract-bound in this synthetic router, and even then the result is explicitly **contract-bound, unverified**. The artifact frontier is an open-PR typed function rather than a signed/serialized impact-report contract; the dependency-coverage receipt is an open-PR schema/runtime. The router does not authenticate either.

## Critical upstream mismatch: IEAC v0.1 events versus graph v0.2

The v0.2 dependency graph uses native identity node IDs of the form:

`<node_type>:identity-sha256:<identity_sha256>`

and includes `source_evidence`, `engineering_artifact` and `engineering_receipt`.

The current invalidation event and impact report v0.1 schemas still use:

`<node_type>:sha256:<artifact_sha256>`

and their node type enums omit those v0.2 built-environment node types. A source-evidence invalidation cannot safely be targeted at a v0.2 `source_evidence` graph node through the v0.1 event/report schema. Do not alias it to `source_measurement` or substitute a content digest for native identity. #650 tracks the additive v0.2 event/report schema and runtime bridge. Until then, the router marks the source lifecycle lane incompatible.

## Routing policy

The registry maps each change class to one or more required families. A single event may require multiple owners. CityMaker joins their report envelopes only when event digest, subject/revision, scope, functional unit and evaluation time agree exactly.

A request for a `not_affected`/NoImpact claim dynamically requires both `source_inventory` and `dependency_coverage`, in addition to the relevant impact family. A descriptor list alone is not source-universe completeness, and a complete traversal is not proof that the dependency graph contains every required edge.

Unsupported or unversioned lanes return `insufficient_evidence`. A wrong or contradictory binding returns `invalid_input`. A complete set of metadata bindings returns only `contract_bound_unverified`; the router never says the owner report contents were authenticated or the impact algorithm was correct.

## Corpus

`CITY_IMPACT_LANE_ROUTER_CORPUS_V0_1.json` contains 35 synthetic known-answer cases:
- correct artifact-frontier plus dependency-coverage envelope;
- wrong-family substitution and missing required lanes;
- source lifecycle v0.1 versus graph v0.2 incompatibility;
- supply-chain, calibration, policy, decision and asset-operation lanes that remain unsupported until exact contracts are pinned;
- event, subject, scope, contract, schema and report-digest mutations;
- incomplete coverage, no-impact source-inventory requirements, duplicate reports and action-authority escalation.

The gold expected lane labels are written into the frozen corpus, independently of the Rust router implementation. These are routing fixtures, not real project evidence.

## Run

From the repository root:

    cargo fmt --manifest-path tools/city-impact-lane-router/Cargo.toml --all -- --check
    cargo test --locked --manifest-path tools/city-impact-lane-router/Cargo.toml
    cargo clippy --locked --manifest-path tools/city-impact-lane-router/Cargo.toml --all-targets -- -D warnings
    cargo run --locked --quiet --manifest-path tools/city-impact-lane-router/Cargo.toml -- \
      docs/research/city-impact-router/CITY_IMPACT_OWNER_LANE_REGISTRY_V0_1.json \
      docs/research/city-impact-router/CITY_IMPACT_LANE_ROUTER_CORPUS_V0_1.json \
      --source-commit <40-lowercase-hex-sha>

The CLI receipt binds the exact registry bytes, corpus bytes and source commit. A PASS means only that the router reproduces the 32 synthetic routing expectations.

## Benchmark targets

- zero wrong-lane false acceptance;
- zero owner/contract/schema/event/subject/scope binding false acceptance;
- zero source-evidence route through incompatible v0.1 event/report schemas;
- zero no-impact route without source-inventory and dependency-coverage lanes;
- zero optimistic fallback for design-only or unversioned lanes;
- exact lane-set and multi-owner join agreement;
- transparent Unknown/Insufficient rates and causes;
- latency and review burden by owner lane.

Do not optimize by reducing the insufficiency rate until the upstream owner contracts actually exist and their exact report bindings are independently verified.

## Claim boundary

The router only checks a declared registry and owner-report envelope. It does not verify the report bytes referenced by a digest, authenticate an owner's identity or authority, recompute impact, prove inventory completeness, certify engineering or issue approvals. Every receipt states `owner_report_content_verified=false`, `owner_authentication=not_performed`, and `action_authority=none`.


## Bitemporal event binding

Every change event now carries separate `valid_time` and `system_time` envelopes, and every owner report must bind to the exact same envelopes:

- `valid_time.effective_at` / `effective_until` and `effective_time_confidence`;
- `system_time.recorded_at`, `trusted_at` and `trusted_timestamp_receipt_sha256`;
- the query's `evaluation_at`.

The event declares `time_basis=UTC_RFC3339_SECONDS`; the router requires canonical UTC RFC 3339 whole-second timestamps. Unknown or disputed effective-time confidence yields Insufficient; a trusted timestamp before the recorded event is invalid; a report with a different temporal envelope is rejected. This is binding and ordering validation only. The router does not authenticate the trusted timestamp receipt or determine the truth of the asserted effective time.

The corpus adds four temporal adversarial controls: missing recorded time, disputed effective-time confidence, reversed recorded/trusted time, and one owner report using a different system-time envelope.
