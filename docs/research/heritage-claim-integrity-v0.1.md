# HDB-13 — Archaeological Construct Validity and Claim Ceiling v0.1

## Motivation

Orengo, Berganzo-Besga & Lumbreras, "Theory and practice of artificial intelligence in archaeology", *Journal of Archaeological Science* 190 (June 2026), DOI 10.1016/j.jas.2026.106571, argues that archaeological ML learns proxies shaped by labels, perception, sampling and source acquisition. It does not directly learn archaeology as an unmediated ground truth. The policy consequence is that the system must identify the construct it measured and the inferential steps that connect that observation to a historical claim.

## Claim chain

1. **Measured signal** — an instrument or survey recorded a signal/geometry.
2. **Proxy interpretation** — a declared archaeological proxy (cropmark, soilmark, relief anomaly, geophysical response, stone morphology) is interpreted from a signal.
3. **Anthropogenic feature candidate** — independent observations make a human-made explanation plausible.
4. **Archaeological feature candidate** — independent evidence includes field/material/stratigraphic support.
5. **Archaeological site interpretation** — the feature-level threshold is met and an identified human reviewer has adjudicated the exact evidence set.
6. **Historical-process interpretation** — the preceding threshold is met and independent chronology or archival evidence constrains the process.

This is a conservative policy ladder, not a universal epistemology of archaeology. Thresholds should be revised only through documented review with archaeological and custodial partners. A higher ceiling permits a claim to be reviewed; it never means that historical truth has been established.

## Independence and evidence dependence

Distinct files, crops, model architectures, derived layers, analysts or citations are not automatically independent evidence.

The v0.1 independent-support counter counts only direct, rights-verified supporting records whose:
- external independence status is marked independently assessed;
- independence group is unique;
- source-lineage root is unique;
- acquisition campaign/event is unique;
- method family is unique;
- source-asset digest is unique.

This conservative deterministic filter detects obvious duplicate/derived-source inflation. It is not a statistical proof of conditional independence. The evaluator checks that an independence assessment reference is structurally present; it does not authenticate its content or the authority of the assessor. The Mycelix / HCEA independence audit remains authoritative for actual independence claims.

## Evidence-origin rules

Direct observation may include raw remote-sensing, geophysical, field, excavation, material, chronometric or historical-source observations. Catalogue assertions, expert interpretations, model predictions and simulations are separate origins/kinds.

Model outputs and simulations are always excluded from empirical support counts. An ML prediction may help prioritise fieldwork; it cannot confirm itself or be counted as a second independent observation because another model produced the same prediction.

## Qualification policy

- Measured signal requires one rights-verified direct observation.
- Proxy interpretation requires a direct remote-sensing/geophysical signal and declared proxy-family identity.
- Anthropogenic candidate requires two independently assessed evidence groups or direct field assessment.
- Archaeological feature candidate requires two independent evidence groups, at least two distinct independent modalities, and at least one direct archaeological field/material/stratigraphic record.
- Archaeological site interpretation requires the archaeological-feature threshold and accepted human adjudication applying to the exact current evidence-ID set.
- Historical-process interpretation additionally requires at least one independent direct chronometric or historical-document record.
- Unresolved contradictions prevent a stronger ceiling. Explicit adjudication must mark contradictions resolved.
- Rights that are pending, denied, unknown or missing required receipts make the evaluation indeterminate.
- Restricted-coordinate policy is preserved in the report; this module does not contain exact geometry and cannot override custodial controls.

The policy does not allow reviewer approval to manufacture missing independent observations. It also cannot authenticate reviewer qualifications, source truth, rights receipts, independence attestations or cultural authority.

## HDB-13 metrics

Report by proxy family, region, biome, sensor and declared claim level:
- claim overreach rate;
- false archaeological-identification rate under geological, agricultural and modern-feature controls;
- proxy-vs-archaeology confusion matrix;
- source-lineage/acquisition/method duplicate inflation;
- contradiction-retention and adjudication-set freshness;
- selective risk/coverage under rights restrictions and OOD shifts;
- claim-probability calibration separately from detector confidence and geometric uncertainty;
- expert agreement and review time;
- worst-region and worst-proxy-family results.

Primary holdout unit is the independent site/landscape, not the number of pixels, tiles, stones, derived assets or citations.

## Negative controls

The synthetic suite must reject:
- model-only and simulation-only "archaeological" claims;
- catalogue-only evidence presented as field-confirmed;
- repeated imagery/derived layers counted as independent;
- same acquisition with different file digests;
- same method family with different software labels;
- pending/denied/unverified rights;
- missing source lineage or malformed digest;
- a stale human adjudication after evidence is added or removed;
- unreviewed or unresolved contradictory evidence;
- a historical-process claim without an independent chronological/archival record;
- public exact-coordinate release when the source access tier is restricted.

## Claim ceiling

A report disposition of qualified_at_declared_claim means only that the structural claim policy is satisfied for the submitted evidence record. It is not a finding of archaeological significance, not proof of historical truth, not an authenticated rights decision, and not fieldwork/excavation authority. Human and custodial review remain necessary.

## Research references

- Orengo, H.A., Berganzo-Besga, I. & Lumbreras, F. (2026). Theory and practice of artificial intelligence in archaeology. *Journal of Archaeological Science*, 190, 106571. https://doi.org/10.1016/j.jas.2026.106571
- The London Charter, Principle 4 (Documentation), requiring explicit representation type, uncertainty and evaluation context: https://www.london-charter.org/documentation.html
- Mycelix evidence-independence / HCEA work is the integration point for externally assessed source and method independence; HDB-13 consumes its attestation rather than recreating a universal independence ledger.
