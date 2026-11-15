# CityMaker cross-repository evidence flow v0.1

**Status:** integration architecture and ownership map. It is not a production protocol, blanket approval, integrated city twin or claim that every referenced connector is implemented.

## 1. Core rule

The city system is a federation of bounded authorities, not a monolithic digital twin.

Every transition must preserve the difference between:

`intent -> design proposal -> analysis -> admissible evidence -> qualified capability -> authorized work -> as-built article -> commissioned asset -> operational service -> measured outcome`

A layer can consume another owner's receipt only within the receipt's stated subject, scope, version, time validity, units, evidence class and claim ceiling. It cannot silently strengthen that evidence.

## 2. End-to-end lifecycle

| Stage | Primary owner(s) | Output consumed by next stage | What the output does **not** prove |
|---|---|---|---|
| Human need, service level, resident constraints | Sol-Atlas #549/#555; Symtropy #1766; relevant civic/community process | Versioned requirements, evaluation scope, population/service needs, non-compensatory floors | Land rights, consensus, legal authority or engineering feasibility |
| Candidate typology and design intent | Symthaea #7790; Symthaea ENG-DESIGN #6005+ | Exact candidate design subject, parameters, parent/seed lineage, unresolved constraints | Structural/code/fire approval or physical existence |
| Site and source context | Sol-Atlas #561; ATLAS-GEO-001 #65; municipal/GCRO/Stats SA/OSM custodians | Pinned geospatial source manifest, access/rights state, CRS, vintage, spatial support, uncertainty | Complete parcel/title, subsurface condition, building occupancy or permission to intervene |
| Climate and environmental context | GreenBook MetroView; Climate.OneBuilding TMYx; authoritative weather/hazard owners | Separate exposure/weather profiles with class, period, scenario, station/grid, source digest and native resolution | Building indoor temperature, medical heat safety or future exact-date forecast |
| Building/facility realization profile | Symthaea #6032 BUILT-ENV, #6036 BIM, #6034 site, #7697 civil | Versioned BuildingProfile / IndustrialFacilityProfile / SiteInfrastructureProfile | That the design has been built, commissioned or is operable |
| Engineering and physical analysis | Symthaea structural/materials/thermal/energy/water owners; Symtropy #1790/#1848/#1900 and appropriate domain solvers | Exact solver/model receipts, input/output digests, convergence, validation scope, uncertainty, limits | Physical measurement, professional sign-off or code approval by itself |
| Material and production capability | Symthaea MFG-PROC #5686, HABITAT-VERTICAL #8051; Mycelix #4814; Mycelix Living-City PR #5035; Symtropy #3768 | Process/capability and supplier/inventory/flow receipts with grade, quantity, unit, lead time, location, uncertainty and owner | Local productive capability from map proximity; deliverable stock from nominal stock; qualified substitution without inspection |
| Circularity and repair pathway | Symtropy #1856/#1984; Mycelix #5035/#5149; Sol-Atlas #569 | Intervention comparison, mass/grade flow, custody/lot and candidate reuse/recovery projection | That mapped material is recoverable, safe, owned/available or accepted for reuse |
| Construction method and cell production | Symtropy #3773 panel cell, #1818 timber robotics, #1834 robot capability matching, #3605 accepted-output; Symthaea #4795/#6040 | Exact work package, tool/robot/configuration/calibration, full-workflow labour and independent acceptance receipts | A simulation pass is a physical qualification; robot cycle time equals human-plus-system throughput |
| Delivery and site receiving | Sol-Atlas #573; Mycelix #4814/#5149; Symtropy route/handling/resource owners | Work-package-specific source lot, quantity, vehicle/load, route, access window, unload, receiving and custody receipts | Accepted installed inventory before receipt/inspection |
| Construction and as-built state | Existing Symtropy Fabrication/Construction execution chain; Symthaea #6040; Mycelix #5149 | Exact as-built artifact, design deviation, metrology/inspection, process and custody lineage | Commissioning, operational service or lifetime performance |
| Commissioning and service realization | Symthaea CIV-SERVICE #5949; CIV-BOOT #5774; utility and equipment owners | Asset-to-service qualification with dependencies, capacity, staffing, utility requirements and maintenance | Citywide accessibility or sufficient capacity across all user groups |
| Operational service / material / energy / water flows | Mycelix Living-City PR #5035 and existing Mycelix service/production owners; Symtropy CityKit #3768 | Owner-native operational observations, physical conservation and service/resource status receipts | Causal impact from correlations or unverified estimates |
| City accessibility and distribution | Sol-Atlas #555/#557; routing references R5/OTP; Mycelix service-flow context | Network reachability, service availability, capacity potential, affordability, eligibility, universal access and uncertainty | Actual allocated capacity or resident preference from a map route |
| Time-dependent continuity and heat | Sol-Atlas #563/#574 and #593/#595; Symtropy #1848/#1900 | Phase-based access/continuity, indoor thermal service evidence, measured response and unknown duration | Hazard probability or a building-level claim from ward-level data |
| Alternative comparison and staging | Symtropy #2529 RDO; Sol-Atlas #131 site Pareto/VOI; #214 portfolio selection | Owner-native dominance certificates, investigation requests, multi-project sequences under shared budgets/dependencies | A socially legitimate winner or authority to procure/build |
| Governance and public decision | Relevant municipality, statutory professionals, land/asset owners, affected residents and accountable organizations; Mycelix #5149 carries scoped approval evidence | Explicit decision owner, permissions, dissent/conditions, signed/revocable approvals and exact scope | That a technical score or system-generated certificate replaces human/legal authority |

## 3. Evidence object requirements

A cross-owner reference must minimally identify:

- owner repository and stable issue/contract/schema version;
- subject ID and revision;
- evaluation scope digest (geography, horizon, population/service frame, functional unit and scenario);
- exact source/data/model/corpus/configuration digests;
- units, CRS/coordinate frames and spatial/temporal resolution where applicable;
- evidence class: synthetic, declared, observed, modelled, calibrated-modelled, measured, independently accepted, unknown, stale, revoked or disputed;
- currentness/validity window and invalidation conditions;
- output/claim ceiling and unsupported cases;
- exact verifier/version/commit and receipt identity.

An HTTP URL, a green workflow badge, a signed supplier assertion or a “PASS” string is not enough without subject/scope/digest binding.

## 4. Claim promotion gates

The federation must never permit these transitions by implication:

- Design intent -> as-built building.
- CAD/BIM/IFC model -> verified geometry/material installed.
- Simulation/process exit zero -> converged/valid analysis.
- Converged analysis -> physical qualification.
- Vendor robot datasheet -> task-specific capability.
- Supplier offer or map point -> qualified available local supply.
- Material stock -> recoverable stock -> qualified secondary input -> deliverable lot -> accepted installation (each is a separate gate).
- Route reachability -> service availability -> adequate capacity -> actual allocation.
- TMY -> heatwave profile; ward population -> building population; modelled temperature -> measured temperature.
- Installed retrofit -> inspected/commissioned retrofit -> measured effect -> population programme impact.
- Pareto frontier -> democratic consensus or build authorization.

Missing, stale, revoked, disputed, scope-mismatched or unit-incompatible evidence must produce a typed Unknown/NotComparable/Unsupported/Blocked state rather than a favorable default.

## 5. How the four repositories should compose

### Symthaea — proposal and engineering meaning
Develops requirements interpretation, design candidates, process and capability semantics, engineering models, predictive reasoning and bounded qualifications. It cannot authorize construction or claim physical state from a generated design.

### Symtropy — physical/resource evaluation and deterministic execution context
Owns domain simulation, construction/workcell geometry and resource/time scenarios, accepted-output benchmarking and physics/operation validation within declared domains. A solver result or simulation state cannot become physical truth without evidence and appropriate authority.

### Mycelix — shared operational facts and accountable multi-party coordination
Owns participant/organization facts, inventory/offer/work/lot events, custody, approval, correction/revocation, urban metabolism and physical flow accounting. It must not infer engineering properties or capability from signatures or commercial offers.

### Sol-Atlas — spatial and decision-support projection
Owns source-aware geography, city morphology, network access, material/logistics/retrofit projections and staged portfolio decision support. It consumes owner-native receipts and preserves spatial resolution, rights, uncertainty and scope. It is not the system of record for engineering simulation, production inventory, material flow or physical commissioning.

## 6. Cross-system test protocol

Before enabling any live connector:
1. Freeze one tiny exact subject in each owner.
2. Produce owner-native synthetic receipts with exact subject/scope/source digests.
3. Verify them in the consuming repository using a separately written adapter oracle.
4. Seed wrong-revision, wrong-scope, wrong-unit, stale, revoked, duplicate, reordered and replayed receipt controls.
5. Prove all mismatches fail closed and that valid receipts retain the original claim ceiling.
6. Change one upstream subject and demonstrate explicit invalidation/re-evaluation downstream.
7. Only then add a rights-cleared recorded-data replay; do not start with live physical actuation or public release.

## 7. Current integration gaps worth prioritizing

- Municipal spatial inventory PDF and data-custodian contacts must be parsed and reconciled to exact source-layer rights.
- GreenBook MetroView's ward-level current/2050 layer methodology, exports, scenario IDs and licence remain to be obtained.
- Johannesburg TMYx station candidates need file-header validation and station representativeness comparison.
- No official current Johannesburg GTFS feed has yet been verified in this inventory.
- CityMorph access and continuity oracles still need exact-head green receipts after recent fixture/lint corrections.
- Heat evidence oracle's 27-case branch needs its latest exact-head receipt; the H6 programme-impact level is reserved as Unsupported.
- Cross-repository live receipt adapters are not yet demonstrated merely because issues are cross-linked.
- The panel-cell physical comparison has not been run; there is no measured productivity, cost or worker-safety superiority claim.

## 8. Claim boundary

This document defines composition and evidence-flow requirements. It does not prove integrated functionality, real local data quality, engineering approval, community consent, procurement rights, safe construction, city resilience or a preferred development portfolio.
