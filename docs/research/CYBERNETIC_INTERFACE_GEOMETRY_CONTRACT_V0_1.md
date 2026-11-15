# Cybernetic Interface Geometry Contract v0.1

**Owner:** Sol-Atlas [CYBERNETIC-INTERFACE-GEOMETRY-001 #708](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/708)  
**Status:** schema and synthetic fixture proposal; not a clinical-fit model or validated body geometry engine.

## 1. Why this contract exists

The current Sol-Atlas core provides renderer-oriented vector/matrix math, globe/geodetic coordinates and data-layer provenance. The inspected source does not yet define a general, calibrated body–device spatial frame graph or revision-bound interface-evidence representation.

The next step is not to place raw scans in Sol-Atlas or Mycelix. It is to define the minimum semantics required to make a body/device relationship interpretable, uncertainty-bearing and traceable across the exact task, article and geometry revisions.

This contract is intentionally separate from:
- Symtropy joint mechanics and physical dynamics;
- Symthaea intent, adaptation and sensory feedback;
- Mycelix serialized article identity, calibration provenance and lifecycle records;
- clinical anatomy, tissue-injury thresholds or fit recommendations.

## 2. Spatial contract

The JSON Schema is `schemas/cybernetic-interface-geometry-v1.schema.json`. A synthetic example is at `schemas/examples/cybernetic-interface-geometry-phantom-synthetic-v0.1.json`.

### Frame graph

Each frame declares:
- stable frame identifier and semantic role;
- parent frame (or the one declared root);
- transform from child coordinates into parent coordinates;
- explicit axis meanings and right-handed convention;
- SI translation and rotation units;
- calibration reference and validity interval when physically measured;
- six-degree-of-freedom uncertainty, or an explicit reason uncertainty is not quantified.

`transform_parent_from_child` is deliberate: it removes ambiguity about transform direction. Translation is metres; quaternion order is `x,y,z,w`; angles are radians. A quaternion's norm and transform composition must be checked by a semantic validator.

JSON Schema cannot prove the graph is connected or acyclic. The eventual semantic verifier must enforce unique IDs, exactly one root, parent existence, no cycles, connectivity, normalized quaternions, valid calibration intervals, consistent units and axis conventions, and no transform composition across incompatible calibration epochs.

### Interface envelope

An interface envelope binds:
- the body/phantom frame and device CAD/manufactured-article frame;
- the local interface region;
- exact geometry artifact reference, digest if available, and revision;
- nominal clearance and plus/minus manufacturing tolerance;
- applicability profiles describing the pose/task/liner/configuration conditions.

A nominal CAD gap is not a measured contact gap. A geometric intersection is not a pressure estimate. An interface envelope is not a clinical fit decision.

### Interface observations

Measured pressure, shear, migration, joint-axis mismatch or contact state must be represented as separate observations with:
- evidence class (simulation, bench, HIL or human study);
- measurement time basis;
- exact calibration references;
- payload artifact reference and digest;
- uncertainty reference.

The observation record must not silently turn a sensor estimate into a biological safety threshold. Do not invent universal safe pressure/shear values.

## 3. Privacy and data minimization

Raw identifiable body scans should remain local or in a user-controlled restricted store by default. Distributed projections should contain only consented, purpose-limited geometry derivatives and the minimum references required to verify provenance.

The schema requires `raw_geometry_exported=false`. For human-linked records, subject references are pseudonymous and an explicit consent-scope reference is required. Direct identity, names, contact details and unfiltered raw scan payloads do not belong in this contract.

Synthetic examples must not be mistaken for human data, clinical anatomy or measured device fit.

## 4. Change and applicability

Geometry validity is revision- and task-dependent. Changes to padding, liners, straps, cuff dimensions, manufacturing tolerances, sensor locations, body-device registration or article identity may affect different interface claims.

A change must produce a new geometry/configuration revision or a successor reference. Historical evidence remains attached to the original inputs. Affected claims can be marked:
- `still_applicable` after a documented impact decision;
- `retest_required`;
- `review_required`;
- `not_evaluated`;
- `superseded`.

Do not invalidate every claim indiscriminately, and do not inherit every old result automatically.

## 5. Qualification ladder

| Stage | Allowed evidence | What it can establish |
|---|---|---|
| Schema fixture | Synthetic transforms and envelopes | Contract parsing and structural semantics only |
| Phantom/fixture | Calibrated geometry and metrology | Registration/clearance agreement for the declared fixture |
| Bench article | Exact manufactured article, instruments and calibration | Observed article geometry and instrumented interface behavior under declared test conditions |
| HIL | Synchronized geometry, motion and interface sensing | Declared integrated bench behavior, not human comfort or tissue safety |
| Human study | Approved protocol, informed consent, appropriate instruments and ethics/regulatory review | Only the outcomes and population actually measured |

No stage automatically promotes evidence to the next one.

## 6. Benchmark programme

The geometry lane should measure separate quantities rather than invent a single “fit score”:

1. **Frame registration:** translation/rotation error against calibrated reference fixtures, by frame and registration method.
2. **Graph closure:** loop residual, transform-chain consistency, disconnected frames and stale calibration rejection.
3. **Uncertainty calibration:** empirical coverage of declared transform uncertainty on held-out fixture poses.
4. **Clearance:** predicted versus measured clearance error by interface region and pose; report worst-case and distribution.
5. **Migration:** relative interface motion over a declared task cycle, with instrument and timing uncertainty.
6. **Joint alignment:** anatomical/mechanical axis mismatch only where the reference and method support that interpretation.
7. **Revision sensitivity:** detection of changed CAD, article, liner, calibration and capture revisions.
8. **Privacy:** zero raw identifiable scans in exported fixtures/projections; explicit consent and disclosure checks for any human-linked derivative.

Use synthetic positive and negative controls first. Later report p50/p95 and per-region error, but set numerical acceptance thresholds from calibrated metrology, engineering tolerances and risk analysis for the particular interface—not universal values guessed in advance.

### Required negative controls

- unknown or duplicate frame ID;
- missing parent frame;
- disconnected or cyclic frame graph;
- invalid/non-normalized quaternion;
- left/right-handed or axis-order mismatch;
- unit mismatch (mm supplied as m, degrees as radians);
- stale, revoked or wrong-instrument calibration;
- transform composition across incompatible calibration epochs;
- geometry revision differs from the manufactured article revision;
- interface observation references a different article or sensor calibration;
- uncertainty omitted but reported as zero;
- synthetic/estimated geometry relabelled as physical measurement;
- raw identifiable scan included in a public projection without an authorized disclosure profile.

A reject-all implementation is not acceptable: a valid synthetic graph and valid calibrated phantom graph must be admitted by their respective profiles.

## 7. Owners to compose

- Sol-Atlas #708 owns the spatial frame/geometry evidence contract.
- Symtropy #4557 and #460 own physical dynamics and physics qualification; neither owns body-scan identity or clinical fit.
- Symthaea #11400/#11403 own task requirements and sensing/intent semantics; neither owns the spatial frame graph or physical haptic renderer.
- Mycelix #5899 owns sensor provenance/calibration lineage; #5985 owns the serialized article and configuration lifecycle.
- Mycelix #3084 owns generic physical product revisions, manufactured units and qualification receipts.

Use those exact owner records and references. Do not create a parallel calibration authority, lifecycle database, sensor identity scheme or clinical recommendation engine.

## 8. Claim ceiling

This proposal establishes a versioned schema and synthetic example only. It establishes no human body measurement, clinically valid fit, tissue safety, pressure threshold, comfort, reduced migration, physical device clearance, manufacturing tolerance, regulatory compliance or clinical benefit.
