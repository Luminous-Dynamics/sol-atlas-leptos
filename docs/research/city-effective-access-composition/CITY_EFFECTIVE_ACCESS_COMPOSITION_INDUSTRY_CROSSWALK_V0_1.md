# City Effective Access Composition — industry and benchmark crosswalk v0.1

This document is a bounded comparison and benchmark plan. It is not a certification, standards-conformity declaration, claim of superiority, or assertion that Luminous has matched real-world city datasets.

## What established systems already do well

| Comparator | Established capability | Boundary relevant to this work | Our benchmark relationship |
|---|---|---|---|
| Conveyal R5 | Multimodal routing engine for one-to-many and many-to-many travel-time and cumulative-opportunity analyses across departure windows | The documented default variability method assumes on-time transit, full schedule information, and unimpeded boarding; reliability/crowding/accessibility evidence must be represented deliberately | Treat R5 as an external routing reference. Match exact GTFS/OSM/network snapshots, modes, departure windows, time percentiles, access/egress and cutoffs. Report differences, do not replace R5 routing without a measured gap |
| OpenTripPlanner 2 | Production multimodal itinerary planning for scheduled transit, walking, cycling and mobility services; builds on GTFS and OpenStreetMap and can incorporate real-time service updates | Journey planning does not establish a service's available capacity, price/eligibility for a beneficiary group, commissioning state, or actual service delivered | Use OTP for route/itinerary parity and interruption scenarios where the same data and assumptions exist |
| r5r | R interface to R5 accessibility calculations, including cumulative opportunities and several distance-decay functions | Outputs depend on the impedance, percentile, cutoff, decay function and data assumptions | Reproduce step/cumulative and selected decay functions on pinned inputs; expose sensitivity rather than choose hidden weights |
| UN SDG 11.2.1 | Standardized indicator for population with convenient access to public transport, disaggregated by sex, age and disability where data permit | The core methodology concerns access to officially recognized stops—typically 500 m network walking distance for low-capacity systems and 1 km for high-capacity systems. Stop access is not service capacity, affordability, reliability, or delivered trips | Reproduce the official indicator as a separately labelled output. Do not silently redefine it as broad access to all urban services |
| ISO 37120 | Indicators and methods for city services and quality-of-life performance that enable comparable and verifiable measurement | It is an indicator framework, not an integrated engineering evidence graph or an assurance receipt for every underlying source. ISO's 2018 edition is identified as due for revision; a 2026 working-draft registration is separately visible | Map every relevant indicator and definition to its source, time, unit and owner. Do not claim ISO conformity from a local crosswalk |
| ISO 37123 | Indicators for resilient cities, intended to be used with ISO 37120 and other strategic frameworks | A resilience-indicator set does not prove an individual facility is physically safe, continuously available or serving its declared population | Map service-continuity and recovery metrics to applicable indicators, but retain the underlying event and owner evidence |

Official and primary references:

- Conveyal R5 repository: https://github.com/conveyal/r5
- Conveyal analysis methodology: https://docs.conveyal.com/analysis/methodology
- OpenTripPlanner project: https://github.com/opentripplanner/opentripplanner
- OpenTripPlanner documentation: https://docs.opentripplanner.org/en/latest/
- UN SDG indicator 11.2.1 metadata: https://unstats.un.org/sdgs/metadata/files/Metadata-11-02-01.pdf
- ISO 37120: https://www.iso.org/standard/68498.html
- ISO 37123: https://www.iso.org/standard/70428.html

## The specific gap this composition benchmark targets

Existing routing, urban-indicator and building-performance methods answer different questions. The composition must not conflate them:

1. **Restoration:** did the responsible service owner qualify the service as restored for the exact scope and stable interval?
2. **Potential opportunity:** can the declared group reach a matching service, at the declared time, within its travel, affordability, eligibility and universal-access envelope, with sufficient *potential* capacity?
3. **Observed operating condition:** did the owner-qualified in-use observation meet the declared service floor during the bound observation interval?
4. **Beneficiary delivery:** is there a separate aggregate, privacy-safe and rights-permitted receipt showing what the declared group actually received against the correct demand denominator?
5. **Lived experience:** what do consented users report, without turning non-response or consent denial into a negative outcome or a technical service measurement?

The proposed contribution is therefore a versioned, time-qualified, group-aware **composition and abstention contract**. It is not a new routing algorithm. The key negative control is that reachable capacity must never be promoted into delivered service.

## Benchmark programme

### Tier 0 — semantic correctness

- 100% exact agreement on all frozen synthetic known-answer cases.
- Zero false-positive restoration, potential-access, operational-floor or delivery claims.
- Zero cases where NotProvided, Unknown, stale, revoked, non-comparable or rights-invalid evidence becomes a favorable state.
- Zero promotion from potential capacity to actual delivered quantity.
- Zero denominator, group, population-frame, functional-unit, service-revision or time-window substitution.
- Aggregate delivery cannot hide a declared subgroup below its mandatory floor.
- Deterministic output under input permutation; duplicate identities and contradictory records fail closed.

### Tier 1 — external routing parity

Run the same network and timetable bundle through CityMorph, R5 and/or OTP. Pin:

- exact OSM/network and GTFS/GTFS-RT snapshots;
- service calendar and date;
- departure windows and time percentiles;
- origin and destination/stop definitions;
- walking/cycling/transit mode settings;
- slope, stairs, step-free and barrier assumptions;
- transfer, wait and access/egress settings.

For finite synthetic graphs, target 100% exact route-existence and known shortest-path agreement. For production routing engines, publish route-set differences and travel-time distributions; use a declared tolerance only after confirming the engines have the same model semantics. Do not hide differences behind a correlation score.

### Tier 2 — effective opportunity and service semantics

Reproduce CityMorph's group outputs, then verify separate evidence for:

- opening/operating window;
- current capacity and capacity units;
- affordability/price and price units;
- eligibility rules;
- step-free and other relevant accessibility constraints;
- scope and population-frame matching;
- unknown, stale and incomplete source evidence.

Target zero favorable outputs when any required dimension is unknown or invalid. A mapped point or a route is never sufficient proof of a usable opportunity.

### Tier 3 — actual delivery and distribution

Where the responsible operator can provide aggregate privacy-safe evidence, report separately:

- group-level delivered units / group-level demand;
- aggregate delivered units / aggregate demand;
- demand-frame coverage and denominator provenance;
- worst-served required group and any failed mandatory group;
- time period and service revision;
- missingness and uncertainty.

Never let high aggregate delivery compensate for a required subgroup's shortfall. Do not ingest individual recipient identities for this composition contract.

### Tier 4 — resilience and longitudinal recovery

For each service class, report a vector rather than one score:

- time to first adequate service;
- time to sustained adequacy under the service owner's stability rule;
- longest and total interval below the floor;
- constrained, inaccessible and unknown duration;
- recurrence/rebound after recovery;
- unmet units/hours only where a valid unit and denominator are supplied;
- delivery coverage over repeated periods where the owner supplies a comparable record.

Use the restoration owner's actual time cuts and do not infer probabilities from a small synthetic corpus.

### Tier 5 — scale and runtime

Only after semantic correctness is established, benchmark:

- 10k / 100k / 1M group-service combinations;
- cold and warm execution;
- source/corpus load and schema-validation time;
- evaluation throughput and p50/p95 latency;
- peak memory;
- incremental recomputation after one source revision changes;
- deterministic receipt hash under repeated runs.

Record hardware, compiler, build flags, corpus digest and commit SHA. Performance targets should be set from a measured baseline; they must never weaken abstention, privacy, source-rights or group-level gates.

## Evidence required before claiming an advantage over SOTA

A credible claim requires matched public/reference datasets, a pinned scenario, exact input semantics, independent known answers, external engine outputs, discrepancy analysis and a reproducible exact-head receipt. A higher single score or faster runtime is insufficient if it relies on different assumptions or silently treats missing evidence as good.

The defensible initial claim is narrower: the reference implementation demonstrates synthetic composition mechanics and explicit non-substitution boundaries. Real Johannesburg validity requires current local network, service, population and operating evidence, lawful rights, and consent-appropriate field validation.
