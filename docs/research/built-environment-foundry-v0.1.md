# Built Environment Foundry: evidence-first site selection and design pipeline

Status: research/implementation RFC v0.1 — proposal and first Rust policy slice, not a validated planning instrument.

## Thesis

The first question for a major work is not “what should we design?” but “where is this project viable, for whom, under which conditions, and with what evidence?” Site selection is upstream of architecture, engineering, cost, carbon, construction logistics, and community outcomes. A wrong site can make an otherwise brilliant design unsafe, unaffordable, ecologically harmful, or socially extractive.

The Built Environment Foundry should connect Symthaea, Mycelix, Symtropy, and Sol-Atlas through a shared, versioned project model. This RFC starts with Sol-Atlas because it already has a Rust core and geospatial visualization. The intended first vertical slice is a synthetic candidate-site evaluator with fail-closed hard constraints and auditable evidence coverage.

## Paradigm shifts

1. **Constraint-first, not score-first.** A weighted sum must never compensate for a verified fatal constraint. Evaluate legal land rights, jurisdiction-specific planning compatibility, ecological/protected-area constraints, hazard screening, geotechnical feasibility, essential services, emergency access, and the required community process as explicit gates. Required gates are configured per project and jurisdiction rather than assumed universal.
2. **Evidence carries its own uncertainty.** Every field has a source identifier, observation time, expiry/freshness policy, evidence state, and optional explanation. Unknown, disputed, and stale are not synonyms for zero, pass, or no risk. Unknown hard gates hold the candidate; a verified failing hard gate excludes it.
3. **Pareto frontier before a single winner.** Preserve non-dominated alternatives across cost, climate risk, access, ecology, carbon, community benefit, and constructability. Let accountable stakeholders choose trade-offs only after seeing them. Sensitivity-test rankings under plausible weight and input changes.
4. **Value-of-information planning.** Recommend the next survey or data request that most reduces decision uncertainty: geotechnical boreholes, title/servitude checks, flood modelling, utility-capacity confirmation, biodiversity surveys, contamination assessments, or community engagement. Site selection becomes an adaptive investigation, not a one-time opaque ranking.
5. **Lifecycle and urban-system boundaries.** Assess not only the parcel but network capacity and externalities: water and energy availability, transit and emergency response, embodied and operational carbon, demolition/displacement, heat, flood pathways, habitat connectivity, material transport, construction staging, adaptive reuse, and maintenance.
6. **Participatory legitimacy is evidence, not a sentiment multiplier.** Record who was consulted, who was not reachable, dissent, benefit distribution, affordability, displacement risk, land tenure, cultural heritage sensitivity, and conditions attached to community feedback. Do not infer consent from silence or online engagement.
7. **Open interfaces and independent verification.** Use OGC API - Features for interoperable geospatial feature access; IFC 4.3 for building and infrastructure exchange where appropriate; and buildingSMART IDS to express machine-checkable information-delivery requirements. Keep the internal model richer than any single export format and test round-trip loss explicitly.
8. **Generative design is downstream of site evidence.** Symthaea proposes alternatives and explains trade-offs; it cannot override hard gates. Symtropy can provide geometry and suitable physical simulation, but structural, geotechnical, fire, wind, seismic, and other specialist analyses must use validated solvers and qualified review. Mycelix can preserve provenance, approvals, resource commitments, and accountable decisions.

## Candidate workflow

1. Define the intended work, occupancy, footprint/envelope, service needs, time horizon, budget range, acceptable impacts, and decision-makers.
2. Identify geographic search areas; resolve parcel boundaries and tenure only from sourced data. Respect local data licensing, privacy, cultural heritage protections, and restrictions on sensitive locations.
3. Run exclusion overlays and project-specific hard gates. Mark missing data explicitly; do not treat absence of a mapped hazard as proof of no hazard.
4. Build evidence-backed metrics and uncertainty ranges for viable candidates. Record coordinate reference systems, resolution, source dates, lineage, and known coverage gaps.
5. Display the Pareto set and scenario-specific rankings, with confidence/coverage and the reasons a candidate is excluded or held.
6. Compute which additional observation is likely to change the decision, then request the right survey or consultation rather than generating false certainty.
7. After site due diligence, pass approved site constraints into parametric massing and design generation.
8. Keep decisions, updates, approvals, simulations, drawings, material quantities, and field changes linked to stable IDs and immutable revision records.

## Shared interface contract

Every spatial fact should carry: stable feature/site ID; geometry and coordinate reference system; unit and measurement method where applicable; source and source rights/license; observation and ingestion timestamps; spatial/temporal resolution; evidence state; uncertainty or interval; expiry rule; transformation lineage; and review owner.

The first Rust implementation in sol-atlas-core::site_selection intentionally handles only policy evaluation once inputs are normalized. It does not fetch data, geocode parcels, infer stale status from timestamps, perform GIS intersections, interpret law, or certify hazards. Those belong in separately testable ingestion, GIS, jurisdiction-policy, and professional-review layers.

## Evaluation and benchmark targets

These are proposed acceptance targets, not observed results.

### Policy correctness (synthetic first)

- 100% of verified failing hard constraints produce Excluded, regardless of preference score.
- 100% of missing, unknown, disputed, stale, or non-verified hard-gate evidence prevent ShortlistEligible.
- Missing/stale/disputed preference values do not silently become zero or appear as usable evidence.
- Identical inputs and policy versions produce byte-stable logical outputs.
- Every disposition includes a machine-readable reason; every score reports evidence coverage.
- Invalid weights, non-finite values, and scores outside [0, 1] are rejected.
- Include at least 30 cases before declaring the policy evaluator stable, with positive controls, missing-data cases, contradictory records, stale-source boundaries, score extremes, and hard-gate negative controls.

### GIS and data quality

- Geometry validity and CRS transformations tested against independently known controls.
- Explicitly test boundary edges, coordinate order, antimeridian, vertical datum/height, parcel overlaps, topology, and resolution mismatch.
- 100% lineage for score-contributing fields; sources with incompatible units/CRS or unlicensed/restricted data are blocked or clearly quarantined.
- Data freshness is policy-specific by source and jurisdiction; no global “30-day fresh” constant.
- Maintain per-layer coverage and uncertainty maps rather than hide no-data areas.
- Add a geographic false-negative suite for hazard/ecology/legal overlays; zero known seeded exclusions may be mislabeled as clear.

### Design and delivery

- Site-selection and building-design decisions remain separable and replayable under pinned inputs.
- Pareto solutions are non-dominated by the defined objectives; ranking sensitivity and scenario changes are reported.
- IFC exchange tests preserve all required IDs/properties/units; IDS validation is an explicit information requirement, not a visual inspection.
- Cost, carbon, and quantities report intervals and model scope instead of spurious precision.
- Do not claim safety or compliance from synthetic benchmarks or a passed generic test suite.

## The four-system split

- **Sol-Atlas:** spatial search, source-aware layers, parcel/site candidates, constraints, infrastructure context, uncertainty maps, and scenario comparison.
- **Symtropy:** deterministic geometry and simulation substrate where its physical models are appropriate; adapters to independently validated specialist engineering solvers.
- **Symthaea:** intent compilation, alternative generation, multi-objective search, explanation, and value-of-information recommendations. It can suggest and critique, not certify.
- **Mycelix:** provenance receipts, permissions, stakeholder/decision records, procurement and material attestations, change approvals, and lifecycle handover.

## First implementation slice

The site_selection.rs module adds typed constraints and criteria, evidence states, deterministic weighted scoring, a fail-closed hard-gate stage, reason strings, and synthetic regression fixtures. It is deliberately a policy kernel, not an end-to-end GIS or engineering system. Follow-on work should add a proper geospatial data catalog, freshness policies, Pareto and sensitivity analysis, and a reproducible GIS benchmark.

## Research basis

- buildingSMART IFC specifications: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- buildingSMART Information Delivery Specification: https://technical.buildingsmart.org/Projects/Information-Delivery-Specification-Ids/
- OGC API - Features: https://www.ogc.org/standards/ogcapi-features/
- NIST, Building Digitization and Semantic Interoperability: https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability
- US EPA EJSCREEN limitations and caveats: https://19january2021snapshot.epa.gov/ejscreen/limitations-and-caveats-using-ejscreen_.html
- USGS National Seismic Hazard Model: https://www.usgs.gov/programs/earthquake-hazards/science/national-seismic-hazard-maps
- NOAA Climate Mapping for Resilience and Adaptation: https://www.coast.noaa.gov/digitalcoast/tools/cmra.html

These standards and screening resources inform interface and uncertainty handling; they do not certify this project. The USGS and NOAA links are examples of US-specific screening layers, not globally portable sources. National screening layers are starting points only and must be supplemented with jurisdictional records, field surveys, professional advice, and local knowledge.
