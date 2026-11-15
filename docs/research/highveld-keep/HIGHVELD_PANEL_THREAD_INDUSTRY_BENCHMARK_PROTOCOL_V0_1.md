# Highveld Panel Thread — Matched Industry Benchmark Protocol v0.1

**Status:** research protocol; targets are proposed, not achieved  
**Scope:** synthetic composition first; physical evidence only after separate qualification  
**Parent:** [HIGHVELD-KEEP-ROBOT-THREAD-001 #622](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/622)  
**Benchmark corpus:** [PR #870](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/870)  
**Evaluator:** [PR #877](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/877)

## 1. Research thesis

The field is already converging on connected digital twins, digital threads, industrial AI and lifecycle data federation. We should not attempt to win by rebuilding CAD, BIM, PLM, CAE, MES, QMS, robot controllers, or asset-management suites.

The research target is narrower and harder:

> Given heterogeneous owner-native product, process, supplier, robot, metrology and as-built records, can an independent system determine exactly what those records establish, what they do not establish, what changed, which prior evidence remains applicable, which proof obligations are still open, and the least-burden admissible evidence campaign that can resolve a decision?

The proposed breakthrough is **decision-sufficient, evidence-first engineering**:
- preserve the owner-native source and its limits;
- compose by exact subject, revision, time, configuration and decision scope;
- treat uncertainty, authority, rights and physical observation as separate dimensions;
- plan evidence acquisition against the decision it can change;
- minimize unnecessary requalification without retaining stale evidence;
- optimize accepted physical output across the complete workflow rather than a subsystem's nominal cycle time.

The result is not a replacement digital twin. It is an independent assurance and decision layer over multiple digital twins and industrial systems.

## 2. Standards and research landscape — checked 15 November 2026

### Standards now published or under development

| Reference | Current state | Consequence for this benchmark |
|---|---|---|
| ISO 23247-5:2026 — digital thread for manufacturing twins | Published June 2026 | Use as a manufacturing lifecycle-thread reference; do not claim that a proprietary graph alone is interoperability |
| ISO 23247-6:2026 — digital twin composition | Published July 2026 | Test integrated, unified and federated composition while preserving component models and schema distinctions |
| ISO/AWI 23247-7 — commissioning of digital twins | Under development | Record commissioning maturity explicitly; do not present it as published |
| ISO/AWI 23247-8 — digital-twin VVUQ | Approved work item, development initiated 23 October 2026 | Design the VVUQ evidence and metrics now, but do not claim conformity to an unpublished standard |
| IFC 4.3.2.0 / ISO 16739-1:2024 | Latest official IFC release identified by buildingSMART | Use IFC for exchangeable built-asset semantics and exact schema-version tests |
| buildingSMART IDS 1.0 | Official buildingSMART standard since 1 June 2024 | Test machine-readable information requirements, but do not treat an IDS pass as geometry, material, installation or physical-performance proof |
| ISO 10218-1:2025 / ISO 10218-2:2025 | Published February 2025; current editions | Use Part 1 for industrial robot safety and Part 2 for application/cell integration, commissioning, operation, maintenance and decommissioning; require competent integrator and safety review for the guarded cell |
| NIST IR 8536 | Final 9 September 2026 | Align provenance composition with cryptographic linkage, selective disclosure, source-native records and cross-industry traceability |
| NIST 2026 AI/ML Roadmap for Smart Manufacturing | Published 3 July 2026 | Include trustworthy AI, industrial data contextualization, robotics, sustainability, reliability and VVUQ |
| NIST TIDAL / SI for Manufacturing research | Active research programme in 2026 | Benchmark layered intelligence, human-machine teaming, integration and digital-twin assurance; do not confuse research direction with a finalized standard |

Primary references:
- ISO 23247-5:2026: https://www.iso.org/standard/87425.html
- ISO 23247-6:2026: https://www.iso.org/obp/ui?_escaped_fragment_=iso%3Astd%3Aiso%3A23247%3A-6%3Aed-1%3Av1%3Aen
- ISO/AWI 23247-7/-8 catalogue: https://www.iso.org/committee/54158/x/catalogue/
- ISO/AWI 23247-8 VVUQ: https://www.iso.org/standard/94858.html
- IFC releases: https://standards.buildingsmart.org/
- IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- NIST IR 8536: https://csrc.nist.gov/pubs/ir/8536/final
- ISO 10218-1:2025: https://www.iso.org/standard/73933.html
- ISO 10218-2:2025: https://www.iso.org/standard/73934.html
- NIST 2026 AI/ML roadmap: https://www.nist.gov/publications/2026-roadmap-artificial-intelligence-and-machine-learning-smart-manufacturing
- NIST Super Intelligence for Manufacturing: https://www.nist.gov/programs-projects/super-intelligence-si-manufacturing
- NIST Digital Twins Workshops Summary (NIST IR 8620): https://www.nist.gov/publications/digital-twins-workshops-summary-report

### What leading commercial platforms already do

This is a capability map based on public product descriptions, not an independent scorecard.

| Platform | Publicly described strength | Fair matched test for our niche |
|---|---|---|
| Autodesk Tandem | Converts BIM/facility data into an operational twin, linking assets, documents, IoT data and integrations | Can a panel's exact design, delivered lot, installed identity, inspection and repair evidence be joined without losing source revisions, provenance or decision limits? |
| Bentley iTwin | Integrates engineering data from diverse tools with reality data and IoT for infrastructure twins | Can federated engineering and as-built records preserve transformation lineage, uncertainty and exact asset identity under conflicting observations? |
| Siemens Xcelerator / Digital Twin Composer | Composes product/process/factory context with simulation and real-time physical information; Digital Twin Composer was launched at CES 2026 and described as early access with selected customers | Can a cross-vendor composition independently detect stale calibration, changed tool/process configuration, unsupported semantics and false confidence before a release decision? |
| PTC Windchill | Enterprise PLM, configuration/change management, BOM transformation, process planning and lifecycle traceability | Can a cross-owner evaluator preserve independent supplier custody, physical measurement and third-party evidence while calculating precisely which downstream evidence needs requalification? |
| Dassault 3DEXPERIENCE | Connects CATIA, SIMULIA, DELMIA and ENOVIA product, simulation, manufacturing and lifecycle processes | Can the evidence layer reproduce a defensible decision across organizational and vendor boundaries without assuming one platform's internal model is the authority for all other owners? |

References:
- Autodesk Tandem: https://www.autodesk.com/products/tandem/overview
- Bentley iTwin Platform: https://developer.bentley.com/itwinplatform/
- Siemens Digital Twin Composer announcement: https://news.siemens.com/en-us/digital-twin-composer-ces-2026/
- PTC Windchill: https://www.ptc.com/en/products/windchill
- Dassault 3DEXPERIENCE: https://www.3ds.com/products/3dexperience/

Important: Siemens' public announcement reports customer deployment results, but the scenarios and baselines are not necessarily comparable to this panel-cell workload. Vendor-reported numbers must not be converted into a leaderboard. For all platforms, a capability not exposed in a given product/configuration should be recorded as **not exposed / not tested**, not presumed absent.

### Industrialized construction adoption context

Autodesk's 2026 Spotlight on Industrialized Construction reports that regular or standard use of productization, DfMA and prefabrication ranges from 37% to 40% among its respondents, while 70% expect industrialized construction to become standard. In supplemental Dodge research, traditional procurement was the leading barrier to wider DfMA/productization adoption (51%). The report describes a survey of 869 industry leaders and experts plus supplemental Dodge research; it is a vendor-sponsored industry survey, not a neutral measurement of all global construction activity or a matched comparison of technical performance.

This is a useful market signal: the differentiator is not one successful prefabrication demo, but repeatable, procurement-compatible delivery with validated quality, traceability and predictable outcomes. Our benchmark should measure hand-offs, scope changes, supplier evidence and accepted output across multiple projects—not simply compare a robot cell's best cycle time.

Reference: https://www.autodesk.com/design-make/articles/2026-state-of-design-make-spotlight-on-industrialized-construction

### Construction-robotics evidence gap and field-work envelope

A June 2026 PRISMA-based systematic review synthesizes 375 construction-robotics papers published from 2023–2025. It reports that applications are concentrated in structural-construction tasks, with planning, maintenance and demolition comparatively underexplored; operator-led workflows remain prevalent. The review is a map of published research, not a census of commercially deployed robots or a validated productivity leaderboard.

This suggests a stronger research opportunity than another structural automation demonstration: a **continuous product-to-service loop** that links design-for-assembly and robotic inspectability to installation, condition observation, repair, disassembly and re-verification. The benchmark should cover pre-construction planning, manufacturing, installation and service/repair—not just the robot's primary task.

Source: Xu et al., *Construction robotics: a systematic review of robot types, applications and human robot collaborations*, Results in Engineering, June 2026, DOI: https://doi.org/10.1016/j.rineng.2026.110406

NIOSH's 7 October 2026 construction-robotics bulletin identifies dynamic workspaces, overlapping trades, uneven terrain, weather, dust, changing light, visual obstructions, dropped/unstable loads, sensor degradation, communications failures, software/firmware changes and recovery behavior as material hazards. These conditions differ from stable manufacturing cells.

Source: https://www.cdc.gov/niosh/bulletin/2026/safety-in-construction-robotics.html

**Benchmark boundary:** the first physical Highveld panel demonstrator remains a guarded, non-occupied, non-load-bearing cell. The dynamic-site cases belong in a separate digital/controlled test suite and later approved field pilot. They must not be used to imply that an indoor cell qualification transfers to mobile, public, occupied or work-at-height operations.

### NIST test and measurement precedents

The NIST Smart Manufacturing Systems Test Bed connects CAD, CAM, inspection, product-data management, CNC equipment, CMM/digital micrometers and MTConnect data. It provides a practical reference for connecting design, fabrication and inspection data in one measured workflow: https://www.nist.gov/laboratories/tools-instruments/smart-manufacturing-systems-sms-test-bed

NIST's collaborative robotics work emphasizes task decomposition, coordination, role allocation, communication, situational awareness and collective human/robot performance—not just robot speed: https://www.nist.gov/programs-projects/performance-collaborative-robot-systems. NIST's robotic performance-assessment work also points to system-level assembly-performance models and test methods rather than isolated robot component speed: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems

NIST's 2026 Super Intelligence for Manufacturing initiative organizes work around TIDAL (Teamed, Integrated, DT-assured and Layered Intelligence), with research on trustworthy intelligence, human-machine teaming, contextualized industrial data, integration/benchmark studies and digital-twin VVUQ. Its planned integration test bench explicitly compares alternative information architectures (including UNS/MQTT, knowledge graphs/type models, I3X and OPC UA profiles), beginning with Tennessee Eastman Process and planning extensions to physical NIST Digital Twin and Additive Manufacturing Data Integration Lab equipment. NIST's 2026 AI/ML roadmap also identifies heterogeneous industrial data, sensing/control integration and trustworthy, explainable, reliable operation as deployment barriers.

These are methodological comparators, not certification programmes. Our differentiation should not be a novel architecture acronym or a generic knowledge graph. It should be independent, reproducible evidence that a cross-owner workflow improves decision correctness, traceability, requalification and accepted physical outcomes under adversarial change. We should reproduce the spirit of physical measurement and task-level metrology, not imply that a synthetic corpus is equivalent to a test bed or that passing our benchmark establishes ISO conformity.

## 3. Cross-repository owners — reuse, do not duplicate

| Concern | Existing owner | What Highveld consumes |
|---|---|---|
| Design intent, IFC/IDS projection, spatial context, exact source revisions | Sol-Atlas #622 and existing ITIP work; #870 corpus | Source-bound design and requirement references; no new BIM authority |
| Engineering change impact and evidence applicability | Symthaea #10543 and #6040 | Affected configuration, evidence applicability and recom­missioning projection |
| Decision-sufficient evidence acquisition | Symthaea #12046 | Candidate evidence requests, decision-critical blocker and abstention result |
| Observation/measurement campaign planning under shared resources and spatial constraints | Symtropy #4988 / #5164, implementation #5240 | Resource-/space-time-feasible multi-observation plan; plan coverage is not accepted evidence |
| Production-option + option-specific evidence-campaign composition | Symtropy #5250 / draft PR #5260, consuming #5238 + #5240 | Existing owner-native option/evidence package and joint Pareto result; Highveld must not duplicate its optimizer |
| Material lot/custody/quality/kit admission | Symtropy #5321 | Exact readiness result and its source evidence |
| Manual / assisted / guarded robotic panel-cell production | Symtropy #3773 | Matched route inputs, whole-workflow labour and accepted panel outputs |
| Robot inspection, as-built reconstruction, repair and re-verification | Symtropy #4043 | Inspection/repair observations and new configuration identity |
| Construction provenance, custody, approval scope, correction and revocation | Mycelix #5149 | Owner-native provenance and approval receipt references |
| Assurance-obligation acquisition / decision-conditioned value of information | Mycelix #6252 | Admissible evidence plans and decision-leverage estimates |

This protocol does not invent field names for any owner package. Until a particular owner profile, schema and version are pinned, that projection is **UNBOUND** and the integration test must not infer semantics from a similar-looking field. The cross-stack envelope should carry exact references and digests, not copy source-of-truth records.

**Important existing composition:** Symtropy #5260 already combines its own production optioneering (#5238) with the option-specific observation campaign planner (#5240) and computes the joint package/Pareto result. Highveld is not proposing a second such evaluator. Its new cross-repository research question is whether Mycelix #6252's proof-obligation coverage and admissibility gates (authority, rights/source-use, safety, schedule, physical realizability and data access) can be bound to those existing Symtropy option/evidence packages without weakening either owner's semantics. The current Mycelix #6252 record is an open research issue; do not claim a production bridge exists.

The first integrated run should be a read-only, synthetic replay. It must not dispatch a robot, reserve an actual production resource, send a supplier request, accept material or authorize installation.

## 4. Benchmark arms

Run the same frozen cases and work package through these arms:

- **B0 — Manual document workflow:** skilled reviewer using the source packages and ordinary search/spreadsheet tools. Record reviewer time and unresolved questions.
- **B1 — Point-to-point integration:** best available exports/APIs/scripts from the participating source systems, preserving the native records.
- **B2 — Owner-native incumbent platform:** use the appropriate Autodesk, Bentley, Siemens, PTC or Dassault product where it is genuinely applicable. Record product version, licensed modules, adapters, configuration and services used.
- **B3 — Standards-first federation:** IFC/IDS and relevant owner/vendor-native digital-thread or traceability standards; no additional evidence-gating intelligence beyond the tested implementation.
- **B4 — Luminous composition:** the same source artifacts with exact-head Sol-Atlas, Symthaea, Symtropy and Mycelix owner outputs, evaluated by the independent panel-thread qualifier.
- **B5 — Expert reference:** two independent qualified reviewers adjudicate each case without seeing B4 output; a third reviewer resolves disagreement.

All arms receive the same source package, time budget, compute budget, disclosure permissions and task instructions. Hide gold labels and adversarial mutations from candidate operators. Publish versions, configuration, adapters, cost model and exclusions. Do not give B4 hidden-oracle access.

## 5. Scenario suites

The initial 24 scenarios in #870 are a seed, not a representative SOTA sample. Expand with controlled mutations and blinded holdouts.

| Suite | Challenge | Required negative controls |
|---|---|---|
| S1 Identity and revision | Exact panel/article, lot, design, drawing, tool and installed identity | Duplicate IDs, fuzzy joins, reused IDs after replacement, revision mismatch |
| S2 Semantic fidelity | Units, coordinate frames, tolerances, process meaning and IFC/IDS properties | Unit conversion without transform, semantic collapse, unknown field silently coerced |
| S3 Time and currentness | Event/effective/observation/knowledge/retrieval times | Stale certificate, clock aliasing, late correction, expired calibration |
| S4 Authority and rights | Signer scope, delegation, source license and permitted purpose | Authenticated-but-unauthorized signer, overbroad disclosure, revocation |
| S5 Material provenance | Lot, custody, receiving, substitution and certification | Counterfeit/duplicate lot, missing custodian, revoked certificate |
| S6 Process/robot capability | Exact tool, recipe, machine, robot, controller and calibration envelope | Tool swap, stale TCP, out-of-envelope process, unsupported variant |
| S7 Metrology and VVUQ | Reference measurement, uncertainty, coverage and model applicability | Predicted observation used as measured, uncertainty hidden, out-of-domain estimate |
| S8 As-built and repair | Installed article, inspection, repair, changed configuration and re-verification | Design intent treated as physical truth, missing inspection, negative evidence deleted |
| S9 Common cause | Independence of certificates, sensors, providers and review sources | Two records share one upstream source but are counted as independent |
| S10 Decision/action boundary | Distinguish plan, recommendation, execution, observation, acceptance and authority | Planner output treated as execution; work-order completion treated as product acceptance |
| S11 Evidence acquisition | Choose measurements/reviews that can change the current decision | High information gain but no decision leverage; rights-blocked request; impossible acquisition |
| S12 Repair/disassembly | Service access, reversible joints, recovery path and material recovery | Missing removal capability, unqualified replacement, repair creates new configuration without retest |

Mutations must be generated from preregistered operators, then independently labelled. Keep scenario generation seed, corpus digest and adjudication record outside the candidate input. Report results separately by suite; aggregate accuracy must never mask a critical false-clear.

## 6. Metrics — report families, never a single composite score

### A. Assurance and semantic correctness

- **Critical false-clear count:** number of hard-negative cases where a candidate recommends a permissive disposition. Primary hard gate: zero on adjudicated hidden critical cases.
- **Critical blocker recall:** target 100% on the adjudicated critical set.
- **Benign false-block rate:** target at most 5% on adjudicated benign/repairable cases, conditional on the critical false-clear gate remaining at zero.
- **Exact disposition accuracy:** target at least 95% on held-out adjudicated cases; report confusion matrix and calibration by severity.
- **Subject/identity precision:** target zero false exact joins; unresolved/fuzzy identity remains Unknown.
- **Required semantic preservation:** target at least 99.5% of adjudicated required fields preserved; 100% of unsupported/lossy transforms explicitly surfaced.
- **Change-impact recall:** target 100% for safety/quality-critical affected evidence and at least 95% overall against the adjudicated dependency oracle.
- **Requalification efficiency:** reduce unnecessary full requalification by at least 25% against the fixed-checklist baseline without increasing critical false-clears.
- **Historical integrity:** zero deletion/overwrite of negative, disputed, superseded or revoked source events.

### B. Evidence acquisition and operator efficiency

- Decision-critical blocker identification accuracy.
- Requests per defensible decision; unnecessary request count.
- Acquisition cost, latency, reviewer-hours and disclosure burden.
- Percentage of candidate requests admissible under authority, rights, source competence, resource and safety constraints.
- Decision regret, robust-safe action-set change and sensitivity to source dependence.
- Abstention correctness when no admissible acquisition can resolve the blocker.

Research targets to test, not results:
- at least 95% decision-critical blocker identification on hidden adjudicated cases;
- at least 30% fewer unnecessary requests than collect-everything;
- at least 20% lower median expert reconstruction/review effort;
- zero selection of blocked, unknown, rights-ineligible or physically infeasible requests;
- zero promotion from a planned acquisition to an observed or qualified measurement.

### C. Digital-twin credibility and measurement

- Calibration of prediction intervals and coverage by output quantity.
- Independent checkpoint error, registration stability and repeatability.
- Uncertainty calibration, out-of-domain detection and selective risk/coverage.
- Frequency of unsupported extrapolation and erroneous certainty.
- Model applicability across configuration, material, tool, process and site-context change.

For a model expected to cover 95% of outcomes, target empirical coverage near 95% on held-out data, with tolerance pre-registered per quantity and a separate sharpness measure. Never improve nominal coverage by making intervals uselessly wide. Use NIST/ASME/JCGM-aligned VVUQ review proportionate to decision consequence.

### D. Physical production and serviceability

The panel-cell route comparison must consume the accepted-output and whole-workflow labour denominator already owned by Symtropy #3773. Report:
- accepted panels per total workflow labour-hour;
- first-pass and final accepted yield;
- p50/p90/p95 cycle and changeover time;
- direct and indirect labour, supervision, kitting, replenishment, setup, cleaning, inspection, rework and recovery;
- defect rates by class, scrap mass and rework hours;
- energy/material/water/transport per accepted panel;
- downtime, availability, tool wear and recovery;
- human exposure removed and new exposure introduced;
- service access, repair time, replacement burden and recovered material.

No physical target is a verified result until the same functional unit, input quality, acceptance predicate, task boundary, equipment envelope and independent metrology are used across routes.

Initial hypotheses for a controlled pilot:
- at least 20% reduction in total accepted-panel lead time versus the matched manual baseline;
- at least 10 percentage-point improvement in first-pass acceptance or equivalent demonstrable quality improvement;
- at least 25% reduction in rework labour;
- at least 15% reduction in material waste per accepted panel;
- no increase in serious safety events or unprotected hazardous exposure.

These are challenge targets, not predictions. Pre-register sample sizes and uncertainty before the physical comparison. A small demo is not enough to substantiate production superiority.

### E. Security, resilience and governance

- Tamper, replay, substitution and source-compromise detection.
- Revocation propagation and time-to-degraded/blocked disposition.
- Reconstruction consistency after coordinator or adapter replacement.
- Common-cause dependency detection.
- Selective disclosure volume and purpose compliance.
- Recovery after partitions, delayed events and corrections.
- Human contestability, review and correction response.

Hard gates remain separate from economic benefit. A safety, legal, rights, authority or material-origin failure cannot be compensated by throughput.

## 7. Independent adjudication materials

Two review artifacts have been added:
- `HIGHVELD_KEEP_PANEL_THREAD_REVIEW_PACKET_V0_1.json` removes the provisional expected disposition, reason codes, requalification domains and criticality labels from each scenario.
- `HIGHVELD_PANEL_THREAD_ADJUDICATION_WORKSHEET_V0_1.md` provides separate first-pass registers for two independent reviewers and a third-party disagreement resolver.

The redacted packet is a convenience to reduce accidental anchoring, **not a hidden benchmark**: the source gold-label corpus remains public and retrievable. The adjudication protocol must state this limitation. True blind evaluation requires an access-controlled holdout or trusted external custodian, and a signed adjudication manifest that binds final labels to exact source and packet digests. The Rust evaluator deliberately does not trust a free-form corpus status string.

## 8. Statistical design and claim thresholds

1. **Development:** the public 24-case #870 corpus; visible and unadjudicated. Use it for implementation debugging only.
2. **Adjudicated validation:** at least 300 cases, two independent reviewers, third-person adjudication of disagreements, frozen gold labels and an explicit disagreement log.
3. **Hidden adversarial holdout:** expand toward at least 3,000 critical mutations stratified by failure family, with families and random seeds not exposed to candidate systems.
4. **Field/physical validation:** collect real observations with calibrated instruments, independent acceptance, lawful rights and controlled operational conditions; keep this as a separate evidence class.

For zero false-clears in 3,000 *independent* critical trials, the simple one-sided 95% binomial upper bound is approximately 0.1% (rule-of-three approximation). This is not a field-risk claim: correlated mutations, shared fixtures or synthetic distributions invalidate the independence assumption. Report exact counts and uncertainty by failure family, and use clustered analysis where cases share a source or generator.

The benchmark must preregister:
- case sampling and mutation generation;
- label protocol and adjudicator qualifications;
- train/development/validation/hidden-holdout separation;
- stopping rules and failure handling;
- compute/hardware/software environment;
- baseline versions and service/configuration cost;
- statistical analysis and exclusions.

## 9. Implementation and execution gates

- **P0 — corpus and exact-head software:** closed schemas, 24-case corpus, deterministic evaluator, mutation controls and pinned owner refs.
- **P1 — independent label adjudication:** no candidate-independent pass status until a signed adjudication manifest binds reviewers, protocol, case labels and corpus digest.
- **P2 — adapter composition:** consume only pinned owner-native projections; unresolved mappings stay UNBOUND.
- **P3 — blinded cross-platform replay:** B0–B5 arms; no vendor claim is presumed.
- **P4 — benign physical cell:** non-load-bearing, non-occupied, guarded workcell; metrology and professional safety gates separately qualified.
- **P5 — production/service comparison:** repeat manual, assisted and guarded-robot trials and include repair, disassembly and maintenance.

A phase is not complete because a CI job is queued, skipped or absent. Each gate requires exact repository and commit identity, fixture digests, toolchain, test result and independent evidence appropriate to the claim.

## 10. The SOTA strategy

Do not claim to beat Autodesk at building-operations visualization, Bentley at infrastructure geospatial composition, Siemens at factory simulation, PTC at PLM workflows or Dassault at integrated product/industrial design.

Aim to outperform a **matched incumbent workflow** on the cross-system tasks those suites need to support but where success is not established by rendering, data connection or workflow completion alone:

1. fewer false identity and semantic joins;
2. faster and more complete change-impact determination;
3. zero critical stale/revoked evidence false-clears;
4. lower evidence acquisition and requalification burden;
5. better calibrated uncertainty and truthful abstention;
6. higher accepted output per full-workflow labour-hour;
7. lower repair/rework/waste cost over the product lifecycle.

Build the independent evaluator, open challenge corpus, adapters and evidence protocol. Buy or integrate specialist CAD/BIM/PLM/CAE/MES/QMS/robot products where they are superior. An open, independently replayable assurance layer can complement proprietary execution suites without claiming to replace their engineering depth.

## 11. Claim ceiling

A benchmark pass establishes agreement with an independently adjudicated, frozen test set under its declared environment. It does not, by itself, establish physical material properties, structural adequacy, production capability, certified robot safety, statutory building compliance, professional sign-off, construction permission or occupancy. Those remain distinct owner and professional-authority decisions.
