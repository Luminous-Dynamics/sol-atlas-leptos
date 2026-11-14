# Home assembly demand and substitution contract v0.1

**Study ID:** `HOME-ASSEMBLY-DEMAND-001`  
**Parent:** [issue #803](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/803)  
**Composes with:** [HOME-FREEDOM-COMPILER-001 #800](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/800), Highveld Keep #607, industry benchmark #617 and fabrication bridge #622  
**Maturity:** `SYNTHETIC_REQUIREMENTS_ONLY`  
**Execution:** `NOT_GEOMETRY_BOUND`

## 1. The real problem is not selecting a material; it is maintaining equivalence under change

A design can look complete while its buildability is fictional. A nominal bill of materials often omits approval scope, product revision, batch identity, interface tolerances, local supply, installation capability, repair access and the effect of substitution on previously checked claims.

The proposed interface starts *before* product selection. It records what the home design requires from each assembly and which owner must establish the relevant evidence. It then gives the downstream South African compliance, product-coverage, material-flow, delivery and fabrication owners an exact demand to satisfy.

This is deliberately not another supplier database or BIM schema.

## 2. Paradigm shift: from BOM to evidence-linked assembly demand

```text
resident brief + selected candidate + exact design revision
                         |
                         v
             required assembly functions
                         |
                         v
     property/approval/interface requirements + owner
                         |
                         v
         geometry-bound quantity and tolerance basis
                         |
                         v
      source-qualified local product / component options
                         |
                         v
       approval, compatibility, buildability and repair
                         |
                         v
      re-evaluate all affected claims after substitution
                         |
                         v
       exact procurement/fabrication/install/as-built
```

A product substitution is not an alias change. It is a controlled change to a dependency graph. If a roof membrane changes, the affected weather, interface, fire, durability, warranty, maintenance and embodied-impact claims may need requalification. An unrelated room adjacency does not. The system must neither under-invalidate nor blindly invalidate every result.

The central new benchmark is **substitution blast radius**:
- all affected claims are invalidated;
- unaffected claims are preserved with a reason;
- new evidence is bound to the exact replacement product/revision/configuration and project scope;
- no claim is re-promoted until the responsible evidence owner requalifies it.

## 3. Owner boundaries

| Domain | Owner | This interface does |
|---|---|---|
| Resident brief and home alternatives | Sol-Atlas #800 / PR #801 | binds the exact candidate and design revision |
| South African building/product approvals | Sol-Atlas #604 | references applicable authority and evidence profile |
| Product families and product chains | Sol-Atlas #602/#603 | emits requirements; does not own products |
| Material recovery and reuse | Sol-Atlas #569 | records demand for circularity/reuse evidence |
| Construction phases and delivery | Sol-Atlas #573 | references delivery package and site sequencing |
| Reference dwelling | Sol-Atlas #607 | consumes actual Highveld Keep architecture when admitted |
| Industry comparison | Sol-Atlas #617 | supplies matched benchmark protocol |
| Design-to-panel/asset bridge | Sol-Atlas #622 | receives exact panel demand after geometry exists |
| Fabrication capability and output | Symtropy #3773/#1818/#1834/#3605 | retains work-package, robot and accepted-output authority |
| As-built survey/repair loop | Symtropy #4043 | owns measured realized geometry and repair/reverification |
| IFC/IDS projection | Symthaea #6036; Sol-Atlas #704 | owns crosswalks and information-loss reports |
| Construction provenance/approval receipts | Mycelix #5149 | owns native provenance and approval receipts |

This work owns only the resident-design-to-assembly-demand contract and its deterministic qualifier.

## 4. Data semantics

### Assembly requirements

Each assembly demand binds:
- brief ID, candidate ID, design revision and candidate-set reference;
- building layer and functional assembly role;
- component roles and required interfaces;
- required property / approval categories and exact evidence owner;
- claim dependency IDs that must be invalidated after a change;
- inspection, repair, replacement and disassembly requirements.

The schema records a requirement category, not an invented numeric limit. The responsible standards, professional or authority owner supplies the applicable threshold for the exact jurisdiction and building use.

### Geometry and quantities

Until a geometry owner provides exact realized geometry and a takeoff method:
- quantity state = `NOT_COMPUTED`;
- measured quantity = null;
- unit = null;
- geometry reference = null;
- takeoff method = null.

No quantity is inferred from a room's floor area, a typical house or an archetype name. After a geometry-bound takeoff, the record must carry the exact geometry revision, coordinate/measurement basis, method, uncertainty and unit.

### Product, approval and supply facts

The P0 fixture intentionally selects no product:
- product selection = `UNSELECTED`;
- product references = empty;
- approval = `NOT_CHECKED`;
- local availability = `UNKNOWN`;
- cost = `NOT_ESTIMATED`, amount null.

A product-family entry, supplier catalogue record, certificate, EPD or manufacturer's declaration does not imply project applicability, current approval, structural performance, fire performance, installation acceptance or supply availability. Every claim needs its exact source, scope, revision, date, rights and owner.

### Manufacturing and repair

A product being listed in IFC, IDS or a catalogue does not establish:
- a local machine/fixture/tool/worker capability;
- safe handling or installation;
- tolerance fit;
- inspection access;
- repairability or disassembly;
- accepted output quality.

Those remain UNKNOWN until their respective owners provide current evidence.

## 5. Initial assembly slices

The paired synthetic case registers four assembly families for each home candidate:

1. Opaque wall / envelope assembly.
2. Roof, ceiling and drainage assembly.
3. Openings and external shading assembly.
4. Wet-service / sanitation core.

A fifth family—the non-load-bearing envelope/service panel—will be admitted when #622 has exact shape and a qualified interface package. This v0.1 fixture contains no selected materials or physical quantities.

## 6. P0 substitution corpus and metrics

Freeze at least 40 independently authored cases before comparative evaluation:

| Test family | Required challenge |
|---|---|
| Identity | same product label, different SKU/revision/batch/configuration |
| Interface | nominal dimension matches but tolerance/connection does not |
| Approval | expired, wrong site/use, wrong system configuration or missing certificate |
| Performance | change invalidates thermal, moisture, fire, acoustic or structural evidence |
| Geometry | source IFC revision changes or takeoff basis is stale |
| Supply | local supplier unavailable, lead-time unknown or regional coverage absent |
| Cost | unknown rate, wrong unit, stale quote, currency/tax/scope mismatch |
| Repair | repair/disassembly claim lacks a procedure or accessible interface |
| Fabrication | required process/tool/fixture/capability missing |
| Positive control | exact qualified equivalent preserves the same function and interfaces |

Proposed hard targets:
- 0 false-equivalence substitution PASS.
- 0 missed dependent-claim invalidations.
- 0 unsupported unrelated-claim invalidations in the exact known-answer corpus.
- 100% source/product references exact or unresolved.
- 100% quantity rows geometry/revision/method/unit-bound or `NOT_COMPUTED`.
- 0 family approval promoted to project permission.
- 0 cost or local availability claim without dated, scoped source evidence.
- 100% positive-control preservation.

Report separate metrics:
- affected-claim recall and precision;
- blast-radius over- and under-invalidation;
- engineering/review effort per substitution;
- product evidence coverage by property and jurisdiction;
- quantity/takeoff error after geometry changes;
- local availability/lead-time coverage;
- cost estimate interval and quote age;
- full-workflow labour/setup/rework/waste per accepted output;
- repair/disassembly/replacement time;
- design intent preserved after a necessary substitution.

A scalar material score is not appropriate: a cheaper or more available item cannot compensate for a failed safety or approval gate.

## 7. Industry comparison and what could be genuinely better

**IFC 4.3.2.0** (published as ISO 16739-1:2024) is the current official IFC4.3 release. **IDS 1.0** is buildingSMART's official computer-interpretable information-delivery specification. Use them for exchange requirements, identities and property-presence validation; neither is a physical test or project approval.

The European Commission describes Digital Product Passports as data containers for product/component/material identity, safety, origin, repairability and environmental information. Its roadmap is staged and product-group specific; it is not a South African building permit or a blanket requirement for this programme.

A January 2026 systematic review of industrialised housing argues that mass-customised kits-of-parts and P-DfMA can improve predictability, time, cost variability, waste and quality, while noting persistent barriers in regulation, procurement, supply-chain capacity and component/workflow standardisation. A 2026 social-housing circular-economy review identifies industrialised construction, design for disassembly, adaptive reuse, product-service systems and digital passports as connected themes.

Our hypothesis is narrower and testable: **a resident-facing design compiler can improve the reliability of the bridge from bespoke functional intent to repeatable, locally qualified assemblies, particularly when products are substituted or supply chains change.** This is not yet demonstrated.

References:
- buildingSMART IFC: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- buildingSMART IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- EU Digital Product Passport: https://single-market-economy.ec.europa.eu/single-market/digital-product-passport_en
- Hedayati et al. (2026), *Industrialised Housing Delivery*, Buildings 16(3), 552: https://doi.org/10.3390/buildings16030552
- *Circular economy strategies for the design of a social housing system* (2026): https://doi.org/10.3389/frsus.2026.1908729

## 8. Claim ceiling

The committed example is a synthetic requirements packet only. It does not identify a real product, manufacturer, certificate, supplier, quantity, local price, material property, fire/structural/thermal rating, manufacturing process, robot task or approved assembly. A complete IFC/IDS exchange does not prove physical equivalence. The only valid P0 outcome is that unknowns are preserved and the demand interface is structurally coherent.
