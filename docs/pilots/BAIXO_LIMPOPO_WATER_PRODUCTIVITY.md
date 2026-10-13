# Baixo Limpopo Water–Food Productivity Pilot

- **Status:** Design only; source artifacts have not been acquired or processed.
- **Reviewed:** 2026-10-13
- **Parent proposal:** [Earth Systems Intelligence RFC](../EARTH_SYSTEMS_INTELLIGENCE_RFC.md)
- **Machine-readable pilot profile:** [baixo-limpopo-water-productivity.json](baixo-limpopo-water-productivity.json)

## Why this pilot

A bounded irrigation-area pilot is a better second scientific test than a generic “water security” demo. It checks whether Sol-Atlas can align rainfall with agricultural water/productivity products while preserving spatial support, processing vintage, missingness, source lineage, and limits on interpretation.

FAO's current WaPOR documentation lists three product levels and identifies a Level III Baixo Limpopo (Mozambique) area in its area list. Level III products are area-specific; actual layer availability and date coverage must be rechecked in the live catalog before selecting files. WaPOR provides a no-credential API for v3 retrieval, so acquisition can be automated after this source check. References: [WaPOR data overview](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data/) and [official access channels](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data-access/en).

CHIRPS v3 offers a long historical precipitation series at 0.05-degree grid spacing, with separate preliminary and final products. The new pilot uses v3 and requires a pinned **final** product for retrospective analysis; CHIRPS v2 production is scheduled to end after December 2026. CHIRPS blends satellite-derived estimates with station observations, so station data that participated in the blend cannot automatically be treated as independent ground truth. References: [CHIRPS v3](https://www.chc.ucsb.edu/data/chirps3) and [CHIRPS overview / v2 lifecycle notice](https://www.chc.ucsb.edu/data/chirps).

The pilot does **not** need GRACE in its first pass. Its broad grid and model-assimilated drought semantics do not resolve irrigation-scheme operations or individual wells. GRACE-derived indicators can be reconsidered only as a regional context layer, never as a local well measurement.

## Scientific question

Within the footprint actually covered by a verified WaPOR Level III Baixo Limpopo product, where do the selected water/productivity indicators depart from their own seasonally comparable history, and are those departures temporally consistent with precipitation anomalies at defensible shared spatial and temporal support?

The first deliverable is a **source-readiness and coverage report**. A descriptive anomaly report is allowed only if source vintages, quality masks, temporal support, and the length/completeness of the common record pass D1–D2. Forecasting, crop-yield estimates, causal attribution, and operational recommendations are out of scope.

## Source admission, in order

### 1. Freeze the spatial and product definitions

Use the current WaPOR catalogue to record the exact Level III area/product IDs, layer names, units, processing version, available dates, nominal and effective resolution, quality mask, no-data semantics, and publication/finality status. Record the area boundary from the source product or an explicitly versioned authoritative boundary; do not trace it from an image or silently substitute the whole Limpopo Basin.

The first candidate WaPOR layers are actual evapotranspiration and interception (AETI), its documented water-balance components when available, and a documented biomass/water-productivity output. The live catalogue—not this document—determines the final subset. Do not assume that all layers exist for the selected area or period.

**Release-history guard:** FAO documents a WaPOR Level III methodology re-release in June/July 2024 and a separate correction to Level III net primary production (NPP) and above-ground biomass production (AGBP) for Baixo Limpopo and other areas announced in March 2025. The provider warns that affected NPP/AGBP files downloaded before 21 March 2025 likely underestimated plant-production variables. If those layers are selected, confirm the correction applies to the exact product, reacquire corrected artifacts, and freeze their hashes and release metadata. Never mix old and corrected vintages without a documented compatibility analysis. See [WaPOR access notes and correction history](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data-access/en).

### 2. Freeze rainfall

Select CHIRPS v3 **final** data matching the pilot time window and temporal aggregation. Save the exact product identifier / filenames, publication timestamp, source README and citation, terms snapshot, acquisition timestamp, file hashes, and valid-data mask. Keep preliminary and final products separate; a replacement final release creates a new artifact identity rather than silently overwriting the older run.

### 3. Preserve source dependence and rights

For each artifact, retain URL, source/product/version IDs, SHA-256, retrieval time, terms/attribution snapshot, coordinate reference system, physical units, temporal interval, pixel support, quality flags, data gaps, and upstream inputs. WaPOR is a derived product; its presence beside CHIRPS is not automatically independent corroboration. Inspect the WaPOR methodology and CHIRPS station-input documentation before making any independence claim.

WaPOR is marked **product-level rights review required** until the exact selected layer's metadata and reuse terms have been checked. CHIRPS v3 is described by its provider as CC BY 4.0; retain attribution and the release-specific citation. Do not infer that one source's rights apply to another source or to a merged output.

## Analysis protocol

1. **Coverage first.** Produce a per-product matrix of dates, missingness, quality-mask coverage, boundary overlap, and valid-cell/valid-period counts. If the common period is too short or incomplete, stop at coverage/time-series description.
2. **Match support.** Aggregate finer products to a common defensible support. Never interpolate coarse rainfall to a fine grid and present it as new fine-scale information. Record spatial aggregation, masks, area weighting and CRS transformation.
3. **Match time.** Compare like with like (e.g., matching monthly or dekadal windows only when both exact product layers support them). Record event time, observation period, retrieval time, publication time, and final/preliminary status separately.
4. **Use an honest baseline.** If the frozen WaPOR layer provides enough complete comparable seasons, compute a predeclared season-aware reference (median and a robust spread such as MAD) using only the training period. If data are insufficient, report coverage and raw time series without anomaly labels. Do not choose thresholds after examining the target period.
5. **Test sensitivity.** Report results under alternative reasonable masks, aggregation choices, season windows and data vintages. Keep uncertainty, coverage, and source disagreement visibly separate.
6. **Validate independently.** Before calling a result validated, seek rain gauges whose contribution to the chosen CHIRPS v3 release is known and excluded from the independent test; also seek irrigation delivery/operations records and official crop statistics with compatible time/geographic support. If input overlap cannot be resolved, classify the comparison as contextual agreement—not independent validation.

## Gates and acceptance criteria

| Gate | Required evidence | Stop condition |
|---|---|---|
| D0 — Inventory | Current product-catalogue snapshot; exact WaPOR layer and CHIRPS v3 final-product choices; terms/reuse review; declared target | Any critical product, date, license, or boundary uncertainty remains unresolved |
| D1 — Intake | Frozen source bytes, SHA-256 digests, metadata snapshots, units/CRS, masks, lineage and reproducible acquisition record | Missing digest/metadata, preliminary-final mix, unreviewed redistribution or unexplained schema drift |
| D2 — Baseline | Coverage report; reproducible seasonal baseline; comparison to a transparent simple baseline; sensitivity analysis | Insufficient comparable seasons, material leakage, or hidden change of product vintage |
| D3 — Independent validation | Reference data with documented independent lineage; predefined splits; calibrated/appropriate metrics and false-alert accounting | The purported reference shares unknown or known target-generating inputs |
| D4 — Independent review | Reviewer checks source admission, methodology, split construction, calculations, limitations and failure cases | Only author self-review or results that cannot be reproduced |
| D5 — Decision support | Versioned outputs with uncertainty/coverage, update policy, user comprehension and accountable decision boundary | Output implies farm-level truth, causality, forecast, or authority not actually validated |

The profile and validator encode **design-only status** today. They do not grant any gate.

## Local independent-evidence leads

Desk research found two candidate paths for local reference records. Both remain unadmitted until the exact table/record, geography, time period, provenance and reuse terms are inspected.

- **Instituto Nacional de Irrigação (INIR), Mozambique:** its official site announced the Plataforma de Irrigação de Moçambique (PIM) on 29 June 2026 and also reported preliminary flood impacts at Regadio do Baixo Limpopo in June 2026, including a report that about 3,500 hectares of rice had been cultivated before the floods. This is a strong lead for locating irrigation-scheme boundaries, operations, or event records; the announcement does not establish an accessible API, export format, full record set, or independent validation labels. See [INIR](https://inir.gov.mz/).
- **Instituto Nacional de Estatística (INE), Gaza provincial yearbook 2024:** the official yearbook includes basic-food crop area and production tables for 2020–2024. These are province-scale aggregates. They can inform broad plausibility checks, but must not be presented as validation of a particular irrigation scheme, individual fields, or 20 m pixels. See the [Gaza provincial statistical yearbook 2024](https://www.ine.gov.mz/documents/20119/334668/ANUARIO_ESTATISTICO_PROVINCIA_GAZA_2024.pdf/facd30cd-f7dc-83ac-1a35-3e92df08ea22?download=true&t=1751365783760&version=1.0).

For either path, first capture the primary source bytes/metadata and its version/date, identify who collected or estimated the values, understand whether the reported footprint overlaps WaPOR or CHIRPS inputs, and review terms. A preliminary flood-impact report is an investigation target—not a ground-truth label until its method and spatial/temporal definitions can be reproduced.

## Metrics if the evidence supports them

Report data availability and valid-area fraction first. If baseline analysis becomes justified, report seasonal anomaly error/agreement against the frozen baseline, sensitivity to masks and product vintages, and performance stratified by season/zone. Only a pre-registered alert threshold should produce precision/recall or false-alert rate. Independent gauge comparisons must disclose whether those stations were absent from the selected CHIRPS blend. Metrics without suitable reference data are not evidence of scientific validity.

## Claim ceiling

Even after D2, this pilot may describe gridded indicators and their co-variation. It does not, by itself, establish:
- farm-level crop yield, farmer behavior, or household food availability;
- exact irrigation efficiency, water delivery to an individual parcel, or an individual's drinking-water access or safety;
- individual well levels, local aquifer depletion, or groundwater over-abstraction;
- that rainfall deficit caused a productivity change, or that irrigation caused or prevented it;
- a forecast, optimized operating policy, infrastructure design, or authority to control irrigation, allocate water, or direct emergency aid.

A real-world decision would need appropriate local evidence, domain review, consultation with relevant users and authorities, and a separate D5 acceptance decision. Integrity hashes identify artifacts; they do not make their scientific interpretation true.

## Current state

No WaPOR or CHIRPS artifacts have been downloaded or processed as part of this pilot. The exact WaPOR Level III layer, period, boundary, CHIRPS file release, rights snapshots and independent reference sources remain unlocked. The machine-readable profile intentionally records those gaps instead of filling them with assumptions.
