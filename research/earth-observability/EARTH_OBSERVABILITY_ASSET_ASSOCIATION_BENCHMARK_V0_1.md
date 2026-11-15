# EARTH-OBSERVABILITY-ASSET-ASSOCIATION-001
## Admissible event-to-asset clustering metrics and adversarial scoring v0.1

**Implementation:** `sol-atlas-core/src/utility_reconstruction_metrics.rs`  
**CLI:** `utility_reconstruction_eval`  
**Input schema:** `ASSET_ASSOCIATION_EVALUATION_V0_1.schema.json`  
**Fixture:** `fixtures/asset-association-evaluation-synthetic-v0.1.json`  
**Pinned analytic metric baseline:** `baselines/asset-association-baseline-v0.1.json`  
**Parent:** Sol-Atlas #771; implementation child #779  
**Status:** Rust scorer, synthetic fixture and pinned metrics authored; exact-head CI pending  
**Claim ceiling:** Abstract analytic clustering only. No physical survey, utility truth, geometry, connected topology, calibration or engineering suitability claim.

---

## 1. Why a new scorer is needed

Event detection metrics cannot tell whether several radar events describe one physical asset, multiple assets or non-utility clutter. Asset-level evaluation must therefore score the partition of observed events into candidate assets.

This evaluator makes two safeguards explicit:

1. The scored denominator contains only independently admissible reference events.
2. Missing predictions are not silently interpreted as abstentions. Every reference event receives an explicit candidate-asset assignment or an explicit abstention.

A candidate asset is never an authoritative owner asset identifier.

## 2. Reference truth tiers

| Tier | Scored? | Meaning |
|---|---:|---|
| `structured_independent_survey` | Yes | Independent structured survey/exposure truth with evidence reference |
| `trench_image_interpreted` | Yes | Reviewed image-derived trench interpretation; supports only the adjudicated target association, not unmeasured 3D geometry |
| `owner_record_only` | No | Record evidence only; not independent physical truth |
| `conflicted` | No | Contradictory unresolved references |
| `insufficient` | No | No admissible event-to-asset truth |

Within an eligible tier, a null `truth_asset_id` means the event is adjudicated non-utility clutter. A null label in an ineligible tier is not clutter and is excluded.

A truth evidence identifier is mandatory for every eligible event. The scorer counts excluded tiers separately rather than dropping them invisibly.

## 3. Prediction semantics

Every reference event has one prediction:

- `candidate_asset`: a non-authoritative identifier in the `candidate:` namespace.
- `abstain`: the model declines to assign the event to an asset.

Abstention is not a negative utility finding. For clustering metrics, an abstained event is treated as a singleton predicted cluster. This preserves the cost of incomplete reconstruction without fabricating an asset or converting a non-detection into absence.

## 4. Metrics

The evaluator emits:

- B-cubed precision, recall and F1, averaged over every eligible event;
- pairwise true-positive event pairs;
- explicit overmerge and oversplit pair counts;
- pairwise precision and recall;
- candidate asset count, false candidate asset count and missed truth asset count;
- event assignment coverage, abstention rate, false associations of clutter and abstained utility events;
- excluded counts for record-only, conflicted and insufficient truth.

B-cubed metrics are event-balanced. Pairwise counts make merges and splits directly interpretable. Neither metric substitutes for independent geometry or topology truth.

### Pair definitions

- **Overmerge:** two events from different reference clusters are assigned to the same candidate asset.
- **Oversplit:** two events from the same reference utility asset are not assigned to the same candidate asset.
- Each adjudicated clutter event is a distinct reference singleton.
- Each explicit abstention is a distinct predicted singleton.

If there are no predicted positive pairs or no reference positive pairs, the corresponding pairwise precision or recall is `null`, not zero and not one.

## 5. Adversarial controls

The Rust unit tests include:

1. Correct grouping yields perfect event clustering.
2. Merging clutter into a real utility produces explicit overmerge pairs and a false clutter association.
3. Splitting one reference utility into two candidate assets produces oversplit pairs.
4. Abstention reduces completeness and does not claim physical absence.
5. Record-only truth is excluded, not promoted to clutter.
6. A missing prediction is rejected rather than silently treated as abstention.
7. Conflicted truth cannot be promoted to a negative example.
8. The committed fixture parses and matches the expected structural metrics.

The CLI tests additionally bind a SHA-256 digest to the exact input bytes and compare the resulting metrics against the pinned analytic baseline.

## 6. Synthetic fixture interpretation

The frozen fixture includes:
- two events belonging to one reference utility;
- one event belonging to a second reference utility;
- one adjudicated clutter event;
- one record-only event that is excluded from scoring.

The candidate output correctly associates the three utility events with two candidate assets, abstains on clutter and makes an arbitrary candidate assignment for the excluded record-only event. The latter does not affect the scored metrics.

The fixture deliberately demonstrates that an unsupported truth tier cannot affect performance merely by being assigned a convenient label.

## 7. Next physical-data use

The first proposed external study is UT-Gold-D, but the benchmark must not assume all trial-trench images are structured geometry. Use the truth-tier ledger and only score event-to-target associations where the image/survey linkage and target interpretation have been independently reviewed.

The UT-Gold-D publication identifies important acquisition limitations: some SEG-Y files lack georeferencing because GNSS obstruction forced use of wheel encoder only; uneven terrain sometimes prevented consistent wheel contact, compromising data throughput and quality. These cases must be preserved as separate metadata/observability strata, not discarded without accounting or filled with invented coordinates.

Dataset source:
- ter Huurne, Olde Scholtenhuis & Dorée (2024), Data in Brief 54, 110329, DOI 10.1016/j.dib.2024.110329.
- 4TU dataset DOI 10.4121/96303227-5886-41c9-8607-70fdd2cfe7c1.

Use the 13 construction projects as project-level holdout clusters. Report leave-project-out performance with all 13 project results visible. Do not treat 125 activities or 959 radargrams as independent sites.

## 8. Nonclaims and promotion rules

This evaluator does not score geometry, depth, route continuity, node/edge topology, false junctions or calibration of utility-world probabilities. Those require their own reference truth and metrics.

A high B-cubed score does not establish a physically correct map. A candidate asset with correct event grouping may still have incorrect position, depth, material, size or connectivity. Do not promote it to an authoritative owner record or a PAS/ASCE quality claim from this score alone.

No physical performance result is claimed until data admission, project-level blind split, independent truth adjudication and exact reproducible evaluation are complete.
