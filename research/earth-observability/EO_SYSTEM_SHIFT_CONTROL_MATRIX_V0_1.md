# EO System-Shift Control Matrix v0.1

**Parent:** [EO-SYSTEM-COMPARABILITY-001 / #829](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/829)  
**Implementation:** [draft PR #830](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/830)  
**Status:** synthetic controls and structural claim contract; no real-data run.

**Machine-readable cases:** [EO_SYSTEM_SHIFT_CONTROL_MATRIX_V0_1.json](./EO_SYSTEM_SHIFT_CONTROL_MATRIX_V0_1.json)  
**Schema:** [eo-system-shift-control-matrix-v0.1.schema.json](../../schemas/eo-system-shift-control-matrix-v0.1.schema.json)

## Purpose

This matrix isolates the measurement-system confounder from the physical-change problem. Each scenario holds geography/support and target semantics constant while varying exactly one source, sensor, collection, processing or asset-measurement dimension.

The benchmark must not award a physical-change claim for a measurement-system shift. It should return:
- `blocked` when a promoted change/no-change claim lacks a valid exact-scope comparability receipt;
- `unresolved` when platform/sensor/configuration identity is insufficient;
- `eligible_for_review` only when the source/measurement bridge is present and independently verified.

A digest-only local reference is not scientific qualification. The external verifier must check the full payload and calibration evidence.

## Canonical measurement-system signature

The current typed signature includes:

1. canonical source-registry identity;
2. platform identifier;
3. source-qualified sensor ID and identity origin (provider-declared or derived);
4. instrument identifier;
5. modality;
6. product collection/family;
7. processing baseline/version;
8. SHA-256 fingerprint of the exact asset-level measurement profile (e.g. bands, polarization, instrument mode and relevant acquisition configuration).

A missing product collection, processing baseline or measurement-profile fingerprint is incomplete identity and forces a comparability receipt or abstention. Two missing values do not prove equivalence. A missing processing baseline on one side differs from a known baseline on the other; it must never be silently treated as equal.

## Synthetic control matrix

| ID | Physical world | Measurement-system change | Bridge | Expected disposition |
|---|---|---|---|---|
| MS-001 | No event | Same source/platform/sensor/collection/baseline/profile | Not needed | Same-system claim may be evaluated under ordinary POD/coverage rules |
| MS-002 | No event | Processing baseline v1 → v2 | Absent | Block promotion |
| MS-003 | No event | Landsat platform/instrument → Sentinel platform/instrument | Absent | Block promotion |
| MS-004 | No event | Same platform/sensor but different product collection | Absent | Block promotion |
| MS-005 | No event | Same platform/product but different bands or SAR polarization profile | Absent | Block promotion |
| MS-006 | Unknown | Asset measurement-profile fingerprint missing | Absent | Block or unresolved; never assume same system |
| MS-013 | Unknown | Processing baseline missing on both observations | Absent | Block or unresolved; two unknowns do not establish equality |
| MS-007 | True event | Sensor/platform transition with an independently qualified bridge | Exact receipt present | Eligible for expert review, not automatic truth |
| MS-008 | No event | Cross-sensor transition | Receipt references wrong observation set | Block promotion |
| MS-009 | No event | Cross-sensor transition | Receipt has wrong support/class/ontology | Block promotion |
| MS-010 | No event | Same fully identified measurement system | Unnecessary receipt supplied | Reject as invalid/unnecessary reference |
| MS-011 | Unknown | Multi-instrument or provider-sensor identity ambiguous | Any | Unresolved; bridge does not manufacture a missing identity |
| MS-012 | Common-mode error | Two sensors share the same misregistration/calibration bias | Apparently agreeing systems | Must remain a separate negative control; agreement is not independent corroboration |

## Metrics

- false physical-change claim rate on MS-002 through MS-006 and MS-008 through MS-012;
- exact-scope receipt acceptance/rejection accuracy;
- unresolved/abstention rate for incomplete source identity;
- bridge coverage (share of multi-system cases with a valid independent bridge);
- false-bridge rate (receipt accepted despite wrong source, asset, support, target, ontology, QA, or configuration);
- recall and area error on MS-007 after a bridge is independently accepted;
- worst-domain performance by source × platform × sensor/instrument × collection × processing baseline × asset measurement profile.

## Hard synthetic gates

- 0 promoted physical-change claims on no-event measurement shifts without a valid exact-scope comparability receipt.
- 0 receipts accepted with an incorrect complete observation-ID set, support, target class or ontology version.
- 0 unknown sensor identities made admissible by a bridge receipt.
- 0 incomplete measurement-profile fingerprints silently treated as equivalent.
- 100% deterministic replay of the frozen control bundle.

These are structural and synthetic gates. They are not estimates of real-world accuracy or field reliability.

## Implementation boundaries

- Sol-Atlas computes the declared observation-system signature and checks local claim/receipt binding.
- STAC profile ingestion preserves provider-native identity and per-asset measurement metadata.
- The source/asset manifest authority supplies the measurement-profile fingerprint; Sol-Atlas does not guess a digest.
- Mycelix/custodian or an independently qualified calibration authority verifies source rights, custody, calibration and the comparability payload.
- Symthaea may generate hypotheses and identify what additional observation would discriminate explanations; it cannot self-certify the receipt.
- Symtropy owns physical/environmental forward models where they are used; a map-difference score is not a physical model.

## Exit criteria for real-data evaluation

Do not begin T2/T3 scoring until:
1. the data rights and use purposes for every source are independently reviewed;
2. each observation has a complete or explicitly unresolved source and measurement profile;
3. the spatial/temporal split is frozen by independent scene/site/event groups;
4. independent reference data are defined and not derived from the same model/source;
5. the cross-system bridge qualification procedure and blind adjudicator are preregistered;
6. exact code, schemas, data manifests, model weights, transforms, scorer and environment digests are frozen.

Unresolved records are retained for investigation: the contract permits an explicitly `Unresolved` mixed-system claim without a comparability receipt, but does not permit CandidateChange, SupportedChange or NoChangeWithinEnvelope promotion without an exact-scope bridge.