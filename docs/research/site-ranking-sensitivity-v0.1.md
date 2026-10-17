# Scenario-Based Ranking Sensitivity for Site and Design Selection

Status: implementation note v0.1. This measures rank stability across an explicit set of preference scenarios; it does not estimate the probability that a stakeholder will choose a scenario.

## Why this is a paradigm shift

A weighted score can conceal how much a site ranking depends on the chosen weights. In a civic or monumental project, access, carbon, ecology, community benefit, logistics and resilience can legitimately point toward different options. A single score should not disguise those value choices as objective truth.

This module takes a **scenario-based sensitivity** approach: stakeholders and planners declare plausible preference regimes up front, the system evaluates candidates under each one, and the result shows winners, rank ranges, ties, pairwise ranking reversals and candidates that cannot be ranked due to missing evidence.

This complements robust Pareto analysis:
- Pareto analysis retains candidates not demonstrably dominated across objectives.
- Scenario sensitivity shows what happens when explicit stakeholder preference weights change.
- Neither automatically determines legal/site eligibility or stakeholder legitimacy.

A spatial multi-criteria decision-analysis study published in 2026 reports that suitability results may be sensitive to criterion weights and interaction/non-additivity parameters, with localized sensitivity that can be hidden by broad averages: https://doi.org/10.1016/j.envsoft.2026.107137. This supports measuring sensitivity, not using its results as evidence about any one project.

## Input contract

- Objective IDs are explicit and fixed across all scenarios.
- Candidate scores are benefit-oriented and normalized to [0, 1] before entering this module. A minimizing metric such as carbon must be transformed by a documented upstream rule.
- Every scenario specifies a finite, non-negative weight for every declared objective and must contain at least one positive weight.
- A candidate only needs usable evidence for objectives with positive weights in that scenario. Missing active scores produce NeedsEvidence; an upstream excluded candidate never ranks.
- A score tolerance groups near-equal values into shared ranks. The tolerance is absolute on the normalized [0, 1] scale.
- Scenarios are descriptive preference regimes, not probabilistic samples. The fraction of scenarios in which a candidate ranks first is a coverage statistic, not a prediction of human choices.

Weights are scaled by their maximum before score accumulation to avoid overflow when users provide unusually large but finite importance weights.

## Outputs

For each scenario, the report provides:
- a candidate status of Ranked, NeedsEvidence, or Excluded;
- the weighted score when rankable;
- a deterministic rank with shared ranks for scores within tolerance;
- the full set of rank-1 candidates, preserving ties.

For each candidate, summary outputs include total and ranked scenario counts, held/excluded counts, number and share of scenarios where it ranks first, rank bounds, mean rank, and mean weighted score.

For each candidate pair, outputs include how many comparable scenarios prefer A, prefer B, or tie, and whether the ordering reverses across scenarios. A robust unique winner is reported only when the same sole winner appears in every declared scenario.

## Synthetic benchmark

The fixture at docs/research/fixtures/site-ranking-sensitivity-v0.1.jsonl covers six cases:
1. Access-, carbon- and community-first scenarios produce different winners.
2. A unique candidate wins across all supplied preference regimes.
3. Equal-score candidates remain tied.
4. A candidate excluded by upstream hard constraints never re-enters the rankings.
5. A candidate can be ranked in a scenario that assigns zero weight to its missing criterion, but is held when that criterion becomes active.
6. A declared score tolerance prevents a false single-winner claim for near-ties.

An independent mirror over the fetched-back fixture should be used alongside the authored Rust tests; the Rust tests are not considered passing until compiled and run on the exact PR head.

## Proposed promotion gates

1. At least 30 scenario fixtures with diverse criteria, weights, score ranges, ties, rank reversals and missing-evidence cases.
2. Input-order invariance and deterministic output on pinned inputs.
3. Explicit documented weight provenance; no hidden default weights or inferred preference probabilities.
4. Sensitivity to score uncertainty as well as weights, once the score model supplies defensible intervals.
5. Global sensitivity analysis (e.g., Sobol indices) only when the inputs and probability/distribution assumptions are justified; distinguish screening from probabilistic inference.
6. Report whether each site remains feasible independently of weighted score, and never allow a preference weight to compensate for a verified hard-constraint failure.
7. Validate any real candidate scores against source-quality, GIS, professional-engineering and community-process evidence. These benchmarks do not certify a site.

## Relationship to the four systems

- Sol-Atlas hosts the evidence-backed candidate catalogue, scenario explorer and ranking-change visualization.
- Symthaea can help elicit criteria, explain trade-offs, propose scenarios and identify missing evidence—but must not invent stakeholder weights or claim consensus.
- Symtropy and validated specialist solvers supply suitable simulated performance indicators, clearly separating exploratory simulation from verified engineering results.
- Mycelix records who selected each policy/scenario, the exact inputs and revisions, dissent/conditions, and the approval trail.

## References

- Spatial MCDA uncertainty and sensitivity research (2026), *Environmental Modelling & Software*, 205, 107137: https://doi.org/10.1016/j.envsoft.2026.107137
- OGC API - Features: https://www.ogc.org/standards/ogcapi-features/
- NIST, Building Digitization and Semantic Interoperability: https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability

This module is a transparent ranking diagnostic, not a safety check, legal decision, public-consent measure or substitute for professional review.
