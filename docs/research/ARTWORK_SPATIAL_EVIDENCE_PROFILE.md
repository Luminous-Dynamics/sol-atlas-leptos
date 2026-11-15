# Artwork Spatial Evidence Profile
**Status:** research/design specification v0.1  
**System owner:** Sol-Atlas  
**Integration:** consumes stable artwork identity from Mycelix and supplies geometry/scene evidence to Symtropy and Symthaea  
**Scope:** spatial intelligence for artworks, sculpture, and installations; not an artwork authentication system

## 1. Objective

Define a portable, rights-aware spatial evidence profile for documenting physical artworks and installations. Sol-Atlas should represent geometry, coordinate frames, object parts, installation context, capture lineage, and uncertainty so that later users can compare captures and reconstruct spatial relationships without confusing a visual rendering with metrologically reliable geometry.

The art domain is an application of Sol-Atlas's general spatial-representation capability; it does not change the system's core identity into a provenance registry or curatorial authority.

## 2. Paradigm shift: from 2D catalogue images to uncertainty-aware spatial records

A spatial record should connect:
- stable artwork/passport ID;
- capture event and operator/device;
- camera/sensor calibration;
- image, point-cloud, mesh, and optional radiance/splat assets;
- coordinate reference frame and unit system;
- transformation chain between local artwork, room, building, and map frames;
- segmentation and semantic labels with authorship/source;
- uncertainty and areas not observed;
- rights, access, and cultural restrictions.

A photorealistic NeRF or Gaussian splat is not automatically a dimensionally accurate 3D model. Keep visual appearance assets distinct from metric geometry assets.

## 3. Spatial data profile

### Object and scene structure
- Artwork root node and stable object identifier.
- Component/part hierarchy for multi-part sculpture and installations.
- Geometry references: point clouds, meshes, depth maps, and calibrated image sets.
- Appearance references: texture maps and optional radiance/splat assets.
- Semantic annotations: material, inscription, join, damage feature, mounting point, or installation element, each with source and confidence.
- Spatial relationships: contains, attached-to, supported-by, adjacent-to, displayed-in, oriented-toward, and part-of.

### Coordinate and capture provenance
- Explicit coordinate frame, handedness, axis convention, units, and origin definition.
- Sensor/camera model, calibration digest, capture settings, timestamps, and operator.
- Registration transform with source/target frames, algorithm, parameters, residuals, and review status.
- Georeferencing only when needed and authorized; never publish exact storage coordinates for high-value or culturally restricted objects by default.

### Uncertainty and completeness
- Per-asset and, where feasible, per-region uncertainty.
- Occlusion and unobserved-surface mask.
- Scale uncertainty and reference-measurement evidence.
- Distinguish measured, interpolated, reconstructed, manually modelled, and unknown geometry.
- Capture quality flags: blur, reflections, specularity, translucency, low texture, motion, calibration drift, and coverage gaps.

### Rights and access
- Separate rights for photographs, source captures, meshes, textures, and derived models.
- Capture consent, cultural permissions, embargo and access profile.
- Public/controlled/restricted/sealed delivery modes.
- Do not expose sensitive object locations or restricted iconography through embeddings, previews, metadata, or search indexes.

## 4. Evaluation protocol

Create a small, rights-cleared benchmark of objects with repeatable reference captures. Include paintings in frames, textured and reflective sculpture, polished metal, translucent materials, carved relief, multi-part installations, and low-texture surfaces.

For each object:
1. Acquire independently calibrated reference geometry (e.g. structured-light or laser scan where appropriate).
2. Acquire the candidate image-based and/or depth-based capture under declared conditions.
3. Freeze calibration, reconstruction version, camera poses, coordinate transforms, and evaluation masks.
4. Compare candidate geometry with reference geometry in the same frame.
5. Repeat from a different operator/device/site to test transfer and repeatability.

Report:
- point-to-plane RMSE and robust percentiles;
- symmetric Chamfer distance and F-score at declared tolerances;
- normal consistency;
- scale and registration error;
- feature/edge/inscription preservation;
- region-specific error rather than only one global average;
- completeness/occlusion and uncertainty calibration;
- colour difference only where capture colour calibration supports it;
- runtime, memory, operator time, and capture cost.

A single global Chamfer score can obscure lost inscriptions, tool marks, thin components, or sharp relief. Add feature-weighted evaluation with a published annotation protocol and compare it with global metrics. Do not claim that a visual splat is metrically accurate without independent geometry validation.

## 5. Baselines and comparison

Use a staged baseline ladder:
1. calibrated multi-view photogrammetry / structure-from-motion and multi-view stereo;
2. depth/structured-light or laser-scan reference where feasible;
3. current best available open reconstruction pipeline for the exact capture conditions;
4. optional NeRF / Gaussian splatting for view synthesis, scored separately from metric geometry.

The target is not merely better image similarity. It is better geometry fidelity at a declared capture budget, with fewer missing critical features, known uncertainty, reproducible transforms, rights-aware access, and independently inspectable evidence.

## 6. Cross-system handoff

- **Mycelix:** provides the canonical artwork ID, capture rights, signed evidence references, and provenance lineage.
- **Sol-Atlas:** owns coordinate frames, geometry, scene graph, transforms, segmentation, and spatial uncertainty.
- **Symtropy:** consumes metric geometry and boundary conditions for bounded physical/transport scenarios.
- **Symthaea:** retrieves and explains spatial evidence, identifies missing views or inconsistent transforms, and helps curate follow-up capture plans.

No system may infer legal ownership or authenticity from geometric similarity. Exact location and sensitive cultural information remain access-controlled.

## 7. Initial acceptance criteria

- A documented schema represents object parts, capture event, assets, coordinate frames, transform lineage, uncertainty, and rights.
- At least 3 distinct geometry classes and 2 independent capture workflows are evaluated.
- Every benchmark result identifies reference instrument, calibration, units, masks, software digest, and capture conditions.
- Geometry and visual appearance results are reported separately.
- No exact private storage location is included in public fixtures.
- A held-out object and held-out capture device/site test are included before any generalization claim.
