# Megastructure Design Context v0.1

**Status:** read-only projection and data-quality contract; not a site-selection or approval system  
**Work item:** [Sol-Atlas #883 — MEGASTRUCTURE-DESIGN-CONTEXT-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/883)  
**Design-quality consumer:** [Symtropy #5384 / PR #5418](https://github.com/Luminous-Dynamics/symtropy/pull/5418)  
**Spaceport geography owner:** [Sol-Atlas #393 / PR #394](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/394)

## 1. Objective

A megastructure is not an isolated CAD object. It occupies, affects, depends on and is inherited within a real place and a wider planetary/orbital system.

Sol-Atlas should project evidence about that context into a design review without becoming:
- a second GIS or canonical evidence registry;
- an orbital mechanics or structural solver;
- a site-selection or permitting authority;
- an environmental-impact assessor;
- a customary-rights or community-consent adjudicator.

The projection is read-only. It makes data lineage, coverage, resolution, uncertainty, access rights, source conflict and claim ceilings visible.

## 2. Governing rule

> A design review must show the candidate in the place and system it will inhabit, using source-qualified context, while never turning a map layer into a truth flag, permit, consent, safety finding or environmental approval.

An empty map area is not proof that no ecological, cultural, rights, infrastructure or hazard feature exists. Missing coverage and no observed feature are different states.

A layer marked official or administrative is not interchangeable with a measurement. A proposed infrastructure feature is not an operating asset. A modelled hazard is not an observation. Synthetic fixtures must never be presented as real geography.

## 3. DesignContextProjectionV1

The projection consumes immutable references to a design candidate and the exact evidence-bearing design package.

### Candidate identity

- stable candidate ID and design-generation ID;
- geometry, footprint, route, deployment envelope or orbital-context artifact digest;
- intended use, service, mission and design boundary;
- relevant review-pack view identities;
- source design-system and configuration identity;
- exact coordinate reference system, vertical datum, epoch and transform identity;
- requested context-layer set and declared spatial/temporal domain.

A geometry, route, orbital envelope, coordinate transform or design-generation change invalidates previous projection receipts for the affected domain.

### Context layer identity

Every context layer must identify:

- stable layer ID, source dataset and source-record identity;
- source publisher/authority and original source URL or resolvable reference;
- publication date, effective date, observation date and retrieval date as separate fields where applicable;
- license, terms of use, attribution requirements and access classification;
- data type: observed, measured, administrative, modelled, inferred, proposed, synthetic or unknown;
- geometry type, horizontal CRS, vertical datum, resolution/scale and coordinate transform;
- positional, temporal, thematic and model uncertainty where known;
- source-health state, parser/validation state and freshness disposition;
- known exclusions, sampling bias, missing coverage and applicability limits;
- transformation / processing lineage and exact projection version;
- claim ceiling and the authority responsible for any subsequent determination.

Do not invent precision when the source does not supply it. Do not infer a confidence probability from an unqualified quality label.

### Feature-level lineage

Where the source provides individual features, preserve a stable source feature identifier and the original feature geometry or immutable source reference. Aggregations, simplifications, reprojections and privacy transformations must be represented as derived data with their own identity. Never overwrite the source semantics with a rendered feature.

## 4. Context families

### 4.1 Ecology and conservation

Potential layers include protected areas, Ramsar wetlands, Key Biodiversity Areas, habitat, ecosystem connectivity and species observations, subject to licensing, sensitivity and scope.

Requirements:
- retain the boundary authority and its positional accuracy;
- distinguish designated area from modelled habitat and individual observation;
- retain seasonal / temporal validity where relevant;
- do not infer absence from an empty observation layer;
- support avoidance-first review for sensitive ecosystems;
- protect exact sensitive species and community locations where disclosure could cause harm.

### 4.2 Climate, natural hazards and physical geography

Potential layers include flood extent, heat, wildfire, coastal erosion, storm exposure, sea-level scenarios, slope, geology and geotechnical proxies.

Requirements:
- retain model version, scenario, emissions / return-period assumptions where applicable, epoch, grid resolution and vertical datum;
- distinguish observed events, administrative hazard zones and model projections;
- never downsample a high-resolution source without recording the transformation;
- show conflicting datasets side by side rather than silently choosing one;
- keep geotechnical investigation and site survey requirements explicit.

### 4.3 Community, access and rights context

Potential layers include settlements, public access, administrative boundaries, customary-tenure evidence, access routes, social infrastructure and affected-community context where lawful and ethically appropriate.

Requirements:
- distinguish administrative boundaries from customary tenure and lived use;
- preserve source authority and disagreement;
- respect license, privacy, consent and disclosure restrictions;
- do not expose sensitive household, sacred-site or community information without authorization;
- do not infer consent from mapped proximity or from a majority preference score;
- a map layer can identify a question for review, not adjudicate a right.

### 4.4 Heritage and cultural landscape

Represent legally designated heritage and community-identified cultural landscape evidence as distinct source types. Preserve authority, date, scope, sensitivity and access controls. Absence of a designation is not evidence that no heritage interest exists.

### 4.5 Infrastructure and services

Potential layers include ports, rail, roads, power generation/transmission, water, communications, tracking stations, emergency services, logistics hubs and industrial capacity.

Requirements:
- distinguish operating, under-construction, planned, proposed, retired and unknown states;
- capacity values require source, unit, time basis, effective date and operational conditions;
- network connectivity does not prove right-of-way, access permission or available capacity;
- service dependencies and common-cause corridors should remain visible;
- preserve source identity where routes have been generalized or privacy-protected.

### 4.6 Night-sky, lighting and visual context

Potential layers include measured night-light observations, official observatory protection zones, source-defined dark-sky designations and project-specific lighting studies.

Requirements:
- show sensor, acquisition date, processing, resolution and observation conditions;
- distinguish measured radiance from modelled spill and from policy boundaries;
- do not invent an unobstructed night-sky view or universal visibility score;
- evaluate illumination, glare, spill and astronomical context within a declared measurement method.

### 4.7 Orbital and off-world context

Consume versioned orbital access envelopes, mission trajectories, debris/eol records and off-world logistics only from their actual owners.

Sol-Atlas must not:
- derive orbital trajectories from illustrative display coordinates;
- treat static circular guides as ephemerides;
- turn a rendered orbital envelope into a collision-risk or station-keeping assessment;
- substitute terrestrial GIS semantics for inertial/body-fixed frame semantics.

Every orbital feature needs epoch, frame, transform/ephemeris identity, covariance/uncertainty and model claim ceiling from the authoritative producer.

## 5. Projection and invalidation lifecycle

1. Resolve the exact candidate and geometry/configuration identity.
2. Validate the requested spatial/time domain and coordinate semantics.
3. Resolve source references and licensing/access restrictions.
4. check source data health, freshness, completeness and known omissions.
5. Transform source data without discarding original source identities.
6. Produce a projection receipt listing accepted, stale, partial, unavailable, restricted and rejected layers.
7. Bind each rendered overlay to the same candidate generation and projection receipt.
8. Invalidate the projection when geometry, coordinate transforms, requested domain or relevant source versions change.

A successful projection means only that the requested layers were projected according to their declared contracts. It does not mean that the site is safe, suitable, legal, ecologically acceptable or socially approved.

## 6. Required UI behavior

- Show source, date, scale/resolution, uncertainty and data-health state for each active layer.
- Make missing/stale/partial/restricted coverage conspicuous.
- Distinguish unknown, unavailable, valid-empty and malformed states.
- Never render an invalid or stale layer as an authoritative overlay.
- Preserve conflicting source records and let reviewers inspect them.
- Support side-by-side alternatives using matched design identities and layer versions.
- Preserve dissent, source conflict and minority evidence in exports.
- Provide accessible labels, textures, keyboard-accessible controls and non-color encodings.
- Keep visualization confidence distinct from physical or legal authority.
- Exports must bind exact candidate, view, source-layer, transform and projection identities.

## 7. Synthetic adversarial benchmark

The companion corpus `megastructure-design-context-cases-v0.1.json` contains eight synthetic cases:

1. stale projection after the design footprint changes;
2. proposed footprint near a conservation boundary with uncertain alignment;
3. no mapped feature where source coverage is missing;
4. two hazard sources with different epochs, resolutions and scenarios;
5. sensitive community data whose license permits internal review but not public display;
6. attractive rendering that hides a critical service corridor;
7. illustrative off-world positions presented as if they were epoch-specific ephemerides;
8. synthetic layer mislabeled as measured or official.

Acceptance:
- 100% stale design identities rejected or visibly marked stale;
- missing coverage never treated as no impact;
- unauthorized sensitive data never disclosed;
- unsupported data never promoted to observed/official status;
- source lineage and claim ceiling survive projection and export;
- no output implies consent, site approval, environmental clearance or orbital safety.

All fixture values and features are synthetic. They are policy/conformance tests, not real geography or a site assessment.

## 8. Industry and assurance anchors

The design-quality consumer should be compatible with NASA's human-centred design practice, which combines CAD, VR, task simulation, mockups, prototypes and human-in-the-loop review. Sol-Atlas supplies the place/evidence context; it does not replace professional architecture or human-factors review.

For human-rated space systems, NASA-STD-3001 Volume 2 Revision F (2026-07-14) is an active reference for applicable human-factors, habitability and environmental-health requirements. Applicability and tailoring belong to the programme's responsible technical authority.

For unmanned spacecraft in Earth orbit, ISO 23312:2026 defines debris-mitigation requirements and recommendations within its scope. It is not a universal orbital-ring, habitat or terrestrial site-planning standard.

NASA-STD-7009B and its implementation handbook are relevant model-and-simulation assurance anchors. A map renderer must not imply that a modelled source layer has been physically validated.

## 9. Ownership boundaries

- **Sol-Atlas:** read-only spatial projection, data-health presentation and source lineage.
- **Symtropy:** design-quality policy, engineering trade-offs and physical qualification.
- **Symthaea:** candidate generation and decision-relevant evidence selection; no self-certification.
- **Mycelix / authorized rights infrastructure:** provenance, permissions and authorized decisions; no automatic physical or legal truth.
- **External data authorities and professional reviewers:** remain authoritative within their own mandates and applicability.

## 10. Claim ceiling

This contract and synthetic corpus define a read-only evidence projection. They do not select a real site, establish customary consent, certify legal rights, assess environmental impact, authorize a project, validate orbital mechanics or establish a megastructure's beauty, safety or feasibility.
