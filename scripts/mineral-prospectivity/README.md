# Mineral prospectivity data preflight

Standard-library-only, read-only archive/file inspector. It hashes the original file and inventories ZIP metadata without extraction. This is not GIS parsing, dataset licensing, or mineral prospectivity inference.

## Usage

    python3 scripts/mineral-prospectivity/inspect_bundle.py /path/to/source.zip --source-id USGS-AFRICA-GIS --title "USGS Africa minerals geodatabase" --source-url https://doi.org/10.5066/P97EQWXP --rights-status unknown --rights-notes "Rights review pending" --output /tmp/usgs-africa-preflight.json

Keep rights status as "unknown" until you have verified the exact artifact's terms. The tool records your input; it does not grant or adjudicate permissions.

To optionally read each archive member for CRC/data checks, use `--verify-crc`. This performs a full decompression pass only when the archive has no unsafe paths and its advertised total expanded size/compression ratio is within hard-coded safety limits. It is not a substitute for full file/schema checks.

## Test

    python3 -m unittest discover -s tests -p test_inspect_bundle.py -v

The inspector flags unsafe paths, duplicates, case/Unicode filename collisions, large members, high compression ratios, and incomplete Shapefile sidecars. It never extracts files. It does not validate CRS, attributes, geometry, or geological meaning; use a controlled GDAL/OGR stage for that.
