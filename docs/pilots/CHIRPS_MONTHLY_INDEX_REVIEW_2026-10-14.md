# CHIRPS v3 Monthly Africa Index Recheck — 2026-10-14

## Observation

- **Reviewed on:** 2026-10-14 (UTC date).
- **Source:** [official CHIRPS v3 monthly Africa-subset GeoTIFF directory](https://data.chc.ucsb.edu/products/CHIRPS/v3.0/monthly/africa/tifs/).
- **Earliest visible monthly file:** `chirps-v3.0.1981.01.tif`.
- **Latest visible monthly file:** `chirps-v3.0.2026.08.tif`.
- **Visible directory timestamp for latest file:** 2026-09-11 19:02 UTC.
- **September 2026 and later:** no matching monthly files were present in the browser-rendered listing checked on this date.

This is a follow-up to the [2026-10-13 browser review](CHIRPS_MONTHLY_INDEX_REVIEW_2026-10-13.md). The latest visible entry and its displayed timestamp were unchanged in this recheck. The directory was inspected through the web rendering, not retrieved as exact response bytes by the CLI; **this is not a raw-index snapshot and has no source-byte SHA-256**.

## Research consequence

Do not assume that September 2026 is available because its expected release window has arrived. Select an analysis window only after:
1. a fresh raw index has been captured by `plan_chirps_monthly.py --snapshot-index`;
2. every requested month is present in that captured index;
3. the requested period intersects the exact selected final WaPOR layer's available temporal coverage and vintage; and
4. there are enough complete, comparable seasons for the declared baseline.

A newer directory observation does not itself prove the files are downloadable, final, scientifically valid, or compatible with the chosen WaPOR records.

## Explicit limitations

- The rendered listing is a browser observation, not an exact capture of the directory-index response bytes.
- The visible filename and server timestamp do not prove the scientific publication time, TIFF integrity, final/preliminary state, quality-mask semantics, or product suitability.
- No CHIRPS rainfall raster was downloaded or processed during this review.
- The WaPOR API catalogue remains inaccessible through the current research fetch path, so this review does not establish an analysis interval.

## References

- CHIRPS v3 scientific/data documentation: https://www.chc.ucsb.edu/data/chirps3
- Official monthly Africa-subset GeoTIFF index: https://data.chc.ucsb.edu/products/CHIRPS/v3.0/monthly/africa/tifs/
- Prior dated observation: [CHIRPS monthly index review — 2026-10-13](CHIRPS_MONTHLY_INDEX_REVIEW_2026-10-13.md)
- Planner implementation: ../../scripts/earth-systems/plan_chirps_monthly.py
