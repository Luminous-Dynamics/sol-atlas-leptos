# Mineral prospectivity data tools

These tools are stages of a fail-closed ingest pipeline, not a prospectivity model.

## 1. Archive preflight (Python standard library only)

    python3 scripts/mineral-prospectivity/inspect_bundle.py /path/to/source.zip --source-id USGS-AFRICA-GIS --title "USGS Africa minerals geodatabase" --source-url https://doi.org/10.5066/P97EQWXP --rights-status unknown --rights-notes "Rights review pending" --output /tmp/usgs-africa-preflight.json

This hashes the original file and inventories ZIP metadata without extraction. It flags unsafe paths, duplicate/colliding names, suspicious sizes/compression, incomplete Shapefile sidecars, and the contents of directory-based FileGDBs such as `Africa_GIS.gdb/`. It does not extract any archive members. Optional `--verify-crc` reads archive members only when there are no unsafe paths and the advertised expanded size/compression ratio is within configured limits.

Keep rights status as "unknown" until you have checked the exact artifact and relevant component-layer terms. The tool records your declared state; it does not determine permissions.

## 2. GIS metadata inspection (GDAL/OGR required)

First manually stage only reviewed, authorized archive members in an isolated, quota-limited workspace. These commands **do not extract archives**; they only inspect a staged GIS file or a staged Esri File Geodatabase directory:

    python3 scripts/mineral-prospectivity/inspect_gis.py /data/Africa_GIS.gdb --source-id USGS-AFRICA-GIS --title "USGS Africa file geodatabase" --rights-status unknown --rights-notes "Component-layer provenance review pending" --output /tmp/africa-gdb-metadata.json

For a file layer:

    python3 scripts/mineral-prospectivity/inspect_gis.py /data/geology.gpkg --source-id SA-CGS-GEOLOGY --title "Candidate CGS geology layer" --rights-status unknown --rights-notes "Reuse terms awaiting review" --output /tmp/geology-metadata.json

The tool invokes `ogrinfo -json -ro` for vector formats and `gdalinfo -json` for raster formats. It reports vector layers, fields/schema, feature counts, CRS/extents, or raster dimensions, bands, CRS and related metadata. It also hashes the exact source file. For a `.gdb` directory, it uses a deterministic, versioned `sha256-tree-v1` digest over sorted relative paths, sizes, and per-file SHA-256 digests; this is a collection/tree digest, not a normal single-file SHA-256. Tree hashing refuses symlinks and is capped at 100,000 files and 20 GiB total.

To inspect raster tiles inside a GeoPackage, override the default vector mode:

    python3 scripts/mineral-prospectivity/inspect_gis.py /data/tiles.gpkg --kind raster --source-id EXAMPLE --title "Raster tiles" --rights-status unknown

The commands use argument arrays rather than a shell, have timeouts and an output-size cap, and do not pass update/write flags. GeoPackage files may contain vector layers or raster tiles, so select the mode that matches the actual content.

A successful GDAL/OGR open is not proof that geometry is valid, metadata complete, rights allow use, or the dataset is geologically suitable. Review warnings, compare CRS/units/extents against the study area, check per-layer source lineage, and only then create a manifest or analytical feature layer.

## Tests

    python3 -m unittest discover -s tests -p "test_inspect*.py" -v

The tests use synthetic files and mocked GDAL/OGR command results. Passing tests do not certify any real source archive or GIS dataset.
