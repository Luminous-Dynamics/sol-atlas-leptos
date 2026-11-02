# Stratified Predicted-Negative Audit v0.1

**Purpose:** estimate false-negative burden without treating unsurveyed, inaccessible, unknown or unobservable landscape as true negatives.

## Why this matters

Field & Dean (2026) reported a Northern San Juan CV deployment with precision 0.65, recall >0.99 and F1 0.79 on its validated image subset (n=10,197). Its broader deployment F1 estimate was 0.68 and depended on extrapolating validation rates to 764,639 unvalidated images; the authors explicitly caution that this extrapolation may mislead. The authors also report predicted-negative review and iterative human validation. This is a useful regional human-in-the-loop baseline, not evidence of global performance.

Reference: https://www.cambridge.org/core/journals/american-antiquity/article/computer-vision-survey-of-the-northern-san-juan/6BE98BA78AE0BFF7A688E579B669BB8F.

## Sampling frame

Stratify the full finite population before sampling. v0.1 supports high-score candidates, medium-score candidates, low-score negatives, predicted-negative landscape, model-abstained and data-unavailable strata. Each stratum declares population N_h, selected n_h, design reference and sampling design. Each sampled unit records its first-order inclusion probability pi_i and landscape/site cluster.

For stratified simple random sampling without replacement, the code checks pi_i = n_h / N_h. For unequal-probability designs, the actual pi_i for every sampled unit is required. Sample and unit IDs must be unique, and each stratum's actual selected count must match the declared sample count.

## Ground-truth semantics

- `confirmed_positive`: field-assessed or independently expert-reviewed, with adjudication, reviewer, authority and rights-receipt references.
- `confirmed_negative`: the same requirements plus a detectability assessment describing whether the survey could detect the declared target size/depth under actual conditions.
- `unknown`: unresolved; never counted as negative.
- `unobservable`: conditions or access prevented meaningful judgement; never counted as negative.

Catalogue-only/model-proposed evidence cannot be used as confirmed positive/negative in this audit. The HERA evidence audit separately checks candidate-specific corroboration and its rights/access provenance.

## Population estimation

For each confusion-matrix cell, v0.1 uses a Horvitz–Thompson total: \(\widehat{T}_{cell}=\sum_{i\in sample} y_{i,cell}/\pi_i\). Precision, recall, F1 and false-negative rate are ratio estimates from the expanded cell totals. Raw audited confusion counts are always reported separately from expanded population estimates.

v0.1 is deliberately strict: if any selected unit is unknown, unobservable or model-abstained, population-level precision/recall metrics are withheld. The report still exposes raw counts, unresolved fractions, sample/landscape coverage and Kish effective sample size. There is no silent nonresponse imputation.

## Uncertainty boundary

The current report does not emit confidence intervals. First-order inclusion probabilities support Horvitz–Thompson point totals but do not suffice for a general design-based variance. The next version needs pairwise inclusion probabilities or an explicitly defined multi-stage/PSU sampling design and cluster variance estimator. Do not calculate pixel-IID confidence intervals for landscape clusters.

## Benchmarks

- Design-weighted recall and false-negative total against a locked external audit.
- False-negative discoveries per 100 reviewer-hours within predicted-negative strata.
- Precision/recall difference between naive validated-only and design-weighted estimates.
- Effective sample size and reviewer-hours per fixed confidence-interval width.
- Stratum- and landscape-specific false-negative burden; unknown/unobservable share.
- Comparison with uniform random, score-proportional and expert-targeted audit allocation at matched budget.

## Leakage and governance

Once a sampled unit is fed back into model training/tuning, it no longer belongs to an untouched final audit. Keep a locked external set; store selection probabilities and frame/version digests; sample only within rights/custodial policy; withhold exact vulnerable-site coordinates from public manifests.

## Claim ceiling

The module validates frame/sample arithmetic and reference presence. It cannot prove sampling was genuinely random, inclusion probabilities are authentic, the frame is complete, survey detectability is valid, or reviewer/custodial authority is genuine. Those require independent design audit and heritage governance.