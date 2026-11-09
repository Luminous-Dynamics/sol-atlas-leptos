# CityMaker industry and research crosswalk v0.1

**As of:** 2026-11-06  
**Status:** benchmark design and external reference mapping, not measured Sol-Atlas / Symtropy / Symthaea / Mycelix performance.

## Executive positioning

CityMaker should not attempt to replace the best specialist products and solvers in urban planning, routing, BIM, digital twins, materials accounting, production scheduling or robot control. Its competitive hypothesis is a **verifiable city design-to-service integration layer** that joins:

- human/civic needs and service requirements;
- urban form, access, public space and distributional incidence;
- building/component product systems and construction methods;
- manufacturing/robotic capabilities and workforce;
- material, energy, water and logistics dependencies;
- construction phases, accepted outputs and commissioning;
- maintenance, repair, adaptation, reuse and recovery;
- provenance, source rights, uncertainty, change and accountable decisions.

The differentiation is the ability to prove which exact assets, materials, capacities, routes, work packages and evidence support each city-service claim—and to show what breaks when an assumption or dependency fails. That is not the same as claiming a better route engine or autonomous construction robot.

## 1. Industry maturity is not delivered performance

### Industrialized Construction Index (ICI), 2026

The 2026 ICI paper proposes a quantitative maturity framework spanning five separate pillars:

1. Standardization.
2. Prefabrication.
3. Modularization.
4. Automation.
5. Digitalization.

The study reports applications across 61 residential projects in Colombia and 33 in France. It argues that the index can distinguish different industrialization pathways across market contexts. These dimensions are a useful reporting crosswalk for CityMaker, but an ICI maturity number is not proof of accepted output, low lifecycle cost, safety or environmental advantage.

Reference: Sanchez R. & Attouri (2026), *Industrialized Construction Index: a project indicator for industrialization assessment*, IGLC 34. https://doi.org/10.24928/2026/0184

### Autodesk 2026 industrialized-construction survey

Autodesk reports that regular or standard use of productization, DfMA and prefabrication is around 37–40% in its worldwide survey; 84% of adopters report better finished quality and 76% report greater schedule/cost predictability. The report also identifies digital maturity and traditional procurement practices as scaling factors.

Interpretation:
- There is an adoption/maturity gap, not merely a robot-hardware gap.
- Repeatable digital product definitions, procurement interfaces, design standardization and information continuity deserve first-class measurement.
- Self-reported adopter benefits are useful industry context, not controlled causal estimates and not universal target values.

Reference: Autodesk, *Spotlight on Industrialized Construction 2026*. https://www.autodesk.com/design-make/articles/2026-state-of-design-make-spotlight-on-industrialized-construction

## 2. Construction robotics: lifecycle-wide, task-specialized, human-centred

A June 2026 PRISMA review synthesized 375 construction-robotics papers published from 2023–2025. It found a concentration of research in structural construction tasks; planning, maintenance and demolition remain underexplored. Operator-led workflows still dominate compared with full autonomy, and the review identifies real-world deployment barriers around autonomy, lifecycle integration and validation.

This changes the CityMaker strategy:

- Do not lead with a universal humanoid robot.
- Build capability-specific cells and equipment: survey/metrology, timber and sheet processing, steel preparation, panel/MEP assembly, material handling, placement assistance, inspection, maintenance and selective disassembly.
- Evaluate each capability at the task and configuration level.
- Treat maintenance, repair, adaptation and end-of-life work as core robotics domains, not future add-ons.
- Preserve human roles, intervention burden, worker exposure and new hazards as separate metrics.

Reference: Xu et al. (2026), *Construction robotics: a systematic review of robot types, applications and human robot collaborations*, Results in Engineering 30, 110406. https://doi.org/10.1016/j.rineng.2026.110406

### Physical safety is a separate evidence tier

NIOSH's October 7, 2026 bulletin notes that robotics can reduce exposure to hazardous and physically demanding tasks, while human–robot interaction introduces risks such as unauthorized entry into operating areas and failures involving sensing, communications, software integrity and coordination. Therefore, a lower manual exposure measure cannot stand alone as a safety success if the robot creates new hazards.

For any physical demonstrator:
- Begin with isolated, low-consequence workcells and non-load-bearing articles.
- Use engineered guarding, interlocks, risk assessment and qualified commissioning before powered processes.
- Measure new hazards, safe stops, false stops, unauthorized zone entry, communication loss, sensor failure and restart behaviour.
- Keep simulator evidence, synthetic contract evidence, bench measurements and supervised field observations distinct.
- Physical ground truth must be independently instrumented; the robot's own perception cannot be its only referee.

Reference: NIOSH, *Safety in Construction Robotics*, 7 Oct 2026. https://www.cdc.gov/niosh/bulletin/2026/safety-in-construction-robotics.html

## 3. Digital twins and VVUQ

NISTIR 8620 (July 2026) summarizes manufacturing digital-twin workshops and highlights interoperability, verification/validation/uncertainty quantification (VVUQ), cybersecurity and workforce readiness as priority challenges.

CityMaker consequence:
- Digital-thread completeness and exact identity must be measured independently of model fidelity.
- Every model output has a declared context of use, version, assumptions and uncertainty.
- As-designed, as-manufactured, as-installed, commissioned and operating states remain separate.
- A hash proves byte identity relative to a digest, not source truth or authorization.
- A simulator is not a physical measurement; a measured result is not automatically representative of a different machine, site or season.

Reference: NISTIR 8620, *Digital Twins Workshops Summary Report*, 21 July 2026. https://doi.org/10.6028/NIST.IR.8620

## 4. The physical scale of the problem

UNEP's 2025–2026 Global Status Report for Buildings and Construction reports that the sector accounts for 37% of global emissions and nearly 50% of global material extraction. These are global sector aggregates, not Johannesburg-specific measurements. They reinforce why CityMaker must include upstream materials, construction, operations, maintenance, replacement, transport and end of life rather than optimizing only the initial build.

Reference: UNEP, *Global Status Report for Buildings and Construction 2025–2026*, May 2026. https://wedocs.unep.org/items/dd0a1d99-dba3-46da-93ab-ef2a26fc3c14

## 5. CityMaker comparison matrix

Compare alternatives using identical functional requirements, study boundaries, service periods, design revisions and acceptance criteria. Preserve all outcome dimensions rather than hiding trade-offs in one score.

| Dimension | Required measures | Comparison rule |
|---|---|---|
| Standardization | product variants, interface reuse, tolerance compatibility, exception count | Compare equivalent released design families |
| Prefabrication | off-site work share, site assembly time, transport/handling, handover defects | Count whole workflow, including temporary works and logistics |
| Modularization | interfaces, interchangeability, replacement time, interface mismatch | Do not assume modularity is always beneficial |
| Automation | task coverage, cycle distributions, human intervention, recovery, guarded operating envelope | Compare manual, assisted and guarded-robot routes |
| Digitalization | identity/lineage closure, model-to-as-built discrepancy, receipt completeness, requalification effort | Preserve design history and evidence generations |
| Productivity | accepted units per total workflow labour-hour | Include setup, kitting, replenishment, monitoring, inspection, rework, cleaning and recovery |
| Quality | first-pass yield, final acceptance, defect classes, rework, scrap | Independent acceptance predicates and fixed functional unit |
| Schedule | end-to-end lead time, p50/p90 cycle, missed windows, schedule reliability | Include supplier, transport, receiving and commissioning delays |
| Cost | installed and lifecycle cost, cost variance, maintenance and replacement | Same study period, scope and currency basis |
| Human factors | exposure removed/introduced, workload, ergonomics, training and role burden | No netting away new hazards with productivity gains |
| Safety | hard-stop misses, zone incursions, safe-stop behaviour, fault recovery | Non-compensatory gate; physical evidence required |
| Circularity | component reuse, grade-retained recovery, downcycling, residuals, virgin input | Distinguish function retention from mass recovery |
| Environment | embodied/operational GHG, energy, water, transport and waste | Declare lifecycle boundary and functional unit |
| Maintainability | diagnosis, repair, replacement, downtime, tool/part families, access | Include operations and end-of-life lifecycle tasks |
| Resilience | continuity, bottleneck exposure, substitute depth, recovery duration | Scenario-based unless probabilities are evidenced |
| Urban outcomes | effective access, affordability, displacement, public-space access and distribution | Report by relevant group and neighbourhood |

## 6. Matched experimental protocol

For the first panel-cell demonstrator:

1. Freeze one non-load-bearing reference panel family, released design/BOM revision, materials and receiving acceptance criteria.
2. Compare a conventional/site-built baseline, manual off-site assembly, human-assisted mechanization, and a guarded robotic route only after the required safety/commissioning evidence exists.
3. Use identical functional output, tolerances, inspection, lot acceptance and process boundaries.
4. Capture complete labour/time/cost, including engineering/setup, kitting, supervision, monitoring, maintenance, rework and rejected articles.
5. Randomize or block trials by material-lot variation, operator/team, shift and relevant environment; document why each allocation is valid.
6. Use independently calibrated measurement equipment and keep the reference system outside the evaluated robot's relevant fault domain.
7. Report raw denominators, exclusions, confidence intervals/uncertainty and all critical counterexamples.
8. Retest after a change in design, tool, calibration, material lot, software or workcell configuration according to the affected evidence scope.
9. Compare maintenance, repair and planned disassembly only after the relevant product/connection qualification is defined.
10. Do not claim a performance improvement until an exact, preregistered matched baseline and independently accepted result exist.

## 7. SOTA comparison strategy by subsystem

| Subsystem | Reference class | CityMaker strategy |
|---|---|---|
| Urban routing/access | Conveyal R5, OpenTripPlanner, published SDG methods | Consume specialist routing, then add evidence-qualified service capacity, eligibility, affordability and universal access |
| Urban metabolism | UN-Habitat/SDG, OECD urban metabolism/MFA, Living-City ledger | Preserve system boundaries, resource balances, inter-city flows and distributional impacts |
| BIM/digital thread | IFC/IDS and established digital-twin workflows | Prove exact identity, revision, as-built discrepancy, commissioning and change lineage |
| Industrialized construction | ICI, DfMA, prefabrication and modular delivery | Compare maturity pillars separately and measure accepted output/lifecycle results |
| Construction robotics | 2026 systematic review, NIOSH safety, owner-native robot qualification | Task-specific systems, physical ground truth, guarded collaboration, lifecycle maintenance/repair |
| Materials/circularity | ISO 20887, ISO 59020, material passports and owner-native lifecycle accounting | Track stock -> recoverable -> qualified -> available -> accessible -> deliverable; no double counting |
| Robust design | Pareto/interval robust optimization owner contracts | Non-compensatory floors, uncertainty semantics and disclosed trade-offs; no opaque city score |

## 8. What would constitute a credible advantage?

Do not claim "better than SOTA" because a synthetic corpus passes or a dashboard shows more layers. Evidence of a real competitive advantage requires:

- semantic parity with reference methods on identical inputs;
- documented extension benefits that the reference method does not claim to provide;
- no regression in safety, quality, uncertainty abstention or data rights;
- independent measurement on representative tasks and sites;
- repeatability across materials, teams, shifts and configurations;
- acceptable total workflow cost and maintenance, not merely a fast machine cycle;
- visible distributional consequences, worker outcomes and stakeholder decision rights;
- exact source, model, configuration and receipt lineage.

An initial success condition should be **non-inferior accepted quality and safety under the same functional requirements, plus a measured improvement in at least one predeclared operational outcome**, with all trade-offs reported. The margin and outcome should be set with competent engineering, procurement, worker and project stakeholders before trials—not invented by the benchmark author.

## 9. Owner composition

- **Sol-Atlas:** geography, public-interest constraints, access/morphology, source coverage, material and delivery projections, scenario comparison.
- **Symtropy:** physical/process/resource semantics, deterministic simulation, construction assurance and benchmark evaluation.
- **Symthaea:** requirements/design hypotheses, material/process/manufacturing/robot capability reasoning and experiment selection.
- **Mycelix:** supplier/lot provenance, custody, commitments, approvals, correction, revocation and dispute.
- **External qualified authorities:** professional design, statutory permissions, structural/fire/building safety, workplace safety, independent inspection and community decision rights.

Never duplicate an owner merely because an adjacent product UI needs a summary. Cross-repository adapters verify owner-native claims; they do not take over their authority.

## Claim ceiling

This crosswalk defines research references and experimental design. It does not establish CityMaker SOTA performance, real city buildability, cost or carbon reductions, structural/fire compliance, robot safety, local supplier capacity, municipal endorsement, community consent or an authorized construction programme.
