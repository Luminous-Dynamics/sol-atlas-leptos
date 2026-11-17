# Highveld Keep v0.1 — preliminary desk review and professional review plan

**Review date:** 7 November 2026  
**Reviewed branch:** `research/highveld-keep-v0.1`  
**Reviewed commit:** `c52a336140d023c3721ba946771fd2499219ccaf`  
**Companion contract-checker PR:** [#610](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/610), stacked on this branch  
**Parent issue:** [#607 HIGHVELD-KEEP-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/607)  
**Review type:** AI-assisted document / software desk review; not a registered-professional review, design certificate, statutory submission or approval.

## 1. Disposition

**Disposition: suitable to continue as an E0 synthetic reference-design and professional-scoping package. Not suitable for site selection, detailed design, procurement, construction, occupation or claims of safety/buildability/compliance.**

This is a favourable review of the *research contract and its present boundaries*, not of a physical building design. The package deliberately contains no selected parcel, surveyed geometry, approved architectural drawings, structural calculations, fire strategy, verified local climate file, product approvals, construction costs, or statutory approvals. The correct disposition is therefore **continue concept work and commission staged independent review; do not promote the evidence tier**.

No registered architect, engineer, fire engineer, quantity surveyor, building-control official, public-health professional or independent third-party reviewer has signed this document. No professional liability is assumed or transferred.

## 2. Exact review scope and evidence

The review inspected the four source documents on the branch:

| Artifact | Blob SHA |
|---|---|
| `HIGHVELD_KEEP_DESIGN_BASIS_V0_1.md` | `84dff04b924477427f6bd4d6adaa423664244d66` |
| `HIGHVELD_KEEP_BENCHMARK_PROTOCOL_V0_1.md` | `ec38e66fb105ee280f1b783ed37d1be97e6e9518` |
| `HIGHVELD_KEEP_SCENARIO_V0_1.json` | `e83a4221b41f9ab187163efd5d7dcf1d3fa5b1fe` |
| `HIGHVELD_KEEP_PARAMETER_SPACE_V0_1.json` | `6888a88f35398fddd495e8db7cd84e62a9414e93` |

Software evidence checked:
- [Design-package PR #608](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/608), exact head `c52a336140d023c3721ba946771fd2499219ccaf`: WASM check, native tests (7 passed) and formatting workflow succeeded in [run 42404806526](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/42404806526).
- [Contract-checker PR #610](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/610), exact head `33c1a57609dc2ccc8009325e6d9fa7bb62eb7f66): 38 isolated Rust tests passed; CLI reported `definition_valid_synthetic_only` with zero violations in [run 42398400572](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/42398400572). Its repository Check workflow passed in [run 42398400591](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/42398400591).

These runs establish code/package checks only. They do not independently verify the design assumptions, validate any weather archive, or prove physical performance. The checker itself correctly states that it is not a geometry, structural, fire, energy, zoning or approval validator.

## 3. Strengths observed

1. **Claim control is unusually clear.** The design brief identifies itself as E0 and synthetic; the scenario keeps parcel, coordinates, survey, zoning, geotechnical report and dolomite assessment unset. The 60 m × 80 m envelope is explicitly synthetic and cannot be confused with a surveyed or zoned property.
2. **Plural alternatives are preserved.** The four spatial families are not presented as one predetermined answer. The benchmark compares multiple material/production paths and requires identical briefs and bounded assumptions.
3. **Hard constraints are not hidden in a weighted score.** Access, fire/life-safety, structural and approval gates are separated from preferences and economic/environmental objectives.
4. **Climate claims are cautious.** The five Gauteng TMYx records remain separate candidates, with licence, header, representativeness and source-hash questions unresolved. Typical-year screening is not substituted for heatwaves, design extremes or future climate.
5. **The delivery model is interdisciplinary.** The package connects intent, geometry provider qualification, specialist solvers, openBIM, materials, fabrication, inspection and lifecycle evidence without claiming that any one repository replaces the relevant professional.
6. **The software contract is reproducible and bounded.** The nested Rust checker and exact-head workflow are good E1 package-integrity evidence.

## 4. Findings requiring closure before a real-site concept review

| ID | Severity | Finding | Required closure |
|---|---|---|---|
| HK-PR-01 | Blocker | No real parcel, title, cadastral survey, orientation, topography, servitudes or zoning evidence exists. | Select a parcel only after owner authority and legal/title checks; bind a surveyor-issued survey and current municipal planning record. |
| HK-PR-02 | Blocker | Johannesburg/Highveld ground conditions, including dolomite risk, are unknown. | Parcel-specific geotechnical investigation and dolomite-risk assessment by an appropriately experienced, registered geotechnical professional before foundation concepts are accepted. |
| HK-PR-03 | Blocker | No architectural geometry or coordinated drawings have been produced. | SACAP-registered architectural professional leads concept design; provide dimensioned plans/sections/elevations, site plan, levels, room/task layout, accessible routes and coordinated model exchanges. |
| HK-PR-04 | Blocker | No structural system, load paths, foundations, retaining design, connections or rational calculations exist. | ECSA-registered structural/geotechnical professionals accept a defined design basis and produce calculations, drawings, assumptions and independent checking appropriate to complexity. |
| HK-PR-05 | Blocker | Fire strategy, means of escape, fire service access, detection/suppression and emergency response are not designed. | Competent fire-safety professional develops a code/jurisdiction-specific fire strategy; coordinate with building control and relevant emergency services. |
| HK-PR-06 | Blocker | Accessibility is currently a task/relationship intention, not a dimensional route or usability proof. | SACAP lead plus an appropriately competent accessibility specialist and resident/user testing; independently measure routes, thresholds, turning, reach, sanitary use and emergency egress against the applicable requirements. |
| HK-PR-07 | Major | Weather records are candidate sources only; no exact archive has been inspected, hashed, licence-cleared or admitted. | Independently inspect exact file headers, units, timestamps, missingness, source period, station metadata, rights and representativeness; compare relevant observations and retain separate extreme/future scenarios. |
| HK-PR-08 | Major | Thermal, ventilation, daylight, indoor-air-quality, moisture and overheating performance are unmeasured and unsimulated for actual geometry. | Run exact, version-pinned specialist analyses after geometry and inputs are fixed; define calibration/validation data and out-of-domain conditions. Field measurements are required before claiming occupied performance. |
| HK-PR-09 | Major | Secure night ventilation and vector exclusion may conflict with resident security, airflow, egress, noise and outdoor-air quality. | Apply the new VH-012 case from [Symtropy #4095](https://github.com/Luminous-Dynamics/symtropy/issues/4095): observe actual occupied night use, jointly test security and airflow, and award no ventilation credit when windows are closed. Use local entomology/public-health input; do not transfer Kenyan pilot effect sizes to Johannesburg. |
| HK-PR-10 | Major | Water, stormwater, wastewater, energy, electrical protection, backup power and utility connection capacities are unverified. | Obtain provider/site data and discipline-specific design; qualify potable/non-potable separation, storage, backflow, drainage, electrical protection and outage transitions. |
| HK-PR-11 | Major | Material/product pathways and robotic fabrication assumptions lack actual supplier, certification, grade, fire/durability, tolerance and maintenance evidence. | Prepare a material/product compliance schedule, supplier traceability and mock-up/testing plan; robotics remain a separate supervised, guarded workcell qualification. |
| HK-PR-12 | Major | No quantity-surveyed cost plan or lifecycle cost evidence exists. | Appoint a South African quantity surveyor for cost plan, contingencies, escalation, fees, professional services, utility/site works, maintenance/replacement and whole-life scenarios. |
| HK-PR-13 | Major | Cultural, aesthetic and resident acceptance are design intentions, not validated outcomes. | Conduct paid, consented resident/community and local creative-practitioner reviews; document dissent and trade-offs rather than compute a universal beauty score. |
| HK-PR-14 | Major | Statutory and regulatory status can change between this research cut and submission. | At each gate, obtain a dated legal/approval register from the local professional team and confirm municipal, SACAP, ECSA, SANS, fire, environmental/heritage and home-builder obligations. |

## 4A. Additional software-assurance finding

A review of the companion checker PR #610 confirmed its stated 38 passing tests and exact claim ceiling, but identified a gap in the parser boundary: the implementation deserializes into generic `serde_json::Value` and validates selected paths. The current tests do not prove that unknown fields at arbitrary nested depths or duplicate raw JSON object member names are rejected.

Follow-up [HIGHVELD-KEEP-STRICT-JSON-001 #626](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/626) now defines the strict-schema and duplicate-key work. This does not mean the current Highveld files contain malformed or hostile JSON; it means the validator should not yet be represented as a complete strict JSON schema boundary. Preserve the current E1 package-integrity claim ceiling until #626 closes.

## 5. South African professional and statutory review pathway

The project is in Johannesburg, so the review team should use the actual City of Johannesburg submission process and current national/municipal rules rather than a generic international checklist.

- **Architecture:** South African Government guidance states that providing architectural services without SACAP registration is illegal. Use a suitably registered, competent architectural professional for the actual service and building-plan work; candidate practitioners work under professional supervision. Source: https://www.gov.za/services/services-residents/world-work/professional-registration/register-architect and SACAP public guidance: https://sacapsa.com/for-public.
- **Engineering:** ECSA registration is required for specified consulting engineering, responsibility and statutory engineering functions. Select the registration category and discipline to match the actual engineering work; verify current registration and scope rather than relying on job title. Sources: https://www.gov.za/services/services-residents/world-work/professional-registration/register-professional-engineer and https://www.ecsa.co.za/enforcement-and-conduct/identification-of-engineering-work/.
- **Johannesburg building control:** the City's Construction Permit Management System states that building-plan submissions are made by a registered professional and provides for municipal review, approval/refusal, commencement notification and occupancy certificates. Source: https://cpms.joburg.org.za/ and https://joburg.org.za/services_/Pages/City%20Services/Building%20Plans/How-do-I-submit-the-plan.aspx.
- **Building law:** the National Building Regulations and Building Standards Act 103 of 1977 remains a core statutory reference; approval must be sought from the relevant local authority. The exact current amendments, applicable SANS 10400 parts/editions and municipal requirements must be checked at submission. Sources: https://www.gov.za/documents/national-building-regulations-and-building-standards-act-16-apr-2015-1302 and https://www.gov.za/sites/default/files/gcis_document/201504/act-103-1977.pdf.
- **Construction health and safety:** use the applicable Occupational Health and Safety Act and Construction Regulations in force at tender and construction; obtain client/design risk assessments and competent-person appointments relevant to the actual work. The official 2014 Construction Regulations are here: https://www.gov.za/nr/documents/notices/occupational-health-and-safety-act-regulations-construction-07-feb-2014. The 2025 draft and the 2026 National Built Environment and Construction Safety Framework are policy/watch items until the relevant final instruments and commencement are confirmed.
- **Home-builder protection:** the Housing Consumer Protection Act 25 of 2024 is published, but the Government's current Act page says commencement is “to be proclaimed.” Do not assume its provisions have commenced; check the latest Gazette and the transitional/current NHBRC regime before any contract or construction. Sources: https://www.gov.za/documents/acts/housing-consumer-protection-act-25-2024-english-siswati-29-jan-2025 and https://www.gov.za/documents/housing-consumers-protection-measures-act.

This desk review is not legal advice. The project team must obtain a dated applicability opinion from its South African professionals and relevant authorities.

## 5A. Professional registration scope clarification — checked 7 November 2026

The review matrix uses discipline- and task-matched practitioners; it does not assume that every architectural task must be signed by a Professional Architect or that every engineering task requires a Professional Engineer specifically.

- SACAP lists four architectural professional registration categories: Professional Architect, Professional Senior Architectural Technologist, Professional Architectural Technologist and Professional Architectural Draughtsperson. The actual service, project type, scale/complexity and legal submission role must be allocated within the person’s registered category and competence: https://sacapsa.com/registration/categories.
- ECSA’s Identification of Engineering Work Rules define and demarcate engineering activities within registration categories and disciplines. ECSA states those Rules and the Enforcement Rules are gazetted and in effect; the official Rules PDF gives an effective date of 19 September 2025. The appointment must therefore match the relevant category, discipline and actual identified work, not rely on a job title alone: https://www.ecsa.co.za/about-ecsa/idoew-and-enforcement-rules/ and https://www.ecsa.co.za/enforcement-and-conduct/identification-of-engineering-work/.
- The Government's registration page confirms that ECSA registration is statutory for consulting engineering work, taking responsibility for engineering work and specified statutory functions (including the National Building Regulations): https://www.gov.za/services/services-residents/world-work/professional-registration/register-professional-engineer.

The professional appointment schedule should record the exact registration category, discipline, IDoEW scope, competence evidence and any specialist registration or authority separately for each work package. Where a service lies outside the reviewer's scope, record the appropriate additional competent professional/authority rather than stretching the first reviewer's remit.

## 6. Required independent review panel

Reviewers should be independent of the design author where practical, appropriately registered/competent for the work, and explicitly appointed to a bounded scope.

| Review seat | Minimum scope | Required submission from design team |
|---|---|---|
| SACAP-registered architectural professional | Brief, spatial planning, building-plan pathway, coordination and design responsibility | Site/architectural drawings, schedules, design basis, compliance register |
| ECSA-registered structural engineer | Structural system, loads, foundations, materials, connections, constructability | Geotechnical report, structural design basis, calculations, drawings, product data |
| ECSA-registered geotechnical professional | Dolomite/ground hazards, foundations, earthworks, stormwater interactions | Survey, geotechnical investigation, site history and proposed foundation assumptions |
| Fire-safety professional | Fire strategy, egress, compartmentation, response access and systems | Occupancy/use assumptions, plans/sections, materials, fire-service access and services |
| Mechanical/building-physics professional | Thermal comfort, ventilation, indoor air, moisture and energy | Weather files, geometry, occupancy schedules, materials, solver versions and validation data |
| Electrical professional | Supply, distribution, protection, backup, PV/battery and safe isolation | Load schedule, utility constraints, single-line diagrams, equipment specifications |
| Civil/water professional | Potable/non-potable, drainage, stormwater, sanitation, site grading | Survey, hydrology, utility letters, catchment and water-demand model |
| Accessibility specialist and resident representatives | Independent use/route/egress and lived-experience review | Dimensioned routes, bathrooms, thresholds, door/circulation schedules, mock-ups |
| Quantity surveyor | Capital and lifecycle cost, risk/contingency, procurement assumptions | Measured quantities, specifications, rate basis, alternatives, lifecycle assumptions |
| Public-health entomologist/vector specialist | Local species, sampling design, vector endpoint and interpretation | Local ecological context, screening details, trap protocols, species/effort metadata |
| Environmental/heritage/landscape specialists as applicable | Ecology, biodiversity, heritage, stormwater landscape and approvals | Parcel/context data, species/material schedule, constraints register |
| Competent construction H&S professional | Design risk, safe fabrication/assembly/maintenance and construction method | Construction sequence, work packages, lifting/access, workcell and maintenance hazards |

A professional's review must state the exact scope and registration category, what was checked, exclusions, unresolved assumptions, findings and required actions. A single general approval must not be inferred from a reviewer who only checked one discipline.

## 7. Review process and acceptance rules

Each review record should bind:
- exact document/model/fixture revisions and SHA-256 digests;
- reviewer identity, professional body, registration number/category and verification date;
- discipline, jurisdiction, appointment and reviewed scope;
- applicable legislation, code and standard editions;
- assumptions and evidence supplied;
- finding severity: Blocker / Major / Minor / Observation;
- disposition: Accept within scope / Accept with conditions / Revise and resubmit / Not reviewed;
- conflicts of interest and limitations;
- reviewer signature/date and any required independent checking.

**No software test, AI desk review, vendor certificate, model confidence or digital receipt substitutes for professional judgement, statutory approval or construction inspection.**

Recommended gates:
1. **G0 — synthetic concept:** present state. Software consistency only.
2. **G1 — site feasibility:** legal/title, survey, zoning, geotechnical/dolomite, utilities, hydrology, environmental/heritage and resident requirements.
3. **G2 — coordinated concept:** SACAP architect plus discipline reviewers identify fatal flaws before detailed design.
4. **G3 — technical design:** exact drawings, calculations, specialist analyses, specifications, cost plan and formal independent checks.
5. **G4 — statutory/tender readiness:** municipal approval, required professional certificates, approved products, contractor/builder registration and applicable home enrolment, H&S file and procurement.
6. **G5 — construction/commissioning:** inspections, tests, as-built records, defects, certificates and occupancy approval.
7. **G6 — in-use performance:** consented, privacy-preserving measured comfort, ventilation, vector, energy/water, defect and resident outcomes.

Progression requires every blocker closed by its responsible professional/authority. Unknown evidence remains Unknown; no weighted aggregate may override a blocker.

## 8. Professional review of Luminous' current research claims

| Claim / proposal | Current review judgement |
|---|---|
| Four spatial families are legitimate alternatives for research | Reasonable as a synthetic exploration set; not an exhaustive or resident-approved design set. |
| Evidence-carrying workflow may outperform connected AEC tools in handoff/revision/traceability | Worth testing; no comparative result exists yet. |
| Secure screened night ventilation can improve actual use and vector exclusion | Important hypothesis; must be tested with local climate, local vector ecology, resident use and fire/accessibility constraints. |
| Robotic or panelized construction may reduce cost | Unproven; compare full accepted output cost including design, tooling, labour, QA, rework, logistics, training and maintenance. |
| Highveld Keep is a safe or buildable dwelling | Not established; no claim permitted. |
| TMYx candidates represent a real Johannesburg parcel | Not established; no licence/header/representativeness admission yet. |
| Synthetic contract checker passed | Established only for its declared internal package invariants at the cited exact head. |

## 9. Next professional-review actions

1. Ask a SACAP-registered architect to review the design brief and determine what is needed for a real site-specific concept.
2. Ask an ECSA-registered geotechnical professional to define the minimum parcel/geology investigation, especially dolomite screening.
3. Ask a structural engineer and fire professional to review the concept families for fatal constraints before geometry becomes detailed.
4. Ask a building-physics/mechanical professional to define the secure-night-ventilation and thermal measurement protocol.
5. Ask a quantity surveyor to design the matched cost model before making any “more affordable” claims.
6. Ask an entomologist/public-health partner to define local vector sampling and the limits of inference.
7. Only after the real parcel and these reviews, commission detailed geometry, solver runs, building-plan preparation and product/fabrication packages.

## 10. Non-claims

This document does not certify design, structure, foundations, fire safety, accessibility, thermal performance, ventilation, vector reduction, public health, cost, buildability, code compliance, zoning, land title, materials, robotics, construction safety or occupancy. It is a preliminary desk review intended to make professional review more efficient and more accountable.
