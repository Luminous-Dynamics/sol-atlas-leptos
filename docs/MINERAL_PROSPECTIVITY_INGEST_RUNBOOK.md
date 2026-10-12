# Mineral Prospectivity Ingestion Runbook

**Status:** Executable procedure scaffold; no real source archive processed yet.  
**Purpose:** Safely inventory the USGS Africa mineral-industries geodatabase, then inspect its GIS metadata without adding source data to Git.

## 0. Source and rights

Primary source: [USGS Africa mineral-industry geodatabase (2021)](https://doi.org/10.5066/P97EQWXP), also listed in the [USGS data release record](https://www.usgs.gov/data/compilation-geospatial-data-gis-mineral-industries-and-related-infrastructure-africa). The USGS catalog identifies the release as public and marked CC0 1.0. This is release-level rights information; preserve component-layer source metadata and assess any restrictions or third-party acknowledgements before republishing derived products.

The assistant execution environment could not resolve external hosts for direct download. Accordingly, the actual archive has **not** been downloaded or inspected here. Obtain it manually from the official USGS/ScienceBase record, record the exact download URL/product filename and date, and never commit the archive to this repository.

## 1. Prepare a controlled workspace

Use an isolated directory on a filesystem with sufficient free space. Keep downloads, extracted source data, metadata reports, and derived products separate from the Git working tree.

With Nix available, an environment containing Python and GDAL can be opened with:

    nix shell nixpkgs#python3 nixpkgs#gdal

Verify the installed tools:

    python3 --version
    ogrinfo --version
    gdalinfo --version

Do not run data ingestion with secrets or unrelated home-directory content mounted into the workspace.

## 2. Inspect the downloaded archive before extraction

Example paths below are placeholders; substitute the exact artifact you obtained.

    python3 scripts/mineral-prospectivity/inspect_bundle.py /data/inbox/Africa_GIS.gdb.zip --source-id USGS-AFRICA-GIS --title "USGS Compilation of Geospatial Data for Mineral Industries in Africa" --source-url https://doi.org/10.5066/P97EQWXP --rights-status unknown --rights-notes "Release is listed CC0; component source metadata and exact downloaded asset still require review" --output /data/reports/usgs-africa-preflight.json

Check:
- `artifact.sha256` matches the hash of the exact downloaded bytes.
- `unsafe_member_names` is empty and there are no unexplained duplicate/colliding member names, symlinks, or compression/size warnings.
- `file_geodatabase_roots` identifies the expected `.gdb/` directory and its internal component suffixes.
- The archive is not empty, and the manifest's file count and size appear plausible for the stated product.

Optional CRC verification:

    python3 scripts/mineral-prospectivity/inspect_bundle.py /data/inbox/Africa_GIS.gdb.zip --source-id USGS-AFRICA-GIS --verify-crc --output /data/reports/usgs-africa-preflight-crc.json

The optional scan is refused when unsafe paths or its advertised size/compression thresholds are exceeded. Do not override that refusal by blindly extracting the archive.

## 3. Stage source data only after human review

The preflight tool deliberately does not extract. If the archive has unexpected paths, symbolic links, duplicate entries, suspicious compression, or undocumented files, stop and investigate the original source before extraction. Otherwise stage it in a dedicated, quota-limited directory with a reviewed extractor and without preserving owners or permissions. Do not use a bulk extraction command against an untrusted archive without checking its extraction behavior.

Do not write extracted source data into the repository. Keep the original archive immutable; record source and extracted-artifact hashes separately.

## 4. Inspect the File Geodatabase with GDAL/OGR

A File Geodatabase typically appears as a directory ending in `.gdb`. The GIS inspector accepts a `.gdb` directory, computes a deterministic tree digest (sorted relative path, size, and per-file SHA-256), rejects symlinks, and enforces caps of 100,000 files and 20 GiB total size.

    python3 scripts/mineral-prospectivity/inspect_gis.py /data/staged/Africa_GIS.gdb --source-id USGS-AFRICA-GIS --title "USGS Africa File Geodatabase" --source-url https://doi.org/10.5066/P97EQWXP --rights-status unknown --rights-notes "Component-layer provenance not yet reviewed" --output /data/reports/usgs-africa-gdb-metadata.json

Review the report for:
- GDAL/OGR driver and version, layer names and feature counts.
- Geometry types, extent, coordinate reference system, fields/schema and missing CRS warnings.
- Layer-specific source metadata, dates, units, and whether extent/coordinates are consistent with the intended analysis region.
- The exact artifact/tree digest and byte/file counts.

For any GeoPackage containing raster tiles, use `--kind raster`; the default for `.gpkg` is vector metadata. No tool here mutates source data.

A successful GDAL/OGR open does not mean the dataset has correct attributes, complete features, a valid license for redistribution, or geological suitability.

## 5. Per-layer triage before joining or modeling

Build a table with one row per layer:
- layer name, geometry type, feature count, fields/schema, CRS, extent, coordinate accuracy, date/vintage;
- upstream source/agency, source URL/citation, terms/license, reuse state, and relevant attribution;
- whether it represents observations, mines/facilities, occurrences, modeled resource tracts, infrastructure, boundaries, or other context;
- any known selection bias, omission, duplicate records, or spatial uncertainty;
- a child manifest ID and data lineage references.

Do not treat the USGS layer of *undiscovered resource tracts* as ground truth. It is itself a model output. Use it as context/benchmark material only, and explicitly avoid leakage if later evaluating predictions against that layer.

## 6. Acceptance gates

- **G0 — Source identified:** canonical source record, exact file list, acquisition date and access terms recorded.
- **G1 — Container inspected:** archive hash, safe-path results, FileGDB root, entry inventory and warnings reviewed.
- **G2 — GIS metadata inspected:** GDAL/OGR report generated; CRS, schema, extents and feature counts reviewed.
- **G3 — Layer provenance reviewed:** every layer classified and component provenance/reuse terms checked.
- **G4 — Spatial QA passed:** expected area/CRS, coordinate ranges, geometry validity, data gaps, duplicates and target label semantics checked.
- **G5 — Eligible for analytical features:** the selected subset and all transformations frozen in a data manifest; no hidden source substitution.

A GIS-readable dataset only passes G2. It is not automatically ready for model training. Keep raw data, derived features, labels and model outputs separate.

## 7. Recommended first use of this dataset

Use the archive to exercise the ingestion, manifest, and provenance machinery and to map the locations/types of existing mineral observations, facilities, and modeled tracts. Then pursue the 2024 South African prospectivity paper's supplement as the primary PGE/Au benchmark. Do not replace the benchmark's labels/covariates with the Africa database or claim discovery based on overlap with existing modeled tracts.
