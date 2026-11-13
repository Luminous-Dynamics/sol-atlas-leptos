# EARTH-OBSERVABILITY-QUAL-001
## Multimodal subsurface evidence fusion, calibrated uncertainty, and adaptive investigation

**Status:** research specification v0.1  
**Date:** 2026-11-13  
**Owner:** Sol-Atlas (spatial evidence integration and benchmark orchestration)  
**Supporting owners:** Symtropy (survey and physical-model qualification), Symthaea (hypothesis generation and investigation planning), Mycelix (provenance, custody, rights and accountable review)  
**Claim ceiling:** research and decision support. This specification does not qualify a site, establish ground truth, certify a survey, approve excavation, or authorize infrastructure access.

---

## 1. Objective

Create a reproducible, vendor-neutral benchmark for building and updating a four-dimensional, evidence-qualified representation of the terrestrial and subterranean environment.

The benchmark asks whether an integrated system can do more than display heterogeneous map layers:

1. preserve the identity, rights, time, coordinate frame, measurement process and uncertainty of every source observation;
2. reconcile surface, borehole, geological, geophysical, hydrological, scan and operational evidence without erasing disagreements;
3. estimate what is known, what is unresolved, and where the current observation process is incapable of seeing;
4. quantify uncertainty in derived geological and infrastructure interpretations;
5. select the next survey or investigation that most improves a specified decision under cost, access, time, disturbance and safety constraints;
6. compare predictions with later independent observations and trigger scoped revalidation when evidence changes.

The unit of progress is not a map tile or terabyte. It is an independently testable reduction in decision-relevant uncertainty at a declared spatial scale and cost.

## 2. Composition: do not create parallel owners

This task is an integration and benchmark owner. It composes existing work:

- Sol-Atlas [#450 — SOLATLAS-CIV-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/450): canonical spatial/temporal provenance and explicit missingness.
- Sol-Atlas [#561 — Johannesburg data readiness](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/561): candidate source inventory and rights/access qualification.
- Sol-Atlas [#134 — spatial investigation sampling and EVSI](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/134): spatial sample-plan optimization.
- Sol-Atlas [#755 — MUDDI-aligned subterranean utility profile](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/755): underground built-infrastructure logical model and disclosure boundary.
- Sol-Atlas [#138 — built-environment evidence graph](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/138): versioned artifacts, evidence links, receipts and revalidation.
- Symtropy [#1607 — survey evidence](https://github.com/Luminous-Dynamics/symtropy/issues/1607): control adjustment, frames, registration, uncertainty and change detection.
- Symtropy [#4910 — Subterranean Utility Fabric](https://github.com/Luminous-Dynamics/symtropy/issues/4910): system economics, utility-fabric requirements and physical workstreams.
- Symtropy [#4972 — distributed fibre sensing](https://github.com/Luminous-Dynamics/symtropy/issues/4972): modality-specific distributed acoustic sensing qualification.

Ownership boundaries:

- Sol-Atlas owns source-preserving spatial integration, claim lineage, temporal/spatial query, coverage/uncertainty projection and benchmark orchestration.
- Symtropy owns modality-specific sensor, survey, physical and simulation qualification. It must not inherit a claim merely because Sol-Atlas can render a result.
- Symthaea may propose hypotheses, compare investigation actions and explain information value; it does not authorize drilling, excavation, access or public disclosure.
- Mycelix may preserve identity, custody, rights, review and approval evidence; a valid record does not make the measured claim physically true.
- Geological, geotechnical, hydrological, land, utility and other competent authorities remain authoritative for their own determinations.

## 3. Paradigm shift

### 3.1 From a single 3D model to a versioned field of claims

Do not create one apparently authoritative voxel volume for the whole Earth.

Represent the environment as linked, scale-aware objects and fields:

- terrain surfaces and point-cloud assets;
- geological units, interpreted boundaries and structural features;
- borehole trajectories, depth intervals, samples and tests;
- geophysical survey geometries, measured responses and inversion products;
- groundwater sites, screened intervals, levels, chemistry and time series;
- known underground spaces, assets and network features;
- excavation, as-built geometry and structural monitoring observations;
- interpretations and model outputs that reference, but never overwrite, their inputs.

A model is a versioned interpretation over a declared domain, scale, time and evidence set. Conflicting interpretations may coexist. A new observation creates a new evidence revision and may invalidate dependent claims; it must not rewrite historical results.

### 3.2 From coverage to observability

A sensor footprint is not the same as the ability to detect a target.

For each acquisition, characterize where and under what conditions the target was observable:

- spatial footprint and depth/range of investigation;
- resolution, sampling geometry and point/trace spacing;
- detection probability or minimum detectable contrast where established;
- signal-to-noise and coupling/penetration conditions;
- calibration, clock, position and orientation quality;
- occlusion, blind zones, environmental confounders and rejected samples.

A non-detection is admissible only within the demonstrated observation domain and detection limit. Outside that domain it is **UNOBSERVED**, not **ABSENT**.

### 3.3 From static mapping to active investigation

A map should not only report uncertainty. It should make uncertainty actionable.

For a named engineering or environmental decision, the system should identify candidate additional measurements, estimate how each could change the posterior and downstream decision, and report expected information/decision value separately from financial cost, delay, access burden and disturbance. Mandatory safety, legal, land, privacy, ecological and cultural obligations are hard constraints, not tradeable weights.

### 3.4 From point accuracy to uncertainty-aware geometry

Every relevant spatial claim needs:

- horizontal CRS, axis order, units and datum;
- vertical datum and height type;
- coordinate epoch where applicable;
- local frame identity, handedness and origin where relevant;
- exact transform operation and versioned grid/parameter resources;
- measurement, control and transformation uncertainty;
- spatial support and dimensionality;
- observation time, valid time and source revision.

An ICP residual or internal bundle-adjustment precision is not absolute accuracy. Registration/calibration observations must be kept separate from independent checkpoints used for qualification.

## 4. Interoperability profile

Reuse domain standards and adapters. Do not invent a universal replacement.

| Domain | Interoperability anchor | Intended use |
|---|---|---|
| Underground built space and utilities | OGC MUDDI Part 1 v1.0 | Cross-model concepts for underground assets, spaces, networks and evidence; concrete local logical profile remains use-case-specific |
| Geological units, structures, boreholes, specimens and analyses | OGC GeoSciML 4.1 | Geological feature and investigation exchange |
| Groundwater | OGC WaterML 2 Part 4 / GWML2 2.2.1 | Groundwater domain model and exchange |
| Sensors, systems and observations | OGC API - Connected Systems 1.0; SensorThings API 1.1 | Systems/deployments, procedures, observations, events and sensor metadata |
| Generic spatial features | OGC API - Features | Read/query access to spatial feature collections |
| Spatiotemporal environmental data | OGC API - EDR 1.2 | Position, area, trajectory, corridor and related query patterns |
| Point clouds, imagery, DEMs and survey artifacts | STAC 1.1.0 and appropriate asset-format profiles | Discovery, metadata, provenance and spatiotemporal cataloguing |
| Large-scale 3D rendering | OGC 3D Tiles 1.1 | Streaming/rendering tiles, not a physical truth or geoscience schema |
| 3D built/urban context | CityGML 3.0 conceptual model | Semantic city objects and their relationships |
| Data-quality description | ISO 19157-1:2023 | Quality elements, measures and reporting |

Adapters must preserve source semantics and report lossy transformations. MUDDI, GeoSciML, GWML2, SensorThings, Connected Systems, STAC, CityGML and 3D Tiles solve different parts of the problem; none alone is the complete Earth-observability contract.

## 5. Evidence and uncertainty contract

Every record must state its epistemic role:

- **RAW_OBSERVATION**: immutable source measurement or artifact;
- **PROCESSED_OBSERVATION**: deterministic/algorithmic transformation of source data;
- **INTERPRETATION**: human or model interpretation, with author/method;
- **MODEL_OUTPUT**: prediction, inversion, interpolation or posterior;
- **INDEPENDENT_VALIDATION**: reference evidence reserved for evaluation;
- **DECISION_OR_REVIEW**: an accountable decision record, separate from observations.

Every record must also carry an availability/currentness state. Minimum distinct states:

**PRESENT**, **PARTIAL**, **UNKNOWN**, **UNOBSERVED**, **NOT_ACQUIRED**, **WITHHELD**, **RESTRICTED**, **STALE**, **CONFLICTED**, **INVALID**, **SUPERSEDED**, **OUT_OF_DOMAIN**.

Do not collapse these states into null, zero or a single confidence scalar.

Uncertainty dimensions must remain distinguishable where material:

1. instrument/metrology uncertainty;
2. positional and frame-transformation uncertainty;
3. sampling and coverage uncertainty;
4. geophysical inverse-problem/model uncertainty;
5. geological spatial variability;
6. epistemic/model-form uncertainty;
7. temporal/change uncertainty;
8. source, rights and custody uncertainty.

Do not add independent variances where errors are correlated. Preserve covariance, ensemble or another declared representation and document assumptions. A confidence score without a calibration domain, target variable, method and validation set is not an uncertainty qualification.

## 6. Benchmark tracks

### Track A — Source and spatial conformance

Test CRS, axis order, units, vertical datum, epoch, coordinate transforms, source digests, temporal semantics, rights state, coverage, schema mapping and deterministic replay.

Negative controls include wrong CRS with plausible coordinates, ellipsoidal/orthometric height confusion, swapped axes, unpinned grid files, future/stale timestamps, source replacement, missing vertical reference, duplicate IDs, invalid transforms, withheld content and restricted utility geometry.

### Track B — Survey geometry and change

Compare control adjustment, scan registration, surface/volume reconstruction and repeat-survey change detection against independent reference control.

Report internal registration residual separately from absolute checkpoint error. Report the minimum detectable change and spatially varying uncertainty; do not treat every surface-difference pixel as physical movement.

### Track C — Subsurface reconstruction

Evaluate declared targets separately:

- geological contact/interface position and depth;
- lithology or material-class distribution;
- bedrock depth;
- fault/fracture geometry where independently labelled;
- groundwater level/pressure surface and its temporal change;
- cavity/void or utility presence only where a suitable reference standard exists.

Compare matched methods: mapped geology/manual expert baseline, borehole-only interpolation, ordinary/indicator kriging where assumptions fit, single-modality inversion/interpreta­tion, specialist implicit geological modelling, and multimodal fusion. Preserve separate benchmark tracks where the target or evidence basis differs.

### Track D — Uncertainty calibration and abstention

For continuous targets report MAE/RMSE/bias, median and P95 absolute error, interval coverage and interval width, and a proper probabilistic score such as CRPS where suitable.

For categorical targets report class-wise precision/recall, confusion costs, Brier/log score and calibration/reliability curves. Include out-of-domain and abstention coverage-risk curves. Aggregate metrics must not conceal a geology/site class with severe undercoverage.

### Track E — Adaptive investigation and value of information

Compare:

1. fixed-grid / fixed-spacing investigation;
2. expert-designed plan, where documented;
3. random or cost-only sampling;
4. greedy uncertainty/information reduction;
5. mechanism- and decision-aware EVSI/ENBS plan;
6. oracle plan using withheld truth (upper bound only).

Hold the total declared budget or field effort constant. Report expected decision loss/regret, uncertainty reduction, accepted decision quality, survey cost, elapsed time, access burden, disturbance, and value-of-information calibration. A plan that selects a more informative but unauthorised or unsafe survey is ineligible.

### Track F — Temporal assimilation and model revalidation

Introduce new measurements, corrected source versions, sensor drift, calibration changes, construction/excavation events and genuine physical change. Check that only dependent artifacts are marked for revalidation, old receipts remain historically reproducible, and stale model outputs cannot silently remain current.

## 7. Dataset and split protocol

Use three evidence levels:

**Level 0 — Synthetic contract fixtures.** Known geometry, properties, noise, covariance, correlated registration drift, modality-specific blind zones, conflicting sources and controlled change. These establish software properties, not field accuracy.

**Level 1 — Public or permissioned retrospective sites.** Exact source snapshots, licences, data dictionaries, survey reports and reference measurements are admitted before model selection. Unknown licences remain non-ingestible for redistribution. Public visibility is not permission to republish.

**Level 2 — Prospective independent field validation.** A separate survey/engineering team holds checkpoint and borehole truth, preregisters tolerances and methods, and evaluates blind predictions before revealing the reference observations. Field validation is specific to the instruments, site, geology, depth, method and conditions tested.

Split by site, survey campaign, borehole, instrument/rig, geological domain and time as appropriate. Never randomly split adjacent traces, points, tiles, depth intervals or repeated windows from the same physical acquisition across training and test. Run asset-identity, spatial-overlap and near-duplicate audits. Keep tuning data separate from independent checkpoints.

## 8. Proposed comparator landscape

The benchmark should not assume that a general evidence platform beats specialist engineering software at its core task.

- Bentley OpenGround is a direct comparator for geotechnical data management, borehole/test/lab/monitoring workflows and collaboration.
- Seequent Leapfrog Works is a direct comparator for implicit 3D geological modelling, borehole-based interpretation, updates and integration of geophysics/hydrogeology.
- GIS platforms and open spatial databases are comparators for feature ingestion, spatial query, cataloguing and visualization.
- Specialist geophysical inversion, surveying and groundwater tools remain reference engines for their own qualified domains.

The defensible initial differentiation is cross-tool evidence lineage, uncertainty/observability semantics, independent validation, multi-modal disagreement management, and decision-aware next-investigation selection. Any claim of numerical superiority must use the same data, reference truth, hardware/time budget, target and metric as the comparator.

## 9. Acceptance gates and research targets

### Non-negotiable software/evidence invariants

- Zero wrong-frame or incompatible-unit records accepted silently.
- Zero missing mandatory datum/vertical-reference/calibration data promoted to qualified geometry.
- Zero unknown or unobserved underground conditions represented as confirmed absence.
- Zero synthetic observations presented as physical measurements.
- Zero raw-source lineage loss for an admitted derived claim.
- Zero validation leakage from fit/control data into independent checkpoint metrics.
- Zero unauthorized disclosure of restricted underground infrastructure details in the frozen access-control corpus.
- Deterministic replay for frozen inputs, transformations, policy and software identity.
- Every model output has an applicability domain and explicit abstention path.
- Every dependent result affected by a changed evidence digest is marked for revalidation; history is not rewritten.

### Quantitative targets (proposed, not achieved)

- Calibration: nominal 90%/95% predictive intervals should meet preregistered empirical-coverage tolerances on blind data, with coverage and width reported by site, geology, modality and depth.
- Reconstruction: improve held-out boundary/depth error and probabilistic scores against each matched conventional baseline; do not set a universal centimetre target before acquisition scale and reference quality are known.
- Investigation efficiency: test a stretch hypothesis of at least 20% lower expected decision loss at equal investigation budget, or at least 20% lower budget for equivalent preregistered decision quality, versus fixed-grid and single-modality baselines.
- Robustness: no degradation hidden by aggregate scores; report worst-domain and leave-one-site/region/rig-out results.
- Evidence quality: 100% of admitted benchmark predictions bind exact inputs, model/build/configuration, transformation lineage and evaluation split.

The 20% efficiency figure is an aspirational hypothesis for the tournament, not a claimed result or guaranteed acceptance threshold. It must be preregistered after baseline variance and field costs are known.

## 10. Initial South African pilot

Use Johannesburg as the surface and source-qualification pilot, while keeping the first engineering target geographically bounded.

Candidate sources:

- City of Johannesburg 2023 orthophotography, LiDAR and contours: City page reports 15 cm imagery, minimum 4 points/m² and 25 cm derived contours across approximately 1,644 km². Exact tile availability, classification, accuracy report, CRS, vertical datum, rights and tile-level quality must be verified before qualification. A density figure alone does not establish a USGS QL or a vertical-accuracy class.
- DWS National Groundwater Archive: DWS describes over 294,500 geosites and explicitly says the NGA is point-focused. Access/export is registration-based; treat data access, reuse terms and field completeness as unresolved until verified.
- DWS stored borehole distribution: March 2026 map documents 283,030 stored records and warns it does not depict all existing boreholes or groundwater potential.
- Council for Geoscience geological maps/data portal: candidate source; identify exact layer/report, native scale, download mechanism, licence and lineage per asset before use.
- Copernicus/other global terrain and satellite data: regional context and change screening, not a substitute for local ground control or site investigation.

The Johannesburg pilot should produce a source/coverage/rights/quality atlas first. The ISMG corridor pilot should consume this profile later, when a particular candidate corridor and its authorized survey evidence have been selected. Do not create sensitive utility, mine-workings or private borehole coordinates for a public demo.

## 11. Phasing

**P0 — Freeze semantics and benchmark protocol.** Define the evidence roles, uncertainty dimensions, spatial/temporal frame contract, benchmark tasks, splits and claim ceiling. Publish synthetic fixtures and negative controls.

**P1 — Interoperability adapters.** Crosswalk MUDDI, GeoSciML, GWML2, OGC Connected Systems/SensorThings, STAC and OGC APIs. Preserve unknown, withheld, conflict and rights states in round trips.

**P2 — Reproduce conventional baselines.** Reproduce simple interpolation and geophysical/survey baselines on frozen synthetic and permitted retrospective data before proposing a new fusion method.

**P3 — Adaptive survey tournament.** Evaluate spatial and method selection under matched budgets, including fixed-grid, expert and oracle comparisons. Integrate with #134 rather than forking the EVSI implementation.

**P4 — Prospective validation.** Obtain permissioned independent control and field data. Keep physical safety and authorization outside the optimizer.

**P5 — Operational feedback.** Consume qualified as-built, TBM, utility, groundwater and structural telemetry; trigger scoped revalidation when changes affect dependent claims.

## 12. Open questions to resolve before real data ingestion

1. Which exact Johannesburg tiles, classification, accuracy reports and licence terms can CGIS provide?
2. Which CGS geological layers can be downloaded, redistributed and cited with versioned metadata?
3. What are the DWS NGA export fields, sampling/quality semantics, owner restrictions and reuse conditions?
4. Which first pilot decision is being improved: corridor alignment, foundation design, groundwater management, utility avoidance or another bounded problem? The relevant observation model and loss function depend on this.
5. Which independent reference measurements can be held blind, and who is qualified to adjudicate them?
6. Which proprietary tools/datasets are available for matched baseline reproduction? If unavailable, record that comparator lane as unrun rather than inferred.

## 13. Research references

- OGC MUDDI Part 1 v1.0: https://docs.ogc.org/is/23-024/23-024.html
- OGC GeoSciML 4.1: https://www.ogc.org/standards/geosciml/
- OGC GroundWaterML 2.2.1: https://www.ogc.org/standards/gwml2/
- OGC SensorThings API 1.1: https://docs.ogc.org/is/18-088/18-088.html
- OGC API - Connected Systems: https://ogcapi.ogc.org/connectedsystems/
- OGC API - Features: https://ogcapi.ogc.org/features/
- OGC API - Environmental Data Retrieval 1.2, published 2026-09-08: https://docs.ogc.org/is/19-086r9/19-086r9.html
- OGC STAC 1.1.0: https://www.ogc.org/standards/stac/
- OGC 3D Tiles 1.1: https://www.ogc.org/standards/3DTiles/
- OGC CityGML 3.0: https://www.ogc.org/standards/citygml/
- ISO 19157-1:2023: https://www.iso.org/standard/78900.html
- USGS 3DEP Lidar Base Specification 2025 Rev. A: https://www.usgs.gov/ngp-standards-and-specifications/lidar-base-specification-online
- USGS topographic quality levels: https://www.usgs.gov/3d-elevation-program/topographic-data-quality-levels-qls
- City of Johannesburg CGIS 2023 aerial products: https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx
- DWS groundwater data management: https://www.dws.gov.za/Groundwater/data.aspx
- DWS NGA borehole distribution, March 2026: https://www.dws.gov.za/groundwater/data/boreholedist.aspx
- Bentley OpenGround: https://www.bentley.com/products/openground/
- Seequent Leapfrog Works: https://www.seequent.com/products-solutions/leapfrog-works/

