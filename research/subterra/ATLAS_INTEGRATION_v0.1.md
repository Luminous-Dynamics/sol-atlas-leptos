# Sol Atlas — Subterra planetary atlas integration v0.1

**Date:** 2026-11-16  
**Status:** Research integration specification; no production renderer/type changes in this increment  
**Related programme:** GeoBio Foundry / Subterra in Luminous-Dynamics/symtropy

## 1. Purpose

Extend Sol Atlas from an Earth-first infrastructure globe into a multi-body planetary atlas capable of preserving, comparing and visualizing subterranean site evidence across planets, moons, asteroids and comets.

This is not accomplished by adding a body name to the existing Earth `Site` record. Planetary mapping requires explicit body identifiers, body-fixed or inertial reference frames, coordinate conventions, orientation models and time validity.

## 2. Repository-grounded assessment

The current public `sol-atlas-leptos` repository provides useful foundations:
- `sol-atlas-core/src/types.rs` defines Earth-oriented records such as `Site`, `GeothermalNode`, `EarthRegion`, `MajorCity`, and `FossilDeposit` with scalar `lat` and `lon` fields but no body identifier or coordinate-frame metadata.
- `Layer::Regions` is labelled `Earth Regions`; the layer list does not currently define a planetary subsurface/site-evidence layer.
- `sol-atlas-core/src/geo.rs` provides a spherical map conversion and a `haversine_km` function that hard-codes an Earth radius of 6,371 km.
- `sol-atlas-core/src/solar_system.rs` contains visual orbital parameters for the Sun, Moon, Venus, Mars, Jupiter and Saturn. These are rendering/layout values, not body-fixed cartographic frames, shape models or surface coordinate transformations.
- `DataProvenance` distinguishes observed, curated and scenario datasets, with source and snapshot date. It is a useful start but does not yet carry per-observation native resolution, calibration, covariance, coordinate-frame lineage, or processing history.

This means Sol Atlas has a multi-body visualization foundation, but its inspected site schema and geodetic helper are Earth-centric. Directly loading lunar or Martian cave coordinates into the existing `Site` type would lose essential reference information.

## 3. Coordinate-system requirements

### 3.1 Use authoritative body-fixed frames

Every planetary location must declare:
- target body identifier;
- reference-frame identifier (for example an IAU body-fixed frame);
- coordinate-system type;
- latitude definition where applicable (planetocentric or planetographic);
- longitude direction (positive east or positive west);
- reference spheroid/shape model and radii where applicable;
- coordinate epoch or validity interval for time-dependent orientation models;
- source/model identity and version.

The IAU Working Group on Cartographic Coordinates and Rotational Elements defines recommendations for body orientation and cartographic coordinates. NASA PDS4 and NAIF SPICE provide relevant data and transformation standards.

References:
- IAU Working Group on Cartographic Coordinates and Rotational Elements: https://www.iau.org/WG100/WG100/Home.aspx
- IAU/WGCCRE 2015 report: https://doi.org/10.1007/s10569-017-9805-5
- NASA PDS4 Standards Reference (spin axes, prime meridians and coordinate systems): https://pds.nasa.gov/data/pds4/releases/en/system_bundleD-20240609/Standards_Ref_v3.4.pdf
- NASA NAIF SPICE body-fixed and reference-frame documentation: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/frames.html
- NAIF planetographic-to-body-fixed rectangular conversion: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/pgrrec_c.html

### 3.2 Do not reuse Earth constants

The current `haversine_km` is Earth-specific. Planetary distance and local surface projection must use the target body's declared shape model and geodetic convention. Irregular small bodies may not be well represented by a sphere or reference ellipsoid; they may require a shape model and body-fixed Cartesian coordinates.

### 3.3 Keep the subsurface in a local 3D frame

A cave interior should use a declared local survey frame anchored to a site reference point:
- frame ID and parent body-fixed frame;
- transform to body-fixed coordinates, including rotation and translation;
- units and axis convention;
- registration method and covariance/uncertainty;
- survey epoch and instrument calibration;
- loop-closure / control-point residuals.

A local cave frame is not the same as latitude/longitude. The entrance tie-point connects the interior survey to the planetary atlas; interior mapping uncertainty must remain explicit and must not be silently forced to the surface map.

## 4. Proposed data model

Keep current Earth records stable. Introduce separate multi-body entities:

- `PlanetaryBodyRecord`: body ID, name, body class and shape/orientation model references.
- `PlanetaryReferenceFrame`: frame ID, frame type, coordinate convention, epoch validity and source.
- `PlanetarySurfaceFeature`: feature ID, body/frame reference, surface coordinates, morphology, evidence and provenance.
- `SubsurfaceSiteRecord`: site/entrance ID, competing geological-origin hypotheses, connectivity, accessibility, surveyed extent, interior-frame ID and preservation constraints.
- `SubsurfaceObservation`: modality, acquisition geometry, native resolution, time, calibration, uncertainty and immutable source reference.
- `SubsurfaceSurveyFrame`: local frame transform, parent frame, control points, covariance and map version.
- `SubsurfaceGeometryAsset`: versioned mesh/point cloud/profile with coverage masks, resolution and digest.

Use the Subterra site-evidence schema from the Symtropy research branch as the cross-system evidence contract:
https://github.com/Luminous-Dynamics/symtropy/blob/research/geobio-foundry-v0.2/research/geobio-foundry/SUBTERRA_SITE_EVIDENCE_SCHEMA_v0.1.json

Use evidence state separately from visualization category. A map layer is not an engineering qualification.

## 5. Proposed atlas layers

Initial layer candidates:
1. Planetary surface context and reference grids.
2. Subsurface access-point candidates (pits, skylights, fractures, collapse chains).
3. Multi-sensor-supported subsurface features.
4. Directly surveyed entrance-connected voids.
5. Interior survey coverage and unobserved regions.
6. Geological-origin hypotheses.
7. Environmental measurements (temperature, radiation, pressure, gas, dust, volatile observations).
8. Robot tracks, access/egress routes and relay locations.
9. Preservation/planetary-protection constraints.
10. Use-specific engineering assessments, clearly labelled candidate/assessed/qualified.

Filters should include body, feature class, evidence state, source instrument, survey epoch, origin hypothesis, connectivity state and preservation status.

## 6. Subterranean visualization rules

- Surface context and cave interior must be visibly distinct.
- Interior geometry must be shown only over surveyed extent; unsurveyed regions remain visibly unknown.
- Geometry confidence/coverage is separate from geological-origin confidence.
- Candidate and inferred geometry must use distinct visual encodings from directly surveyed geometry.
- Every feature must link to source observations and their native resolution.
- A cave visualization must not imply pressure containment, structural stability or habitability.
- Time-dependent observations and coordinate transforms must be versioned to prevent stale layers being rendered in an incorrect frame.

## 7. Integration responsibilities

| System | Responsibility |
|---|---|
| Sol Atlas | Planetary coordinate frames, multi-body site catalogue, layers, provenance, observation lineage and spatial visualization |
| Symtropy | Local 3D survey profiles, volumetric terrain, geometry processing, physics, robotics and replayable scenarios |
| Symthaea | Active observation selection, competing-hypothesis reasoning, survey planning and anomaly diagnosis |
| Mycelix | Resource/sample/robot custody, access agreements and independently attested claims |
| GeoBio Foundry | Resource/process graph, site-to-capability compilation and industrial process accounting |

## 8. Implementation sequence

1. Add body/frame-aware data types alongside current Earth types; do not silently reinterpret old records.
2. Add a frame transformation interface backed by versioned IAU/PDS/NAIF reference data.
3. Add a synthetic Moon/Mars/Venus site dataset with explicitly synthetic provenance.
4. Add map-layer filtering by body and evidence state.
5. Add a surface-to-local-cave frame tie-point and interior geometry asset.
6. Add coverage/uncertainty display and source-observation navigation.
7. Add read-only adapters to Subterra site records from Symtropy.
8. Only after coordinate transforms and evidence semantics are reviewed, add production layers or renderer changes.

## 9. Explicit non-goals for v0.1

- No direct production type changes.
- No invented planetary coordinate constants.
- No reuse of Earth haversine distance for other bodies.
- No inference of cave connectivity from a dark pit marker.
- No model-generated geometry displayed as surveyed geometry.
- No safety, pressure, habitat or industrial suitability claim from a globe visualization.

The initial value of this integration is a trustworthy planetary spatial memory for the Subterra programme. Sol Atlas should make the evidence inspectable; it must not upgrade the evidence.
