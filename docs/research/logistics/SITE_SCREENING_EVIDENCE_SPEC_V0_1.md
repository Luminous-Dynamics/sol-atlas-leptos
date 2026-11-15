# Sol-Atlas: Site Screening and Geospatial Evidence for Regenerative Homes and Logistics Nodes

**Status:** Research specification v0.1; geospatial integration proposal  
**Date:** 2026-11-15

## 1. Thesis

Sol-Atlas should provide geographic and environmental context for choosing where homes, shared logistics hubs, drone receiving stations, EV carrier depots, charging facilities and aerial-access properties might be viable.

It must not silently become a land-title database, cadastral authority, aviation approval system, certified digital elevation model or engineering design solver. Each layer must identify its source, coordinate reference, resolution, update time, uncertainty and claim ceiling.

The key design shift is to move from a beautiful map of candidate locations to a geospatial evidence package that explains why a site is feasible, infeasible or still unknown.

## 2. Existing public repository substrate

The public Sol-Atlas repository already contains:
- Rust geographic functions for latitude/longitude conversion and distance calculations;
- parsed supply-route, regional, climate, infrastructure, emergency-shelter, health-facility and robotics-dispatch layers;
- energy and industrial infrastructure layers;
- geological and environmental event datasets;
- heritage paired-comparison machinery with explicit claim ceilings and frozen report bindings.

These are useful building blocks for contextual mapping and evidence discipline. They do not establish parcel-level site feasibility, legal land access, obstacle clearance, aviation permission or a surveyed terrain model.

## 3. Site classes

Screen at least these site types independently:
1. Regenerative detached home.
2. Shared residential logistics hub.
3. EV carrier depot and charging facility.
4. Community parcel receiving and cold-storage hub.
5. Drone receiving point / landing site.
6. Air-accessible private estate.
7. Remote rural residence and emergency access site.
8. Repair, sorting and containerization workshop.

A site may be attractive for housing while unsuitable for an aircraft facility, or vice versa. Never collapse all criteria into one unqualified score.

## 4. Required spatial evidence layers

### Terrain and physical access
- surveyed or appropriately sourced elevation model;
- slope, aspect, roughness and local relief;
- drainage and flood susceptibility;
- landslide, erosion, wildfire and other relevant hazard layers;
- road and track access, bridge/culvert capacity and seasonal access;
- emergency ground access and alternate route;
- construction logistics and crane/large-component access.

### Climate and environmental conditions
- wind speed, direction, gust distribution and seasonal variation;
- precipitation, temperature, humidity, solar resource and extreme heat;
- wildfire smoke, dust, salt exposure and coastal conditions;
- flood and water availability;
- ecosystem and habitat sensitivity;
- noise-sensitive receptors and quiet periods.

### Infrastructure
- grid connection, power quality and outage history where available;
- renewable generation potential and storage siting;
- water, wastewater and stormwater infrastructure;
- road, rail, public transport and logistics networks;
- communications coverage and emergency services;
- charging and maintenance service proximity.

### Land, rights and heritage
- cadastral parcel boundary and authoritative source;
- registered ownership or tenure evidence;
- zoning and land-use permissions;
- servitudes, easements and access rights;
- conservation and protected-area restrictions;
- heritage resources, cultural landscapes and community rights;
- indigenous/local knowledge with consent and provenance.

### Aviation and drone constraints
- airports, heliports, aerodromes and known traffic patterns;
- controlled/restricted airspace, temporary restrictions and published notices;
- obstacle data and approach/departure surface requirements;
- utility lines, masts, cranes and tall structures;
- population exposure and ground-risk zones;
- noise and wildlife impacts;
- suitable contingency/forced-landing alternatives;
- operator-specific routes, weather limits and written regulatory determinations.

Sol-Atlas should consume authoritative and current aviation data through a dedicated integration. A general geographic dataset or visually plausible map is not a basis for flight authorization.

## 5. Evidence model

Each geospatial feature should include:
- stable feature ID;
- geometry and coordinate reference system;
- source organization and exact dataset/version;
- observation/publication timestamp;
- spatial resolution and accuracy;
- vertical datum, where relevant;
- confidence/uncertainty and known missing regions;
- license and permitted reuse;
- transformation/projection history;
- evidence digest and retrieval receipt;
- claim ceiling (contextual, screening, surveyed, statutory or independently certified).

Unknown data must remain unknown. Missing flood evidence is not evidence of no flood risk. Missing obstacle data is not clear airspace. Missing ownership evidence is not permission to build.

## 6. Multi-criteria site assessment

Do not reduce a site to one weighted score without preserving hard constraints and trade-offs.

Use three output classes:
- **Reject:** a verified hard constraint makes the proposed use infeasible.
- **Conditional:** the site remains plausible but permissions, surveys, environmental assessments or infrastructure upgrades are missing.
- **Candidate:** all required screening evidence is present at the specified claim level; still not a construction or aviation permit.

Rank surviving candidates only after hard constraints are applied. Provide separate sub-scores for housing, energy, ground logistics, aviation, ecology, accessibility, heritage and lifecycle burden.

The report should show Pareto alternatives rather than hiding conflicts behind one scalar score.

## 7. Cross-system use

### Mycelix Commons / Transit
Verify property, tenure, community access and infrastructure agreements through authorized records; identify suitable shared logistics hubs and service territories; support route and facility planning; record community review and land-use disputes.

### Symthaea
Propose candidate sites and explain trade-offs; identify missing information and request further evidence; compare design requirements against site constraints; never infer permission from a predicted score.

### Symtropy
Import terrain/context into scenario scenes; simulate ground access, obstacles, logistics transfer and environmental conditions; preserve geometry and source digests; delegate FEA, CFD, hydrology, traffic and building physics to validated tools.

### AeroWeft
Evaluate aircraft-specific performance envelopes against surveyed and weather evidence; declare required landing/receiving footprints, clearances, approach volumes and charging infrastructure; never use generalized aircraft categories as a substitute for a named configuration.

## 8. South African pilot screening

A South African pilot should initially screen ground-accessible residential and community sites before assuming an air-accessible site is viable.

For each candidate:
- verify title/tenure, zoning, servitudes and local authority requirements;
- obtain a current topographic and obstacle survey at suitable resolution;
- examine local wind, rainfall, flood, fire and biodiversity conditions;
- identify community, noise and wildlife impacts;
- verify access for emergency services and conventional deliveries;
- obtain a written SACAA determination for the exact aircraft and operating concept before planning routine aerial operations.

The SACAA's published UAS information and the 2026 Civil Aviation Regulations amendments must be reconciled for the specific operation. The statement that a category is excluded from Part 101 is not a grant of permission or evidence that it is unregulated under all applicable law.

## 9. Benchmarks

- geospatial source completeness per required layer;
- feature positional and vertical accuracy against independent survey;
- source freshness and update latency;
- percentage of sites correctly rejected by known hard constraints;
- false-safe rate (target zero in frozen hard-negative cases);
- false-reject rate and uncertainty calibration;
- reproducibility of a site report from frozen layer digests;
- sensitivity to missing, stale or contradictory layers;
- ecological/noise/heritage impacts retained as distinct criteria;
- human review time and appealability;
- proportion of candidate sites with a verified alternative access and emergency plan.

Do not publish a site-feasibility claim when underlying data only supports contextual screening.

## 10. Initial implementation slices

1. Add versioned schema for a SiteEvidenceLayer and SiteScreeningReport.
2. Reuse Sol-Atlas Rust geo utilities; add explicit CRS and uncertainty metadata.
3. Build a small South African demonstration with clearly cited public layers and visible missing data. Do not fabricate parcel, airspace or obstacle layers.
4. Add hard-negative tests: known flood zone, protected habitat, missing title evidence, stale obstacle layer, blocked emergency route.
5. Produce a site-specific evidence report with separate housing, energy, ground-logistics, aviation, ecology and heritage outcomes.
6. Integrate site reports with Mycelix authorized property records only after identity/permission contracts are defined.

## 11. Non-claims

This document does not claim that Sol-Atlas currently contains complete cadastral parcel or zoning data; surveyed, construction-grade terrain; authoritative live aviation/obstacle data; validated flood/hydrology, geotechnical or fire models; landowner consent or building permission; or regulatory approval for a helistop, vertiport or UAS operation.

Its role is geographic context and evidence management, with explicit integration to authoritative sources and specialist reviewers.
