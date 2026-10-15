# Black Sea Grain Initiative: Source-Readiness Review

- **Status:** `design_only`; selected for source-readiness review, not yet qualified for analysis.
- **Catalog domain:** `global-conflict-and-supply-chain-resilience`.
- **Machine-readable gate:** [black-sea-grain-initiative-readiness.json](black-sea-grain-initiative-readiness.json).
- **Review date:** 15 October 2026.
- **Claim ceiling:** source-discovery and publisher-reported observations only. No causal estimate, live warning or global impact predictor is produced.

## Why this is the first candidate

This case has an unusually clear public mechanism record: commercial shipping to and from Ukrainian Black Sea ports was halted in February 2022 on safety grounds; the Initiative was signed on 22 July 2022; and official Joint Coordination Centre updates publish monthly cargo totals and voyages under the agreement. The UN's 17 July 2023 update reports 32,817,650 metric tonnes and 1,003 voyages by 14 July, with monthly values from August 2022 through a partial July 2023. The agreement was not renewed after 18 July 2023. Sources: [UN Joint Coordination Centre update](https://www.un.org/en/node/207702), [UN Initiative data page](https://www.un.org/en/black-sea-grain-initiative/data), and [UN achievement summary](https://www.un.org/en/black-sea-grain-initiative/achievements).

FAO provides a monthly Food Price Index dataset from 1990 onwards, with distinct cereal and vegetable-oil group indices. Its published notes discuss the index's peak in March 2022 and changes into 2023. That makes it useful for temporal context, not a clean downstream causal outcome: the overall index is a weighted basket, is revised, and is exposed to many simultaneous supply, weather, energy, currency and policy effects. Sources: [FAO index and CSV download](https://www.fao.org/worldfoodsituation/FoodPricesIndex/en/) and [FAO index methodology](https://www.fao.org/worldfoodsituation/FoodPricesIndex/en/).

This is a good **first integration test** because the observed corridor series has direct published monthly quantities; it is not a good basis for a simple before/after causal conclusion. The agreement's own series only starts in August 2022. Months before that are **not observed values of zero** and cannot be filled with zero.

## What has been established during source discovery

- The official UN update provides a cumulative reported total, a voyage count and a monthly table by departure month and designated port.
- The monthly figures transcribed into the JSON are source-reported values found during research. They sum to the reported cumulative total of 32,817,650 tonnes. That is a useful transcription-consistency check, but not an independent validation of the publisher's data.
- July 2023 is explicitly partial through 14 July, so it must not be plotted or compared as a full month.
- FAO's page links the monthly CSV and explains the overall index's basket structure. The available browser reader did not ingest the raw CSV media type, so this review has **not** acquired the file bytes, recorded a digest, or inspected its row-level metadata.
- UCDP's official download center provides the Georeferenced Event Dataset version 26.1 and codebook; its data covers 1989–2025 and is licensed CC BY 4.0 with required citation. The browser reader did not ingest the ZIP archive, so no archive acquisition or digest is claimed. UCDP GED records georeferenced lethal organized-violence events; it is conflict context, not a port-closure feed. Sources: [UCDP download center](https://ucdp.uu.se/downloads/) and [GED 26.1 codebook](https://ucdp.uu.se/downloads/ged/ged261.pdf).
- The exact UN page bytes, FAO CSV bytes, UCDP archive and Comtrade responses have not been acquired and hashed. Data access, product terms and attribution still need artifact-specific recording.

## Candidate windows—do not freeze until data review

| Period | Intended meaning | Important limit |
|---|---|---|
| 2017-01 to 2021-12 | Candidate monthly trade baseline for Ukraine HS 1001 wheat and HS 1005 maize, if Comtrade coverage and classification can be qualified | Includes COVID-era structural shocks; not a sufficient control by itself |
| 2022-02 to 2022-07 | Conflict and export-corridor disruption context before Initiative shipment reporting begins | Do not substitute missing Initiative records with zero total Ukrainian exports |
| 2022-08 to 2023-06 | Full calendar months in the published Initiative corridor series | Represents Initiative shipments from designated ports, not all Ukrainian grain exports |
| 2023-07 through 2023-07-14 | Last partial month present in the cited update | Explicit partial period; not comparable to a complete month without a documented adjustment |
| After non-renewal in July 2023 | Possible future phase for alternative-route and post-Initiative export evidence | Requires separately acquired route/trade data; the Initiative's series cannot by itself describe later total exports |

The study window, query filters, controls and outcome definitions remain unfrozen. If a source cannot supply sufficient matching monthly/commodity/geography coverage, we will narrow or abandon that comparison instead of imputing values.

## Evidence path

1. **Conflict and maritime context:** use a pinned UCDP release only for the declared conflict-event question. Record that it uses a lethal-organized-violence event definition. It cannot alone establish a port closure or shipping risk.
2. **Direct corridor mechanism:** capture the UN Joint Coordination Centre's original source page/table or a publisher-linked downloadable asset. Store exact bytes and SHA-256, retrieval time, publisher/version, parser version, rights state, month/port units and partial-period labels. Reparse the table from those captured bytes.
3. **Monthly market context:** capture FAO's exact CSV, preserve exact columns/raw values and revisions, identify the cereal and vegetable-oil index series, and document source terms. Keep the overall FFPI as an explicitly broad contextual index.
4. **All-route trade baseline:** query UN Comtrade for reporter Ukraine, HS 1001 and HS 1005, monthly periods, matching flow/partner/classification dimensions across the chosen baseline and follow-up years. Capture request parameters, response snapshots, completeness/estimated state, unit and valuation metadata, pagination and limits. Aggregate data must stay distinct from shipment-level evidence.
5. **Controls and sensitivity:** after data readiness, preregister suitable comparison origins, commodities, time windows, seasonality treatment and exclusion rules. Inspect pandemic-period effects, other exporters, freight/energy costs, weather, policy and concurrent conflict activity. Do not choose controls after seeing which combination produces a preferred result.
6. **Optional maritime context:** add a specific IMF PortWatch series only if its temporal/geographic definition, version and terms match the question. Port activity is not commodity-specific cargo volume or a bill of lading.

## Required tests before C0 can close

- Reparse exact captured UN source bytes and independently verify every monthly value and the published cumulative total.
- Assert that the final July observation is partial and that no pre-August initiative values are imputed as zero.
- Preserve source-reported versus independently recomputed values separately.
- Parse FAO values from exact captured CSV bytes; test date uniqueness, raw-value preservation, missingness, revisions and index labels.
- Verify Comtrade query dimensions, row/aggregate status, classification, unit, currency/value semantics, completeness and cross-page duplicate handling.
- Run source-ablation and negative-control tests after the source sample is frozen; do not treat multiple summaries of the same JCC table as independent corroboration.
- Obtain independent review of the captured data, preregistered analysis and claim ceiling.

## Resulting claim ceiling

After C0, an acceptable result may describe the Initiative's **publisher-reported corridor flows** and their timing relative to a separately captured global food-price series. It must not claim that the agreement caused the global index change, that the series represents all Ukrainian grain exports, that any destination had a shortage, or that a modeled path identifies current inventory or a real supplier.

No dispatch, procurement, price forecasting, humanitarian allocation or operational action is enabled by this pilot.
