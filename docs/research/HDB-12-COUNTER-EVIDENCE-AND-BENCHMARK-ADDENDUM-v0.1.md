# HDB-12 counter-evidence and benchmark addendum v0.1

## Why this increment exists

A reconstruction must not erase evidence merely because it does not support the selected geometry. The original v0.1 contract correctly required positive source evidence to have a `supports` relation. It did not yet provide an element-level place to retain contradictory or contextual records without either promoting them to support or omitting them from the dependency graph.

## Association semantics

Each geometry element now carries three disjoint evidence associations:

- `source_evidence_ids`: records marked `supports`; only this set can satisfy positive empirical support and direct-measurement requirements.
- `counter_evidence_ids`: records marked `contradicts`; these are preserved as counter-evidence and never count as positive support.
- `context_evidence_ids`: records marked `context_only`; they inform interpretation but do not establish or contradict the element's geometry claim.

An evidence ID may occur in only one association class for a given element. Every associated evidence record must resolve, and every associated ID must appear exactly once in the element's source-evidence dependency set. The association class must match the evidence record's declared relation.

Rights state remains explicit. A rights-pending or rights-unknown counter-record may be retained as a reference, but is not thereby cleared for content retrieval, public disclosure, redistribution or model training. Verified rights still require a receipt. Positive support retains the existing verified-rights requirement.

The committed synthetic known-answer input now demonstrates all three associations on one observed geometry element: one direct measurement supports it, one rights-pending field assessment contradicts it, and one rights-verified historical document provides context. Its expected report records each association category separately. A dedicated Rust regression also confirms that a model prediction can be retained as counter-evidence without being promoted to measurement or positive support.

## Benchmark extension

Add the following matched evaluation tracks to HDB-12:

1. **Counter-evidence retention:** recall and precision of independently adjudicated contradictory records, measured per claim and per site.
2. **Relation integrity:** fraction of support / contradict / context-only label mutations rejected; target 100% on the registered adversarial corpus.
3. **Conflict visibility:** fraction of material contradictions surfaced to reviewers rather than silently dropped or merged into a scalar confidence score; target 100% for in-scope registered cases.
4. **Non-promotion:** zero counter-evidence or context-only records satisfying positive support or observed-geometry admission.
5. **Dependency completeness:** 100% of attached support, counter and context records resolve and are digest/lineage bound; no unreferenced association or dependency-set mismatch.
6. **Rights non-escalation:** zero denied, pending or unknown records treated as permission to fetch, publish, redistribute or train.
7. **Reviewer utility:** time to identify material contradictions and inter-reviewer agreement, reported separately from model accuracy.

Use the same source packages and held-out sites for each baseline. Compare omission-based pipelines, support-only claim graphs, contradiction-aware evidence graphs and expert-curated dossiers. Do not allow a higher geometric score to compensate for lost contradictions.

## Research position and claim ceiling

This is a structural interchange and validation increment. It does not determine which interpretation is correct, weigh evidence, resolve archaeological disputes, authenticate rights receipts, establish historical truth or make a cultural authority decision. Material contradictions should trigger review or preserve competing hypotheses; they should not automatically veto a reconstruction.

## Negative controls

- A `supports` record listed as counter-evidence is rejected.
- A `contradicts` record listed as positive support cannot satisfy empirical support.
- A record cannot appear in both support and counter/context associations for one element.
- An attached counter/context record without a matching source dependency fails.
- A rights-pending contradiction remains retained but is not promoted to licensed source material.
- A model output remains a model output even when it contradicts an element.

## Report visibility

The structural report exposes separate counts for support, counter-evidence and contextual evidence associations, plus the number of elements carrying counter-evidence or contextual records. These are association counts, not unique evidence-record counts: one evidence record may be related to multiple elements. This makes conflict presence visible to downstream reviewers without collapsing it into a confidence score or deciding which interpretation is correct.
