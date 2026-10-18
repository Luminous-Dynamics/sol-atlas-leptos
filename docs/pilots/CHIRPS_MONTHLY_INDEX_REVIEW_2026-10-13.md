# CHIRPS v3 Monthly Africa Index Review — 2026-10-13

## Observation

- **Reviewed on:** 2026-10-13 (UTC date).
- **Source:** [official CHIRPS v3 monthly Africa-subset GeoTIFF directory](https://data.chc.ucsb.edu/products/CHIRPS/v3.0/monthly/africa/tifs/).
- **Earliest visible monthly file:** `chirps-v3.0.1981.01.tif`.
- **Latest visible monthly file:** `chirps-v3.0.2026.08.tif`.
- **Visible directory timestamp for latest file:** 2026-09-11 19:02 UTC.
- **September 2026:** `chirps-v3.0.2026.09.tif` was not present in the browsed index on this review date.

The directory page is a live HTML listing. This review was made through the browser view and **is not an acquired raw-index snapshot**: no exact response bytes or index SHA-256 were captured by that browser read. It is useful evidence for planning only. Before selecting a final analysis interval, run `plan_chirps_monthly.py` against the live directory with `--snapshot-index` and retain the generated plan and raw HTML. That command checks every requested month and records the exact index SHA-256.

## Research consequence

Do not define the pilot as “through the present” or infer a latest available month from the calendar. For a historical study, select a time interval that:
1. is explicitly present in the captured CHIRPS monthly directory snapshot;
2. intersects exact final WaPOR layer coverage and the selected correction vintage;
3. contains enough complete, comparable seasons for the declared baseline; and
4. does not combine preliminary and final product vintages silently.

If an updated directory adds later months, record a new dated index snapshot and diff it against the previous snapshot. Do not edit this historical observation in place.

## Explicit limitations

- The index lists filenames and server-side timestamps; it does not certify TIFF content, units, QA masks, retrieval success, license applicability to derived outputs, or scientific validity.
- A file's directory timestamp is not necessarily its observation period or scientific publication date.
- The current-month release cadence can change; rerun the planner when beginning acquisition.
- No rainfall raster was downloaded or processed during this review.

## References

- CHIRPS v3 scientific/data documentation: https://www.chc.ucsb.edu/data/chirps3
- Monthly Africa-subset GeoTIFF index: https://data.chc.ucsb.edu/products/CHIRPS/v3.0/monthly/africa/tifs/
- Planner implementation: ../../scripts/earth-systems/plan_chirps_monthly.py
