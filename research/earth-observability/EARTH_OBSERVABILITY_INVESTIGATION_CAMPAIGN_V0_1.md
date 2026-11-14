# EARTH-OBSERVABILITY-INVESTIGATION-CAMPAIGN-001
## Project-macro tournament aggregation and worst-domain reporting v0.1

**Implementation:** `sol-atlas-core/src/investigation_campaign.rs`  
**CLI:** `investigation_campaign_eval`  
**Schema:** `INVESTIGATION_CAMPAIGN_V0_1.schema.json`  
**Fixture:** `fixtures/investigation-campaign-synthetic-v0.1.json`  
**Pinned baseline:** `baselines/investigation-campaign-baseline-v0.1.json`  
**Parent:** Sol-Atlas #771 / investigation tournament draft #793  
**Status:** Rust campaign scorer, fixture, schema, CLI and pinned baseline authored; exact-head CI pending  
**Claim ceiling:** Three synthetic placeholder project cases. No UT-Gold-D project IDs or physical outcomes have been ingested.

---

## 1. Why project-level aggregation is necessary

A benchmark with 13 construction projects must not report one pooled score over 959 radargrams and imply 959 independent replications. Observations from the same project share site, geology, acquisition equipment, practices, access constraints and often common coordinate/processing error.

The campaign layer therefore treats each project case as one equal-weight evaluation unit. It preserves individual project outcomes and reports both:
- unweighted macro-average loss and regret across projects;
- worst-project loss and regret, with the project identity retained.

It also preserves the full per-project result rows. A macro-average is not allowed to conceal a failing project.

## 2. Campaign invariants

Every campaign requires:
- one declared dataset snapshot;
- one common currency, direct-cost budget, crew-hour budget and maximum disturbance class;
- at least two distinct project IDs and evaluation IDs;
- identical policy IDs and policy kinds across every project case;
- one valid action catalogue, decision-loss table and frozen policy output per project.

The evaluator rejects budget or dataset-snapshot drift, duplicate projects/evaluations, policy-set drift, and invalid project cases.

The action catalogue and authority gates remain checked by the single-case evaluator. Campaign aggregation does not bypass those checks.

## 3. Per-project metrics and aggregate outputs

For each policy/project pair, the receipt retains:
- realized decision loss and regret;
- direct spend and crew-hours;
- planner-supplied information-gain diagnostic.

For each policy, it reports:
- project count;
- unweighted project macro-average loss and regret;
- worst-project loss and regret plus the specific project IDs;
- average and maximum spend;
- average and maximum crew-hours;
- mean information gain only if every project supplied a value.

No trace-count or activity-count weights are used. Policy metrics are sorted by policy identity; per-project result rows are sorted by policy and project identity. Worst-project ties are broken lexicographically so receipts remain deterministic.

The campaign does not claim that the projects are statistically independent samples from every future deployment environment. With UT-Gold-D's 13 clusters, report all 13 project outcomes, macro-average, between-project dispersion, worst case and the small-cluster limitation.

## 4. Synthetic example

The fixture contains three abstract project cases and five common policies. The per-project loss assigned to the entropy-greedy policy differs by project (100, 80, 60), yielding a macro-average of 80 and a worst-project loss of 100 on project-a. The decision-aware policy has losses 10, 15 and 20, yielding a macro-average of 15 and worst-project loss 20 on project-c.

The expert and cost-only policies remain separate comparators, and the hidden-truth oracle remains a non-deployable upper bound. These fixture values exercise arithmetic and reporting; they are not measured project performance.

## 5. Why this is useful for a physical 13-project tournament

Once UT-Gold-D source admission and project identity linkage are verified, each leave-project-out fold can supply one project case. The campaign should only aggregate policies after the same policy versions, action eligibility rules, cost boundary and truth-adjudication process are applied consistently.

For each blind project:
1. freeze its evidence snapshot and allowed action catalogue;
2. freeze policy versions and submitted plans before truth reveal;
3. retain independent adjudication and loss provenance;
4. score each policy under the common resource/authority constraints;
5. store the per-project receipt and input digest;
6. aggregate only after every planned project has been scored under the same protocol.

A project used in training or model selection must not remain a blind evaluation project for a later claim. The actual UT-Gold-D project IDs are not represented by the synthetic fixture IDs.

## 6. Nonclaims

The campaign aggregator does not:
- establish independent sampling or statistical power;
- calculate confidence intervals or hierarchical uncertainty;
- compute EVSI/ENBS;
- authenticate truth, policy blindness or authority;
- establish physical sensor performance or utility-network correctness.

It is an aggregation and reporting layer that prevents project outcomes from disappearing into one pooled event-level metric. Statistical inference and physical qualification remain separate.
