# Highveld weather source register v0.1

This is a **candidate inventory**, not an admitted data archive. It is maintained under Sol-Atlas #611.

## Source families

- Five Gauteng 2011–2025 Climate.OneBuilding TMYx products: Tambo, Botanical Gardens, Grand Central, Rand and Lanseria.
- South African Weather Service archive candidate: SAWS describes selected-station hourly data (wind, temperature, humidity, pressure, sunshine) from 1950 onwards; station periods, identity, QC and product-specific rights must be obtained.
- ARC Agrometeorology candidate: the portal describes approximately 1,900 historical station records, including 1,100 manual and 763 automatic; 460 AWS are active and daily data are collected. It offers ad-hoc hourly reports, but station-specific hourly coverage must be confirmed.
- NOAA NCEI GHCND Johannesburg International daily-summary comparator, not an hourly thermal-model input.
- ERA5 gridded reanalysis solar comparator, not a ground observation.

## Do not conflate

SAWS' current national observation network count, ARC's operational programme count, ARC's broader archive inventory and the TMYx file count represent different institutions and scopes. No totals are summed. TMYx is typical-year output, not an actual-year heatwave or future scenario. No station IDs are assumed identical across WMO/WIGOS/ISD/GHCND/provider systems without an effective-date crosswalk.

## Admission workflow

1. Obtain exact archives/metadata from providers.
2. Inspect EPW/STAT/DDY/Rain headers or native station file metadata.
3. Record archive and extracted-content SHA-256.
4. Audit coordinates, elevation, timezone, instrument/relocation history, variables, missingness, QC and periods.
5. Establish pricing, commercial use and redistribution rights.
6. Compare only compatible periods/variables and preserve source disagreement.
7. Keep a candidate blocked while any critical identity, rights, temporal, quality or representativeness field remains Unknown.

## Current claim ceiling

No source in this register is locally qualified. It does not establish weather at any parcel, a design extreme, heatwave, future projection, energy-model input or building-performance result.
