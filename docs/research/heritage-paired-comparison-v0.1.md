# HDB-B9B — paired human-only vs AI-assisted operational comparison v0.1

## Purpose

This evaluator compares two completed HDB-B9 reports on the same sealed challenge and independently adjudicated evidence. It is designed to test a productivity hypothesis without allowing different sites, label coverage, surveyed areas or adjudication standards to contaminate the comparison.

The comparison report is descriptive and explicitly does not authorize population-level superiority claims. Paired HDB-20 site/region/acquisition bootstrap inference is still required.

## Required pairing

The baseline and candidate scorecards must share:
- challenge ID and immutable challenge-manifest digest;
- reference-adjudication protocol digest;
- independent evaluation-unit kind;
- exact evaluation-unit ID set;
- identical adequately assessed area for each unit;
- identical confirmed constructed-wall, ambiguous, not-observable and not-assessed reference counts;
- identical hard-negative reference counts per class.

The evaluator validates every report's arithmetic before comparing it: per-unit P/R/F1, TP/FP/FN, resolved/unresolved/candidate counts, area/hour totals, confirmed and unresolved-inclusive false-positive rates, and macro/micro aggregates. Report IDs must match report model IDs. Each report is bound by a source SHA-256 and receipt reference; this evaluator checks syntax and binding, not digital-signature authenticity.

Thresholds may differ between workflows, but each must have its own validation-selected, frozen threshold receipt. The challenge test partition must not be used to choose either threshold.

## Decision gates

The preregistered policy supplies:
- maximum absolute recall loss;
- maximum tolerated increase in confirmed false-positive candidates per adequately assessed km²;
- maximum tolerated increase in the unresolved-inclusive false-positive upper bound per km²;
- minimum candidate/human-only verified-findings-per-expert-hour ratio.

The comparison requires:
1. all recall-eligible independent units to remain within the recall-loss margin;
2. micro and macro recall differences to remain within the same margin;
3. confirmed false-positive density to remain within its tolerance for every unit and aggregate;
4. unresolved-inclusive false-positive upper bounds to remain within their separate tolerance for every unit and aggregate;
5. verified-findings-per-expert-hour ratio to meet the productivity target.

The productivity ratio is undefined when human-only baseline productivity is zero. It is not infinite, and the productivity target cannot pass from an undefined ratio. Recall for a unit with zero confirmed positive references is not assigned a fabricated value; that unit remains available for false-positive burden comparison but is excluded from recall comparisons.

## Why two false-positive burdens?

The HDB-B9 scorecard reports both confirmed false positives and an upper bound that treats every unresolved candidate as a potential false positive. The paired evaluator retains both. This prevents the AI-assisted workflow from seeming better merely by leaving ambiguous predictions unresolved.

## Statistical limits

No confidence interval is produced here. HDB-20's paired cluster bootstrap and spatial effective-information analysis must resample the declared independent units, not stones, pixels or chips. Until that work is completed with adequate preregistered site/region count and power, the comparison is exploratory and no population superiority claim is authorized.

## Stretch target

A proposed hypothesis is ≥2× verified findings per expert-hour while remaining within a partner-approved recall-loss margin (initially 2 absolute percentage points) and false-positive-density tolerances. The tolerances must be set with archaeologists before the challenge partition is unsealed and supported by the assessed negative-area sample size.

A 2× productivity result is not meaningful if it is achieved by sacrificing recall, increasing the false-positive burden, shrinking assessment coverage, or excluding difficult areas.

## Synthetic qualification

Tests include successful matched comparison, input-order invariance, challenge/manifest/protocol mismatch, unit-set and reference-coverage mismatch, undefined human baseline productivity, and invalid policy/report receipts. Synthetic fixtures are not field performance evidence.