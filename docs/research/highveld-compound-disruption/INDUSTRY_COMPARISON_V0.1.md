# Highveld industry comparison and differentiation hypothesis v0.1

**Status:** desk research and proposed comparison protocol, 12 November 2026. This is not a product test, procurement recommendation, benchmark result, or SOTA-superiority claim.

## Executive position

Do not attempt to replace each incumbent specialist with one Luminous engine.

- Use **Forma** for early massing, daylight exploration and its native design handoffs where those tasks fit.
- Use **TestFit** for site feasibility, generative layout and pro-forma-oriented exploration where those tasks fit.
- Use **EnergyPlus** or another qualified physics backend for building energy/thermal simulation. Pin the exact version and input files.
- Use **ASHRAE Standard 140** for the applicable building-simulation software capability tests.
- Use **BOPTEST** for control/FDD comparisons on supported emulator cases.
- Test Luminous on the difficult connective tissue: whether evidence remains scoped, current, rights-valid and decision-relevant as a building, service, resident requirement, event trace, instrument, supplier or solver changes.

A credible result is therefore **incumbents + Luminous evidence composition**, compared against the same incumbents without that composition. This design tests incremental value rather than confounding tool quality with orchestration.

## Public capability snapshot

| Industry system / benchmark | What current public material establishes | What it does not establish for Highveld | Luminous response |
|---|---|---|---|
| Autodesk Forma Building Design | On 4 Nov 2026 Autodesk documented Indoor Daylight in public beta for occupied-hour analysis over a typical year, parking layout, furniture, DXF import and DWG export. [Autodesk update](https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/) | A typical-year daylight feature is not a historical extreme-event record, a calibrated Highveld thermal model, a household-service trajectory or a rights/consent-aware outcome ledger. | Use Forma outputs as versioned inputs. Test whether event/weather assumptions and downstream decisions remain traceable after design revisions. Do not rebuild its schematic design features. |
| TestFit Site Solver | Public product material describes site/parcel data, 2D/3D topography, flood/wetland/easement exclusions, zoning/parking/FAR constraints, cut/fill and financial feasibility; current Site Solver pages advertise large option filtering and deal evaluation. [Site Solver](https://www.testfit.io/product/site-solver) | Product claims and US-oriented data integrations do not establish South African statutory/site applicability, physical survey accuracy, detailed building physics, household services or operational resilience. | Use it for matched site-feasibility tasks where source coverage and jurisdiction fit. Test import/export lineage, site-evidence coverage and change propagation across downstream decisions. |
| EnergyPlus | Version 26.2.0 was released 30 Sep 2026. Official material describes whole-building energy simulation, subhourly outputs, advanced fenestration/HVAC/control capabilities and FMI import/export. [Release notes](https://energyplus.net/epupdate.htm) [EnergyPlus](https://energyplus.net/) | A solver version or converged run does not validate a specific Highveld geometry, material set, occupancy schedule, weather file, control strategy or physical measurement. | Integrate it as a specialist backend. Bind executable/version, geometry, materials, schedules, weather, outputs, convergence/warnings and model applicability into evidence receipts. |
| ANSI/ASHRAE Standard 140-2023 | Specifies test methods for technical capabilities and applicability of building performance simulation software. The 2023 revision includes weather-data-format tests. [ASHRAE resource index](https://www.ashrae.org/technical-resources/bookstore/ashrae-building-energy-simulation-and-performance-modeling-resources) | Software capability testing is not empirical calibration or validation of the project-specific Highveld model. The user manual distinguishes analytical, comparative and empirical testing; Standard 140-2023 has no empirical test cases. [User manual](https://data.ashrae.org/standard140/140UsersManual-PartI-Final%20%28050825%29.pdf) | Do not reinvent the suite or claim Standard 140 compliance from generic unit tests. Run the applicable suites, document applicability/exclusions, then qualify the project model separately against measurements and uncertainty. |
| BOPTEST / IBPSA Project 1 and 2 | The current official test-case page identifies release v0.9.0 and lists nine available emulators: BESTEST Air, BESTEST Hydronic, BESTEST Hydronic Heat Pump, Single Zone Commercial Hydronic, Two Zone Apartment Hydronic, Multizone Residential Hydronic, Multizone Office Simple Air, Multizone Office Simple Hydronic and Multizone Office Complex Air; Single Zone Commercial Air and Flexible Research Platform are under development. Test cases bundle dynamic models, embedded controls, weather, electricity prices and carbon-emission factors with predefined API scenarios. [Current test-case list](https://ibpsa.github.io/project1-boptest/testcases/index.html) [IBPSA project](https://ibpsa.github.io/project1-boptest/index.html) | BOPTEST cases are not automatically Highveld residential cases and do not themselves provide local utility event evidence or resident-defined service floors. | Reuse BOPTEST on supported cases for controller comparisons. Add separate Highveld cases only when a properly scoped emulator and measured reference are available. |
| Brick / Haystack / ASHRAE 223 | Existing building semantic models and interoperability work address machine-readable building equipment, points and relationships. ASHRAE 223P remains a proposed standard: NIST's 2 Nov 2026 project update says comments are being addressed and the committee vote for second public review is pending. Do not call it a finalized published standard. [NIST project status](https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability) [ASHRAE TPS](https://resourcecenter.ashrae.org/technical-resources/standards-and-guidelines/titles-purposes-and-scopes) | An ontology or semantic tag does not prove physical measurand quality, consent, authority, instrument calibration or a causal relationship. 223P itself remains a proposal in development; use a pinned draft and explicit profile/version if testing interoperability. | Map to existing semantics where appropriate; keep identity, validity, source rights, resident authority and evidence claims in explicit assurance envelopes. |

## Where the research gap is real

A 2026 systematic scoping review of 176 studies/frameworks found that only 25% supplied operational definitions of energy resilience, the literature used 29 building-scale indicators, and fewer than 10% of the included studies addressed the Global South. The review also found limited treatment of multi-hazard trajectories, district dynamics and equity. These are statistics about that review's corpus, not universal counts for all research. [Ramesh & ur Rehman, Energy and Buildings (1 Nov 2026)](https://doi.org/10.1016/j.enbuild.2026.118111)

The opportunity is not to invent another “resilience index”. It is to make the claims behind a decision inspectable and reproducible:

1. **One evidence graph from concept to in-use outcome.** Preserve design, source, model, instrument, procurement, commissioning and post-occupancy revisions with claim scopes and invalidation paths.
2. **Compound services, not isolated hazard flags.** Keep external hazard, observed evidence, service loss/restoration, reserve state, repair delay and resident task outcomes distinct. Do not infer causality from time overlap.
3. **Resident capability as a hard constraint.** Track each declared resident/task service floor; never allow a weighted average to hide a mandatory failure.
4. **Multi-fidelity decisions with preservation checks.** Use fast design tools for exploration and high-fidelity physics/reference checks for the decisions that could change. Measure false-feasible decisions, Pareto-set loss and rank reversals.
5. **Decision-sufficient measurement.** Choose the next survey or instrument only when its possible outcomes can alter robust feasibility, a Pareto choice or a critical blocker; stop when the decision is stable across the remaining states.
6. **Change-aware truth maintenance.** A source revision, revoked licence, geometry change, sensor replacement or solver change invalidates only dependent claims—neither too little nor indiscriminately everything.
7. **Repair and recovery as first-class design variables.** Measure access, parts delay, service loss, repair labour, repeat failure and stable recovery, not just nominal efficiency at handover.

These themes are a differentiation hypothesis. Related work already exists in resilience, multi-objective design, model calibration, digital twins and measurement design; novelty must be demonstrated by matched experiments and independent review, not asserted by terminology.

## Fair head-to-head protocol

Use the benchmark arms in HIGHVELD_BENCHMARK_CONTRACT_V0.1.json:

- B0: documented manual/conventional process.
- B1: incumbent specialist toolchain (each tool only on tasks it supports).
- B2: the same incumbent engines plus Luminous evidence composition.
- B3: the best-performing hybrid, with every retained/replaced tool disclosed.
- B4: exhaustive finite oracle on tiny synthetic worlds only.

Freeze and publish the same candidate geometry/design revision, household/task requirements, exogenous trace bytes/digest, initial state, information budget, output units/horizon, compute budget and human-hours. Pin software versions, licenses, data snapshots and exclusions. Do not compare different tasks or model scopes under one score.

### Mandatory output panels

1. **Evidence:** trace pairing, rights/scope/revision integrity, unknowns, missingness, conflicts and false-PASS controls.
2. **Decision:** hard feasibility, robust-feasible set, Pareto precision/recall, rank reversals, false-feasible rate and residual blockers.
3. **Resident services:** per-resident/task service floors, unserved service-hours, reserves, first breach, first restoration and stable recovery.
4. **Physics/measurement:** Standard 140 applicable suite results, model-specific residuals on held-out measurements, uncertainty budget, convergence and applicability domain.
5. **Lifecycle:** human-hours, cycle time, data preparation, review/rework, repair time, part delay, requalification work and direct costs.
6. **Distribution:** who benefits, who bears service loss, accessibility/task preservation and per-household or per-resident distribution; do not bury tails in an average.

No single composite score is allowed to override a hard gate. Hypervolume/regret may be reported only with frozen objective bounds/reference points and a declared reference set.

## What “better than SOTA” should mean

| Claim | Minimum credible evidence |
|---|---|
| More trustworthy decisions | Zero critical false PASS on seeded controls; exact oracle agreement on finite cases; external blind review of evidence and receipts. |
| Better design decisions | Same candidates and constraints; more robust-feasible or better Pareto outcomes confirmed by an independent higher-fidelity/reference evaluator; uncertainty disclosed. |
| Better resilience | Same exogenous trace, initial state, resources and information budget; per-resident/service floors and stable-recovery trajectories measured under qualified physical or explicitly synthetic scope. |
| Faster engineering | Lower human-hours and elapsed cycle time at equal or better decision quality, with all setup/rework and software costs counted. |
| Better use of measurements | Lower measurement cost/time than uniform capture while preserving the decision across all remaining admissible states; real savings only after field measurement. |
| Better lifecycle performance | Fewer stale decisions, less rework, correct scoped invalidation, better repairability and verified post-occupancy service outcomes. |

## Current recommendation

The near-term win is a **benchmarkable evidence layer over established tools**, not a new monolithic building platform. The strongest first demonstration is a small, blind, paired Highveld study:

1. Freeze 3 architecture candidates and a small set of design counterfactuals.
2. Run manual and specialist-tool baselines on the exact same brief.
3. Replay the seven compound synthetic traces with identical trace digests.
4. Seed rights, timestamp, stale-revision, missingness, scope-escalation and future-leak failures.
5. Show whether evidence composition changes a decision, catches an unsupported PASS, or prevents a rework/requalification mistake.
6. Only then promote the strongest decision-sensitive cases to the unoccupied #4531 test cell.

The physical layer remains unqualified until instrument admission, independent reference methods, model-specific VVUQ and a safety-reviewed test protocol exist. NIST AMS 100-85 (5 Oct 2026) supports risk-proportionate VVUQ scaled to model influence and decision consequence; it does not waive domain-specific validation. [NIST AMS 100-85](https://doi.org/10.6028/NIST.AMS.100-85)
