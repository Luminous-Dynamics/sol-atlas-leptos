# Home Freedom Compiler v0.1

**Study ID:** `HOME-FREEDOM-COMPILER-001`  
**Parent:** [issue #800](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/800)  
**Maturity:** `RESEARCH_CONTRACT_NOT_EVALUATED`  
**Execution:** `NOT_EXECUTED`  
**Climate reference:** Johannesburg / Highveld is the first regional profile, not a universal default.

## 1. Thesis: make the home user-authored, not architecturally generic

A person's dream home is not a particular architectural style. It is a reliable setting for the life they want to live: relationships, solitude, work, cooking, sleep, care, learning, hobbies, guests, outdoor life, privacy, mobility, cultural practices and future change.

The design problem is therefore not "generate a pretty floor plan." It is:

> Given a resident-authored functional brief, a real site, binding authority constraints, a budget and uncertainty, return several safe and feasible ways to realize the intended life—then show what each option costs to build, operate, maintain and adapt.

The resident authors the requirements and chooses among the trade-offs. The tool can explain, search, calculate and reject unsupported claims. It cannot decide what a household ought to value, authorize construction, certify engineering or manufacture missing evidence.

This is a mass-customization problem: standardize interfaces, safety-critical details, repeatable components and procurement where that reduces cost; personalize the spatial arrangement, room relationships, privacy, materials, finish, outdoor use and expansion path where residents value choice.

## 2. The paradigm shift: a home compiler

The pipeline is an evidence-preserving compilation process:

```text
Resident story and preferences
        |
        v
Typed functional brief (hard constraints != preferences)
        |
        v
Site / rights / climate / municipal authority envelope
        |
        v
Parametric design grammar + open building interfaces
        |
        v
Candidate plans and 3D/BIM representations
        |
        v
Specialist checks: structure, fire, thermal, daylight, air, water, access
        |
        v
Pareto-feasible alternatives + quantities + uncertainty-bounded costs
        |
        v
Qualified professional review and resident decision
        |
        v
Phased construction, inspection and as-built evidence
        |
        v
Operation, maintenance and safe adaptation
```

Every stage emits a revision-bound artifact. If a wall moves, a material changes, a room is added or a system is substituted, only the dependent claims are invalidated and re-evaluated—but none are silently inherited.

### 2.1 A household brief is an executable contract

The brief schema separates:

- **Activities:** what people need to do, including work, rest, care, cooking, study, hobbies, storage and outdoor activities.
- **Spatial relationships:** adjacency, separation, sightlines, privacy, noise, circulation, daylight and service access.
- **Hard requirements:** requirements that no generated candidate may violate.
- **Preferences:** choices the household may trade against cost, space, materials or other preferences.
- **Accessibility intent:** functional needs chosen by residents, expressed as design requirements rather than inferred diagnoses.
- **Adaptation intent:** household changes and future additions the design should accommodate.
- **Budget envelope:** initial capital, whole-life boundary, stage funding and uncertainty. Unknown rates remain unknown.
- **Consent and privacy:** a local-only default; explicit purpose-scoped permission before any data leaves the household's control.

No names, personal identifiers, precise home addresses, photographs of occupants, biometrics, audio or behavioural surveillance are required to produce a design brief. Site-specific data can be referenced by a controlled artifact ID, with actual access governed separately.

### 2.2 The site is a first-class input

A design is not feasible merely because a floor plan fits inside a rectangle. The site envelope includes:

- survey revision, boundaries, title/servitudes and land-use rights;
- topography, soil/geotechnical evidence, drainage and flood exposure;
- road, fire/rescue, pedestrian and service access;
- utility capacity and connection conditions;
- sun path, shading, local wind, temperature, humidity and future climate;
- vegetation, existing structures, heritage constraints and neighbours;
- applicable municipality, occupancy/building classification, code edition, amendments and submission requirements.

An unverified survey, land right, soil state, code route or connection capacity remains `UNKNOWN`. The compiler may present a conditional concept, but it must not label that concept buildable or approved.

### 2.3 Open building: protect the long-life layers

Use stable, reviewable interfaces between long-life and short-life elements:

| Layer | Typical contents | Design responsibility |
|---|---|---|
| Site and rights | boundaries, access, servitudes, drainage | site/authority evidence |
| Primary structure | foundations, load paths, lateral stability | qualified structural design |
| Weather and fire envelope | roof, external walls, fire separation, openings | qualified envelope and fire design |
| Building services | water, sanitation, electrical, ventilation, data | inspectable, serviceable routes |
| Space plan | room arrangement, partitions, storage | resident-directed within safety envelope |
| Infill and finish | cabinets, fittings, finish, furniture | replaceable and repairable |
| External realm | courtyard, shade, garden, work yard, water feature | water, safety and access constraints |

The benefit is not that every layer is modular. The benefit is that a kitchen change should not require an unreviewed structural intervention, and a future room should not block an escape route, remove daylight from a neighbour or overload drainage.

### 2.4 Incremental does not mean incomplete

A household should be able to build in stages without being forced into an unsafe shell or an expensive one-time project. The initial phase must be a complete, safe and serviceable home for its declared occupancy.

Every planned future addition needs a reserved and checked envelope for:

- structural support and foundations;
- fire separation, escape and emergency access;
- daylight, ventilation, privacy and neighbour impacts;
- sanitation, water, electrical and drainage capacity;
- weatherproofing and construction access;
- ownership, common-property and municipal approval conditions.

This lesson matters: incremental housing can create real resident agency, but published follow-up on Quinta Monroy documents how uncontrolled extensions can compromise habitability. Future growth needs a pre-engineered and governed interface, not simply an empty bay.

## 3. Highveld architecture families to generate

These are design grammars and test archetypes, not fixed products or universal winners.

### A. Solar courtyard home

A north-oriented living edge and protected courtyard can support winter solar access while external shading and seasonal screens reduce high summer sun. High thermal mass, insulated roof/envelope, secure night ventilation and a shaded outdoor room are tested together. The courtyard is a microclimate zone with its own wind, shade, mean-radiant-temperature and privacy evidence—not an automatic cooling device.

### B. Row or terrace home with service yard

A compact party-wall form can reduce exposed envelope area and use repeatable structural/service interfaces. A private, ventilated service yard accommodates laundry, repair, storage, rainwater components and future equipment without consuming the primary living area. Acoustic and fire separation between attached units are hard constraints.

### C. Expandable compact detached home

A complete one- or two-bedroom core is designed with a prequalified expansion bay and reserved service routes. The design is tested against the "no expansion" baseline as well as future rooms. Expansion may be rejected if it harms sunlight, drainage, privacy, structure or safe access.

### D. Duplex / intergenerational home

Two independently usable zones can share selected services while maintaining privacy, acoustic separation, safe circulation and a lawful access/utility arrangement. The model must distinguish shared ownership from shared occupancy assumptions.

### E. Accessible single-level home

A step-free route, usable turning space, adaptable bathroom, reachable storage and low-effort controls are planned from the start. The profile captures functional design requirements; it does not infer a person's health status. Exact dimensions are taken from the applicable standards and professional review, not invented by a generic template.

### F. Live-work / maker home

A work zone has an explicit acoustic, fire, ventilation, storage, delivery and visitor boundary from sleeping/living spaces. The model distinguishes low-impact desk work from workshop activity; it does not assume every trade is safe or lawful inside a dwelling.

### G. Clustered homes with shared amenities

Homes can share a workshop, laundry, guest room, garden, play area, storage or a water/energy asset. Sharing is optional. Each household retains a protected service floor and safe fallback; access, capacity, queues, maintenance, governance and cost allocation must be explicit.

### H. Retrofit-first / maintain-existing home

The system must be willing to recommend repair, insulation, shading, ventilation, service upgrades or reconfiguration instead of new construction. "No build" is a real candidate, not a failure state.

## 4. Water-aware thermal architecture is optional and evidence-gated

Highveld thermal design starts with a competent no-water baseline: orientation, shade, roof insulation, envelope, thermal mass, controlled ventilation and safe night flushing. Water systems are added only when a matched test shows a useful benefit under a declared water budget.

Candidate additions include roof-water harvesting with first-flush and filtration, non-potable reuse where permitted, a covered cistern or pool coupled through a qualified heat exchanger, and selectively operated low-flow evaporative features. Potable, rainwater, greywater and reclaimed water remain distinct source classes. No pool, fountain or spray system is presumed to improve passive cooling.

The Highveld Commons work in PR #741 and issues #745/#766 owns the neighbourhood water/thermal contracts. This home compiler consumes those qualified interfaces; it does not duplicate the pool physics or water-rights ledger.

## 5. Mass customization without bespoke cost explosion

The proposed design grammar uses a small set of stable interfaces and a large but bounded set of resident-visible choices:

- structural grids and repeatable spans, with site-specific structural qualification;
- standardized opening, shading and service-penetration details;
- configurable room modules and reversible partitions;
- locally available material families with documented substitutions;
- service cores that reduce duplicated plumbing and maintenance access;
- interchangeable infill, cabinets, storage and finish packages;
- pre-engineered expansion nodes and future-service capacity;
- standard quantity takeoff and inspection evidence.

A material substitution is not equivalent merely because its marketing name or nominal dimension matches. The system records the actual material, source, tested property, installation method and affected design claims.

The first prototype should be deliberately modest: one synthetic site, one anonymized brief, at least two distinct alternatives, a visible Pareto comparison, a quantity/cost ledger with unknowns preserved, and an exported open model. No automatic permit submission, procurement or construction action.

## 6. Benchmark design

### 6.1 Functional unit and matched inputs

The comparison unit is **one dwelling design revision that meets one resident brief over one stated service horizon**. Freeze the following across comparators:

- identical brief and hard constraints;
- identical site survey and rights/authority envelope;
- identical climate/weather and future-weather assumptions;
- identical occupancy/use assumptions;
- identical material availability, labour skills and delivery assumptions;
- identical cost boundary, price date, currency and uncertainty method;
- identical specialist physics versions and compute/human-review budget.

A concept rendering is not compared to a stamped construction drawing. Each comparator is evaluated at the same maturity level and human-input budget.

### 6.2 Comparator roles

| Comparator | What it already does well | Correct use in this programme |
|---|---|---|
| Architect-led workflow | contextual judgement, client collaboration, professional accountability | human baseline for time, fee, options, rework and quality |
| TestFit Site Solver | parcel/site configuration, code parameters, pro forma, cut/fill and takeoffs | site feasibility and developer workflow comparison |
| Finch3D | rule-bound plan generation, plan-library constraints, accessibility/code-aware layouts | closest plan-generation comparison |
| Autodesk Forma | early-stage site design and daylight analysis | site/daylight lane where product coverage supports it |
| IFC / openBIM | interoperable building information exchange | geometry, space identity and data portability |
| Brick / ASHRAE 223 / Haystack-XETO | operational semantic representation | use Sol-Atlas #704 mappings; do not create a new ontology |
| EnergyPlus / OpenStudio | whole-building energy, water and supported thermal simulation | thermal/energy reference engine, not the resident-intent engine |
| Local trades / construction-system baseline | procurement, fabrication, skills and site reality | buildability, delivery, substitutions and maintenance |

Pin versions, geography, product entitlement, input scope and measurement method. Report unsupported tasks as `NOT_COMPARABLE`, not as an incumbent failure.

### 6.3 Outcomes are vectors, not a single score

Report at least:

1. **Agency:** hard-requirement preservation, preference retention, number/diversity of feasible options, resident-selected change history.
2. **Function:** activity fit, circulation, adjacency, privacy, storage and acoustic separation.
3. **Health and access:** step-free usability, safe movement, daylight, indoor air, moisture, thermal comfort and injury hazards.
4. **Compliance readiness:** evidence coverage by authority/discipline, unresolved items and professional review status.
5. **Buildability:** quantity accuracy, material availability, substitution rate, skilled labour hours, logistics, waste and phase disruption.
6. **Cost:** capital, whole-life cost, cost variance, contingency, unknown rate coverage, stage affordability and maintenance.
7. **Performance:** occupied-hour operative temperature, discomfort degree-hours, humidity/dew point, indoor air, energy and water per service-year.
8. **Carbon and resources:** embodied and operational carbon, water sources/losses, repair/reuse and waste.
9. **Adaptability:** time/cost to adapt, reused components, change-induced downtime and essential-service continuity.
10. **Resilience:** service-floor violations, outage duration, fallback independence, repair time and household-specific unmet outcomes.
11. **Privacy and rights:** unsupported data reuse, consent errors, unnecessary personal data and owner-revision mismatch.

Do not combine these into one universal "dream home score." A reduction in energy cannot compensate for a fire-safety failure; a high average satisfaction cannot compensate for one household's essential-service failure.

### 6.4 P0 qualification targets

| Gate | Target | Evidence required |
|---|---:|---|
| False-feasible hard-constraint cases | 0 | independent oracle and seeded failures |
| Required resident constraint retention | 100% | brief-to-candidate trace |
| Exact known-answer agreement | 100% | frozen manifest, independent evaluator |
| Quantity rows with revision/unit/source/method | 100% | BOM provenance check |
| Cost rows with unknown/exclusion/uncertainty semantics | 100% | independent cost ledger |
| Unsafe occupied phase accepted | 0 | phased safety negative controls |
| Unreviewed modification marked buildable | 0 | dependency invalidation tests |
| Semantic loss in required export fields | 0 | round-trip IFC/open-data tests |
| Unauthorized procurement/permit/actuation | 0 | seeded authority-boundary controls |
| Valid positive controls preserved | 100% | known-feasible reference cases |

Time-to-option, design diversity, option count, cost estimation error and iteration speed are measured against the matched manual/product baselines first. Set improvement targets only after those baselines are observed.

## 7. Initial negative controls

- A brief requests more floor area than the legal/buildable envelope permits.
- A site has an unresolved servitude, survey conflict or unknown soil condition.
- A code profile is stale or belongs to another municipality/occupancy class.
- An accessible route is broken by a step, door swing, slope or future extension.
- A future room blocks egress, fire access, daylight, drainage or neighbouring privacy.
- A live-work proposal has unqualified fire, acoustic, ventilation or visitor separation.
- A proposed structural span or foundation is outside the declared engineering envelope.
- A low-cost material is unavailable locally or lacks the required evidence.
- A substitution invalidates a fire, thermal, structural or warranty claim.
- A cost rate is absent, outdated, or in a different currency/unit and is silently treated as zero.
- A water feature consumes protected household water or runs without qualified source quality/permission.
- A high-efficiency option is uncomfortably cold during a Highveld winter night.
- A phase leaves the dwelling weather-exposed, without sanitation or without safe occupancy.
- A model uses the same observations for calibration and held-out validation.
- Resident preferences are altered after the system sees the result, or an optimizer silently changes a hard constraint.
- A privacy-restricted brief is exported to an external service without exact consent.
- The no-build/repair option is omitted even when it is a feasible lower-cost alternative.

## 8. South African / Highveld application

The first profile is Johannesburg / Highveld, but all climate and authority data are versioned inputs. SANS 10400-XA:2021 is a mandatory building-energy framework for relevant new buildings, with prescriptive and performance/reference-building routes; building-plan review and other National Building Regulations remain jurisdiction- and building-specific. The tool must track the currently applicable edition, amendments, municipal processes and professional sign-off. It is not a compliance oracle by itself.

Johannesburg's climate makes a seasonal design necessary: summer shading and ventilation are not enough if winter night comfort, insulation, solar gain and condensation are ignored. Use a pinned TMY for early comparison, but also recent actual years, hot/cold extremes and future climate scenarios for lifecycle studies. Grid/reanalysis climate data are not a substitute for a site or courtyard weather station.

## 9. Implementation sequence

**P0 — Contract and synthetic corpus**
- resident brief schema;
- archetype and scenario manifest;
- hard-gate / negative-control evaluator;
- independent known-answer cases;
- versioned comparison protocol.

**P1 — Minimal candidate generator**
- two or more explainable alternatives for one synthetic brief;
- stable interface library and parametric layout primitives;
- revision-bound space/function graph;
- quantity/cost unknown propagation;
- no external actions.

**P2 — Specialist adapters**
- IFC/openBIM exchange and #704 semantic crosswalk;
- EnergyPlus/OpenStudio thermal lane;
- qualified structural, fire, daylight, acoustic, moisture and accessibility tools;
- local material catalogue with evidence and revision.

**P3 — Participatory design and delivery**
- resident review / edit history;
- qualified professional review;
- local contractor and trades workflow;
- staged delivery, inspection and as-built records.

**P4 — Occupied-life learning**
- consent-based post-occupancy evaluation through #633;
- performance gap and maintenance tracking;
- safe change/extension requalification;
- only then expand to multiple climates and regions.

## 10. Claim ceiling

This v0.1 artifact is a research and benchmark contract only. It contains no generated construction-ready dwelling, no validated building-physics model, no local price database, no approved code engine, no material certification, no professional review and no resident study. Its synthetic targets are proposed acceptance criteria, not achieved results. Any future "better than SOTA" claim must be matched, independently reproducible and scoped to a specific task where the baseline is actually comparable.

## 11. Research and reference anchors

- SANS 10400-XA:2021 policy record: https://www.iea.org/policies/8412-sans-10400-xa2021-national-building-regulations-energy-usage-in-buildings
- City of Cape Town explanation of SANS 10400-XA 2021 Ed. 2: https://www.capetown.gov.za/work-and-business/planning-portal/regulations-and-legislations/energy-usage-regulations-for-buildings/
- EnergyPlus 26.2 documentation: https://energyplus.readthedocs.io/en/v26.2.0/index.html
- OpenStudio: https://openstudio.net/
- TestFit Site Solver: https://www.testfit.io/product/site-solver
- Finch3D product overview: https://www.finch3d.com/product
- WHO Housing and Health Guidelines: https://www.who.int/publications/i/item/9789241550376
- MoMA, Elemental / Quinta Monroy (2003–05): https://www.moma.org/interactives/exhibitions/2010/smallscalebigchange/projects/quinta_monroy_housing.html
- SciELO, *Beyond the freedom to build: Long-term outcomes of Elemental's incremental housing in Quinta Monroy*: https://www.scielo.br/j/urbe/a/ZCgQWz9QtCjQhSdxvxQQY6q/
- Johannesburg passive building guidance (University of Johannesburg-hosted City guide): https://www.uj.ac.za/wp-content/uploads/2022/11/coj-design-guidelines-for-energy-efficient-buildings-in-johannesburg-2008.pdf
