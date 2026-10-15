# Live Source Capture Review — 2026-10-14

## Capture provenance

Both read-only source-capture jobs in [Earth systems catalog validation run 38623641960](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/38623641960) completed successfully on commit `03f8d8d0eb7370a80582989557839c8f092a01b4`. The deterministic catalog/pilot/index-planner/WaPOR tests and the Rust `fmt`, WASM check, and native test jobs also passed on that exact commit.

The GitHub artifact pages are [CHIRPS index snapshot 11933703363](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/38623641960/artifacts/11933703363) and [WaPOR mapset snapshot 11933893661](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/38623641960/artifacts/11933893661). Both artifacts were downloaded and inspected after capture. Artifact retention is 14 days; they expire on 2026-10-28 unless refreshed or the evidence is preserved separately.

## CHIRPS v3 monthly Africa index

- Source URL: https://data.chc.ucsb.edu/products/CHIRPS/v3.0/monthly/africa/tifs/
- Retrieved at: `2026-10-14T22:53:37Z`
- Exact captured HTML size: **107,582 bytes**
- SHA-256 of captured HTML (independently recomputed): `851fd43a24e0e4337da19f504c3e952e71fdcb0653aa5d168328d3da50a26654`
- Parsed index: **548 monthly GeoTIFF links**, 548 distinct months, no duplicate month keys and no gaps within the listed range.
- Earliest/latest listed files: `chirps-v3.0.1981.01.tif` through `chirps-v3.0.2026.08.tif`.
- September 2026 and later months were not present in the captured index.
- The exact index hash and byte count match the uploaded manifest. **No GeoTIFF was downloaded.**

This improves the earlier browser-only observation: the end month is now supported by a retained byte snapshot for the captured time. It still does not prove any remote TIFF downloaded successfully, that a file's data are scientifically valid, or that its final/preliminary product status is suitable for the pilot. The study period remains unselected.

## FAO WaPOR v3 mapset catalog

- API endpoint: https://data.apps.fao.org/gismgr/api/v2/catalog/workspaces/WAPOR-3/mapsets
- Three raw response pages were captured at approximately `2026-10-14T22:53:14Z`.
- Total mapset records: **42**, all codes unique; page counts were 20, 20 and 2.
- Raw responses were independently rehashed. Every file's SHA-256 and byte count match both the inventory manifest and its corresponding page metadata.

| Page | Records | Bytes | SHA-256 |
|---|---:|---:|---|
| 1 | 20 | 8,583 | `bfa8c5cd354e46f068c439ea3effa1f9899228c2e0eb5ba2daa232955384076e` |
| 2 | 20 | 8,666 | `e7742848a076626ed564013181bad3a32fd3e8906fe96c9b3b50af71ffe883d1` |
| 3 | 2 | 1,230 | `ca3816371afee6fd1e00062ffc28536f6e132cd2b6bee1641fe3a338f759e5ed` |

The captured mapset listing contains national/continental/global products such as `L2-AETI-M` (actual evapotranspiration and interception; national; monthly; 100 m; unit `mm/month`; scale metadata 0.1) and `L1-AETI-M` (global; monthly; 300 m). It also contains a location-specific Jericho crop-map product. It contains no mapset code beginning `L3-`, and no mapset metadata matched `MBL`, Baixo Limpopo, Limpopo or Mozambique.

**Interpretation limit:** this is a verified observation of the specific `/mapsets` endpoint at a particular time, not proof that the Baixo Limpopo Level III products are unavailable everywhere in WaPOR. The public FAO overview separately lists Baixo Limpopo as a Level III area, and FAO says the continuously updated portal catalog is the authority for current product availability. The inventory API response does not expose a Baixo-specific Level III mapset record among these 42 entries. No Level III layer or period may be inferred from the national monthly mapset.

The next useful query is the raster metadata for exact catalogued mapset code `L2-AETI-M`, as a clearly labelled **national-resolution fallback investigation only**. It is not a substitute for the intended Level III pilot and must not be silently used to claim 20 m irrigation-scheme evidence.

## Local validation lead

FAO's 21 August 2026 update reports that the Irrigation Performance Assessment and Diagnostic Tool (IPADT) supports Lower Limpopo irrigation schemes including Chókwè and Xai-Xai, combining WaPOR with water-supply information at block level. This is a useful lead for requesting provenance, definitions, access route and suitable reference data from INIR/FAO/IWMI; it is **not evidence that those operational records are public, downloadable, or independent of WaPOR**.

Source: https://www.fao.org/in-action/remote-sensing-for-water-productivity/news-and-events/news/news-detail/mozambique-strengthens-use-of-satellite-data-to-assess-irrigation-performance/en

## Decision

- CHIRPS index inventory: captured, hashed and reviewed.
- WaPOR mapset inventory: captured, hashed and reviewed.
- Exact Baixo Limpopo Level III layer/time window: **not yet identified from the captured mapset listing**.
- Raster bytes downloaded/processed: **none**.
- Independent validation: **not performed**.
- Pilot status remains `design_only`; all evidence gates remain unchanged.

A SHA-256 digest establishes byte identity, not truth. A catalog record establishes a catalog observation, not valid layer coverage or reuse rights.
