# Evidence-Backed Construction Method Screening v0.1

Status: implementation proposal; hosted Rust validation is pending. All current fixtures are synthetic.

## Paradigm shift: optimize the life of the assembly, not just the build event

A construction method is not a label such as “timber”, “modular”, “concrete”, “3D printed” or “robotic”. It is a scoped design and delivery system: components, connections, tolerances, supply chain, logistics, inspection, installation sequence, interfaces, service access, maintenance, repair, adaptation and eventual recovery.

The first evaluator increment therefore uses three distinct questions:

1. **Can this option proceed through the required gates?** Regulatory approval path, structural verification, fire/life safety, moisture durability, ground/foundation interface, worker safety, quality assurance, service access, repair capacity and recovery planning are non-compensatory when configured as required hard gates.
2. **Is there comparable evidence for its performance?** Metrics must be scoped to the same project, exact design revision, functional basis and study period. A result for a wall is not comparable to a result for an entire building; a 30-year scenario is not silently compared with a 60-year scenario.
3. **What trade-offs remain robust under uncertainty?** Cost, carbon, duration, labour, maintenance/repair, adaptability and recoverable-material intervals form a multi-objective comparison. No single weighted score can make a safety or statutory failure disappear.

The implementation deliberately does not select a construction family by default. Off-site volumetric modules, panelized kits, precast systems, light-gauge steel, timber, earth-based systems, additive manufacturing and conventional on-site methods are candidate families, not universal winners. Their suitability is context-, design-, supply-chain-, climate-, code- and workforce-dependent.

## Lifecycle accounting and circularity

Use a shared, declared functional basis (for example, a square metre of usable floor area delivering a specified service for 60 years), with the service and study period fixed across candidates.

Keep different carbon questions distinct:
- **Upfront carbon A0-A5:** materials, product manufacture, transport and construction-stage processes within the chosen assessment boundary.
- **Embodied carbon A-C:** the declared product/construction/use-maintenance/end-of-life embodied-carbon boundary, with its module coverage and exclusions recorded.
- **Operational carbon B6-B7:** report energy- and water-related operational assumptions separately when they differ between methods; they should not be inferred from a structural method label.
- **Whole-life carbon:** only call the result this when the relevant lifecycle modules and assumptions have actually been assembled and documented.
- **Module D:** report potential benefits/loads beyond the asset system boundary separately. Do not subtract uncertain future reuse/recycling credits from the primary A-C result to manufacture a low-carbon winner.

The RICS Whole Life Carbon Assessment standard provides a lifecycle accounting framework for constructed assets and explicitly covers production, construction, operation, maintenance, repair, replacement, end-of-life and benefits beyond the asset life. Its current hub and reporting guidance are starting points, not substitutes for project-specific quantities, EPD quality and assessment scope:
- https://www.rics.org/profession-standards/rics-standards-and-guidance/sector-standards/construction-standards/whole-life-carbon-assessment
- https://www.rics.org/profession-standards/rics-standards-and-guidance/sector-standards/construction-standards/whole-life-carbon-assessment/whole-life-carbon-assessment-implementation-guides-and-supporting-documents

## Design for disassembly and adaptability

ISO 20887:2020 defines principles and guidance for design for disassembly and adaptability across buildings and civil-engineering works; ISO confirms the edition remained current after review in 2025:
- https://www.iso.org/standard/69370.html

A 2024 review of disassembly-assessment methods found fragmented criteria and methods, and highlights the importance of connection-type assessment:
- https://doi.org/10.1016/j.autcon.2024.105521

Consequences for future versions:
- Record connection type, access to fixings, removal sequence, destructive versus non-destructive disassembly, component identification, separation of incompatible materials, recovery yield and realistic routes to reuse.
- Distinguish “technically detachable” from “economically/logistically reusable at end of life”.
- Evaluate adaptability as a serviceable design capability, not only a circularity percentage.
- Avoid treating recoverable mass as guaranteed recovered material or an automatic carbon credit.

## Construction robotics and human factors

A 2026 systematic review of 375 construction-robotics studies organizes the field around robot types, task applications and human-robot collaboration, while identifying deployment barriers:
- https://doi.org/10.1016/j.rineng.2026.110406

NIOSH's 7 October 2026 construction robotics bulletin notes that robotics can reduce worker exposure to hazardous tasks, but human-robot interaction can introduce new hazards:
- https://www.cdc.gov/niosh/bulletin/2026/safety-in-construction-robotics.html

Consequently, automation must be assessed by task-level safety exposure, setup/relocation, reliability, maintainability, energy, specialist dependence and the consequences of a failed or unavailable robot. “More automated” is not itself an objective. Early practical targets should favor simple, modular, repairable tools and fixtures that reduce dangerous or fatiguing work, rather than assume a fully autonomous construction site.

## Interoperability

The method contract can map to project/product data exchanged with IFC 4.3 (ISO 16739-1:2024) and related construction classifications, while preserving the evaluator's explicit scope and evidence receipts:
- https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/lexical/IfcClassification.htm

Protocol/schema conformance does not prove that a source is accurate, current, authoritative or appropriate for a design decision.

## Rust contract

`sol-atlas-core::construction_methods` is a pure data/policy layer with no renderer or network dependency. It:
- binds candidates to project scope, design revision, functional basis and reference study period;
- validates required evidence receipt structure, digest syntax, units and freshness;
- confirms a hard-gate pass only from current `Verified` evidence;
- excludes an option on any current verified hard-gate failure, even when other gates or performance data are missing;
- holds missing, stale, estimated or disputed hard-gate evidence at `NeedsEvidence`;
- keeps unaccepted/stale performance metrics out of the eligible comparison;
- uses robust interval dominance rather than silently aggregating objectives; and
- marks the reported frontier conditional when any alternative remains unresolved.

A hash-shaped string is only syntax validation in this module. It does not hash the underlying artifact bytes or authenticate the issuer; that belongs in the source-ingestion and evidence-verification boundary.

## Benchmark targets

Proposed promotion targets:
- 30 pinned gate/evidence cases and at least 12 additional adversarial Rust tests.
- Zero false clears: estimated, unknown, disputed, stale or missing hard-gate evidence must never be interpreted as pass.
- Zero hard-gate resurrection: attractive carbon/cost/time/reuse metrics cannot offset a verified hard-gate failure.
- 100% agreement with an independent reference evaluator on the fixed gate corpus and hand-computed interval-dominance cases.
- Input-order invariance and byte-identical serialized reports for identical input/toolchain.
- At least 100 method portfolios with overlapping and non-overlapping intervals, missing evidence, changed study period and method-specific hard gates.
- Scale measurements for 10,000 method options × configured metric objectives, including runtime and peak memory; set latency targets only after measuring.
- Real-world claims require independently verified bills of quantities, project-specific EPDs, consistent service performance, relevant approvals, ground/climate context, construction logistics and competent professional review.

## Scope / claim ceiling

This is a *screening and trade-off evaluator*. It does not design a building, calculate member sizes, prove fire/moisture performance, check a jurisdiction's building code, authenticate evidence, generate a bill of quantities, or certify safety, price, carbon or circularity. Synthetic benchmark success is not permission to build. Mandatory statutory, structural, fire, health, land-rights, environmental and worker-safety requirements remain outside weighted trade-offs.

## Follow-on assembly research

Issue https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/149 tracks a physical/testable design direction: reversible and serviceable connections, low part/tool complexity, accessible service routes, repair-first layers, measured disassembly yield and task-level worker exposure. That work should prototype and test assemblies before proposing a universal construction system.
