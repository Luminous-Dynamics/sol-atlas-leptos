# Interval-Censored Event Chronology v0.1

**Scope:** evaluate when a known change event occurred when the evidence only constrains it to a time interval, and evaluate chronology only when event intervals are disjoint.

## Why a new temporal target

Satellite revisit gaps, cloud, seasonal acquisition schedules, and delayed field confirmation make exact event months frequently unknowable. A model that returns a single month should not be scored as though the event occurred at that exact instant when the evidence only shows it happened between the last unaffected and first affected observations.

Microsoft WATCH is the current temporal baseline to reproduce: monthly PlanetScope RGB+NIR mosaics at 4.7 m/pixel from 2017–2024, 1,943 Afghanistan sites, six geospatial foundation models plus handcrafted features, and three scoring pipelines—TED (temporal embedding distance), SSCD (self-supervised change detection) and WS (weakly supervised temporal localization). WATCH evaluates Recall@K and symmetric/directional temporal margins and reports generalization sites in Syria, Turkey, Pakistan and Egypt. Its repository is MIT-licensed but does not include the imagery or checkpoints: https://github.com/microsoft/WATCH.

A complementary fine-scale result is Liu et al. (2026), which evaluates phenology-aware temporal Siamese change detection on three East Asian landscape-heritage sites. It reports average F1 89.31%, false-alarm rate 6.42%, miss rate 8.17%, cross-season false-alarm rate 7.94%, and 27.3 days mean early-warning lead across 184 verified true-positive events. These UAV garden/conservation results are not directly comparable to regional satellite monitoring: https://doi.org/10.1038/s41598-026-63132-8.

## Formal target

An event time is represented by an inclusive admissible interval \([t_s,t_e]\), supported by adjudication and evidence references. The model supplies a conditional discrete posterior \(p(t\mid event, observations)\) over possible month ordinals.

Per-event measures:
- `interval_mass = sum(p(t) for t in [t_s,t_e])`.
- `interval_NLL = -ln(interval_mass)`. Zero mass fails closed.
- MAP-in-interval indicator.
- Recall@K: any of the K highest-probability months intersects the admissible interval.
- MAP distance to the nearest interval endpoint, zero when MAP is within the interval.

Pairwise chronology:
- If event A's admissible interval ends before event B's interval starts, truth establishes A before B.
- If intervals overlap, their order is unknown and the pair is counted as ambiguous, not forced into a chronology.
- Predicted order probability is \(P(T_A<T_B)=\sum_{t_A<t_B}p_A(t_A)p_B(t_B)\).
- Pairwise Brier score evaluates this probability against the known order. Pairwise accuracy excludes exact 0.5 ties and reports their count separately.

## Separation of tasks

1. Detection: did a physical change event occur? Use site/event detection and false-alert metrics.
2. Temporal localization: when did the event occur? This module evaluates event-time posterior distributions.
3. Cause attribution: what may have caused it? Use the separate cause evaluator and candidate-specific evidence. It must retain competing explanations and abstention.
4. Significance/impact: what does the change mean for heritage values and response? This remains a separate human/authority-governed judgement.

An alert is not proof of an event; an event is not proof of looting or damage; a cause hypothesis is not an accusation.

## HDB-9 benchmark

Reproduce WATCH TED/SSCD/WS on the same lawful source imagery, site splits, label cutoffs and temporal range. Preserve its Recall@K and symmetric/directional margin results for matched comparison. Add interval mass, interval NLL, MAP distance and pairwise chronology Brier. Include nearest-observation midpoint, uniform distribution over admissible interval, peak-score month, symmetric-margin, directional-margin and random-uniform-interval baselines.

Stratify by site, geography, sensor, observation cadence, admissible interval width, event type and seasonal regime. Missing observation months remain missing; they are not interpreted as no-event months. The final interval labels must be created by independent adjudicators from dated source evidence and locked before model tuning.

## Provisional targets

- At least 10% lower interval NLL than the strongest matched WATCH baseline on a preregistered held-out-site test set.
- At least 10% lower pairwise chronology Brier than nearest-month/midpoint baselines.
- Preserve or improve WATCH Recall@K at matched review budget and event tolerance.
- Calibrated posterior coverage and abstention reported separately from event detection.

These are research targets, not measured Sol-Atlas results.