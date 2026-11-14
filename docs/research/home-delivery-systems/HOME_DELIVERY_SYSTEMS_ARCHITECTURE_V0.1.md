# Home Delivery Systems Architecture v0.1

**Status:** research and integration architecture; not an implemented design compiler, engineering authority, product specification or construction release.
**Programme:** Home Freedom / Highveld household service and delivery.
**Purpose:** connect resident intent, site evidence, spatial design, foundations, assemblies, manufacturing, utilities, appliances, installation and lifecycle outcomes without duplicating domain owners.

## 1. Thesis

A home is not a floor plan. It is a resident-directed system of spaces, structures, assemblies, services, equipment, supply dependencies and lifecycle capabilities.

The compiler's purpose is to preserve resident intent while turning it into increasingly precise, reviewable and locally deliverable design artifacts. Each transformation must declare:
- the exact input revisions and source identities it consumed;
- the requirements it preserved, transformed or could not represent;
- its assumptions, units, uncertainty, exclusions and responsible owner;
- its output digest and evidence maturity;
- the downstream evidence that becomes stale after a relevant revision.

The product should optimize for **qualified resident outcomes over the whole home lifecycle**, not render count, gross floor area, lowest initial shell price or a single opaque sustainability score.

## 2. End-to-end architecture

`resident intent
  -> spatial / service programme
  -> site and context evidence
  -> candidate geometry
  -> structural and specialist design
  -> assembly demand and material qualification
  -> DfMA / fabrication release
  -> logistics and installation
  -> inspection and as-built reconciliation
  -> household service operation
  -> repair, adaptation, reuse and end-of-life`

These are not one linear certainty ladder. The compiler should retain parallel alternatives, unresolved requirements and explicit professional hold points. A downstream stage may only consume the evidence maturity it requires.

## 3. Domain ownership and current public work

| Layer | Current owner / work | Boundary |
|---|---|---|
| Resident brief and candidate design | [Home Freedom Compiler #800](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/800) | Owns resident intent and alternatives, not professional design authority |
| Spatial intent | [Spatial intent graph PR #827](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/827); [resident spatial programme #833](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/833) | Activity, relationship, privacy and resident-stated functional constraints |
| Geometric qualification | [Geometric layout PR #831](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/831) | Current scope is synthetic orthogonal 2D geometry, integer millimetres, openings as connectivity portals; not architectural or code approval |
| Whole-home revision impact | [Revision impact #832](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/832) | Typed change events and evidence-specific dependency invalidation |
| Site / ground / foundations | [Site-to-foundation qualification #836](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/836); [site-investigation EVSI #134](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/134) | Evidence intake, sampling decision support, revision binding and hold-point records; no geotechnical or structural calculation authority |
| Assembly demand | [Home assembly demand #803](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/803) | Declares what assemblies a design requires; does not invent quantities, products or approvals |
| Product/material qualification | [Home material intelligence #811](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/811); [PR #812](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/812) | Exact product/version/certificate/quote evidence and dependent-claim invalidation |
| DfMA / manufacturing / installation | [Home DfMA delivery #814](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/814) | Compares site-built, panelized, modular and hybrid modes on whole-workflow evidence |
| Construction and productive systems | [CityMaker #549](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/549); [construction delivery #573](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/573) | Composes production geography, logistics, human/robot work and accepted assets; not a robot dispatcher |
| Household service integration | [Highveld household service #694](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/694); [appliance integration #837](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/837) | Binds resident-selected appliance services to the home and its utility interfaces |
| Appliance product engineering | Symthaea [HOME-SVC #10322](https://github.com/Luminous-Dynamics/symthaea/issues/10322), [WaterLoop #10323](https://github.com/Luminous-Dynamics/symthaea/issues/10323), [HeatBank #10329](https://github.com/Luminous-Dynamics/symthaea/issues/10329), [AirCare #10330](https://github.com/Luminous-Dynamics/symthaea/issues/10330), [metrology #10341](https://github.com/Luminous-Dynamics/symthaea/issues/10341), [utility coordination #10342](https://github.com/Luminous-Dynamics/symthaea/issues/10342), [repairability #10358](https://github.com/Luminous-Dynamics/symthaea/issues/10358) | Product-specific physics, endpoints, controls, safety and qualification remain with their domain owners |
| Provenance / approvals / lifecycle | Mycelix construction provenance [#5149](https://github.com/Luminous-Dynamics/mycelix/issues/5149), plus owner-native evidence records | Authentication, custody, approvals and durable lifecycle records; no self-authorized engineering claims |

## 4. Three key integration contracts

### A. Site-to-foundation evidence

`site context -> investigation design -> field/lab observations -> professional interpretation -> exact foundation design -> construction hold points -> as-built evidence`

The system can preserve boundaries, source identity, coverage, spatial reference systems, observation times, methods, reports, professional identity and revision dependencies. It must not calculate or infer bearing capacity, settlement, founding depth, ground class, foundation dimensions or structural adequacy.

Site screening, public geology layers and remote sensing are not geotechnical investigation. An investigation plan must be tied to the proposed structure and the decision it can change. Where probability assumptions are defensible, decision-value metrics may be reported; otherwise use robust/worst-case analysis rather than fabricating probabilities from intervals.

### B. Geometry-to-manufacturing

`candidate geometry -> exact takeoff input -> assembly demand -> product/material evidence -> manufacturing capability -> fabrication release -> installation -> accepted as-built`

Quantities remain `NOT_COMPUTED` until an exact geometry owner, measurement method, units, revision and tolerances provide them. A BIM property pass is not a manufacturing pass. A supplier listing is not local availability. A panel model is not a qualified lift plan. Every manufactured item must retain design, assembly, component and batch/lot genealogy.

Delivery-mode comparisons must include setup, factory overhead, yield, scrap, packaging, storage, transport, permits, lifting, site labor, weather downtime, QA, rework, commissioning and lifecycle repair. The no-build, repair, retrofit and partial-retention options remain in the choice set.

### C. Resident-selected services and appliances

`resident capability need -> service contract -> product option -> space / electrical / water / thermal / air / acoustic interfaces -> maintenance and repair -> measured service outcome`

Appliance choice changes the building. For example, a water hub influences separate pipework, tank location, access, drainage and maintenance; a heat pump changes electrical peak, heat rejection and acoustic envelope; ventilation affects openings, ducts, filter access, pressure and noise; warewashing affects hot/cold water, drainage, power, consumables and kitchen workflow.

Prefer independent service modules with documented, replaceable interfaces over one monolithic controller or bespoke appliance for every resident. Integrate shared resource/heat loops only after measuring both benefit and common-mode, hygiene, isolation and repair risks. Keep product safety interlocks and local manual controls inside the product owner.

## 5. Evidence and revision model

Every derived claim must bind to:
- exact home candidate and immutable revision;
- source artifact digest(s), product/assembly/configuration revision(s);
- method/rule/standard edition and jurisdiction;
- spatial frame, units, temporal basis and measurement boundary where applicable;
- uncertainty, coverage, scope and exclusions;
- owner, issuer, review state and claim ceiling;
- dependency edges and supersession/revocation history.

A change invalidates its transitive dependent evidence, not every record in the project. Examples:
- move a load-bearing wall: requalify affected geometry, structure, foundation loads, assemblies, service routes, quantities, cost and fabrication; preserve unrelated appliance performance results;
- change appliance model: requalify spatial/electrical/water/thermal/acoustic/maintenance/cost interfaces; do not invalidate unrelated site boreholes;
- change paint preference: invalidate only claims whose dependencies actually include finish selection;
- revise site drainage or levels: invalidate affected site, foundation, moisture and external-works evidence, not unrelated appliance cleaning tests.

Unknown, stale, disputed, revoked and not-computed states must remain distinct from pass/fail. A queued CI run is not a pass. A schema-shaped receipt is not source authentication. A synthetic geometry pass is not a physical inspection.

## 6. Open standards and exchange

- Use **IFC 4.3.2.0 / ISO 16739-1:2024** as the current official open built-asset exchange basis where appropriate.
- Use **IDS 1.0** for machine-readable information delivery requirements and object/property checks.
- Do not treat IDS as a geometric clash, clearance, structural, fire, moisture or performance verifier. Keep those checks with appropriate geometry/physics/specialist owners.
- Investigate **ISO 23387:2025** data templates for consistent lifecycle information about construction objects.
- Preserve source IDs and report semantic loss on every adapter. Do not invent a proprietary replacement for IFC, IDS, product data templates or jurisdictional approval systems.

References:
- https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- https://www.iso.org/standard/85391.html

## 7. Benchmark programme

Hard gates are never averaged away by cost, speed or aesthetic quality.

| Track | Primary measurements | Initial hard gate |
|---|---|---|
| Intent preservation | Requirement traceability, unrepresented statements, resident edits and preference preservation | 100% of non-negotiables preserved; zero silent inference or preference-to-hard-constraint promotion |
| Geometry | Exact area/identity, overlap, containment, opening and route parity | Zero false pass on frozen adversarial controls; 100% exact known-answer agreement |
| Site / foundation evidence | Source coverage, investigation decision value, stale evidence, hold points, professional packet completeness | Zero unsupported design or construction promotion; 100% exact site/design/report linkage |
| Assembly and substitution | False equivalence, missed/extra invalidation, quantity error and product-scope coverage | Zero unsupported substitutions; no quantity PASS without exact geometry/revision/method |
| Manufacturing and delivery | Total installed cost, lead time, first-time-right, factory/site labor, tolerance, waste, rework, damage and repair | Zero unqualified releases; 100% traceability and required inspection evidence |
| Appliance services | Accepted service per kWh/L/functional unit, unmet demand, noise, reliability, repair time, household and unit variance | Zero unsafe/unsupported service claims; all utility and endpoint measurements boundary-bound |
| Lifecycle outcomes | Adaptation cost/time, service continuity, maintenance, reuse, deconstruction and actual post-occupancy performance | No extrapolation from synthetic or single-prototype data to population-level benefit |

Matched experiments should give every comparator the same resident brief, site envelope, utility availability, jurisdiction edition, material choices, operating/weather profile, output functional units and cost boundaries. Report full distributions and exclusions, and retain independent blind confirmation sets.

## 8. Competitive position

Do not benchmark against a floor-plan renderer using an image metric and then claim superiority in construction. Compare equivalent outcomes and evidence.

- **Generative design/BIM products:** benchmark time to first professionally reviewable candidate, requirement preservation, design-change rework, interoperability loss and independent expert defect detection.
- **Conventional architecture and engineering:** compare the same brief/site/authority scope, with professional time, client revision burden, unresolved assumptions and independently adjudicated defects.
- **Offsite and modular construction:** compare all-in delivered work, including factory setup/overhead, logistics/lift, installation, tolerance correction and warranty/rework. Offsite production may shift cost or labor rather than remove it.
- **Appliance manufacturers:** compare accepted service at matched inputs and outcome quality, across multiple representative units and household duty cycles—not nominal capacity alone.
- **Retrofit/repair alternatives:** retain repair-first and staged upgrade paths; new construction is not automatically the best result.

The potential differentiator is **continuous evidence continuity across the whole home lifecycle**: a resident can see why an option fits, what remains unknown, what a change invalidates, what can be made locally, how it is inspected, and how it can be repaired later. This is a research and product thesis, not yet a demonstrated market advantage.

## 9. Staged execution

1. Keep PR #831's exact-head validation separate from new tracks. Current exact head at the time this architecture was created: `9ab240c639a81b1d0b8afe393cab846b61b79961`; hosted Check run was queued, with native tests, WASM and format jobs queued. No Rust or CI pass is claimed here.
2. Implement #833 resident dimensional/service programme and #832 typed revision invalidation.
3. Add machine-readable evidence intake and independent synthetic qualifier for #836 site/foundation records.
4. Add #837 HomeServiceInterface schema and four-product synthetic composition using the owner-native Symthaea contracts.
5. Bridge #803 assembly demand and #814 delivery with #622/#3773 accepted panel/fabrication evidence.
6. Extend geometry only after the current contract is stable: wall thickness, doors/windows, irregular polygons, structural grids, multiple floors and service voids should be separate maturity levels with independent expected results.
7. Export/import through IFC/IDS with explicit semantic-loss and revision receipts.
8. Move from synthetic tests to blinded historic datasets, independent expert adjudication, calibrated bench measurement, production-representative units and finally permissioned site/household pilots.

## 10. Claim ceiling

This architecture and its synthetic tests do not prove site suitability, soil conditions, foundation adequacy, building-code compliance, fire or accessibility compliance, structural safety, product safety, energy savings, water quality, manufacturability, local supplier capacity, affordability, construction savings or resident satisfaction.

Professional design, controlled standards, qualified inspections, product-specific safety tests, permits and measured as-built/operational evidence remain separate obligations.
