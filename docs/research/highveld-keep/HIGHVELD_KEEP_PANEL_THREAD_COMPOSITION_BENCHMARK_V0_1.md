# Highveld Keep — Panel Thread Composition Benchmark v0.1

**Parent:** [HIGHVELD-KEEP-ROBOT-THREAD-001 #622](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/622)  
**Related integration pilot:** [Symthaea BUILT-ROB-PILOT-001 #6040](https://github.com/Luminous-Dynamics/symthaea/issues/6040)  
**Status:** synthetic benchmark specification; gold adjudication and executable evaluator pending  
**Safety boundary:** non-load-bearing envelope/service panel; guarded, low-energy laboratory context; no occupied-home robot execution

## 1. Purpose

This benchmark tests one missing question across the existing Luminous owners:

> When individually valid design, manufacturing, provenance, robot, inspection and as-built records are composed, do they still describe the same exact physical article, configuration, time frontier and authorized decision?

It is a **composition benchmark**, not another CAD/BIM system, product-lifecycle database, digital-twin graph, solver, manufacturing execution system, provenance ledger, robot controller or approval authority.

The first test article remains a synthetic, non-load-bearing Highveld envelope/service panel. It can exercise panel joints, fasteners, insulation placement, service access, replaceability, inspection access and disassembly. It cannot support claims about structural capacity, fire rating, whole-building energy performance, weatherproofing, legal compliance or occupied-building suitability without separate qualified evidence.

## 2. Existing owner map — compose, do not duplicate

| Owner | Canonical responsibility | Existing work consumed |
|---|---|---|
| Sol-Atlas | Highveld project context, source-preserving composition, IFC/IDS projection and project-level lifecycle | [#622](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/622), [#341](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/341), [#354](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/354), [PR #355](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/355), [PR #660](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/660) |
| Symthaea | Design intent, engineering reasoning, semantic change impact, process semantics and IFC/IDS information projection | [#10543](https://github.com/Luminous-Dynamics/symthaea/issues/10543), [#5686](https://github.com/Luminous-Dynamics/symthaea/issues/5686), [#6036](https://github.com/Luminous-Dynamics/symthaea/issues/6036), [#6040](https://github.com/Luminous-Dynamics/symthaea/issues/6040) |
| Symtropy | Physical simulation, panel-cell comparison, exact physical identity, material readiness, as-built/metrology and disassembly/repair evidence | [#3773](https://github.com/Luminous-Dynamics/symtropy/issues/3773), [#4043](https://github.com/Luminous-Dynamics/symtropy/issues/4043), [#4150](https://github.com/Luminous-Dynamics/symtropy/issues/4150), [#5302](https://github.com/Luminous-Dynamics/symtropy/issues/5302), [#5321](https://github.com/Luminous-Dynamics/symtropy/issues/5321) |
| Mycelix | Participant authority, source provenance, custody, manufacturing and cross-organization receipt semantics | [#5149](https://github.com/Luminous-Dynamics/mycelix/issues/5149), [PR #5708](https://github.com/Luminous-Dynamics/mycelix/pull/5708), [PR #4686](https://github.com/Luminous-Dynamics/mycelix/pull/4686) |

The table is an owner crosswalk, not a declaration that every referenced PR has merged or every adapter is production-ready. Each consumer must pin the exact source revision, schema/profile version and disposition it actually uses. If an owner artifact is unavailable, stale, unqualified or semantically unmapped, the consuming state remains Unknown or blocked.

## 3. Paradigm shift: qualify the joins, not just the components

Conventional lifecycle tooling is strong at managing models, product structures, documents, workflows, simulation and operations inside its respective platform. The critical risk at a multi-vendor boundary is that a join can look plausible while silently changing meaning.

This benchmark focuses on five join failures:

1. **Identity laundering:** same part family/name/geometry is incorrectly treated as the same revision, variant, lot or physical article.
2. **Semantic laundering:** IFC/IDS or a robot projection passes its own information contract while geometry, tolerance, coordinate frame or process semantics have been lost.
3. **Temporal laundering:** a once-valid certificate, calibration, approval or inspection is treated as current at a later decision frontier.
4. **Authority laundering:** identity, authentication, a planner result or a signed assertion is mistaken for permission, professional approval or physical execution authority.
5. **Evidence laundering:** a model result, supplier assertion or robot self-report is promoted to independent physical acceptance.

The desired result is not a single universal twin. It is a source-preserving federation in which each owner retains its model and authority, while composition checks whether the joined evidence is sufficient for the exact downstream question.

## 4. Benchmark artefacts

- `HIGHVELD_KEEP_PANEL_THREAD_CORPUS_V0_1.json`: 24 synthetic scenario specifications and expected terminal dispositions.
- `HIGHVELD_KEEP_PANEL_THREAD_CORPUS_SCHEMA_V0_1.json`: closed Draft 2020-12 schema for the scenario manifest.

The corpus is currently `synthetic_specification_unadjudicated`. The expected dispositions are proposed gold labels, not a validated oracle. Before candidate systems are scored, at least two independent assessors should adjudicate critical scenarios, record disagreements, resolve them under a documented protocol and freeze a versioned answer key. The evaluator must not see or accept candidate-supplied gold labels.

### Case families

| Family | Cases | Main question |
|---|---:|---|
| Complete reference trace | 001 | Can the system preserve a complete synthetic chain without promoting it to approval? |
| Revision / identity / projection | 002–004, 011 | Do source identity and semantics survive each transformation? |
| Material / process readiness | 005–008, 012, 015 | Is the exact lot, custody, kit, capability and approval scope current? |
| Robot / metrology / safety | 009–010, 013, 020 | Are exact tool, calibration, workspace and independent acceptance conditions met? |
| As-built / repair / temporal | 014, 019, 021–024 | Does the realized article match the released generation, and does repair trigger scoped requalification? |
| Governance / rights / dependence | 016–018 | Are authority, rights and evidence independence actually established? |

No case has an expected terminal state of generic `PASS`, `APPROVED`, `RELEASED` or `AUTHORIZED`. The most favorable outcome is `REVIEW_ONLY`.

## 5. Evaluation protocol

### Baseline arms

Run identical scenarios and context budgets through these distinct baselines:

1. **Document-and-spreadsheet workflow:** conventional human coordination across disconnected source systems, with actual reviewer time and omissions recorded.
2. **Native enterprise stack:** use the best-fit available vendor modules and supported connectors for the scenario. Candidate stacks and source baselines include: [Autodesk Forma](https://www.autodesk.com/products/forma/overview), [Autodesk Construction Cloud / Forma construction workflows](https://construction.autodesk.com/), [Autodesk Tandem](https://intandem.autodesk.com/explore-autodesk-tandem/); [Bentley iTwin Platform](https://developer.bentley.com/itwinplatform/); [Siemens Digital Twin Composer](https://news.siemens.com/en-us/digital-twin-composer-ces-2026/) and its broader [digital-twin portfolio](https://www.siemens.com/en-us/company/digital-twin/); [PTC Windchill](https://www.ptc.com/en/products/windchill); and [Dassault Systèmes 3DEXPERIENCE](https://www.3ds.com/products/3dexperience/). These are role-specific product families, not interchangeable products. Pin the actual release, licence, enabled modules, integration configuration, source/connector coverage and human intervention for every run. Digital Twin Composer was described by Siemens as early access with select customers at its CES 2026 announcement; do not treat announced features as generally available unless the test environment proves access. Do not compare a single vendor product against the entire Luminous stack.
3. **Open-standard integration:** IFC 4.3 / IDS, source-native formats and applicable manufacturing/traceability profiles, with a declared integration effort and information-loss report.
4. **Luminous federation:** the existing Sol-Atlas ITIP, Symthaea engineering/change-impact/process owners, Symtropy physical/manufacturing/robotics owners and Mycelix provenance/authority owners.

Use the same panel family, reference geometry, change events, physical evidence packet, acceptance criteria and resource budget. Report product capability claims as vendor claims; report only independently observed benchmark outcomes as measured outcomes. If a licensed vendor environment is unavailable, mark that comparison arm unrun rather than substituting marketing material for a test.

### Core correctness metrics

- **Critical-impact recall:** fraction of independently adjudicated critical changes that trigger every required block or requalification obligation. Target: 100% on the frozen synthetic critical corpus; any critical miss fails the gate.
- **Stale-evidence false-current rate:** count of stale, revoked, out-of-scope or superseded evidence records accepted as current. Target: 0.
- **Exact-lineage closure:** fraction of critical artifact edges bound to exact owner, subject, revision, payload digest and relationship. Target: 100% for P0 critical edges.
- **Projection-loss detection:** critical semantic losses, unsupported mappings and undeclared unit/frame transformations detected. Target: 100% of seeded critical losses.
- **Disposition correctness:** per-class confusion matrix for review, requalification, unknown, conflict and authority-blocked states. Target: at least 95% macro precision after independent adjudication, with zero critical false negatives.
- **Unknown/abstention quality:** unknown states surfaced when input evidence cannot support a conclusion; unsupported guesses and false equivalence counted separately. Target: 100% of seeded unsupported/ambiguous cases remain Unknown or blocked.
- **Authority escalation:** benchmark outputs that mutate approval, release, custody, inventory or robot authority. Target: 0.
- **Replay integrity:** deterministic results for identical pinned inputs and evaluator version; exact digests for all inputs and outputs. Target: 100% repeatability in the deterministic fixture profile.

### Physical-production metrics — separate from semantic correctness

For later manual / assisted / robotic cell trials, compare equal accepted function, material, geometry and quality gates. Report:

- finally accepted panels per total workflow labor-hour;
- total elapsed time, setup/programming, training, supervision and inspection labor;
- first-pass yield, final yield, defect distribution, rework, scrap and failed attempts;
- material, consumables and energy per finally accepted panel;
- metrology uncertainty and tolerance distributions;
- safety stops, near misses, exposure and human intervention;
- inspection, disassembly and replacement success rates and time;
- maintenance burden, downtime and availability.

Do not use robot cycle time as the productivity denominator. Do not let a faster but lower-quality workflow win by delivering fewer accepted panels. Do not collapse quality, safety, circularity and cost into one weighted score that can hide a critical failure.

## 6. Proposed acceptance targets — hypotheses, not achieved results

| Gate | Prospective target |
|---|---:|
| Critical seeded failure detection | 100% |
| Stale/revoked evidence falsely treated as current | 0 |
| Critical lineage edges bound to exact source subjects | 100% |
| Critical transformation losses detected | 100% |
| Critical unknowns silently guessed | 0 |
| Benchmark-generated authority/actuation/release mutations | 0 |
| Deterministic replay on frozen fixtures | 100% |
| Macro precision across adjudicated dispositions | ≥95% |
| Avoidable requalification relative to full-requalification baseline | ≥30% reduction, only if critical recall and stale-evidence gates remain unchanged |

The 30% target is a research hypothesis, not a measured saving. It must be revised or withdrawn if matched trials show the full-requalification baseline is already cheaper, if gold-label uncertainty is too high, or if any critical impact is missed.

## 7. Standards and interoperability profile

- [ISO 23247-5:2026](https://www.iso.org/standard/87425.html) is published and covers the manufacturing digital thread across product lifecycle stages.
- [ISO 23247-6:2026](https://www.iso.org/standard/87426.html) is published and specifies integrated, unified and federated digital-twin composition.
- [ISO/AWI 23247-8](https://www.iso.org/standard/94858.html) is an approved work item under development as of 23 October 2026; its proposed VVUQ scope should inform our research, but it is not a published requirement to claim conformity against.
- [NIST IR 8536](https://csrc.nist.gov/pubs/ir/8536/final), finalized 9 September 2026, gives an industry-neutral manufacturing supply-chain traceability meta-framework built around linked, verifiable provenance and selective disclosure.
- [IFC 4.3](https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/) and [IDS](https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/) should remain exchange and information-requirement inputs, not substitutes for geometry verification, physical metrology, engineering analysis or legal approval.

This profile makes no ISO/IEC conformity, NIST certification, building-code compliance or professional-approval claim.

## 8. Implementation sequence

1. **Freeze the composition corpus** (this PR): scenario specifications, owner map, schema and proposed reason codes.
2. **Independent gold adjudication:** two-reviewer critical labels, disagreement log, frozen answer key and evaluator mutation controls.
3. **Owner adapter confirmation:** pin exact schema/profile versions and fields from ITIP, Symthaea, Symtropy and Mycelix owners; mark unconfirmed paths `UNBOUND` rather than inventing API fields.
4. **Standalone Rust qualifier:** consume only owner-qualified references/projections; produce a deterministic, non-authorizing report. Keep it outside the Sol-Atlas browser/runtime path.
5. **Cross-repo fixture replay:** compose the already-owned ITIP, provenance, material-readiness, design-change, built-asset and disassembly test fixtures; do not copy their semantics or source payloads.
6. **Benign physical pilot:** only after the exact lab-cell work package, tools, safety envelope, professional reviews and measurement plan are independently accepted. Start with the non-load-bearing panel in an unoccupied controlled cell.
7. **Comparative trials:** manual, assisted and robotic workflows; full-workflow labor and accepted-output denominators; publish negative results as well as positive ones.

No CI runner is required to author, review or statically inspect the corpus. Exact-head software validation, human adjudication, real supplier/material evidence and physical trials remain separate gates.

## 9. Explicit non-goals

This tranche does not:
- define another canonical asset graph, provenance ledger, requirements graph or approval authority;
- implement IFC, IDS, QIF, MTConnect, OPC UA, a PLM platform, BIM authoring, CAD, solver or robot controller;
- assert that a digital representation is a physical observation;
- infer real Highveld parcel/site conditions or local supplier availability;
- authorize fabrication, installation, occupancy, robot actuation or use in a load-bearing/fire/life-safety application;
- claim a Luminous SOTA advantage or any measured production improvement.

**Success condition:** the same exact physical article can be followed from intent through source-owned engineering, manufacturing, custody, inspection, installation and repair without losing identity, semantics, time, rights, evidence status or authority—and every deliberately broken join is detected before it can be used to justify a downstream action.
