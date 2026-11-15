# Luminous Habitats: competitive and SOTA benchmark plan v0.1

**Status:** research/benchmark specification. The thresholds below are targets, not measured results. No SOTA leadership, production readiness, city-scale validation or real Johannesburg outcome is claimed.

## Strategic position

Do not build a weaker replacement for every mature BIM, PLM, GIS, routing, asset-management or robotic test system. Build the **evidence-qualified service and asset composition layer** that can consume their owner-native outputs without losing identity, scope, time, authority, rights or claim limits.

A useful definition:

> A habitat is qualified only to the extent that its required services are delivered to declared beneficiary groups, through maintainable physical assets, with current evidence that binds the design, supply, construction, commissioning, access and in-use operation to the same declared scope.

The evidence chain is:

`need -> service requirement -> facility/product -> design/BOM -> qualified supply -> construction -> acceptance -> commissioning -> access potential -> operation -> realized use -> maintenance/recovery`

Every arrow is an explicit owner-to-owner join. A pass in one lane cannot promote an unknown or failed lane.

## 1. Industry baseline: what we should reuse and what we should add

| Industry system / method | Established capability | Luminous strategy |
|---|---|---|
| **Bentley iTwin** | Open APIs/services for infrastructure twins; integrates engineering, reality, IoT and enterprise data, synchronizes changes and supports lifecycle collaboration | Treat as a possible infrastructure-twin owner. Benchmark whether Luminous can retain exact source identity, revisions, temporal cuts, rights and independent assurance when ingesting its outputs |
| **Autodesk Tandem / Forma / operations ecosystem** | Autodesk is connecting design, construction, manufacturing and live operations; its September 2026 announcement describes Tandem integration with Forma, FlexSim and MaintainX | Do not rebuild the operations platform. Add independent qualification of cross-system claims and a service-outcome evidence graph |
| **Siemens Teamcenter** | PLM digital thread from plan and develop through delivery, manufacturing, service and suppliers | Use Teamcenter as a PLM/BOM/configuration owner where present. Luminous should verify cross-owner compatibility and lifecycle claim dependencies, not duplicate PLM |
| **Conveyal R5 / Analysis** | Optimized many-to-many, time-window travel-time and access-to-opportunities analysis; regular grid resolution and departure-time variability | Use it as a routing/accessibility reference. Compare identical networks, service opportunities, windows, modes and percentile settings; add service capacity, eligibility, price, opening, accessibility and provenance gates |
| **OpenTripPlanner 2** | Production passenger-information and multimodal routing | Use for trip-planning interoperability. OTP2 explicitly says transit analytics work has shifted to specialist projects such as R5; do not treat OTP2 as the city-access benchmark engine |
| **ArcGIS Urban** | Scenario metrics, parcel suitability and weighted-criteria analysis, shadow and land-use impacts | Compare scenario inputs and raw criteria. Keep hard floors and Pareto comparisons separate from weighted suitability scores |
| **buildingSMART IDS 1.0** | Computer-interpretable IFC information requirements and model checking | Consume IDS/IFC conformance evidence. Add cross-owner receipt identity, scope/time/rights binding and independent result agreement; do not replace IDS |
| **IGLC Industrialized Construction Index (ICI)** | Five maturity pillars: standardization, prefabrication, modularization, automation and digitalization; the 2026 paper reports case deployment across 61 Colombian and 33 French residential projects | Use ICI as a maturity comparator. Add measured accepted output, service availability, lifecycle maintenance and beneficiary outcomes |
| **NIST robotic performance assessment** | Task-driven measurement for perception, mobility, dexterity, safety and composed assembly performance | Use NIST/ASTM test methods for robot capabilities. Extend to whole construction tasks, site variability, human exposure, rework and accepted habitat output |

### What we must not claim

- Bentley, Autodesk or Siemens do not lack digital-twin or lifecycle integration; they are established platforms with substantial capabilities.
- R5 is not a weak baseline: it models many-to-many, time-window travel-time variability. Its documented default assumes perfect transit reliability, full schedule information and unimpeded boarding; these are assumptions to expose and stress-test, not flaws to conceal.
- A route engine does not establish available service capacity, affordability, eligibility, quality or actual use.
- A digital twin, IFC model or IDS pass does not prove physical conformance, accepted installation, sustained service or resident benefit.
- An ICI maturity score is not proof that a particular district is deliverable or equitable.
- A robotic demonstration is not an end-to-end construction productivity, quality or safety result.

## 2. Standards and public-interest baselines

### ISO 23247-6:2026 — digital twin composition

Published July 2026, this International Standard identifies integrated, unified and federated digital-twin composition and provides guidance on communication, aggregation and interoperability across vendors and independently developed twins.

**Luminous contribution to test:** bind composed *claims*, not merely composed data. Require common subject, scope, revision, time, authority and evidence-state agreement. Do not present this contribution as replacing ISO 23247-6.

### buildingSMART IDS 1.0 and its implementation problem

IDS 1.0 became an official buildingSMART standard in June 2024. Its September 2026 community survey received 186 responses; 131 respondents (70%) expressed interest in some form of software certification. The survey also reports implementation inconsistencies, including different checking results from the same IDS file in different tools.

**Target:** test implementation agreement as a first-class metric. For each supported IDS profile, replay the same frozen models/specifications across at least two independent checkers and the Luminous adapter. Report every semantic divergence; never treat one vendor's PASS as universal conformance.

### ISO 37120 Edition 3 — draft, not a published new standard

As of 9 November 2026, Edition 3 is an ISO Draft International Standard in the enquiry phase (stage 40); ISO 37120:2018 remains the prior published edition. Do not encode unpublished draft clauses as binding requirements or claim certification against the draft.

**Target:** maintain a versioned indicator registry with edition, clause, unit, calculation method, source owner, disaggregation, applicability and evidence ceiling. Reproduce published indicators exactly before proposing extensions.

### UN SDG 11.2.1 and the Johannesburg measurement gap

The UN metadata uses network walking-distance criteria (500 m to lower-capacity transit and 1 km to higher-capacity systems) and adds considerations such as accessibility for special-needs customers, peak frequency and safe/comfortable stops. The City of Johannesburg's published 2020–2021 domesticated proxy reports 93.56840621%, but its method is based on Quality of Life survey responses about walking time to the nearest public-transport access point (under 20 minutes), not the same calculation as the UN network-distance method.

**Target:** implement both as separately labelled indicators. Never put a locally domesticated proxy in the same trend line as the UN definition without a documented method bridge.

### Access-to-opportunities baselines

Conveyal documents a regular grid model (up to five million cells) and travel-time distributions over departure windows. Its defaults assume perfect transit reliability, complete schedule knowledge and unimpeded boarding. R5/r5r support percentiles and step/exponential/logistic decay functions.

**Target:** reproduce published R5 outputs on matched network/time inputs. At minimum compare p50, p90 and p95 door-to-door travel time, access-to-opportunity counts, sensitivity to departure window and sensitivity to unknown/inaccessible edges. Preserve the difference between a travel-time percentile and an accessibility-distribution percentile.

### Construction robotics

NIST's performance framework composes task-driven measures of perception, mobility, dexterity and safety into system-level assembly performance. NIST's autonomous-construction work also studies steel assembly, site metrology, tracking, sensor exchange and design-data integration. A 2026 systematic review of 375 construction-robotics studies reports that field use remains heavily operator-led and that planning, maintenance and demolition are less explored than structural work.

**Target:** compare robots on the whole task and environment, not isolated manipulator specs. Keep safety, accepted quality, productivity and lifecycle economics visible together.

### Building in-use and measurement & verification

CIBSE TM63 and IPMVP provide specialist methods for measured in-use energy performance and efficiency measurement & verification. Their presence does not justify the bridge making an energy-savings or causal claim from a meter reading alone.

**Target:** separate design prediction, as-built/commissioning, raw observation, normalized observation, calibrated model, M&V eligibility, accepted savings report and causal impact. Require the exact method, baseline, post-period, adjustments, coverage and owner approval before stronger claims.

## 3. Benchmark suite: how to exceed the current baseline

### A. Evidence composition and trust

| Metric | E0 target | E1/E2 target |
|---|---:|---:|
| Frozen known-answer agreement | 100% | 100% on 500+ cases |
| False favorable promotion on adversarial controls | 0 | 0 across 10,000 mutation/replay cases |
| Common-scope identity binding | 100% of required owners | 100% with two independent implementations |
| Unknown/stale/revoked/late evidence promoted to PASS | 0 | 0 |
| Input/owner-order permutation invariance | 100% | 100% |
| Exact-head receipt reproducibility | 100% | 100% across clean rebuilds |
| Unbound/duplicate source rows accepted | 0 | 0 |

A false acceptance is more serious than a false rejection. Publish both, but keep the former at zero for all safety/eligibility/authority hard gates.

### B. Access and service equity

Report a vector by group, time window and service category. Do not hide tails in a district-wide mean.

- p50 / p90 / p95 door-to-door time by mode and departure window;
- share of each declared beneficiary group with at least one usable service;
- reachable capacity / declared demand, by compatible physical unit;
- service hours available / required and time-of-day coverage;
- affordability and eligibility pass/fail/unknown;
- step-free / universal-access status and evidence coverage;
- group-level p50, p90, worst decile and between-group gaps;
- unknown population, demand and source coverage as explicit denominators.

**Target:** reproduce UN SDG 11.2.1 exactly; reproduce R5 access outputs on matched cases; then demonstrate that Luminous catches cases where proximity, a route or aggregate capacity overstates effective service opportunity. Use 100% exact agreement on small deterministic network fixtures and report all divergences on larger matched networks.

### C. Restoration and operational delivery

- time to first adequate service;
- time to sustained adequate service under the owner's stability contract;
- unavailable, constrained, inaccessible and unknown service-hours;
- recurrence/rebound and longest contiguous degradation;
- service operational floor coverage;
- repair/rework rate and accepted output;
- energy, water, material and maintenance cost per compatible service unit.

**Target:** no restoration claim from a closed work order or single sample. Every service must satisfy owner continuity, required inspection/commissioning, and the operational floor declared by its contract.

### D. Industrialized construction and physical production

Use ICI's five pillars as one maturity baseline; extend it with actual production evidence:

- first-pass accepted modules/panels;
- defect/rework and replacement rates;
- schedule reliability and delivery variance;
- installed cost per functional unit;
- embodied carbon and material loss per compatible functional unit;
- maintainability, component replaceability and serviceability;
- factory-to-site chain-of-custody and revision correctness.

**Target:** compare retrofit/maintain-first, conventional site-built and panelized/prefabricated alternatives on the same service basket, functional units, budget boundary, climate scenario and acceptance rules. Report Pareto fronts and hard-floor failures; do not hide them in a weighted score.

### E. Construction robotics and automation

For each task family, publish:
- accepted task completion rate;
- defect and rework rate;
- cycle time and throughput per shift;
- setup, calibration and recovery time;
- human intervention and teleoperation minutes;
- near-miss/injury exposure and safety stop rate;
- performance under dust, occlusion, uneven terrain, dynamic obstacles and load variation;
- energy and cost per accepted functional output;
- uptime, mean time to repair and task reconfiguration effort.

**Target:** benchmark perception, mobility, dexterity and safety separately, then end-to-end task outcome. A robot that is fast but produces unsafe or unaccepted work does not pass.

### F. Performance and scale

Do not optimize throughput before semantic correctness. After correctness is frozen, publish cold/warm latency, p50/p95/p99, memory, CPU, incremental recomposition cost and receipt-verification cost for 10k, 100k and 1M scope/evidence records, with exact hardware and corpus sizes.

**Target:** determine an empirical performance budget against the reference implementation; do not claim a speed advantage until the same inputs and correctness rules have been run against the selected baseline.

## 4. Paradigm shifts with the highest leverage

1. **Service-first digital thread:** the primary entity is the delivered service and beneficiary group, not the building model.
2. **Non-compensatory assurance:** a clinic that is inaccessible cannot be rescued by excess capacity elsewhere; a failed safety floor cannot be averaged away by schedule performance.
3. **Federated owner authority:** each supplier, operator, resident-facing service and professional retains authority over their own evidence. A composition layer may qualify reliance, not commandeer decisions.
4. **Temporal truth:** record valid, recorded, registered and ingested times; enable historical replay without future evidence leakage.
5. **Evidence-aware uncertainty:** Unknown is a first-class state. Missing transport, demand, occupancy, rights or calibration evidence is not silently replaced with a favorable assumption.
6. **From simulation to field correction:** every operational variance creates a versioned learning item for future designs, BOMs, procurement, construction method and maintenance—not an unverified model update.
7. **No universal habitat score:** publish independent vectors and distributions; let communities and competent authorities choose priorities within explicit hard floors.

## 5. Execution order

### Immediate — independent of existing runners

- Finish exact-head qualification of PR #687 and retain any failing case as a permanent regression.
- Add canonical owner schema adapters for the actual #564, #635, #667 and #624 receipt formats.
- Add one deliberately adversarial case for each source: stale, revoked, mis-scoped, duplicate, late-ingested, unit-incompatible and rights-restricted.
- Extend benchmark report generation to include source SHAs, corpus/schema digests, reason-code counts and claim ceilings.
- Update the reference-district integration manifest with explicit owner-native schema readiness and unresolved scope lanes.

### Next — independent baselines

- Add a matched R5 reference suite for the CityMorph routing cases.
- Add official SDG 11.2.1 known-answer cases and a separate Johannesburg proxy reproduction.
- Add a cross-check harness for IDS 1.0 implementations and IFC revision bindings.
- Add the ICI five-pillar baseline to the district scenario comparison.
- Add NIST-inspired task cards for panel handling, positioning, fastening, inspection, repair and recovery.

### Later — evidence collection and field pilots

- Register a real, rights-cleared Johannesburg service frame and population denominator.
- Conduct consent-based field checks for route and service usability; keep survey non-response and sampling bias visible.
- Integrate operational service receipts and actual usage/encounter evidence only where lawful, purpose-bound and independently owned.
- Compare retrofit, conventional and industrialized construction on one measured reference district.
- Validate lifecycle performance across seasons and post-maintenance interventions.

## 6. Decision rule for claiming “better than SOTA”

We only claim superiority when all of the following hold:

1. The comparator has been run on the same dataset, scope, units, assumptions and hardware where relevant.
2. The comparator and Luminous outputs have the same semantics and claim ceiling.
3. The benchmark is independently reproducible from pinned inputs and software versions.
4. Accuracy, safety, uncertainty, missingness and worst-group performance are reported—not only averages.
5. The improvement is statistically/materially meaningful and does not weaken any hard gate.
6. The result is independently reviewed or externally reproduced.

Until then, state the capability gap, the proposed test and the exact evidence needed. The ambition is to create a new assurance class—not to use an impressive benchmark number as a substitute for proof.

## Sources

- ISO 23247-6:2026: https://www.iso.org/standard/87426.html
- buildingSMART IDS and 2026 survey: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/ ; https://www.buildingsmart.org/ids-survey-results-2026/
- Bentley iTwin Platform: https://developer.bentley.com/itwinplatform/
- Autodesk operations update, 15 September 2026: https://adsknews.autodesk.com/en/news/autodesk-connected-operations-au-2026/
- Siemens Teamcenter: https://www.siemens.com/en-us/products/teamcenter/
- Conveyal R5 methodology: https://docs.conveyal.com/analysis/methodology
- OpenTripPlanner 2 analysis boundary: https://docs.opentripplanner.org/en/latest/Analysis/
- ArcGIS Urban analysis: https://doc.esri.com/en/arcgis-urban/latest/get-started/get-started-analysis.html
- IGLC 2026 Industrialized Construction Index: https://www.iglc.net/papers/details/2492
- NIST robotic-system performance assessment: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems
- NIST autonomous construction: https://www.nist.gov/publications/nist-research-autonomous-construction
- UN SDG 11.2.1 metadata: https://unstats.un.org/sdgs/metadata/files/Metadata-11-02-01.pdf
- Johannesburg SDG proxy metadata: https://sdg.joburg.org.za/11-2-1/
- ISO/DIS 37120 Edition 3 status: https://www.iso.org/standard/90316.html
- IPMVP: https://evo-world.org/en/products-services-mainmenu-en/protocols/ipmvp
