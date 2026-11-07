# Vector exposure atlas v0.1

Status: rights-aware spatial evidence specification; synthetic-only initial corpus.

Issue: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/618

## Purpose

Provide public-health and community partners with a geospatial evidence layer for environmental covariates, surveillance coverage and intervention follow-up. The atlas should help identify where a new measurement could change a decision. It is not a disease prediction service and not a map of infected people, households or vulnerable communities.

## Existing owners

- Sol-Atlas CITY-HEAT-RETROFIT-ATLAS-001 #593 — Johannesburg heat and retrofit context.
- Sol-Atlas CITY-MORPH-SAFE-001 #591 — resident and neighbourhood safety.
- Sol-Atlas CITY-NEIGHBOURHOOD-REFERENCE-001 #596 — integrated district.
- Sol-Atlas CITY-MORPH-ROBUST-001 #563 — service continuity under disruption.
- Symtropy VECTOR-HABITAT-001 #4095 — coupled housing/vector/thermal design benchmark.
- Mycelix OPS-INTEL-TEST-001 #2942 — evidence frontier and authority separation.

## Policy and scientific baseline

- WHO Global Vector Control Response 2017–2030: https://www.who.int/publications/i/item/9789241512978
- WHO Malaria Surveillance, Monitoring and Evaluation reference manual, 2nd edition (2025): https://www.who.int/publications/i/item/9789240112476
- WHO Guidelines for Malaria (10 September 2026): https://www.who.int/publications/i/item/guidelines-for-malaria/
- South Africa National Vector Control Strategy 2023–2027: https://www.health.gov.za/wp-content/uploads/2024/07/National-Vector-Control-Strategy-2023-2027-signed.pdf

The national strategy identifies surveillance requirements including vector receptivity, density/seasonality, insecticide resistance, intervention coverage/quality and surveillance gaps. The atlas must preserve source resolution and sampling effort so it does not overstate local certainty.

## Paradigm shift

Replace a static, overconfident hotspot map with an evidence-and-coverage atlas:

- environmental suitability is not observed vector presence;
- observed vector presence is not human exposure;
- human exposure is not infection or disease incidence;
- a missing sample is not a negative sample;
- stale observations are not current;
- ward-level information is not household-level truth;
- a dense cluster of reports may reflect sampling density rather than actual prevalence;
- maps should expose uncertainty and surveillance blind spots, not stigmatize communities.

## Observation contract

Each source layer or observation must retain:
- source publisher and stable dataset identity;
- exact version, source digest, licence and permitted-use statement;
- retrieval time, observation time, effective interval and expiry;
- coordinate reference system, transformation lineage and spatial precision;
- temporal resolution, season, sampling effort and observation denominator;
- vector species and taxonomic confidence, where applicable;
- method, units, equipment/calibration and quality flags;
- coverage, missingness and known sampling bias;
- classification as observed, reported, modelled, inferred or missing;
- sensitivity class and minimum publication/generalization threshold.

## Candidate layers

Only after rights and quality review:
- weather, rainfall and climate covariates;
- drainage, water service and standing-water inspection records;
- land cover / built-form context;
- vector surveillance provided by authorized health or research custodians;
- intervention installation and maintenance inspections;
- aggregated health outcomes supplied by a lawful custodian.

A visible web map is not proof that its layers are downloadable, reusable, current or sufficiently accurate. Do not scrape or redistribute restricted records.

## Privacy and safety

Default to local/federated custody and coarse public views. Do not expose exact household disease observations, victim locations, private property conditions, sensitive ecological sites or small-cell counts that permit re-identification. No individual risk ranking, predictive policing, automated deployment or enforcement use.

## V0 synthetic geography

Freeze:
- two seasonal conditions;
- one rainfall / heat event;
- multiple synthetic drainage catchments;
- unequal sampling effort and explicit coverage gaps;
- mixed temporal and spatial resolutions;
- stale and conflicting observations;
- two species with separate identity and evidence histories;
- before/after intervention observations;
- wrong-CRS and displaced-coordinate mutations;
- protected household samples that must be generalized;
- a sparse sample that must not become a population estimate.

## Metrics and proposed gates

- 100% preservation of source, time, CRS, resolution, coverage, species and sampling metadata.
- Zero promotion of environmental suitability to vector presence or of vector presence to disease incidence.
- Zero household-level re-identification in public projections under the frozen threat model.
- Invalid CRS, stale source and licence-incompatible inputs are rejected or quarantined.
- Uncertainty and coverage completeness reported beside counts and model outputs.
- Calibration is stratified by source, species, season and spatial scale.
- Every generated map layer binds the exact source and processing digests.
- An independent fixture verifier reproduces the known-answer admission and disclosure decisions.

## Non-goals

No clinical prediction, person-level or household-level risk score, disease hotspot publication, predictive policing, insecticide recommendation, biological-control deployment, automated treatment or public-health authority substitution.
