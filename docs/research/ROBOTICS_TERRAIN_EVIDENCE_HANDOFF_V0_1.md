# Robotics Terrain Evidence Handoff v0.1

**Status:** typed spatial-evidence contract and synthetic unit tests; no route, terrain-model, or robot-performance result is claimed.

## Purpose

Robots need terrain context that can be traced back to a versioned map, model, coordinate frame, sensor observation, and calibration. A coarse label such as “gravel” or a route length is not enough to establish traversability. Sol Atlas should own the spatial reference and its coverage/uncertainty declarations; Symtropy should own scenario interpretation and simulation; Symthaea should own policy decisions; Mycelix should preserve mission/observation provenance and authorization.

This module is deliberately **not** a planner and does not authorize motion. It adds a versioned record which downstream systems can reference without copying raw coordinates into every scenario.

## Envelope fields and fail-closed rules

- The map snapshot, terrain tile, and optional raw observation are identified by SHA-256-shaped digests. The module checks digest format, not authenticity or truth of the referenced bytes.
- A site, coordinate frame, model/version, observation time, as-of time, optional validity deadline, evidence class and source refs are explicit.
- Geometry fields (slope, cross-slope, step height, rut depth) are optional when not observed. Unknown is not coerced to zero.
- Evidence state and spatial coverage are separate. An available result cannot claim unknown/outside coverage; outside-coverage state must agree with the coverage label.
- Hardware-in-the-loop and field evidence require a raw-observation digest, calibration reference and hardware-configuration reference. Independently reviewed evidence additionally requires reviewer attribution.
- Quantified uncertainty requires a non-negative absolute radius and a declared confidence level. Qualitative or unassessed uncertainty cannot carry an unexplained numeric bound.
- References and hazard lists must be sorted and unique for deterministic serialization.

## Hazard semantics

Each hazard is independently typed as detected, absent, unknown, stale or conflicting. A detected/absent label requires confidence and source evidence. Unknown/stale/conflicting labels cannot carry a fabricated confidence value. The helper that determines whether a clearance claim is blocked treats every state except explicit evidence-backed absence as present-or-unresolved.

That helper is **not** a safety controller: the absence of a hazard record does not prove that an area is safe, and a valid envelope does not prove terrain-model fidelity or physical robot capability.

## Evidence classes

Synthetic fixture, simulator, hardware-in-the-loop, physical field and independently reviewed evidence remain distinct. Do not pool these into a single performance number. Moving a record to a higher evidence class requires new evidence, not a metadata edit.

## Integration points

- **Sol Atlas:** authoritative spatial snapshot, terrain tile/model identity, coordinate-frame reference, coverage, uncertainty and hazard evidence.
- **Symtropy:** binds the envelope to a scenario ID and tests planner/controller behavior under matched terrain/fault cases.
- **Symthaea:** consumes typed context and estimates action success, recovery or abstention; it must not silently convert unknown terrain/hazard evidence to clear.
- **Mycelix:** can link mission receipt, raw observation, calibration, reviewer and repair/inspection records without changing this spatial source of truth.

## Testing and claim ceiling

The unit tests exercise schema version, deterministic serialization, digest shape, stale evidence, evidence-class requirements, uncertainty consistency, canonical references, hazard provenance and the distinction between unknown and absent hazards. They are synthetic contract tests.

A passing test proves structural invariants only. It does not validate a digital terrain model, establish route feasibility, authenticate source artifacts, satisfy a robot safety standard, or qualify a physical system.
