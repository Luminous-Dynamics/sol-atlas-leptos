# ANTARCTIC-ENERGY-ATLAS-001 — V0 spatial evidence contract

This draft establishes the data semantics for a read-only Antarctic operational-energy atlas. It is not yet wired into the Sol Atlas UI or live spatial pipeline.

## Why spatial precision is a scientific correctness problem

A map gives a strong visual impression that every value belongs to the pin underneath it. That impression is false if we map an annual programme-level total onto a single station, assign a reanalysis grid value as if it came from a mast, or silently stretch a historical monthly series across missing periods.

V0 therefore keeps **site identity**, **observation provenance**, **spatial support**, **temporal support**, and **evidence class** as separately versioned fields. A valid point location does not imply that every associated energy number is site-specific.

## Station locator data

The companion file `sites-v0.json` records AADC-published coordinates for Casey, Davis, Mawson and Macquarie Island. Macquarie Island is classified as subantarctic, not as a continental Antarctic station. Coordinates have only the precision published in the metadata; they do not describe asset subcomponents or energy equipment locations.

Historical AADC SOE Indicator 59 (electricity) and Indicator 56 (generator/boiler fuel) are source candidates. Their landing/metadata pages have inconsistent temporal-start statements versus stated archive spans. Do not resolve the difference by assumption. A source artifact must be downloaded, hashed, inspected and reconciled before observations become chartable.

## Observation scope rules

- AADC historical monthly series can become station-specific only after the raw dataset confirms its station identifier, time coverage, units and missingness.
- DCCEEW Annual Report 2024–25 Table 73 is a programme-level annual aggregate. Keep these observations in an aggregate layer; never distribute them over station pins.
- The renewable-electricity row names Mawson wind and Casey solar, but that row remains a combined reporting aggregate. Do not derive annual output by station from it.
- ERA5 is a gridded atmospheric reanalysis (0.25° regular lat/lon grid for the cited hourly product). It is not a site-level measurement. If later shown, display the grid support and interpolation/retrieval method, not a measured-station label.
- Synthetic scenario data belongs in a separate scenario layer with explicit 'synthetic' labels; never overlay it as if it were observed telemetry.
- No data are considered ingested merely because a landing page, DOI or download link is known.

## Evidence-aware UI contract

Every visible record should disclose:
- source/publisher and stable record ID;
- metric, numerical value, unit and exact reporting boundary;
- observation interval/time basis and time zone if applicable;
- spatial support: site, multi-site programme, region or grid cell;
- measurement/evidence class: direct measurement, published aggregate, reanalysis, model output, synthetic fixture or unknown;
- source version/hash, transformation version and data quality flags;
- uncertainty/missingness and all unresolved metadata conflicts.

UI aggregation and map selection may filter or display records, but must not silently infer site ownership, interpolate missing observations, or turn 'unknown' into zero.

## V0 verification

- Pin metadata coordinates exactly to source precision.
- Keep the four station records unique; preserve Macquarie's subantarctic classification.
- A programme aggregate must have no station-level value mapping.
- Unresolved historical time coverage remains an explicit quality flag.
- No ERA5 record can be labelled site-measured.
- Deterministic serialization of the source-pinned manifest.
- A source hash is recorded only after the actual downloaded source bytes are available.

## Sources

- AADC SOE Indicator 59 — monthly station electricity: https://data.aad.gov.au/aadc/soe/display_indicator.cfm?soe_id=59
- AADC SOE Indicator 56 — monthly generator/boiler fuel: https://data.aad.gov.au/aadc/soe/display_indicator.cfm?soe_id=56
- AADC monthly fuel metadata and citation (2019, DOI 10.26179/5d2ea1b98cba3): https://data.aad.gov.au/metadata/records/SOE_generator_boiler_fuel_usage
- AADC linked archive endpoint advertised by Indicator 59: https://data.aad.gov.au/aadc/portal/download_file.cfm?file_id=1272
- DCCEEW Annual Report 2024–25, Table 73: https://www.dcceew.gov.au/sites/default/files/documents/annual-report-2024-25.pdf
- Copernicus ERA5 hourly single-level reanalysis, DOI 10.24381/cds.adbb2d47: https://cds.climate.copernicus.eu/datasets/reanalysis-era5-single-levels?tab=overview
- Implementation contract: [Sol Atlas issue #304](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/304)
