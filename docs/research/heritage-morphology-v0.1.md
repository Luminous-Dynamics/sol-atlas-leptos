# HDB-23: Multi-Scale Architectural Morphology v0.1

## Purpose

HDB-23 creates a reproducible descriptive layer between individual measured building stones and higher-order architectural groups. It asks what measured morphology can be described across wall segments, walls, buildings, households, compounds, insulae, neighbourhoods and settlements before any historical interpretation is attempted.

The first Rust slice is descriptive only. It does not fit hierarchical population models, run spatial significance tests, infer construction chronology, identify social groups or establish political boundaries.

## Measurands

The v0.1 input supports:
- calibrated 2D plan-footprint area (m²);
- normalized MRR regularity ratio = stone plan-footprint area / minimum-rotated-rectangle area, in (0,1]; values near 1 indicate a more regular footprint under this definition;
- elongation ratio = long axis / short axis, greater than or equal to 1.

These quantities are distinct from 3D surface area, volume, wall-face area, masonry volume, stone count or packing density. Future descriptors require their own dimensionality, calibration and uncertainty contracts.

## Identity and membership

There is one selected measurement observation per physical stone in one declared snapshot. Repeated captures of one physical stone must be resolved upstream and cannot silently inflate the stone count.

Every stone carries one explicit membership state at each of eight named grouping levels: assigned, ambiguous, unassigned or not assessed. An ambiguous stone lists at least two distinct eligible candidate groups and is not included in assigned-only statistics. Candidate group summaries record how many ambiguous stones could belong to them.

Membership records are supplied alongside an architecture graph digest. Version 0.1 checks their shape and uniqueness but does not independently join or prove each membership edge against the graph. HDB-17 remains responsible for source-bound physical assembly relations. The current descriptive report also does not calculate all alternative-membership sensitivity scenarios; it preserves candidate sets so a later sensitivity evaluator can do so.

Household and compound are included as distinct grouping dimensions because Shivta research reports household/insula aggregations while HDB-17 provides compound structure. They are not asserted to be universally equivalent or to have a universal parent-child relationship.

## Descriptive aggregation

For each (hierarchy level, group ID) the evaluator reports:
- assigned physical stone count;
- distinct ambiguous candidate count;
- area, MRR regularity and elongation summaries where available;
- mean, nearest-rank p10, median and nearest-rank p90.

Each input measurement is a lower/upper interval. The summary propagates interval endpoints into descriptive bounds. These are **not** confidence intervals, credible intervals or population sampling intervals. Pixels/stones within a wall are not independent site-transfer replicates; this module explicitly claims zero independent site replicates.

Unassigned, ambiguous and not-assessed membership states are separately counted for every grouping level. Missing shape descriptors remain absent rather than being converted to zero.

## Provenance and limits

The input binds dataset, architecture graph, measurement protocol, rights decision, measurement geometry, source capture, method build and measurement receipt identifiers/digests. SHA-256 fields are syntax-checked; the code does not authenticate the underlying bytes, source custody, calibrations, rights documents or the truth of a membership relation.

## Benchmark and negative controls

The synthetic test suite covers deterministic aggregation, interval means/quantiles, ambiguous membership retention, unassigned vs not-assessed distinction, repeated physical identity, duplicate observations, missing/duplicate group levels, invalid status combinations, out-of-range measurement intervals, missing descriptors, malformed digests and input-order invariance.

Next benchmark stages should add:
- membership perturbation across all eligible parents and resulting summary sensitivity;
- alternate wall split/merge and missing-stone corruption;
- shared systematic calibration covariance via HDB-18;
- cluster/site bootstrap via HDB-20 evaluation issue #543;
- spatial autocorrelation and conditional randomisation tests;
- sensor/material/preservation/vegetation confounding controls;
- independent chronology/archival/field evidence before any temporal settlement claim.

## Scientific and ethical claim ceiling

A difference in stone size or shape is a measured morphological difference under a declared sampling and calibration regime. It does not by itself establish ethnicity, wealth, status, labour organisation, household function, political affiliation, religion, chronology or cultural identity. Such claims require independent evidence, domain expertise and appropriate custodial participation.

Model-generated or simulated geometry remains separate from observed physical measurement. The report claim ceiling is descriptive architectural morphology only.
