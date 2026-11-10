# South African GMD Data Access and Evidence Matrix v0.1

**Date:** 2026-11-10  
**Purpose:** Identify usable South African and external comparator sources for a rights-aware GIC / geomagnetic-resilience pilot.  
**Status:** Desk research only. No private portal registration, commercial agreement, direct utility data transfer or raw restricted data access has been completed.

## 1. Executive finding

South Africa has a meaningful historical foundation in geomagnetically induced current research. The first practical step is not to deploy sensors blindly; it is to inventory the existing SANSA, utility, university and legacy Sunburst records, their data rights, sensor metadata and event coverage.

The most actionable first-party discovery path is **SANSA SANDIMS**. SANSA describes this as the repository for geomagnetic, ionospheric and magnetospheric measurements from its instrumentation network. It provides instrument/site/sampling metadata and date-filtered bundles. SANSA states that academic access is free after registration, while commercial use requires a contract. This is an important distinction for Luminous Dynamics' intended commercial work.

SANDIMS is not evidence that transformer-neutral GIC, utility topology, transformer design data or MT impedance are all present or freely downloadable. Those require separate dataset-level and owner-level qualification.

## 2. Source register

| Source / owner | Evidence class | Known scope / access route | Rights / access status | What it can establish | What it cannot establish on its own |
|---|---|---|---|---|---|
| **SANSA SANDIMS** — https://sandims.sansa.org.za/ | Geomagnetic, ionospheric and magnetospheric instrument data + metadata | Search by instrument site, type and sampling interval; date-filtered data bundles via temporary FTP login. Network extends across South Africa, Namibia, Gough and Marion islands and SANAE IV. Metadata includes GPS location, manufacturer, PI and sampling interval. | **Academic:** free after registration. **Commercial:** SANSA contract required. Contact listed by SANSA: dap@sansa.org.za. Dataset-specific terms and fields must be verified. | Instrument locations, cadence, source metadata and some local space-environment observation records. | Transformer-neutral GIC, complete grid topology, transformer parameters, MT impedance, asset susceptibility or permission for commercial derivatives unless explicitly present and authorised. |
| **Grassridge / EPRI Sunburst historical records** | Historical utility GIC measurements | Published work documents monitoring beginning in 1998 and use of 29–31 October 2003 Grassridge GIC data, with 2-second measurements averaged to one-minute for published modelling. | **Published evidence:** verified. Raw archive/custodian and current access rights: **not established**. Ask the relevant utility, historical EPRI Sunburst custodian and authors. | Historical measured GIC for one site/event and comparison against geomagnetic observations and a layered conductivity model. | Current network risk, modern transformer configuration or national spatial transfer. |
| **Hermanus geomagnetic observatory** | Ground magnetic field | Used in historical Grassridge/Hermanus studies; should be inventoried through SANDIMS and any observatory-specific distribution path. | SANDIMS academic/commercial terms apply if distributed there; exact products and licensing not yet checked. | Local geomagnetic input for event reconstruction and local source-field characterisation. | Transformer GIC or site-specific GIC without Earth response, network and transformer modelling. |
| **Hartebeesthoek geomagnetic observations** | Ground magnetic field | Used alongside Grassridge/Matimba in published 2003/2015 southern-hemisphere proxy validation. Check station inventory and record availability in SANDIMS. | Dataset-level access and rights not yet checked. | Additional regional magnetic observations and source-station transfer studies. | Direct transformer-neutral GIC; proxy indices are not ground-truth current measurements. |
| **Botrivier 400-kV line / SANSA 2015–16** | Under-line magnetometer evidence | SANSA's 2015/16 Annual Report documents magnetometers installed under/near a 400-kV line around Botrivier and records significant GIC-associated magnetic response during the 17 March 2015 storm. | Agency report is public. Raw sensor time series, calibration, current installation status and transfer rights: **not established**. | Confirms historical instrumentation activity and a potential local line-current/field measurement pathway. | Do not equate an under-line magnetic measurement with transformer neutral current unless the measurement definition and inversion are established. |
| **Matimba power station / utility GIC records** | Historical transformer/site GIC evidence | Used with Grassridge records in the 2022 southern-hemisphere GIC-proxy validation study. The authors note good-quality GIC records at the two sites did not overlap in time. | Published research evidence verified; raw time series, current owner, data-rights and metadata access not established. | Historical site-level validation for a second South African region / network location. | Simultaneous cross-site validation if event windows do not overlap; present network vulnerability. |
| **South African MT / conductivity records** | Earth-response / geophysical impedance | The 2008 Grassridge/Hermanus work used a multilayer conductivity model; SANSA 2015 work also reports Hermanus MT impedance. University and geophysical custodians must be identified for the raw MT sites, transfer functions and uncertainty. | Model/paper existence is verified; current raw site inventory, frequency coverage, licence and redistribution rights not established. | Local Earth-response functions for geoelectric field modelling where spatial/frequency coverage is valid. | National 3-D conductivity coverage or a universal conductivity model. |
| **SANSA 2015 field / E-field modelling** | Published model-to-measurement comparison | SANSA geomagnetic network and MT-based E-field reconstruction compared with measured GIC for the March 2015 event. | Published paper available; underlying raw data access not established. | A second historical event and method comparator. | Generalisation to all transformer types, all of South Africa or future network revisions. |
| **INTERMAGNET / global geomagnetic products** | Ground magnetic field and observatory metadata | Useful supplementary source for station data and quality context; local South African station access still needs exact product/rights verification. | Per-station and per-product terms must be checked. | Independent magnetic source comparison and station quality context. | Direct GIC or MT data. |
| **BPA public operational GIC feed** — https://transmission.bpa.gov/business/operations/gic/gic.txt | Transformer-neutral GIC comparator | BPA publishes a five-minute rolling feed from 11 substations, with a four-day rolling window and a last-updated timestamp. The page says currents are measured on transformer neutral-to-ground connections. | Public endpoint. Retention is rolling; preserve a timestamped snapshot and digest. Confirm reuse before redistributing derivatives. | Operational ingestion, source-freshness, cadence, missingness and multi-site monitoring tests; reference for telemetry-style data contracts. | A long historical storm corpus, South African topology or permission to infer other utilities' risk. |
| **SWERVE May 2024 US GIC dataset** — https://github.com/lucywilkerson/SWERVE-2024-05-10 | Multi-source measured and modelled event data | Four TVA measured 1-second series and matching 1-minute operator calculations, plus NERC and GMU reference model families. Dataset DOI: https://doi.org/10.5281/zenodo.20090223. | Repository declares MIT; third-party raw data may have source-specific conditions. The PR's replay pins exact commit/blob hashes and does not mirror the dataset. | Reproducible ingestion/aggregation/polarity/reference arithmetic and event-held-out benchmark development. | South African earth conductivity, topology, transformer parameters or transferability. |
| **New Zealand Transpower GIC monitoring** | Operational transformer-neutral GIC | A 2025 paper reports observations expanding from 36 transformers at 13 substations in 2001 to 93 transformers at 28 substations by October 2024. | Operator-owned; published aggregates and selected measurements, broader raw access requires partner terms. | Mature operational monitoring, multi-site measurement and transformer-level comparison model. | Transferability to South African grounding, network and conductivity conditions. |
| **UK SWIMMR / British Geological Survey MT** | Operational models, field observations and MT coverage | UKRI describes SWIMMR as a £20m, four-year operational monitoring/modelling programme. A 2025 study reports new long-period MT from 53 British sites (2021–2024), plus legacy sites for 70 total; another paper describes nine operationalised models for grid, pipeline and rail impacts. | Published programme and science; source model/data reuse terms are per product. | Research-to-operations benchmark for integrated instrumentation, local MT coverage and infrastructure-specific services. | Direct template for South African capability without local owner data and validation. |

## 3. Key South African references

1. **Ngwira et al. (2008), Grassridge/Hermanus:** multilayer conductivity modelling from measured GIC and geomagnetic observations, October 2003 event. DOI: https://doi.org/10.1029/2008SW000408.
2. **Ngwira et al. (2009), spatial transfer:** found model skill degrades with magnetometer distance and the model was specific to a station/site pair. DOI: https://doi.org/10.1029/2009SW000478.
3. **Nahayo et al. (2019), SANSA March 2015:** geomagnetic network and MT-based E-field modelling compared with measured GIC. DOI: https://doi.org/10.17159/sajs.2019/5204.
4. **Nahayo et al. (2022), Grassridge/Matimba:** southern-hemisphere LDi/LCi GIC-proxy validation using historical site records; quality windows at both sites were not simultaneous. DOI: https://doi.org/10.1029/2022SW003092.
5. **SANSA Annual Report 2015/16:** Botrivier under-line monitoring context. https://www.sansa.org.za/wp-content/uploads/2018/05/SANSA-AR-2015-16.pdf.
6. **African GIC proxy study (2026):** Hermanus included among five African observatories; proxies remain a separate evidence class from measured transformer GIC. DOI: https://doi.org/10.1016/j.asr.2026.04.031.

## 4. Data-rights and governance checklist

No dataset enters the public GMD benchmark corpus until these fields are answered:

- Legal data owner and authorised point of contact.
- Public / academic / commercial / research-partner scope.
- Allowed purposes: research, product development, model training, operational monitoring, public visualisation and derivative distribution are separate permissions.
- Raw data retention, local cache and deletion obligations.
- Whether exact station coordinates, transformer identities and topology are restricted.
- Whether trained model weights, feature statistics and derived indicators count as redistributable derivatives.
- Attribution / citation obligations and change notification.
- Data-processing agreement, security classification and incident reporting.
- Whether a derived result may be shared with a utility, regulator, researcher or public.
- Expiration and revocation behaviour; a previously valid result may become ineligible for current reliance.

The system must represent `RightsUnknown`, `AcademicOnly`, `CommercialContractRequired`, `PartnerRestricted`, `PublicRedistributable` and `Revoked` as distinct states. A valid byte digest or source signature does not create a data licence.

## 5. Priority data acquisition sequence

### P0 — Public portal and metadata inventory
- Register for SANDIMS under an appropriate research pathway only if the intended purpose fits its published terms.
- Enumerate geomagnetic/variometer sites, sampling intervals, file products, processing level, time range, location and metadata completeness.
- Keep every downloaded product pinned by digest and source metadata.
- For any commercial product development, obtain and record SANSA's commercial agreement before use.

### P1 — Historical utility records
- Ask the relevant utility/legacy Sunburst custodian about Grassridge and Matimba raw data, sampling rate, polarity, sensor calibration, maintenance, grounding, transformer connections and event coverage.
- Ask SANSA for Botrivier raw magnetometer data, instrumentation design and present/retired status.
- Determine whether a historical direct transformer-neutral GIC trace is available, not just an under-line magnetic response or published proxy.

### P2 — MT and Earth response
- Build a site/frequency/uncertainty map for local MT transfer functions and conductivity models.
- Record measurement date, instrument, site coordinates, frequency bands, tensor convention, remote reference, inversion model, uncertainty and depth sensitivity.
- Do not extrapolate a dense-looking national grid beyond the measured MT support without a model discrepancy / out-of-domain disposition.

### P3 — Utility partnership / shadow pilot
- Only with permissioned, owner-bounded topology and asset metadata.
- Join measured GIC, magnetic and MT data to the exact transformer/grounding/topology revision valid during each event.
- Start read-only and shadow-only; operator remains the authority.

## 6. Cross-region lessons for the benchmark

- **Locality matters:** a 2026 New Zealand study comparing OAKS and EYR magnetic sources found substantially different modeled currents; for Henderson T1, the 24-hour comparison was OAKS correlation 0.85 / performance parameter 0.98 / RMSE 2.47 A versus EYR 0.55 / 0.77 / 7.19 A. The distant source (>200 km south) could overstate local GIC. DOI: https://doi.org/10.1029/2026SW004997.
- **Monitoring scale matters:** Transpower NZ reports dozens of transformer-neutral monitors across 28 substations; one-off studies are not a substitute for persistent monitoring and maintenance history.
- **Operational integration matters:** the UK SWIMMR programme pairs measurements, MT, multi-hazard models, operational services and expert support.
- **Tail and duration matter:** published New Zealand transformer work identifies both brief >50 A episodes and sustained moderate current as separate response regimes. Benchmark peak detection, duration, harmonics and cumulative thermal state independently.
- **No global-index shortcut:** Kp/G-scale cannot substitute for local ground field, MT impedance, network orientation or transformer data.

## 7. Open gaps and honest status

| Question | Status |
|---|---|
| Does South Africa have published historical GIC studies and observations? | Yes — Grassridge, Matimba and Botrivier-related work is documented. |
| Is there a live SANSA data portal? | Yes — SANDIMS, with instrument metadata and date-filtered bundles. |
| Is commercial access automatically permitted through SANDIMS? | No — SANSA says a commercial contract is required. |
| Does the SANDIMS landing page prove availability of transformer-neutral GIC? | No — its described repository is geomagnetic, ionospheric and magnetospheric instrument data. |
| Is there a verified open, continuous, multi-site South African transformer-neutral GIC + network topology + MT dataset for current qualification? | Not established by this desk research. |
| Are current utility asset/topology data and transformer parameters publicly redistributable? | Not established; treat as owner-restricted until confirmed. |
| Can a regional station's magnetic observations be used as a local substitute without spatial validation? | No — published South African and New Zealand studies show station/site transfer limitations. |

## 8. Connected work

- [Sol-Atlas #692 — spatial exposure and infrastructure dependencies](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/692)
- [Sol-Atlas #697 — South African historical-data source qualification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/697)
- [Sol-Atlas #699 — offline Rust known-answer replay oracle](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/699)
- [Symtropy #4461 — GIC physical vertical slice](https://github.com/Luminous-Dynamics/symtropy/issues/4461)
- [Symthaea #11358 — consequence-centric inference](https://github.com/Luminous-Dynamics/symthaea/issues/11358)
- [Symthaea #11370 — causal calibration](https://github.com/Luminous-Dynamics/symthaea/issues/11370)
- [Mycelix #6018 — offline emergency federation](https://github.com/Luminous-Dynamics/mycelix/issues/6018)

**Claim ceiling:** This is a desk-research source inventory. It does not establish current South African grid risk, present sensor health, transformer vulnerability, national spatial coverage or commercial permission to use any data.