# Temporal Availability and Frontier Knowledge Boundary V1

## Purpose

This note defines the V1 meaning of temporal availability in Sol Atlas civilizational projections. It clarifies existing behavior; it does not change the meaning of the temporal availability fields or frontier manifest hashing. Temporal admission receipts may carry an optional frontier manifest hash so a newly produced receipt can bind to the exact content-addressed frontier; this receipt-level field is migration-compatible and does not alter the V1 availability semantics.

## Three distinct concepts

- **Temporal extent** describes when an assessment or interpretation applies, or when an evidence artifact/source snapshot is situated. It is not, by itself, a statement that the record was available to a historical observer.
- **Record availability** (`available_by`) is the earliest year by which the complete record is asserted to have been available for use by the projection system. For argumentation metadata, V1 treats this as availability of the completed external argumentation record.
- **Frontier knowledge horizon** (`known_by_year`) is the cutoff represented by an evidence frontier. It determines which already-described records can be admitted into that historical reconstruction.

These concepts are related but not interchangeable. An old artifact can have a late availability year; an assessment can concern an earlier period while the assessment record itself was only available later.

## V1 admission rule

A temporal record is eligible at a frontier only when its metadata validates and:

`available_by <= known_by_year`

The comparison is inclusive: availability in the frontier's cutoff year is eligible. A record with a later `available_by` is not eligible, even if its subject matter, artifact time, or assessment extent is much older.

For evidence and source snapshots, the strict temporal manifest validator checks that each admitted record has valid metadata and that its availability does not exceed the frontier horizon. Argumentation admission additionally matches the assessment/interpretation identity pair and checks its availability against that same horizon. Frontier admission establishes temporal eligibility only; it does not establish claim applicability, epistemic qualification, historical truth, or scholarly consensus.

## Temporal extent consistency

For argumentation metadata, V1 also requires an explicitly bounded temporal extent not to end after `available_by`. If an extent is open-ended, its known start must not be after `available_by`. This is a conservative consistency rule for the completed external record, not a claim that the assessment or interpretation began only when it became available.

Evidence `artifact_time` and `validity_time` remain separate descriptors. They should not be silently substituted for `available_by`. Publication and capture years, when supplied, cannot be later than `available_by`.

## Append-only lineage

A child frontier may advance `known_by_year`, but it cannot rewrite inherited evidence, source, or argumentation metadata. New availability can be introduced as new records in a later frontier; it must not be retroactively assigned to inherited records. A later horizon therefore does not, on its own, upgrade a claim's `QualificationStatus`.

Historical replay must validate the selected frontier and its verified prefix. A later frontier's additional knowledge must not leak into a replay selected at an earlier cutoff.

## Why V1 does not add another field

The current contract intentionally uses `available_by` as the record-availability boundary and `known_by_year` as the reconstruction cutoff. They answer different questions without requiring a schema or hash change. A future need to represent separate first-observed, published, ingested, or system-indexed dates should be handled through a versioned metadata contract, not by silently changing V1 meanings or historical manifest interpretation.

## Standards alignment and limits

W3C PROV distinguishes entity generation from activity intervals and emphasizes event ordering while minimizing assumptions about synchronized physical clocks. Sol Atlas's year-based V1 fields are a domain-specific, coarser contract; they should not be described as a complete PROV event model. This note borrows the separation of temporal concepts, not a claim of PROV conformance.

## Invariants to preserve

- `available_by == known_by_year` is eligible.
- `available_by > known_by_year` is ineligible, regardless of older subject-matter dates.
- Later frontier horizons do not mutate inherited metadata or claim qualification.
- Temporal eligibility is not a truth or authority judgment.
- Any future semantic expansion is explicitly versioned and preserves historical replay compatibility.
