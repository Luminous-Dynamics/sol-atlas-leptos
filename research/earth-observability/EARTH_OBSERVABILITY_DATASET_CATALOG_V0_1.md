# EARTH-OBSERVABILITY-DATASET-CATALOG-001
## Candidate public and permissioned subsurface benchmark sources v0.1

**Parent:** Sol-Atlas #761 — EARTH-OBSERVABILITY-QUAL-001  
**Status:** source-discovery and acquisition planning; no data ingestion claimed  
**As-of date:** 2026-11-13

This catalogue identifies candidate sources for a reproducible Earth-observability benchmark. The entries are not automatically admitted datasets. Public visibility, a downloadable file, a source index, or a published paper does not by itself establish redistribution rights, field completeness, measurement quality, or independent ground truth.

## 1. Recommended source strategy

Do not wait for a single ideal all-in-one geoscience corpus. Build a staged corpus from complementary, well-described sources:

1. **Borehole inventory and coverage:** use the British Geological Survey (BGS) Single Onshore Borehole Index (SOBI) as a rights-clear starting point for spatial metadata, coverage, source provenance and duplicate/near-duplicate auditing.
2. **Multidisciplinary subsurface observations:** use the Netherlands Geological Survey / DINOloket and BRO data as a candidate source family for boreholes, groundwater, CPT, electrical soundings, geological logs, seismic and laboratory observations.
3. **Digital borehole geophysics:** use released NLOG or USGS GeoLog Locator records when exact release status, data format, and reuse conditions are verified.
4. **Johannesburg surface/subsurface readiness:** continue the City of Johannesburg / DWS / Council for Geoscience acquisition qualification in #561. Do not infer rights, accuracy, or completeness from a map viewer.
5. **Prospective validation:** obtain permissioned field evidence with independent checkpoints and blind reference data only after the relevant site and competent survey/engineering partners are identified.

## 2. Candidate source register

| Source | Verified discovery facts | Appropriate benchmark use | Admission blockers / limitations |
|---|---|---|---|
| **BGS Single Onshore Borehole Index (SOBI)** | BGS describes an open index of more than one million borehole, shaft and well records across Great Britain. It is GIS point data and is free for commercial, research and public use under the Open Government Licence. BGS also provides access to scanned individual borehole records. | Spatial coverage, inventory quality, point metadata, source lineage, duplicate detection, borehole density and survey-planning priors. | SOBI is an index, not proof that every borehole has a complete digital log or laboratory dataset. Admit each linked record/document separately with its own source identity, rights, date, coordinates and quality. |
| **Geological Survey of the Netherlands (GDN), DINOloket** | GDN reports approximately 500,000 drillings, 143,000 probings and 130 million groundwater measurements. DINOloket allows anyone to view, request and download subsurface data and models free of charge. The DINO database includes borehole, groundwater, CPT, vertical electrical sounding, geological/chemical/mechanical analyses, borehole logs and seismic data. | Multimodal profile reconstruction, borehole/CPT/groundwater integration, temporal groundwater baselines, geological modelling and source conflict tests. | “Free of charge” is not a universal reuse licence. Verify exact dataset terms, attribution, export rules, data dictionaries, spatial/vertical reference, dates, sampling and quality for each record family. |
| **BROloket (Dutch Key Registry of the Subsurface)** | GDN describes BRO as a portal to access and download online subsurface data and models supplied under the Dutch Key Registry of the Subsurface Act. | Structured, standardized subsurface information and reproducible administrative data exchange. | Verify the exact BRO object type, schema/profile version, completeness, effective date and permitted downstream publication. BRO and DINOloket are related but not interchangeable data products. |
| **NLOG (Netherlands)** | NLOG exposes energy/mineral extraction data. Its borehole pages list administrative data, reports/logs, digital LIS/LAS logs, samples, core measurements, lithostratigraphy and deviation data. Well logs and reports become available after a statutory five-year confidentiality period; most seismic data follow a five-year period, while multi-client seismic follows a ten-year period. Complete datasets can be downloaded only for a restricted set of data types. | Digital downhole geophysics, deviation-aware borehole geometry, lithostratigraphic intervals, core/log alignment and well-data interoperability. | Verify per-well/seismic confidentiality expiry, exact downloadable data types, license, LAS mnemonic/unit dictionary, depth reference, datum and any operator-specific restrictions. |
| **USGS GeoLog Locator** | USGS describes more than 12,000 digital borehole-geophysical logs at over 2,500 locations. The archive serves logs approved for public release; where possible, logs are exported in Log ASCII Standard (LAS) v2.0. | LAS parsing, borehole geophysical log provenance, unit/mnemonic handling, depth intervals and observation-vs-interpretation semantics. | The archive is USGS-centric and approved-release-scoped, not a globally complete inventory. Verify each record's location, release status, acquisition method, calibration and rights. |
| **Open Well Data catalogue** | Its 2026-11-10 catalogue build lists 507 public sources across 79 countries/territories and 8,045 channel mnemonics, including wells, logs, drilling, production, cores, seismic and CO2 storage. It reports 139 sources whose licence allows copies; its full catalogue check was explicitly not complete. The index lists three South African sources, none marked copy-allowed at that build date. | Discovery index for finding candidate national surveys, regulators, core archives and data repositories; country-specific source inventory benchmarking and license discovery. | As of 2026-11-13, the site warns that Zenodo dataset downloads are temporarily unavailable during an account review. The catalogue is an index, not the underlying dataset; read each source's own license and current access status. Its count and licence labels are dated catalogue metadata, not a grant of reuse rights. |
| **University of Utrecht / Netherlands subsurface data publication** | A public data-description record describes 10,599 2D seismic lines, 280 3D seismic cubes, 6,590 wells and 4,752 composite well logs, with seismic and well datasets in platform-independent formats alongside a Petrel project. | Candidate high-value well/seismic benchmark for cross-modal geologic interpretation. | The exact current download endpoints, version, license/redistribution conditions, data dictionary and completeness must be independently checked before admission. Treat the counts as the publication's description, not a current live inventory. |
| **City of Johannesburg 2023 aerial products** | The City states 15 cm orthophotos, minimum 4 points/m² LiDAR and 25 cm derived contours over approximately 1,644 km²; some products are available on request from CGIS Projects. | Surface terrain, urban morphology, drainage context, surface change and a South African acquisition-readiness pilot. | Exact tiles, classification, accuracy report, CRS, vertical datum, survey control, tile timestamps, licensing and reuse permission remain unqualified. A point-density figure alone does not establish a USGS quality level. |
| **DWS National Groundwater Archive / borehole distribution** | DWS describes more than 294,500 geosites in the NGA and publishes a March 2026 stored-borehole distribution map with 283,030 records; the map warns that it is not a complete borehole inventory or a groundwater-potential map. | Groundwater-source discovery, inventory coverage, metadata audit, site/time context and South African source-qualification work. | The NGA is point-record-oriented; access/export uses registration. Verify export fields, source lineage, permission, coordinates, vertical datum, well construction, screen intervals, measurement dates and observation semantics. |
| **Council for Geoscience (CGS)** | CGS provides a geological information portal and map/data discovery entry point. | Regional geology, map-sheet and geological-unit reference, published reports and South African validation context. | Identify the exact layer/report, edition, native scale, coordinate/vertical reference, access route, license, reuse terms and field-validation provenance for each asset. |

## 3. Why the BGS + Netherlands + South Africa sequence is useful

These sources cover different parts of the measurement problem. The Open Well Data catalogue also quantifies an important access asymmetry: as of its 2026-11-10 build, the United States, Canada and Australia account for 61% of single-country catalogue sources, while South Africa has three indexed sources and none is marked copy-allowed. This does not prove that no South African source is reusable; it shows that permission and provenance discovery must be a first-class workstream rather than assuming a globally open corpus.

- BGS SOBI offers a large, clearly licensed point index for testing source provenance, spatial completeness and inventory semantics before attempting complicated geological inference.
- DINOloket/BRO offers a much richer combination of borehole, probing, groundwater, electrical, seismic and laboratory observation types, making it a strong candidate for multidisciplinary evidence integration.
- NLOG and USGS GeoLog Locator provide digital downhole log candidates for testing depth intervals, LAS units/mnemonics, deviation surveys and geophysical trace provenance.
- Johannesburg supplies the local surface and groundwater context needed to test whether the architecture works with a South African source landscape and its access/rights constraints.

No one source should be treated as the ground-truth authority for all others. A borehole log, a seismic interpretation, a geological map and a groundwater time series observe different quantities at different supports and dates.

## 4. Dataset admission checklist

Before any source becomes a benchmark input, record:

- canonical publisher, owner and source ID;
- exact product / layer / well / survey / record identifier;
- exact download URL, API query or request mechanism;
- access status and reuse licence separately;
- attribution, derivative-work, redistribution and commercial-use terms;
- source edition/version, publication date, observation date, effective time and retrieval time;
- original file digest and file size;
- schema, field dictionary, units, null/missing conventions and coordinate axis order;
- horizontal CRS, vertical datum, height type, coordinate epoch and transform requirements;
- spatial coverage, native scale/resolution, acquisition geometry and temporal support;
- instrument, calibration, sampling, quality flags and known limitations where available;
- data-owner contact and correction mechanism;
- privacy, sensitive infrastructure, land-access and cultural/ecological restrictions;
- whether independent validation truth is available and how it is kept separate.

Admission states must distinguish VerifiedPublic, PublicWithConditions, RequestRequired, Restricted, Unavailable and Unknown. Do not turn an inaccessible source into a missing-value fill or treat a public viewer as a redistribution permission.

## 5. Proposed benchmark data ladder

### D0 — Open source registry benchmark
Use BGS SOBI and the Open Well Data catalogue to test source discovery, rights-state handling, freshness, stable identifiers, duplicate detection and spatial inventory coverage. Do not claim geological reconstruction performance at this level.

### D1 — Borehole geometry and log benchmark
Select a rights-qualified subset of BGS scans, NLOG digital LAS records and/or USGS GeoLog Locator data. Test trajectory/depth semantics, units, intervals, LAS mnemonic interpretation, source provenance and withheld independent wells.

### D2 — Multimodal geological reconstruction benchmark
Select a permissioned DINOloket/BRO subset containing boreholes plus one or more complementary observations (CPT, electrical sounding, seismic, laboratory or groundwater data). Define one target variable and one independent reference method before model training. Compare borehole-only, conventional interpolation/kriging, modality-specific interpretation and multimodal fusion under spatially blocked holdout.

### D3 — Johannesburg pilot
Qualify actual City orthophoto/LiDAR products and DWS/CGS candidates. Start with a coverage/rights/quality atlas. Do not infer site suitability, subsurface safety, absence of buried utilities or groundwater potential from incomplete coverage.

### D4 — Prospective site validation
Only after a concrete investigation objective, permissions and an independent field team exist, preregister instrument, calibration, control, checkpoint, borehole and adjudication procedures. Keep independent checkpoints and withheld boreholes out of model fitting and acquisition planning.

## 6. Evaluation cautions learned from current research

- The October 2026 borehole + surface-wave probabilistic fusion paper (Engineering Geology 109128) reports a bounded 26% local false-positive-area reduction under inverse-CV weighting, but its fusion boreholes cannot also serve as independent field truth; its data/code are available from the corresponding author on reasonable request. Reproduction must source new independent validation evidence.
- The September 2026 TBM continual-learning study (IJRMMS 106610) reports a large static-to-cross-line transfer drop and recovery through updating; its geological labels are operational proxy states, not strict lithological ground truth. Do not compare its accuracy directly to a lithology classifier or geological-boundary reconstruction.
- Published numerical results, source catalog counts and open data portals do not substitute for a common frozen benchmark with compatible target definitions and blind independent truth.

## 7. References

- BGS SOBI: https://www.bgs.ac.uk/datasets/boreholes-index/
- BGS Open Government Licence: https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/
- Geological Survey of the Netherlands subsurface portals: https://www.geologischedienst.nl/en/mapviewers-subsurface/
- DINOloket: https://www.dinoloket.nl/en/about-us
- NLOG boreholes: https://www.nlog.nl/en/boreholes
- USGS GeoLog Locator: https://webapps.usgs.gov/geologlocator/
- Open Well Data: https://openwelldata.org/
- University of Utrecht Netherlands subsurface data description: https://geo.public.data.uu.nl/vault-netherlands-subsurface-database/research-netherlands-subsurface-database%5B1683014156%5D/original/Data_description.pdf
- City of Johannesburg CGIS: https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx
- DWS groundwater data: https://www.dws.gov.za/Groundwater/data.aspx
- DWS borehole distribution map: https://www.dws.gov.za/groundwater/data/boreholedist.aspx
- Council for Geoscience data portal: https://maps.geoscience.org.za/
