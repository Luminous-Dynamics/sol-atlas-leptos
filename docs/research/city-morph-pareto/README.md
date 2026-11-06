# CityMorph Pareto corpus v0.1

This is a 24-case synthetic known-answer corpus for hard constraints, interval uncertainty and Pareto dominance. It contains no actual city data or preferred urban form.

## Semantics

- Every comparison has a shared evaluation-scope digest.
- Every objective has a unit, direction and definition digest.
- Hard constraints are non-compensable. A known violation makes that alternative Infeasible even if its objective values are excellent.
- Missing, stale, revoked or unknown required evidence makes an alternative Unknown.
- A minimization objective robustly compares A no-worse-than B only when A.upper <= B.lower.
- A maximization objective robustly compares A no-worse-than B only when A.lower >= B.upper.
- Robust dominance requires no-worse on every objective and strict separation on at least one.
- Overlapping intervals never establish dominance.
- Exact ties and cross-cutting trade-offs remain on the frontier.
- Only fully evidenced Feasible alternatives participate in dominance.
- No weighted score or ranked "best to worst" list is produced.

The reference oracle must recompute each case from inputs and must not read the expected output when making decisions. The expected fields are frozen known answers used only by tests and the receipt CLI.

## Claim ceiling

A Pareto frontier is non-dominated only for the metrics, constraints, evidence and evaluation scope that are explicitly represented. It does not establish community consensus, represent every stakeholder value or replace legal/professional planning decisions.
