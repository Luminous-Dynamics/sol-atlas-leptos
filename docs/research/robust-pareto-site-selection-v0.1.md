# Robust Pareto Analysis for Site and Design Alternatives

Status: implementation proposal v0.1. This is decision-support research, not statutory, environmental, or engineering certification.

## Paradigm shift: uncertainty-aware alternatives

A weighted sum forces unlike values (access, carbon, ecology, cost, culture, resilience) into one selected set of weights. It can be useful as a scenario-specific preference view, but it can hide rank instability and let a strong score compensate for a critical weakness. A Pareto set preserves alternatives that are not demonstrably worse across all selected objectives.

This implementation adds interval-aware robust dominance. Each metric carries a plausible lower and upper bound. Candidate A robustly dominates B only if A's worst plausible outcome is at least as good as B's best plausible outcome on every objective, and strictly better on at least one, within an explicit, criterion-specific tolerance. Overlapping uncertainty intervals do not justify eliminating a candidate. Incomplete intervals are held for evidence. Upstream excluded candidates can never re-enter the frontier.

This is deliberately conservative. It may retain more candidates than point-estimate Pareto analysis; that is preferable to false certainty during early site screening.

## Mathematical contract

Objective direction is explicit per criterion. A minimization objective is transformed by negating its interval, then the same robust-dominance rule is applied to benefit-oriented intervals.

For every objective i:

- A is no worse under robust comparison when lower_benefit(A_i) + epsilon >= upper_benefit(B_i).
- A is strictly better on at least one objective when lower_benefit(A_i) > upper_benefit(B_i) + epsilon.

If the first condition fails for any objective, A does not robustly dominate B. If no objective is strictly better, the candidates do not dominate one another. Tolerances are optional and criterion-specific, expressed in each objective's raw units; an omitted tolerance defaults to zero. Unit transformations still need pinned semantics and unit-scaling invariance tests before any real-world use.

## Disposition model

- Frontier: complete objective intervals, upstream gate eligible, and no robust dominator.
- RobustlyDominated: at least one complete, gate-eligible candidate robustly dominates the candidate; all dominator IDs are returned.
- NeedsEvidence: upstream gates are unresolved or one or more required objective intervals are incomplete.
- Excluded: the upstream hard-constraint evaluator excluded the candidate.

The Pareto module consumes gate dispositions. It does not reinterpret land rights, zoning, ecology, hazards, or engineering checks. A frontier result is not a site recommendation or approval.

## Synthetic benchmark

The fixture at fixtures/pareto-site-benchmark-v0.1.jsonl covers clear dominance, a real access/carbon trade-off, hard-gate exclusion, overlapping intervals, a missing interval, unresolved upstream gates, minimization direction, and equal-objective ties.

Promotion gates before using this method with real site data:

1. At least 30 reference cases covering mixed maximize/minimize objectives, ties, interval bounds, unit scales, tolerances, missing and conflicting evidence, duplicate identifiers, non-finite values, and schema mismatches.
2. Independently hand-calculated expected frontiers and a second implementation that does not share production code.
3. Property tests for input-order invariance, direction transformations, irreflexivity, and exclusion monotonicity.
4. Sensitivity analysis across criterion-specific scales and policy versions; no implicit global weights.
5. Report frontier size, rank reversals under perturbation, held-for-evidence candidates, and reasons.
6. Synthetic correctness must never be represented as validation of GIS coverage, source quality, policy legitimacy, law, or physical safety.

## Research and interoperability anchors

- OGC API - Features provides interoperable feature-level access to geospatial datasets: https://www.ogc.org/standards/ogcapi-features/
- Official IFC 4.3.2.0 (ISO 16739-1:2024): https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/index.html
- buildingSMART IDS defines machine-interpretable IFC information requirements: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- NIST building digitization and semantic interoperability work: https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability

These standards support data exchange across tools; they do not validate the selection policy or certify a site.
