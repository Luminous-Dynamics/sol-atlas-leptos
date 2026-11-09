# Highveld Keep — Benchmark Protocol v0.1

**Scenario:** `HIGHVELD-KEEP-SYNTHETIC-001`  
**Parent:** [HIGHVELD-KEEP-001 #607](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/607)  
**Status:** proposed protocol; no candidate results or comparative advantage claimed  
**Evidence tier:** E0 until frozen synthetic inputs and independent evaluators exist

## 1. Evaluation question

Can a human-led, evidence-bounded Luminous workflow produce a more complete set of feasible, beautiful, climate-responsive and manufacturable dwelling alternatives than matched conventional workflows, while preserving hard life-safety/accessibility constraints and maintaining traceability from intent to fabrication?

The benchmark does not ask whether an AI can draw a castle. It asks whether the workflow improves the quality, variety, feasibility, evidence completeness and production readiness of design decisions.

## 2. Matched alternatives

All spatial families share the same resident/task brief:
- A — Courtyard Cloister.
- B — Winged Great Hall.
- C — Compact Keep.
- D — Garden Pavilions.

For every spatial family, compare at least two material/production pathways:
- M1 — conventional masonry / reinforced concrete / steel baseline;
- M2 — masonry/stone expression using a conventional qualified structural frame;
- M3 — timber/timber-hybrid roof or upper structure, only where supply/grade/fire/durability/moisture evidence supports it;
- M4 — panelized or precast package;
- M5 — bounded robotic fabrication of a selected timber/panel subassembly;
- M6 — narrowly qualified additive component or formwork, only if a real application and acceptance method are established.

A material pathway is not presumed feasible because it can be represented in a model. Every candidate carries a specific product/system approval evidence state and manufacturing route.

## 3. Competitive baselines

Compare the same task brief, inputs and output acceptance requirements across:

1. **B0 — Professional conventional workflow:** architect-led design, established BIM authoring, qualified consultants and specialist solvers.
2. **B1 — Parametric workflow:** Rhino/Grasshopper or equivalent parametric authoring with specialist environmental/structural tools.
3. **B2 — Connected AEC workflow:** Forma/Revit-class connected planning/design/BIM workflow.
4. **B3 — Open toolchain:** IFC/IDS exchange with specialist thermal/structural/daylight tools, where applicable.
5. **B4 — Luminous orchestration:** Symthaea requirements/alternative exploration, exact qualified geometry exchange, Symtropy solver adapter coordination, Sol-Atlas context, Mycelix evidence/provenance, and qualified professionals.
6. **P0 — Physical production controls:** manual and assisted construction/fabrication.
7. **P1 — Physical robot-assisted route:** same accepted physical component produced using a guarded, calibrated, capability-matched workcell and trained operators.

Do not compare a Luminous render with an industry-approved construction package and call it a fair contest. Each workflow must deliver the same declared package, or its missing deliverables must remain visible.

## 3A. Design-search algorithm benchmark

The generative-search comparison must test the *quality and cost of decisions*, not the number of rendered alternatives.

A 2026 comparative study of building-performance optimization compares multi-objective algorithms using Hypervolume, IGD+, GD+, epsilon indicators, repeated stochastic runs and matched problem settings. It reports that algorithm suitability varies with the building optimization problem; its reported RBFMOpt/HypE results are not evidence that either is universally best or will win on Highveld Keep. A 2025 systematic review of 153 performance-driven building-design studies identifies fragmented metric conventions, strong emphasis on envelopes/massing and energy/daylight/thermal comfort, and limited cross-case/field validation. These findings support a matched, reproducible benchmark rather than a claim based on design count or visual novelty.

### Algorithm families to compare

Choose only methods appropriate to the declared variable/objective structure; pin implementation and settings:
- transparent baselines: exhaustive enumeration on tiny discrete cases, random sampling and space-filling designs;
- evolutionary multi-objective: NSGA-II, NSGA-III, SPEA2 and HypE;
- decomposition-based: MOEA/D;
- surrogate-assisted: RBFMOpt or an explicitly documented RBF workflow, Gaussian-process, radial-basis, random-forest or neural-network surrogates;
- a transparent human/architect-led candidate set from the same design brief.

Do not choose an algorithm based on reputation. Pre-register the comparison and publish failures, exclusions and hyperparameter/seed settings.

### Fair budget protocol

- Give all algorithms the same candidate variable bounds, hard constraints, objective definitions, initial-design budget and high-fidelity solver-call budget.
- Count all high-fidelity calls used for surrogate training, tuning, validation and final verification. Report low-fidelity calls separately.
- Report wall time, CPU/GPU resources, peak memory, solver-license limits, failed runs and human intervention; do not hide surrogate training or failed evaluations.
- Run stochastic methods across at least 20 fixed, published seeds as an initial comparison; increase the sample size where power/variance analysis requires it.
- Use identical exact high-fidelity re-evaluation for every candidate entering the final comparison. A surrogate prediction is never a final building-performance result.
- Keep training, calibration and held-out cases separate. A surrogate that performs well on its training weather/material/site envelope may still be out of domain on another.
- On small synthetic subproblems, calculate an exact feasible Pareto reference by exhaustive enumeration. Do not treat an optimizer's own output as the reference front.

### Optimization and diversity metrics

Report the following as a vector, with metric normalization and reference points frozen before each run:
- feasible-candidate fraction and hard-gate violations (hard failures cannot be repaired by the objective score);
- Hypervolume with a predeclared reference point;
- IGD+ and GD+ only when a defensible reference Pareto set exists;
- epsilon indicator and spread/distribution indicators where suitable;
- Pareto coverage and regret on exact small fixtures;
- high-fidelity evaluations to first feasible candidate and to stable Pareto coverage;
- total simulation calls, compute time, failures and repeat-run variability;
- sensitivity to seeds, objective scaling, preference perturbations and uncertain inputs.

Also evaluate **architectural diversity**, separately from optimizer-distribution diversity. Cosmetic changes to colour, texture or facade material alone do not count as structurally distinct alternatives. Use inspectable signatures such as spatial topology/adjacency, courtyard organization, circulation sequence, storey count, massing decomposition, service routing and structural/material family. A diverse front that contains inaccessible or infeasible candidates is not a successful design-search result.

### Multi-fidelity verification

1. Use geometry-only and cheap rule checks to screen malformed or clearly infeasible candidates.
2. Use inexpensive approximations for exploratory ranking, with model/domain/version and uncertainty attached.
3. Promote only candidates with a declared reason to specialist structural, thermal/energy, daylight or other solvers.
4. Re-evaluate final Pareto candidates with the exact pinned high-fidelity tools and complete input manifests.
5. If a high-fidelity result changes feasibility or ranking, update the Pareto set and preserve the discrepancy; do not overwrite the surrogate result as if it had been correct.

A surrogate is a resource-allocation tool for deciding which candidates deserve expensive evaluation. It is not a substitute engineering authority.

## 3B. Functional relationship and task-network benchmark

The scenario's 11 relationships form an explicit task-level graph, not a geometric floor plan. Benchmark the graph in two classes:

**Hard relationships (7 instances):**
- step-free links from arrival to the great hall, kitchen/dining, work/making and outdoor living;
- arrival-level access to the accessible washroom;
- visitor route from arrival to the great hall avoids sleeping rooms;
- service/maintenance access between service plant and arrival avoids sleeping rooms.

These must be reported as satisfied, failed or unknown by a qualified accessibility/spatial/route evaluation once exact geometry and the applicable professional criteria exist. A missing route or missing geometry proof is Unknown, not satisfied. A known hard failure cannot be offset by an aesthetic or energy gain.

**Preferences (4 instances):**
- kitchen/dining adjacency to outdoor living;
- library/quiet acoustic separation from the great hall;
- sleeping-room acoustic separation from the great hall;
- work/making adjacency to service/plant.

Preferences remain negotiable and should be evaluated through resident review and appropriate spatial/acoustic analysis. They must not silently become hard statutory constraints or be inferred from household identity.

**Metric separation:**
- requirement-graph integrity: valid endpoints, unique IDs, type/class consistency and complete reasons;
- hard-relationship coverage: satisfied / failed / unknown, with source and reviewer;
- preference outcomes: explicit distance/acoustic evidence where available plus resident-rated trade-offs;
- path/access metrics: actual measured geometry, gradients, clear widths, turning/threshold constraints and egress analysis only after an exact geometry provider and qualified criteria are available;
- acoustic outcomes: specialist solver or measured evidence, not inferred from room adjacency.

Do not assign a numeric distance or geometric path to these symbolic links in E0. “Should be adjacent” is an intent label, not a claim that two rooms are within a specified number of metres. “Acoustically separated” is a preference, not a claim of an acoustic rating.

### Relationship-graph negative controls

Add exact mutation fixtures:
- target function does not exist in the functional programme;
- source or target ID duplicated in the functional programme;
- relationship ID missing or duplicated;
- a `hard` route relation is changed to `preference`;
- a negotiable adjacency/acoustic relation is promoted to `hard` without resident/authority review;
- `via_excluded` references an unknown/private function not in the programme;
- a relationship adds `distance_m`, geometry, path coordinates or another unversioned geometric field;
- a hard path is marked satisfied despite absent exact geometry/qualified accessibility assessment;
- preference disagreement is collapsed into one fabricated household preference.

The v0.1 manifest checker covers cross-file graph integrity and type mapping only. The remaining geometry, route, accessibility and acoustic evaluation requires later qualified providers and reviewers.

## 3C. Climate-source comparison and data-fitness protocol

The first source candidates are five Climate.OneBuilding Gauteng 2011–2025 TMYx files (Tambo, Botanical Gardens, Grand Central, Rand Airport and Lanseria), preserved as a regional station ensemble rather than a single chosen site file. A second provider methodology, EPW.tools, is recorded separately as research-only. Its own pages describe station-first fusion, ERA5/MERRA-2 gap filling, satellite solar where available, and AMY/TMY/future products; its published ASHRAE and SURFRAD validation is provider-reported and has not been independently reproduced locally.

Compare source methods without collapsing their product classes:

| Product class | Proper role | Not a substitute for |
|---|---|---|
| TMY / TMYx | Typical-year energy/design screening | Heatwave, hottest-year, future climate or operational calibration |
| AMY | Exact-hour historical model calibration and measurement/verification when the same year has metered building observations | Typical design year, design extreme or future scenario |
| Historical heatwave/extreme episode | Stress testing against an explicitly sourced actual episode | TMY average, future projection or statutory design condition |
| Future climate series | Lifecycle sensitivity under declared GCM/SSP/period/method | Observed future weather or guaranteed forecast |
| Site/station measurements | Local validation and calibration, subject to sensor quality and coverage | A citywide trend or distant station without a representativeness analysis |

For every provider/file, record:
- provider and file/product identity;
- source datasets and source period;
- exact coordinates, elevation, time zone and spatial grid/station;
- hourly variables, units, missingness, substitutions and solar provenance;
- EPW/STAT/DDY headers and all comments;
- archive and extracted-file SHA-256;
- reuse licence and redistribution restrictions;
- station-to-site distance/elevation and representativeness uncertainty;
- independent comparison to local station measurements where possible;
- exact downstream model and claim using the file.

EPW.tools' Johannesburg trend page is based on a 0.5-degree NASA POWER/MERRA-2 grid cell and explicitly notes that it averages the city and surroundings; do not treat its reported +0.84 °C 1981–1990 to 2016–2025 change as a central-city thermometer or parcel-specific trend. Its published station validation should be reported with its actual scope (e.g. the seven NOAA SURFRAD sites for hourly checks), not generalized to South Africa.

The source benchmark should score file fitness/coverage and uncertainty, not decide a provider winner by marketing claims. Do not average two TMY files from different locations or providers without an explicit, justified method and sensitivity analysis.

## 3D. Station identity and location-history benchmark

The source crosswalk contains candidate metadata for Johannesburg Botanical (WMO 683610) and Johannesburg International (WMO 683680 / GHCND `SFM00068368` / WIGOS `0-20000-0-68368`). Published catalogues disagree slightly on Johannesburg International coordinates and provide different periods and data types. The station registry itself may distinguish a reference point from individual instrument coordinates.

Before using any station series:
- retrieve the direct WMO OSCAR/Surface and SAWS station records;
- obtain NOAA NCEI GHCND and Global Hourly/ISD station history, including historical coordinates, equipment, reporting schedule and data inventory;
- reconcile WMO/USAF/WBAN/GHCND/ICAO/SAWS/WIGOS IDs and effective dates;
- retain original and revised coordinates with provenance and time validity;
- measure station-to-parcel distance/elevation and exposure differences;
- determine whether the TMYx and observation series share the same upstream records (avoid false independence in validation);
- distinguish daily rainfall or daily summary records from hourly temperature/wind/solar data;
- keep secondary aggregators as discovery aids, not primary station authorities.

A source coordinate update or station move must invalidate dependent representativeness assessments and any weather model using the affected station snapshot. Never rewrite historic coordinates to the current station point without evidence of instrument continuity or a homogenization method.

## 3E. 2026 industry baseline and fair comparison

The Luminous workflow must be compared with working industry capabilities, not a deliberately weak manual baseline. Product capabilities below are drawn from vendor/standards-owner publications checked 7 November 2026. They are evidence of advertised/current product workflows, not independent proof of performance, regional availability or fitness for a specific South African project.

| Industry baseline | Publicly documented capability | Highveld comparison target |
|---|---|---|
| Autodesk Forma Building Design | Cloud schematic design; site context; automated floor plans, modular façades and unit mix/area metrics; integrated sun-hours, façade/window daylight potential and total-carbon analysis; geolocated native Revit handoff. | Match the same task/site input; compare option throughput, data fidelity into detailed BIM, uncertainty/source lineage, and local climate/regulatory fit. Do not claim Forma lacks early performance analysis. |
| Autodesk Forma Building Design, 4 Nov 2026 update | Indoor Daylight public beta; indoor daylight analysis over typical occupied hours; AutoCAD DXF import and DWG floor-plan export; updated parking/furniture workflows. | Include the current release, not an obsolete Forma baseline; independently verify the same indoor daylight cases and transfer fidelity. |
| Autodesk Forma Site Design | Area, sun-hours, daylight potential, wind, microclimate and embodied-carbon analyses in pre-design/schematic workflows. | Compare site/context/environmental screening on matched inputs. Do not claim Luminous invents these analyses. |
| TestFit Generative Design | Vendor documents zoning filters including height, FAR, building coverage, impervious coverage and dwelling units/acre; unit and parking mix targets. | Compare constraint capture, jurisdiction-specific source provenance, constraint-version updates, and ability to keep unknown or conflicting zoning rules from becoming inferred passes. |
| Autodesk Revit | Detailed model-based BIM remains a central authoring workflow; Autodesk describes Forma as extending connected, outcome-based work earlier while keeping Revit for detailed model workflows. | Treat Revit as a serious detailed-authoring baseline and test lossless/explicitly-bounded transfer rather than positioning Luminous as a Revit replacement. |
| OpenBIM / buildingSMART IFC and IDS | IFC 4.3.2.0 is an official IFC release (ISO 16739-1:2024). IDS 1.0 is an approved information-requirements standard for IFC alphanumeric attributes, classifications, values, units and relations; IDS does not cover geometric checks. | Use IFC/IDS as interoperability contracts; separately test geometry, topology, clashes, coordinate frames, as-built correspondence and calculated values. Do not reinvent IDS or pretend IDS proves geometry. |
| Specialist performance software | Dedicated thermal/energy, daylight, structural, fire, wind and acoustics solvers retain their domain-specific model and validation authority. | Orchestrate exact pinned versions, complete inputs, warnings/convergence, scope and independent benchmarks; do not replace them with a generic model score. |

Primary references:
- Autodesk Forma Building Design capability overview: https://www.autodesk.com/products/forma-building-design/overview
- Autodesk Forma Building Design 4 November 2026 update: https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/
- Autodesk Forma environmental analysis: https://www.autodesk.com/products/forma-site-design/environmental-impact-analysis
- Autodesk/Forma and Revit direction, 30 June 2026: https://adsknews.autodesk.com/en/views/forma-revit-evolution/
- TestFit Generative Design zoning and unit/parking mix: https://www.testfit.io/blog/testfit-5-2-better-data-brings-better-generative-design
- IFC release register: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- IDS standard scope and limitations: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/

### What Luminous must not claim

- "We invented generative building design" — false; the industry already has mature optioneering and performance-analysis workflows.
- "Our AI is more accurate" — unproven until matched solver/measurement comparisons exist.
- "Our BIM is more interoperable" — unproven until round-trip and as-built comparisons demonstrate it.
- "Our designs are code-compliant" — unproven without current jurisdiction-specific requirements, competent professionals and authority review.
- "Our climate data is better" — unproven until exact South African station/file identity, quality, rights and representativeness are qualified.
- "Our buildings are safer, cheaper or greener" — unproven until independent design and physical/operational evidence supports those specific claims.

### Credible differentiation to test

The strongest hypothesis is not another all-in-one authoring tool. It is an **evidence-carrying, authority-preserving design-to-operation compiler**:

1. Compile resident task intent and hard requirements into versioned machine-readable contracts.
2. Generate genuinely different alternatives while preserving conflict and uncertainty.
3. Carry source, station, material, solver and approval provenance into each candidate and dependent claim.
4. Use market-leading authoring and domain solvers through explicit exchanges rather than replacing them.
5. Keep a claim-specific proof graph from input -> geometry revision -> analysis -> approval -> fabrication -> inspection -> as-built -> maintenance.
6. Invalidate only evidence whose dependencies changed, while retaining unaffected evidence with exact dependency receipts.
7. Close the loop with measured indoor conditions, water/energy use, defect and maintenance outcomes—using consent and privacy-preserving telemetry.

The product advantage must therefore be shown as lower evidence loss, fewer false passes, better uncertainty calibration, faster detection of infeasible alternatives, more complete delivery packages and stronger as-built continuity under matched project budgets.

## 4. Frozen inputs

Before scoring:
- freeze household/task program and all hard requirements;
- freeze synthetic site envelope and local coordinate frame;
- freeze weather files and event profiles by exact digest;
- freeze occupancy/use schedules and material/system assumptions;
- freeze budget interval and cost boundary;
- freeze candidate geometry and revision identity;
- pin all authoring/solver/adapter versions and configurations;
- record licences and data-use permissions;
- publish exclusions, incomplete sources and unknowns.

All five Gauteng TMYx candidates remain unqualified until exact archives are downloaded, header-inspected, metadata/coordinates/period checked, hashed, rights-checked and reviewed for station/site representativeness. Compare them as separate station-exposure scenarios; do not average them blindly or choose one by distance alone. A TMY file is not a design-extreme or heatwave case. Use additional historical extremes and future scenario data only with exact sources and scope.

A real site study requires its own survey, geotechnical/dolomite, legal, utility, hydrology, environmental and resident inputs. Never reuse the synthetic site envelope as if it were a surveyed or zoned parcel.

## 5. Synthetic qualification corpus

Freeze deterministic scenarios that expose trade-offs and fail-closed behavior. Minimum 24 cases:

1. Four spatial families satisfy the same declared task graph but differ in form.
2. A visually preferred alternative violates a declared accessible route hard constraint.
3. An alternative improves area efficiency but increases route distance and maintenance burden.
4. Courtyard daylight is improved in winter but creates summer glare/overheating in a separate weather scenario.
5. Thermal mass improves one day/night trajectory but worsens another when night purge is unavailable.
6. Insulation improves winter behavior but creates summer overheating without shading/ventilation changes.
7. Cross-ventilation is unavailable during a period of poor outdoor air quality.
8. Natural ventilation is disallowed by a declared security/noise/weather condition; no free cooling is assumed.
9. Rainwater storage is proposed with no site rainfall/catchment/rights data: output remains Unknown.
10. A water feature increases maintenance/evaporation burden without a declared benefit.
11. A rain garden is proposed on an unknown geotechnical/hydrological condition: site-specific review remains mandatory.
12. A timber alternative lacks product grade/durability/fire/supply evidence.
13. An additive component lacks reinforcement/interlayer bond/inspection acceptance evidence.
14. A material substitution invalidates dependent structural, fire, thermal and procurement receipts.
15. A solver exits successfully but reports non-convergence/severe warnings: no PASS.
16. Weather file is TMY but claim requests heatwave resilience: reject the substitution.
17. Weather source is stale or its archive digest changed: invalidate dependent results.
18. A site parcel is not surveyed or zoning/title conditions are missing: no buildability claim.
19. A site geotechnical/dolomite report is missing where hazard status is unknown: no foundation/site suitability claim.
20. A roof form is aesthetically attractive but its support/load path is undefined: remain unverified.
21. A tower/bridge feature adds visual identity but creates accessibility, fire, maintenance or fabrication conflict: preserve trade-offs.
22. A garden path becomes inaccessible after level/threshold changes: hard-gate failure.
23. A maintenance route is blocked by a fixed decorative screen: require redesign or access evidence.
24. Robot/tool capability or calibration does not cover a fabrication work package: task not executable.
25. A component is manufactured but the as-built scan is outside the declared tolerance: not accepted.
26. A candidate has a beautiful rendering but missing material, fire, geometry or solver evidence: E0 only.
27. A resident changes a privacy or mobility requirement: affected alternatives are invalidated/re-evaluated; unrelated evidence remains current.
28. Two household members have conflicting preferences: preserve both and expose the Pareto trade-off; do not infer a winner.
29. Budget interval tightens: show which alternatives become infeasible and which compromises are explicit.
30. A component design changes after inspection: invalidate all downstream receipts tied to the changed revision.

Synthetic fixture values are never presented as engineering measurements or local climate predictions.

## 6. Outcomes and metrics — no universal score

### A. Resident and architectural outcomes
- blinded aesthetic/experiential review by residents, architects, artists and craftspeople;
- spatial legibility and arrival/threshold experience;
- resident-stated task completion and adaptability;
- privacy, autonomy and perceived safety;
- accessibility and wayfinding.

Report evaluator composition, disagreements, confidence/inter-rater reliability and context. No universal beauty or meaning score.

### B. Spatial and environmental outcomes
- usable area and circulation area;
- accessible route continuity;
- daylight availability, glare and views;
- thermal comfort trajectories under declared profiles;
- ventilation and indoor-air quality where model/measurements support them;
- energy demand and peak load;
- potable/non-potable water demand and storage;
- stormwater performance and landscape maintenance.

Do not collapse daylight, glare, temperature, air quality, acoustics, privacy and energy into one comfort score.

### C. Engineering and evidence outcomes
- exact geometry/revision lineage;
- source, unit, coordinate frame and model-scope completeness;
- solver convergence and warning preservation;
- numerical agreement with independent analytical/validated reference within declared domain;
- uncertainty and held-out error;
- stale/missing/unsupported input detection;
- changed-input dependency invalidation;
- IFC/IDS property-preservation and independent geometry comparison.

IDS is for alphanumeric IFC requirements; geometry checks are separate. IFC exchange fidelity does not prove structural correctness.

### D. Constructability and production outcomes
- design-to-fabrication package completeness;
- part counts and unique components;
- tooling/fixture/workspace coverage;
- handling/lift constraints and access;
- tolerance stack-up and measured placement/fit-up error;
- first-pass yield and accepted component output;
- rework, scrap, setup/changeover and cycle-time distribution;
- energy, operator time, guarding, maintenance and downtime;
- scan-to-model mismatch and defect closure.

### E. Whole-life outcomes
- initial and whole-life cost with boundary and discount assumptions;
- embodied/operational carbon and water;
- repair access and component replacement time;
- maintenance cost and frequency;
- adaptability to changed household tasks;
- disassembly/reuse quality;
- local supply, repairability and skills.

Always normalize to accepted functional output where comparing production routes, not nominal part counts or model quantities.

## 7. Proposed P0 gates

These are proposed acceptance targets for synthetic software and evidence workflows, not claims of real building performance:

- 100% traceability of declared critical requirements to source and revision.
- 100% of unknown, stale, missing, unsupported and out-of-scope inputs remain Unknown/limited.
- Zero false PASS across seeded hard-gate and stale-dependency cases.
- Zero solver PASS when convergence, required inputs or validation scope is absent.
- 100% invalidation of downstream receipts when a depended-on candidate, geometry, material, weather or profile revision changes.
- 100% of candidate exchanges preserve declared units, coordinate frame and stable IDs, or explicitly report loss.
- Zero geometry correctness claims based on IDS alone.
- Zero aesthetic preference override of accessibility, fire, egress, structural or legal hard constraints.
- Zero robotic task execution authorization derived from simulation alone.

## 7A. Highveld programme benchmark ladder and target gates

The programme needs a ladder of separate benchmarks. A win at one rung does not confer a pass on the next.

| Rung | Benchmark | P0 target | Evidence needed for a superiority claim |
|---|---|---|---|
| B0 — Definition integrity | Intent, task graph, source manifests, parameter schemas, hard/soft classes, revisions | 100% critical items trace to source and revision; zero seeded false PASS; unknown remains unknown | Independent mutation suite, reproducible exact-head validation, externally reviewed contracts |
| B1 — Requirement fulfillment | Task completion, hard path/access requirements, privacy, room use, conflicts | 100% hard requirements classified; no hard requirement silently downgraded or passed without evidence | Exact geometry, professional accessibility/fire review, resident acceptance tests |
| B2 — Design exploration | Four architectural families and further genuinely distinct Pareto candidates | Same brief/bounds/evaluation budget for every method; all final candidates re-evaluated with exact qualified solvers | Equal high-fidelity calls, at least 20 published stochastic seeds for stochastic optimizers, exact fronts on enumerable fixtures |
| B3 — Interoperability | IFC/IDS, geometry, coordinate frames, IDs, materials and downstream model transfer | Zero loss of mandatory information; geometry and alphanumeric property tests reported independently | Round-trip export/import into independent tools; stable-ID/property diff; geometry comparison under declared tolerance |
| B4 — Engineering | Thermal/energy, daylight, structure, fire, wind, drainage, water and acoustics | Zero non-converged, missing, out-of-domain or unvalidated result promoted to PASS | Version-pinned specialist solver, applicable software test suites, independent reference cases and/or calibrated physical tests |
| B5 — Production | Design-to-fabrication, tolerances, components, safety, inspection and rework | 100% fabricated critical parts trace to exact design/material/tool revisions; no robot task without capability/safety authority | Calibrated metrology, accepted physical specimens, measured yield/rework/cycle time and independent inspection |
| B6 — In-use habitat | Actual comfort, utilities, maintenance, accessibility, resident experience and adaptation | No claim of operational performance from simulation or render alone | Consent-based longitudinal measurement, calibrated sensors, resident feedback, maintenance records and independent replication |

### Critical reliability metrics

Report these explicitly:
- **False-PASS rate:** unsafe, unsupported, stale, incomplete or out-of-domain cases incorrectly called valid. P0 target: zero seeded false PASS.
- **False-REJECT rate:** valid alternatives rejected by an overly conservative checker. Report separately; zero false PASS must not become a reason to hide false rejects.
- **Evidence completeness:** percentage of critical claims with complete input, method, version, scope, reviewer and dependency lineage.
- **Invalidation precision/recall:** when inputs change, all dependent results are invalidated and unrelated results remain reusable.
- **Unknown calibration:** the observed frequency with which predictions marked Unknown/low-confidence are actually outside validated scope.
- **Surrogate calibration:** empirical interval coverage, sharpness, rank correlation and out-of-domain detection against held-out high-fidelity evaluations.
- **Human disagreement:** report evaluator variance and conflicting resident preferences, not merely an averaged score.

### Geometry and openBIM targets

Use the current official IFC release selected by the project (IFC 4.3.2.0 is currently listed as official) and a version-pinned IDS specification for alphanumeric information requirements. Then:
- test IFC schema validity and IDS compliance;
- test stable object/component ID survival through independent import/export;
- test property, unit, classification, material and relation preservation;
- independently compare geometric representation, placements, coordinate systems, openings, boundaries and topology;
- test clashes and clearance/path conditions using a geometric engine or specialist checker;
- compare the imported model with the source model and, later, calibrated as-built scans.

IDS alone cannot be the geometry oracle. It does not validate clashes, boundary offsets, total floor areas, calculated values or as-built matching.

### Robustness and resilience targets

Each candidate should be tested against a declared scenario set:
- ordinary typical-year weather;
- historical heat and cold events;
- plausible future climate horizons and model ensembles;
- grid interruption during adverse weather;
- municipal water interruption and safe storage depletion;
- indoor air quality/noise/security conditions that disable natural ventilation;
- supplier/material substitutions and construction tolerance variation;
- maintenance failures and component unavailability;
- changed occupancy/task needs and conflicting household preferences.

Report performance distributions, lower-tail outcomes, failure durations and recovery paths—not annual averages alone. Use CVaR or another declared tail metric only when its loss definition and stakeholder interpretation are explicit. No objective aggregation can compensate for a hard life-safety or legal failure.

### Industry-comparison protocol

For each competitor/toolchain, record exact product edition, feature availability, regional limitations, licences, setup effort, human intervention, data sources, file exchange path and version. Run:
1. the same synthetic functional brief and parameter set;
2. the same synthetic site envelope, then later an identical surveyed site;
3. the same qualified weather/material/occupancy inputs;
4. the same mandatory output package;
5. the same high-fidelity solver-call budget and independent verification plan.

Compare a **best-of-breed toolchain** as well as individual products: Forma/Revit + specialist solvers, TestFit + BIM authoring, an open parametric toolchain + IFC, and Luminous orchestration using the same specialist authorities. The strongest competitor may be a coordinated human/team workflow, not one product.

### What “better than SOTA” must mean

A claim must name the task and metric, not claim universal superiority. Acceptable examples after testing:
- “At equal high-fidelity solver-call budgets, the workflow found feasible alternatives with higher median hypervolume and no increase in hard-gate violations on the frozen Highveld test set.”
- “The IFC/IDS transfer preserved all declared mandatory properties and achieved geometric deviation within the preregistered tolerance across the selected importer/exporter set.”
- “The local climate-source qualification workflow detected a station identity/coverage mismatch missed by the compared pipeline.”
- “The accepted panel package reduced measured first-pass rework per installed functional unit under the same safety and inspection requirements.”

A claim is not acceptable if it compares different sites, weather files, geometry, budgets, tool versions, quality gates or human effort; uses a proprietary solver's output as its own validation oracle; or improves cost/speed while hiding accessibility, safety, privacy, worker-health or durability regression.

## 7B. Social/environmental parity and bidirectional resilience benchmark

The 2026 literature establishes a useful challenge. In 123 reviewed LP/MILP construction-optimization studies (1985–2024), 89% optimized exclusively for economic outcomes, 11% incorporated environmental metrics and 0% explicit social metrics; 53% were deterministic. These findings are limited to the review's LP/MILP selection, not every AEC product or optimization workflow. A separate 2026 building sustainability/resilience review reports sequential integration as common and dynamic bidirectional coupling as rare.

Highveld's comparison should require declared measures from five distinct domains:

| Domain | Example measurable outputs | Non-compensatory constraints |
|---|---|---|
| Resident/social | step-free task fulfillment, accessible service reach, indoor habitability distribution, privacy conflicts, resident preference satisfaction by household member, affordability burden, adaptation effort | life-safety, rights, accessibility requirements and consent/privacy boundaries |
| Workforce | full workflow labor, hazardous exposure, ergonomic workload, safe-abort / near-miss rate, human interventions, training and skill requirements | occupational health/safety; no productivity claim that hides human labor or unsafe exposure |
| Environment | whole-life embodied + operational carbon, water use/runoff, material reuse/repair, waste, biodiversity/soil disturbance, transport impact | no unapproved material/toxic exposure; declared LCA/system boundary |
| Economics | CAPEX, NPV/LCC, maintenance/replacement, local supplier risk, schedule variance, accepted output per full-workflow labor-hour | transparent discount horizon, prices/escalation and cost omissions |
| Resilience | time series of thermal/air/water/energy/access service, outage duration, lower-tail habitability, recovery time and failed-service episodes | minimum essential service and life-safety floors |

Do not aggregate these into a universal “best home” scalar. Report Pareto vectors, subgroup distributions and trade-off explanations. Apply hard constraints before preference scoring. Where residents disagree, preserve the disagreement and show the affected options; never infer one household utility from a majority or a demographic profile.

### Dynamic coupling test

A design alternative must be evaluated as a time-varying system, not a stack of independent static reports. Example coupling paths:
- shading -> daylight/solar gains -> cooling demand -> ventilation constraints -> indoor air quality;
- water storage -> supply resilience -> hygiene/maintenance -> embodied material and space use;
- material substitution -> thermal/moisture behavior -> carbon/cost -> repairability -> robot tool compatibility;
- power outage -> ventilation/filtration availability -> overheating/air quality -> habitability trajectory.

Record model identity/version, units, coupling variables, time step, boundary conditions, exchange cadence, convergence/stability criteria, lag/iteration limits, numerical errors, uncertainty and dependency graph. A converged coupling loop does not by itself validate the component models. Each solver retains its independent validation authority.

### New benchmark metrics

- **Social distribution:** median and lower-tail access/habitability outcomes by resident task, with privacy-safe aggregation and resident validation; do not publish individual sensitive traces.
- **Worker burden:** task-attributed labor, exposure, ergonomic risk, safe-abort/near-miss and intervention rates per accepted functional unit.
- **Environmental completeness:** percentage of lifecycle stages/material quantities with source-qualified factors and uncertainty; explicit unmodeled boundary share.
- **Dynamic consistency:** coupling convergence/failure rate, energy/water/mass balance residuals where applicable, time-step sensitivity and event-order robustness.
- **Change propagation:** recall of affected evidence invalidated after a cross-domain model/material change, and precision of unaffected evidence retained.
- **Decision value of information:** fraction of measurements that change candidate admission or decision ranking, value gained versus measurement cost, and unresolved uncertainty after measurement.

P0: no social/workforce/environmental dimension may be silently omitted from the benchmark manifest; Unknown must remain visible. P1: at least one coupled, time-dependent heat/energy/ventilation case and one material/production/repair coupling case must be executed on qualified model interfaces before any integrated superiority claim.

## 8. P1 comparative superiority protocol

Do not choose arbitrary improvement percentages before a benchmark baseline exists.

For each selected outcome:
1. preregister comparator, metric, functional unit and acceptance tolerance;
2. freeze candidate inputs and data;
3. conduct independent review of fixture expectations;
4. run matched tools under version-pinned conditions;
5. blind reviewers where subjective assessment is involved;
6. repeat physical component experiments with calibrated measurement;
7. report median, p90/p95, variance, confidence intervals and failures;
8. preserve the entire feasible Pareto frontier;
9. identify any quality, safety, privacy, affordability, accessibility, worker-health or ecological regression;
10. claim improvement only if it is practically and statistically meaningful without unacceptable regression.

The first competitive question should be: can Luminous deliver a more complete, traceable and constructable package than a matched workflow with fewer hidden assumptions and less evidence loss? Cost/speed superiority comes later, after accepted physical output is measured.

## 9. Physical demonstrator sequence

1. Synthetic design alternatives and requirements fixtures.
2. Exact geometry provider qualification through Symtropy #683.
3. One simple structural reference case through #1790, with independent analytical comparison.
4. One small-zone EnergyPlus case through #1900, with weather/model provenance and relevant ASHRAE 140 capability qualification.
5. One non-structural timber/panel coupon or joinery mock-up with calibrated metrology.
6. Guarded workcell dry-run and supervised assembly under #1818/#1834.
7. Independent inspection and as-built comparison.
8. Only later, a real site/pilot with resident consent, legal/site diligence, professional design and statutory approvals.

## 9A. Reproducible benchmark-result auditing

The implementation work is tracked in [HIGHVELD-SEARCH-EVAL-001 #619](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/619), with the first offline Rust auditor in [draft PR #621](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/621).

The v0.1 evaluator consumes a versioned benchmark-result bundle. It is designed to make false superiority claims harder by checking:
- identical fixture and objective-schema digests across methods;
- equal per-run candidate, high-fidelity-call and human-time budgets;
- explicit accounting for high-fidelity training, tuning, validation and final calls, plus low-fidelity calls and human minutes;
- 20 or more unique seeds for stochastic methods;
- required hard-gate status, current dependencies, high-fidelity/exact result, convergence, complete inputs and validation scope before a candidate enters the feasible Pareto set;
- false-feasible declarations and false-reject counts;
- explicit complete-enumeration provenance before an exact reference front is emitted;
- a predeclared, normalized reference point for 2D hypervolume.

**Current limitation:** v0.1 computes exact two-objective hypervolume and the feasible Pareto set. IGD+, GD+ and epsilon are deliberately reported as unavailable; higher-dimensional hypervolume is also unavailable. The unit fixture uses mathematical objective vectors only and is not an architectural performance simulation. It validates the auditor's rules and arithmetic, not NSGA-II or any competitor.

A real Highveld benchmark bundle must wait for qualified geometry and exact specialist-solver results, and every objective must carry its own units, evidence scope, input revision and validation authority. Never transfer the E1 mathematical fixture's metric results into a building-performance claim.

## 10. Evidence tiers

- E0: concept and synthetic assumptions.
- E1: frozen synthetic corpus and independent deterministic checker.
- E2: exact specialist solver result within validated model scope.
- E3: calibrated physical component/mock-up evidence.
- E4: supervised real-site pilot, resident consent and independent professional review.
- E5: sustained operational measurements and external replication.

Evidence tiers apply claim-by-claim and revision-by-revision. A successful E1 suite does not qualify a building.

## 10A. Design-search research references

- Lu et al. (2026), *Comparative Study of Multi-Objective Optimization Processes for Building Performance*, Buildings 16(21), 4192: https://doi.org/10.3390/buildings16214192. The paper discusses problem-dependent algorithm performance and metrics including Hypervolume, IGD+, GD+, epsilon indicators, repeated runs and matched evaluation settings.
- Huang et al. (2025), *Performance-Driven Generative Design in Buildings: A Systematic Review*, Buildings 15(24), 4556: https://doi.org/10.3390/buildings15244556. The review maps 153 core studies and identifies fragmented metrics, concentration on particular scales/objectives and limits to cross-case/field validation.
- Zhang et al. (2026), *AI-driven surrogate modeling for performance-oriented parametric optimization in sustainable building design: a systematic review*, Energy and Buildings 358, 117238: https://doi.org/10.1016/j.enbuild.2026.117238. The review covers 114 studies; surrogate-model selection should remain problem- and output-dependent.
- Silva et al. (2026 issue; article first online 23 October 2024), *Multi-objective optimization algorithms for building performance assessment – A benchmark*, International Journal of Architectural Computing 24(1), 35–59: https://doi.org/10.1177/14780771241296263. It compares seven algorithms across nine building-related problems.

## 11. Competitive claim

The potential Luminous advantage is not a universal AI architect. It is an authority-preserving, standards-based design-to-production loop that coordinates requirements, parametric alternatives, specialist solvers, local compliance evidence, material provenance, capability-bound fabrication and as-built/maintenance records.

This claim must be tested against conventional professional BIM workflows, parametric authoring, connected AEC platforms and open specialist-toolchains. No current SOTA superiority is asserted.
