# HDB-23B — exhaustive architectural membership sensitivity v0.1

## Purpose

HDB-23's first morphology report preserves ambiguous parentage but excludes ambiguous stones from assigned-only summaries. That is the correct baseline; however, it does not quantify how much an architectural group's descriptive signature could change under alternative candidate memberships.

HDB-23B adds a bounded, deterministic sensitivity evaluator. For one requested hierarchy level, it keeps explicit assignments fixed, keeps unassigned and not-assessed stones out of every group, and enumerates every combination of the supplied candidate group IDs for ambiguous stones.

This changes the output from a single assigned-only summary to a **set-valued sensitivity envelope**. It does not turn candidate assignments into probabilities.

## Exact semantics

For each ambiguous stone i, let C_i be its declared candidate group IDs. The evaluator enumerates the Cartesian product of those candidate sets and evaluates the group summaries under every complete assignment. Assigned stones remain fixed. Unassigned and not-assessed observations are never made eligible by the evaluator.

The report exposes, per possible group:

- minimum and maximum assigned stone counts, including zero when the group disappears under a scenario;
- number of scenarios in which the group is non-empty;
- mean, p10, median and p90 for plan-footprint area, MRR area ratio and elongation;
- an enclosing measurement interval across every enumerated assignment;
- separate ranges for the lower and upper summary endpoints, so assignment-driven movement is visible rather than being hidden in a single pooled interval.

Every candidate assignment is unweighted. Equal enumeration is a sensitivity design, **not** a claim that alternatives are equiprobable. The report's enclosing intervals are descriptive bounds, not confidence or credible intervals.

## Important v0.1 limitation

The current HDB-23 membership input contains candidate IDs, but does not prove that all cross-stone combinations are jointly compatible with HDB-17's architecture graph. HDB-23B therefore enumerates an **unconstrained Cartesian product**. Its envelopes are conservative outer stress bounds and may include combinations that a graph-aware solver would later rule out.

Do not interpret a scenario as a validated architectural reconstruction. A follow-on graph-constrained evaluator must consume source-bound HDB-17 membership/edge evidence, preserve competing graph hypotheses, and demonstrate that all constraints are independently validated.

## Bounded computation

The evaluator refuses to sample, truncate, or silently drop alternatives:

- at most 4,096 complete assignment scenarios;
- at most 100,000,000 preflight work units, where the preflight estimate accounts for stone dispatch/group lookup and a conservative six n·ceil(log2(n)) endpoint-sort budget across the three descriptors;
- checked multiplication for scenario and work estimates;
- deterministic candidate and physical-identity ordering.

Requests outside these bounds return typed errors. The caller must narrow the target analysis scope or explicitly review a later algorithm; it must not quietly weaken the ceiling.

## Executable qualification

The Rust tests cover:

1. exhaustive two-way assignment and endpoint envelopes;
2. input-row and candidate-order invariance;
3. refusal above the scenario cap;
4. refusal above the work budget before enumeration;
5. preservation of unassigned and not-assessed states;
6. one exact scenario when no ambiguity exists;
7. malformed input and empty-ID rejection.

The synthetic report fixture is validated against Draft 2020-12. The preflight budget is a conservative deterministic complexity proxy, not a wall-clock guarantee. These are software-contract tests, not archaeological field validation.

## Research and benchmark programme

The next scientific comparison should hold the measured stones fixed and compare:

- assigned-only baseline;
- unconstrained full Cartesian sensitivity (this v0.1 implementation);
- graph-constrained candidate assignments after HDB-17 evidence integration;
- expert-adjudicated reference assignments, with disagreement retained rather than collapsed.

Report per-group count range, descriptor endpoint movement, rank stability, group disappearance/creation, sensitivity width and expert adjudication effort. At higher levels, the independent evaluation unit remains the site/region/acquisition—not the stone count. HDB-20's cluster-bootstrap contract (#543) is required before attaching population confidence intervals.

A useful preregistered target is not “sensitivity width equals zero.” It is:

- 100% of ambiguous eligible stones are included in at least one enumerated alternative;
- 100% of candidate combinations are either exhaustively evaluated or rejected by the declared cap;
- 100% of group count and metric ranges reproduce for identical canonical inputs;
- 0 probabilistic claims from unweighted scenarios;
- 0 historical/social claims from morphology sensitivity alone;
- graph-constrained and expert-adjudicated sensitivity reported separately from the unconstrained outer bound.

## Paradigm shift

The objective is no longer to force the best single wall/household assignment and then report a precise-looking group metric. It is to report what remains stable across the admissible evidence alternatives, and to mark any conclusion whose direction or magnitude changes with membership as fragile.

This is a bridge from object detection toward **evidence-qualified architectural measurement**. The software is designed to expose the limits of the reconstruction, not manufacture certainty.
