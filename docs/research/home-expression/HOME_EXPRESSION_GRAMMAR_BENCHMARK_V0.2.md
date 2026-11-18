# Home Expression Grammar v0.2 — research and benchmark protocol

**Issue:** [HOME-ARCHITECTURAL-GRAMMAR-002 #934](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/934)  
**Parent work:** [Home Expression Library #904 / PR #906](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/906)  
**Status:** Contract, catalogue and synthetic fixture proposal; not a design generator  
**Date:** 2026-11-18

## 1. Paradigm shift: from style presets to a composable architectural grammar

A style is an expressive reference, not a building system. Home Expression v0.2 therefore separates five axes:

1. **Spatial typology:** detached, courtyard, row/terrace, duplex, apartment, pavilion, clustered/multi-generational, live-work, incremental-core and retrofit-first arrangements.
2. **Architectural expression:** form, composition, proportion, ornament, colour, light, thresholds and symbolic intent. The catalogue supplies reference families; residents can blend, reject, rename or leave them undecided.
3. **Material and tectonic intent:** stone, brick, earth, timber, steel, hybrid and repairable assembly preferences. A material name is not a specified assembly, EPD, structural detail or procurement proof.
4. **Environmental strategy intent:** orientation, shading, daylight, ventilation, thermal mass, water strategies and landscape relationships. A style's appearance never proves that one of these strategies works on a specific site.
5. **Experience and programme:** optional spaces and services that change area, structure, fire, services, access, cost, water, energy, sourcing and maintenance.

The composition contract preserves unresolved conflicts. It does not collapse axes into an aesthetic score, infer a style from resident identity, or automatically map a collage of references into geometry. The v0.2 catalogue now publishes the shared vocabularies as versioned arrays: 11 spatial typologies, 34 expression families, 8 material/tectonic options, 10 environmental strategies and 26 experience packages. The configurator consumes these same IDs rather than maintaining a separate UI-only list; Symthaea adapters should consume this versioned catalogue when their owner accepts the contract.

### New catalogue families

The 20 v0.1 IDs are retained. Fourteen additional expression families are added: Cape Dutch historic reference; Cape Georgian influence; Victorian; Edwardian; Georgian/Neoclassical; Arts & Crafts/Craftsman; Art Nouveau/Jugendstil; Prairie School; Bauhaus; International Style; Nordic/Scandinavian; Spanish Colonial/Mission; Alpine/Chalet; and Postmodern.

Two important classification choices:
- **Karoo corbelled-stone architecture is not a general-purpose style preset.** It is a separately typed, conservation-only Great Karoo heritage reference. UCT research documents its nineteenth-century form, complex frontier context and contested/multiple heritage associations. Exact asset provenance, local community engagement and conservation review remain necessary. Sources: Patricia Kramer (2012), [UCT thesis](https://open.uct.ac.za/handle/11427/12087); Caroline Hancock (2018), [UCT thesis](https://open.uct.ac.za/items/5d247a02-4bab-4a92-8f6f-2ce9d5fb04fb).
- **No generic "African vernacular" preset is introduced.** Community-specific grammar modules require attribution, community-authorized description, source provenance, rights, review and a correction/withdrawal route. Cape vernacular scholarship itself describes a layered tradition with place, material, technology and cultural influences, including Georgian shifts; it should not be flattened into an unchanging motif set. [University of Pretoria source](https://repository.up.ac.za/server/api/core/bitstreams/fc087af1-69b5-4f2f-9e74-9ed9fe58716e/content).

## 2. What is new versus what is deliberately not built

This increment adds:
- additive v0.2 catalogue, keeping v0.1 byte-for-byte and digest-pinned;
- separate JSON Schema for the catalogue and five-axis grammar composition;
- a synthetic Highveld grammar example whose site/climate/jurisdiction evidence remains UNKNOWN;
- the resident configurator's catalogue selection updated to v0.2, with an explicitly unvalidated local draft;
- deterministic Rust integrity checks and mutation self-tests.

This increment does **not** add:
- floor-plan or 3D geometry generation;
- structural, fire, moisture, energy, water or acoustic calculations;
- quotes, cost estimates, bills of quantities or supplier availability;
- municipal approval, heritage consent or professional sign-off;
- persistent resident data, user authentication or household consensus.

The UI remains local reactive state only. The v0.2 grammar fixture is a contract example, not a buildable proposal. The existing v0.1 profile, decision receipt, synthetic source and qualifier remain pinned to v0.1.

## 3. Industry and research baseline — matched scopes only

| Comparator | Demonstrated product/research scope | Fair test | Our differentiation target |
|---|---|---|---|
| Finch | Generative building layouts, firm plan libraries, unit/floor-plan exploration and BIM workflows | Same brief, floor plate, plan-library access, code assumptions, output editability and human setup budget | Preserve resident intent and provenance across candidate, decision and later evidence revisions |
| TestFit | Site/development feasibility, building configuration, unit mix, quantities and early cost workflows | Same parcel, zoning evidence/date, unit mix, quantity scope and rate source | Whole-household service, lifecycle and affordability outcomes; no opaque assumptions |
| Autodesk Forma | Early massing/site analysis including sun, daylight, wind, microclimate, noise, solar and carbon studies | Same geometry, site, weather file, date, terrain and analysis scope | Carry site evidence into resident-led alternatives and downstream qualification |
| Hypar | Programmable design logic, test-fit and space planning with metrics and Revit handoff | Same program, geometry exchange, team edits and output handoff | Keep the grammar declarative, open, resident-owned and independently verifiable |
| LayoutGen (2026 research) | Retrieval-grounded floor-plan generation with deterministic checks for selected topology/geometric/dimensional rules | Same dataset split, plan type, encoded rules, generation budget and blind evaluator | Expanded evidence ownership, multiple hard-gate domains, uncertainty and resident control; report scope mismatch |
| FloorPlanBench (2026, public research benchmark) | Exact validator for integer-decimetre rectilinear room layouts; 28 programmes / 4 tiers / 10 seeds / 50,000 evaluation-equivalents; greedy, SA, greedy→SA, NSGA-II, CP search, random and analytic placement | Reproduce the pinned canonical run, identical programme set, evaluator, 50k-equivalent budget and 10 seeds; compare all methods, not NSGA-II alone | Extend the benchmark with five-axis resident grammar, source lineage, change invalidation and qualified downstream physical evidence; seek measurable gains without weakening validity |
| Talami et al. (2026) | Surrogate-assisted multi-objective office optimization; paper reports 93.8% fewer simulations than exhaustive runs in one case | Re-run on same case/model and compare against exact oracle and declared exhaustive budget | Test robust multi-fidelity search without weakening high-fidelity final verification |
| EnergyPlus / OpenStudio | Building energy and water simulation | Identical geometry, constructions, weather, occupancy, schedules, systems and solver version | Do not claim to beat physics engines; improve the search/decision process around them |
| IFC / IDS and professional BIM workflow | Open model exchange and computer-interpretable information requirements | Identical models, IDS files, software versions and round-trip task | Exact lineage, loss reports and claim-scoped evidence that survives tool changes |

Official/product descriptions: [Finch](https://www.finch3d.com/product), [TestFit](https://www.testfit.io/), [Autodesk Forma](https://www.autodesk.com/za/products/forma/environmental-impact-analysis), [Hypar](https://docs.hypar.io/). Product marketing statements are not independent benchmark results.

Relevant published baselines:
- **FloorPlanBench (Gary Wang, canonical paper run dated 2026-09-06, commit 7dfcca3):** 28 programmes (249 rooms), four tiers, seven methods, 10 seeds and a 50,000 evaluation-equivalent budget. The exact validator checks overlap, containment in rectilinear/L/U boundaries, dimensions, area bounds, required adjacency, separations, door validity, entrance connectivity and a bounded accessibility criterion. Its seven methods are random, greedy, simulated annealing, greedy-seeded SA, NSGA-II, purpose-built constraint search and analytic placement. In the paper's canonical run, Greedy→SA solved 92% of runs with median 5-objective hypervolume 0.355; greedy also solved 92%, while NSGA-II solved 44% and had median hypervolume 0.098. [Paper](https://floorplanbench.vercel.app/paper), [MIT-licensed source repository](https://github.com/Gariyuuu/floorplanbench).
- **Reproduction warning:** the current live benchmark overview reports different headline hypervolume values (e.g. Greedy→SA 0.363 and NSGA-II 0.196) from the paper's stated canonical run (0.355 and 0.098). Do not mix these as one result. Pin the source commit, core hash, task files, per-run artefacts and generated summary before using either. FloorPlanBench is a useful public benchmark/reproduction target, not an independent certification or peer-reviewed universal SOTA verdict. Its geometry is restricted to axis-aligned rectangular rooms in rectilinear envelopes; daylight and carbon are declared proxies, not physical simulations.
- LayoutGen reports 3.7 to 0.3 measured violations per plan across 200 runs, but only for the specific checks in its loop; the paper explicitly excludes some whole-building/site checks from that count. [Kembhavi et al., 2026-10-15](https://link.springer.com/article/10.1007/s44163-026-02481-9).

- **Learned layout-generation benchmark (RPLAN test):** the SOTA2 aggregation reports HypergraphFormer at ATC 100 on its access-graph protocol, HouseDiffusion 96.7, and DiffPlanner 89.2; a separate vector/raster protocol reports TLC-Plan FID 1.84 alongside IoU, SSIM, GED and graph metrics. These are different evaluation protocols. They measure learned representation/topology/visual similarity outcomes on RPLAN, not hard building-code correctness, site feasibility, thermal performance or construction readiness. Report the exact split, output representation and metric; do not mix ATC, FID, GED, IoU and FloorPlanBench hypervolume into a single leaderboard. [RPLAN test ATC](https://www.sota2.com/research/sota/floor-plan-generation-on-rplan-test), [RPLAN FID/vector metrics](https://www.sota2.com/research/sota/floor-plan-generation-on-rplan).
- **Autodesk Forma Building Design (4 November 2026):** current documented capability includes Indoor Daylight (public beta), parking layout with accessible/EV stalls and aisle parameters, furniture, DXF import and DWG export, plus Revit handoff. These are schematic-design capabilities to include in the matched workflow, not proof of regulatory or physical qualification. [Autodesk release](https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/).
- **Autodesk Forma Building Layout Explorer (14 October 2026):** Autodesk describes multifamily unit-mix/circulation/core layout generation and comparison, but explicitly calls it early research and warns outputs are starting points rather than directly reliable designs. Treat it as a research comparator until a generally available release and a matched test protocol exist. [Autodesk AU 2026 announcement](https://blogs.autodesk.com/forma/2026/10/14/what-autodesk-forma-announced-au-2026/).
- **TestFit Site Solver:** current vendor documentation describes parcel/topography/exclusion-zone workflows, zoning filters (FAR, DU/AC, parking), automated or manual cores/stairs/firewalls/elevators, cut/fill and earthwork-cost optimisation, conceptual quantities, pro forma filtering and Revit/CAD/SketchUp exports. TestFit is a strong development-feasibility comparator, especially for multi-family, mixed-use and industrial projects; compare its supported task scope fairly rather than calling it a whole-home design/qualification system. [Site Solver](https://www.testfit.io/product/site-solver), [solve-filter documentation](https://support.testfit.io/knowledge/generative-design-solve-filters).

- **LayoutGen (Kembhavi et al., 15 October 2026):** for single-storey residential plans drawn from RPLAN/MSD (studio–3BHK), axis-aligned rectangular rooms and three Indian municipal rule repositories, the reported loop reduced counted violations from 3.7 to 0.3 per plan over 200 runs (92%). That count covers topological/geometric and encoded room-area, width and critical-width checks; setbacks, FAR, egress, staircases, parking and ventilation were outside the generation loop. It is a strong compliance-guided generation comparator for its scope, not whole-building compliance. [Paper](https://link.springer.com/article/10.1007/s44163-026-02481-9).
- **MO-ISDO residential energy/GHG benchmark (Amani et al., Energy, 1 October 2026):** a multi-storey building in Tabriz's cold semi-arid climate, EnergyPlus simulation, 14 initial variables reduced to 10 by standardized regression coefficients, and comparison against NSGA-II and MO-GWO. The paper reports 17 non-dominated solutions and 32.36% lower annual energy / 22.84% lower GHG relative to its reference building. This is a case-specific result, not a transferable South African saving or a solver-call reduction. [Paper](https://www.sciencedirect.com/science/article/pii/S0360544226021870).
- **Mediterranean residential EnergyPlus/NSGA-II benchmark (Romeos et al., Energy and Buildings, October 2026):** reports 59 non-dominated energy/cost solutions and approximately 14% space-conditioning energy reduction for a selected compromise relative to its case baseline, with knee-point pruning used to return a small portfolio. The article record states that no data were used, so exact independent reproduction may require the authors' model/input package. [Paper](https://www.sciencedirect.com/science/article/pii/S0378778826009370).
- **Central African multi-climate low-carbon benchmark (Buildings, 11 September 2026):** EnergyPlus + NSGA-II across eight representative Central African climate zones, optimizing operational energy, initial cost and operational carbon subject to comfort constraints. The paper reports 53.13–59.85% energy savings against its reference buildings and PV-enabled net-negative operational carbon. This is useful adjacent regional evidence, not a South African validated baseline; climate file, archetype, tariffs, carbon factors and system boundaries must be matched before comparison. [Paper](https://www.mdpi.com/2075-5309/16/18/3626).
- **IDS implementation consistency signal (buildingSMART survey, 2026):** 186 responses, with 58% reporting real-project IDS use and 70% wanting some form of IDS software certification. Respondents also report cases where the same IDS file is checked differently by different tools. This motivates a checker differential-conformance benchmark, not a claim that IDS itself validates geometry. [Survey results](https://www.buildingsmart.org/ids-survey-results-2026/).
- Talami, Costa and Stouffs report 93.8% fewer simulations in one Nottingham office case, with surrogate accuracy and low deviation reported for their studied objectives. This is a strong search-efficiency baseline, not a residential/South-African generalization claim. [Energy and Buildings, online 2026-10-20](https://www.sciencedirect.com/science/article/pii/S0378778826014775).
- The available [Talami et al. manuscript](https://papers.ssrn.com/sol3/Delivery.cfm/cf644962-4e07-468a-8c8f-d49ec1862029-MECA.pdf?abstractid=6981724&mirid=1&type=2) reports 85–92% coverage of the true Pareto front; use the manuscript's exact definition when reproducing this benchmark.
- EnergyPlus 26.2.0 was released 2026-09-30. Pin its executable/package digest and all inputs. [EnergyPlus release notes](https://energyplus.net/epupdate.htm).
- ASHRAE Standard 140 is a method for evaluating building-performance simulation software; it should anchor solver verification rather than an ad hoc model leaderboard. [ASHRAE/IBPSA resources](https://www.ashrae.org/technical-resources/bookstore/ashrae-building-energy-simulation-and-performance-modeling-resources).
- IFC 4.3.2.0 is the official IFC 4.3 release, published as ISO 16739-1:2024. IDS 1.0 is an official information-requirement standard, but its first version does not validate geometric rules; complement it with geometry-aware checks rather than overclaiming. [buildingSMART IFC](https://www.buildingsmart.org/standards/bsi-standards/industry-foundation--classes/), [ISO 16739-1:2024](https://www.iso.org/standard/84123.html), [IDS](https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/), [IDS limitations](https://github.com/buildingSMART/IDS/blob/development/Documentation/UserManual/specifications.md).

## 4. Benchmark suite and proposed targets

All targets below are **research targets**, not observed results. Report each separately; never combine them into one weighted score.

### P0 — correctness, rights and resident intent
- 0 false-feasible / false-PASS cases on frozen hard-gate oracles.
- 100% preservation of explicit non-negotiables, avoided motifs, undecided fields and dissent.
- 100% source/revision/method identity for any claim-bearing evidence.
- 100% rejection or quarantine of missing rights, withdrawn references, unbound heritage claims and stale evidence.
- 0 automatic promotion from style/intent to geometry, cost, climate performance, code compliance or heritage approval.
- Positive controls must demonstrate that valid, evidence-complete candidates can pass; an always-blocking system is not acceptable.
- 100% of regional style/typology/heritage references retain source, geographic/historical scope, attribution/context note and rights state; zero identity inference from name, address, nationality, photograph or budget.
- 0 heritage/conservation-only references are silently promoted into contemporary construction details or new-build style presets.
- No universal cultural-authenticity, beauty or symbolism score; assess provenance and resident/community validation separately.

### P1 — grammar quality and compositional behaviour
- 100% of valid grammar fixtures preserve independent axis identity; 0 silent conversion of typology into style or material preference into a specified assembly.
- 100% detection of seeded incompatible/underspecified combinations; unresolved cases remain visible rather than silently "fixed".
- At least 3 genuinely distinct topology/functional candidates on a tractable non-overconstrained brief, with duplication measured structurally rather than by render similarity.
- ≥95% retention of resident-stated requirements across candidate revision; 100% of removed or transformed requirements must have a traceable explanation and resident-visible status.
- Counterfactual locality: changes should propagate only through declared dependencies; publish the exact affected/unaffected receipt set.
- 100% exact invalidation closure on frozen dependency-DAG fixtures; zero evidence receipt remains current after one of its declared dependency digests changes.
- 0 unrelated evidence invalidations where the dependency graph is complete and proves independence; report graph coverage/completeness as a separate metric because a missing edge is not evidence of independence.
- 100% preservation of the distinction between architectural expression, spatial typology, tectonic intent, environmental strategy and experience/programme; no style label is a geometry or material assembly parameter.
- Source-aware regional references must pass a positive-control attribution test and negative controls for missing/withdrawn sources, absent context, wrong type assignment and attempted identity inference.

### P2 — search and physical performance

**P2a — floor-plan search (software benchmark, not physical validation)**
- First reproduce FloorPlanBench's pinned canonical run: all seven baselines, all 28 programmes, four tiers, 10 seeds and 50,000 evaluation-equivalents. Do not compare only with NSGA-II: the paper's Greedy→SA hybrid is the strongest reported method on aggregate median hypervolume and matches greedy's 92% solved-run rate.
- Preserve its shared exact validator and its validity-first rule: invalid plans receive no objective values and never enter the Pareto archive. Publish task-level results, solved-run rate, budget to first valid plan, raw objective vectors, hypervolume, front diversity, 300-weight sensitivity, paired tests and confidence intervals.
- Proposed held-out extension target: at the same evaluation-equivalent budget, maintain ≥92% solved runs and improve median five-objective hypervolume by ≥10% over the reproduced Greedy→SA baseline, with zero invalid plans admitted to result archives. Report per-tier and per-task outcomes; if the target is not met, publish the negative result and identify which constraints caused degradation.
- The v0.2 five-axis grammar is an additional input treatment. Benchmark the same underlying programmes with (a) neutral/default intent, (b) grammar-preserving constraint compilation, and (c) resident partial-preference data. Do not award points for subjective style labels; measure hard-requirement preservation, topology diversity, option novelty, and the cost of preserving undecided/conflicting intents separately.

**P2b — high-fidelity building-performance search (different benchmark, different unit)**
- On the Talami et al. Nottingham office case, reproduce its reported 93.8% simulation reduction and its 85–92% true-front coverage using the paper's exact definition before claiming any comparison.
- Proposed held-out residential stretch target: ≥95% exact-oracle Pareto-front coverage, ≥95% exact-oracle hypervolume, and ≤5% of exhaustive **high-fidelity solver calls**. These metrics are separate; never equate a FloorPlanBench evaluation-equivalent with a building-physics simulation call.
- Calibrate surrogate uncertainty by climate, typology and assembly; out-of-distribution inputs must abstain. Independently re-run final Pareto candidates with the qualified high-fidelity solver.
- Report hypervolume, IGD+, epsilon indicator, feasible-set recall, scenario-wise regret, wall time, energy use by end use, peak demand, comfort hours, water balance and lifecycle cost separately.
- Complete ASHRAE 140 testing before claiming building-simulation fidelity. A surrogate is an exploration aid, not a replacement for the qualified solver.

**P2c — cross-climate residential replication**
- Reproduce the MO-ISDO Tabriz case against its reported NSGA-II and MO-GWO references with the exact EnergyPlus version, 10 retained variables, weather, occupancy, schedules, envelope, HVAC and emission factors. Report the paper's energy/GHG outcomes, then add held-out climate/occupancy uncertainty; do not carry its percent savings into a different climate or archetype.
- Reproduce the Mediterranean 59-solution EnergyPlus/NSGA-II case and preserve both its full Pareto set and its knee-point subset. Measure portfolio loss caused by pruning; no single utopia-distance selection is a default resident choice.
- Build an eight-zone Central African comparison using its published reference building definitions before using it as regional evidence. Extend—not conflate—the cases to South African climate zones with SA-specific weather, tariffs, grid emissions, water constraints and household occupancy; record the source and rights for each dataset.
- Compare each method at equal high-fidelity solver-call, wall-clock and energy budgets. Report call counts, exact-front coverage, HV, IGD+, epsilon, calibration, worst-scenario regret and scenario-wise failures. Do not infer algorithm superiority from percentage savings against different reference buildings.

### P3 — interoperability and professional workflow
- For frozen IFC+IDS fixtures, report import/export round-trip property retention, geometry deviations, classification preservation and semantic-loss receipts. Initial target: ≥99.5% required-property retention, with zero silent loss of required items.
- Keep IDS information checks distinct from geometry, dynamic calculations and jurisdiction-specific professional reviews.
- Add an IDS differential-conformance suite: identical IDS + IFC inputs should yield the same normalized findings across supported checker versions; proposed target is 100% disposition agreement on frozen fixtures, with explicit version/error receipts. Geometry, quantities, external references, dynamic calculations and jurisdictional reviews remain separate test suites.
- Compare against an incumbent architect/engineer workflow on the same brief, time budget and deliverable maturity; report human setup time, intervention count, rework, error classes and review outcome.
- Do not make permit, safety or professional-practice claims from benchmark fixtures.

### P4 — resident experience and legitimacy
- Use consented and compensated residents; pre-register tasks and analysis.
- Measure requirement-comprehension, ability to express "none/undecided", preference correction, perceived influence, accessibility task completion, privacy comprehension and dissent preservation.
- Compare decisions to resident-declared priorities, not synthetic LLM personas or a universal satisfaction score.
- Evaluate whether residents can add, remove, rename, reject and leave references undecided; measure agency, correction latency and whether residents understand which references are historical/conservation-only versus selectable spatial or architectural directions.
- Determine sample size by a power analysis for the chosen outcomes; do not invent a user-study result from a prototype.

## 5. Execution order

1. Freeze v0.1 and make v0.2 additive.
2. Validate the v0.2 catalogue and grammar schema with deterministic mutation tests.
3. Add the new catalogue to the local-only configurator; retain an explicit unvalidated draft status.
4. Extend the grammar compiler in Symthaea #7790/#7801 only after its owner accepts the contract; do not add a second optimizer in Sol-Atlas.
5. Route realized geometry, simulation and spatial constraints to Symtropy #1766/#1790/#1848 and existing owner APIs; no duplicate solver.
6. Compose with Sol-Atlas Home Freedom and #832 revision invalidation; exact changes, not intent labels alone, determine re-evaluation.
7. Build a small exact oracle and independent benchmark fixtures before large simulation campaigns.
8. Conduct a resident/architect evaluation only with consent, compensation and a pre-registered protocol.

## 6. Status ledger

- Catalogue v0.2 and grammar contract: **authored / not yet exact-head CI verified**.
- Thirty-case grammar conformance corpus (5 positive / 25 negative): **authored; Rust runner and exact-head execution pending**.
- FloorPlanBench and Talami reproductions: **protocol target only; not run by this project**.
- UI: still an inspiration-only, local working draft; no save, geometry, price, or qualification.
- Synthetic grammar fixture: **synthetic / not evaluated**.
- Physical, cost, climate, code, heritage and professional validation: **not performed**.
- Benchmark comparisons: **protocol and targets only; no SOTA victory claimed**.
