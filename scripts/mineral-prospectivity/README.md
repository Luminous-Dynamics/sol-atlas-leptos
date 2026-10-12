# Mineral prospectivity data tools

These tools are stages of a fail-closed ingest pipeline, not a prospectivity model.

## 1. Archive preflight (standard library only)

    python3 scripts/mineral-prospectivity/inspect_bundle.py /path/to/source.zip --source-id USGS-AFRICA-GIS --title "USGS Africa minerals geodatabase" --source-url https://doi.org/10.5066/P97EQWXP --rights-status unknown --rights-notes "Rights review pending" --output /tmp/usgs-africa-preflight.json

This hashes the original file, inventories ZIP metadata without extraction, and flags unsafe paths, duplicate/colliding names, suspicious sizes/compression, and incomplete Shapefile sidecars. The optional `--verify-crc` reads archive members only when no unsafe path is present and advertised expanded size/compression ratio is within configured limits.

Keep rights status as "unknown" until the exact artifact's terms have been checked. The tool does not determine permissions.

## 2. GIS metadata inspection (GDAL/OGR required)

First stage an authorized and manually reviewed file in a controlled local workspace. This command does **not** extract archives; it only opens one already-staged GIS file in read-only mode:

    python3 scripts/mineral-prospectivity/inspect_gis.py /data/geology.gpkg --source-id SA-CGS-1M-GEOLOGY --title "Candidate CGS geology layer" --rights-status unknown --rights-notes "Reuse terms awaiting review" --output /tmp/geology-metadata.json

The tool invokes `ogrinfo -json -ro` for vector formats and `gdalinfo -json` for raster formats, then writes a compact metadata report containing driver, layers, schema/fields, feature counts, CRS, extents, raster dimensions, bands, hash, and warnings. It uses a timeout, shell-free subprocess invocation, an output-size cap, and does not pass update/write flags. GeoPackage raster content may require a separate raster-mode extension; the current .gpkg default inspects its vector interface.

A successful GDAL/OGR open is not proof that geometry is valid, metadata complete, rights allow use, or the dataset is geologically suitable. Review warnings, compare CRS/units/extent against the study region, then add the report digest to the dataset manifest.

## Tests

    python3 -m unittest discover -s tests -p "test_inspect*.py" -v

Synthetic tests require only Python's standard library. GIS command behavior is mocked for tests, so a passing suite does not certify any real GIS file.
