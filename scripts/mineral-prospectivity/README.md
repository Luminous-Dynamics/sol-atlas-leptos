# Mineral prospectivity data preflight

Standard-library-only, read-only archive/file inspector. This is a metadata and archive-safety smoke test, not GIS parsing, dataset licensing, or mineral prospectivity inference.

## Usage

Run:

    python3 scripts/mineral-prospectivity/inspect_bundle.py /path/to/source.zip --source-id USGS-AFRICA-GIS --title "USGS Africa minerals geodatabase" --source-url https://doi.org/10.5066/P97EQWXP --rights-status unknown --rights-notes "Rights review pending" --output /tmp/usgs-africa-preflight.json

Keep rights status as "unknown" until you have verified the exact artifact's terms. The tool records your input; it does not grant or adjudicate permissions.

## Test

Run:

    python3 -m unittest discover -s tests -p test_inspect_bundle.py -v

The inspector hashes the exact input bytes, reads the ZIP central directory, lists member names, identifies candidate geospatial formats by extension, and flags unsafe paths, duplicate names, large members, high compression ratios and incomplete Shapefile sidecars. It never extracts files. It intentionally does not read and CRC-check every expanded member, to avoid unbounded decompression on very large bundles. It does not validate CRS, attributes, geometry, or geological meaning; those require a next stage such as GDAL/OGR and schema-specific checks.
