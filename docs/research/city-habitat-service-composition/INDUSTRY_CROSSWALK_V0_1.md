# Industry and standards crosswalk — habitat service composition v0.1

This is a benchmark design comparison, not a certification or a claim of outperforming the systems below.

| Comparator / framework | Established contribution | Boundary the composition benchmark addresses |
|---|---|---|
| ISO 23247-6:2026 | Defines integrated, unified and federated digital-twin composition, with communication, aggregation and interoperability guidance | Add explicit claim-level scope, time, authority and evidence-state checks to one human-habitat service decision |
| buildingSMART IDS 1.0 | Computer-interpretable exchange requirements and IFC information checking | Extend from information presence/conformance into cross-owner evidence and outcome qualification; retain IDS for IFC checks |
| Conveyal R5 / Analysis | High-resolution, schedule-aware access-to-opportunities analysis and travel-time distributions | Do not rebuild its routing. Bind its output to actual service capacity, eligibility, affordability, universal access and operations evidence |
| OpenTripPlanner 2 | Production multimodal journey planning and passenger information | Do not treat trip-planning output as district access analytics or proof of an operating service |
| UN SDG 11.2.1 | Comparable convenient-access indicator with attention to sex, age and disability | Reproduce the official indicator; report capacity, price, service windows, effective access and local field verification separately |
| ISO 37120 | City service and quality-of-life indicators; Edition 3 is under development as a Draft International Standard | Provide verifiable source/owner/window/denominator bindings behind the indicator values, not a replacement indicator set |
| CIBSE TM63 / TM54 and IPMVP | Methods for measured in-use building energy performance, modelling comparison and M&V | Keep measured building/service operation separate from accessibility potential, and do not infer savings or causal improvement without the relevant accepted method and evidence |
| NIST robotic performance assessment | Task-driven composition of perception, mobility, dexterity and safety measures; test methods and system-level task performance | Extend the same discipline to habitat delivery: service task, environment, human constraints, lifecycle performance and evidence assurance |

## Paradigm shifts

### 1. From asset-centric to service-centric assurance

The unit of success is not a commissioned building; it is the sustained service that the building is meant to provide to a declared population.

### 2. From proximity to effective opportunity

A point on a map is not a service. A route is not access. A reachable service with insufficient capacity, eligibility barriers, excessive price, limited operating windows or missing step-free evidence is not a usable opportunity.

### 3. From simulated/digital success to evidence-qualified outcomes

A digital twin can explain a candidate state. It does not itself prove physical acceptance, stable operations, actual access or resident benefit. Keep predictions, simulations, owner declarations, measurements and consented experience as separate evidence classes.

### 4. From opaque city scores to non-compensatory state vectors

Do not allow a strong energy result to compensate for inaccessible health care, or abundant theoretical capacity to compensate for a failed operational floor. Keep hard floors, group-level distributions and Unknown states explicit.

### 5. From static evidence to temporal replay

Use valid time, recorded time, registration time and ingestion time. Replaying a decision at an earlier evidence cut must not see data ingested later. Revocation and rights changes must affect current reliance without rewriting historical facts.

## Benchmark roadmap

| Tier | Benchmark | Target |
|---|---|---|
| E0 correctness | Frozen adversarial cases | 100% exact agreement; zero false-positive promotion |
| E1 implementation | Two independent evaluators, same inputs | 100% semantic agreement on disposition, reason codes and claim ceiling |
| E2 interoperability | Actual owner receipts, owner-native source IDs preserved | Zero unbound records; zero silent source/scope coercion |
| E3 local validity | Rights-cleared, field-verified Johannesburg pilot | Report calibrated access, capacity, operational delivery, missingness and subgroup uncertainty separately |
| E4 resilience | Revocation, stale sources, time-cut replay, common-cause and partial-owner outage tests | Fail closed; no unknown-to-pass coercion |
| E5 scale | 10k / 100k / 1M origin-group records and incremental updates | Publish latency, memory, throughput and update cost on declared hardware; never weaken evidence gates to meet speed |

## External reference links

- ISO 23247-6:2026: https://www.iso.org/standard/87426.html
- buildingSMART IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- Conveyal Analysis methodology: https://docs.conveyal.com/analysis/methodology
- OpenTripPlanner analysis boundary: https://docs.opentripplanner.org/en/latest/Analysis/
- UN SDG 11.2.1 metadata: https://unstats.un.org/sdgs/metadata/files/Metadata-11-02-01.pdf
- ISO 37120 Edition 3 draft: https://www.iso.org/standard/90316.html
- NIST robotic performance assessment: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems
- EVO IPMVP: https://evo-world.org/en/products-services-mainmenu-en/protocols/ipmvp
