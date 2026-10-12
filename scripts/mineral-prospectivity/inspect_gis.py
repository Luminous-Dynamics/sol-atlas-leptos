#!/usr/bin/env python3
"""Read-only GIS metadata inspector using installed GDAL/OGR command-line tools."""
from __future__ import annotations
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

RASTER_EXTENSIONS = {".tif", ".tiff", ".vrt", ".img", ".asc", ".dem", ".jp2", ".nc", ".grib", ".hdf"}
VECTOR_EXTENSIONS = {".shp", ".gpkg", ".geojson", ".json", ".kml", ".gml", ".fgb", ".parquet", ".sqlite", ".gdb"}
MAX_REPORT_CHARS = 8 * 1024 * 1024


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def run_json(command: list[str], timeout_seconds: int) -> tuple[dict[str, Any], str, str]:
    completed = subprocess.run(command, shell=False, check=False, capture_output=True,
                               text=True, encoding="utf-8", errors="replace",
                               timeout=timeout_seconds)
    if completed.returncode != 0:
        details = completed.stderr.strip() or completed.stdout.strip() or "no diagnostic output"
        raise RuntimeError(f"command exited {completed.returncode}: {details[:3000]}")
    if len(completed.stdout) > MAX_REPORT_CHARS:
        raise RuntimeError(f"metadata JSON exceeds report limit ({len(completed.stdout)} characters)")
    try:
        parsed = json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"GDAL/OGR did not return valid JSON: {exc}") from exc
    if not isinstance(parsed, dict):
        raise RuntimeError("GDAL/OGR JSON root must be an object")
    return parsed, completed.stdout, completed.stderr


def summarize_vector(doc: dict[str, Any]) -> dict[str, Any]:
    layers, warnings = [], []
    raw_layers = doc.get("layers", [])
    if not isinstance(raw_layers, list) or not raw_layers:
        warnings.append("No vector layers were reported")
        raw_layers = []
    for layer in raw_layers:
        if not isinstance(layer, dict):
            warnings.append("Unrecognized layer record in OGR metadata")
            continue
        geometry_fields = []
        raw_geometry_fields = layer.get("geometryFields", [])
        for gf in raw_geometry_fields if isinstance(raw_geometry_fields, list) else []:
            if not isinstance(gf, dict):
                continue
            crs = gf.get("coordinateSystem") if isinstance(gf.get("coordinateSystem"), dict) else {}
            projjson = crs.get("projjson") if isinstance(crs.get("projjson"), dict) else {}
            geometry_fields.append({
                "name": gf.get("name"), "type": gf.get("type"), "extent": gf.get("extent"),
                "crs_wkt": crs.get("wkt"), "crs_name": projjson.get("name"),
            })
        fields = []
        raw_fields = layer.get("fields", [])
        for field in raw_fields if isinstance(raw_fields, list) else []:
            if isinstance(field, dict):
                fields.append({k: field.get(k) for k in
                               ("name", "type", "subType", "width", "precision", "nullable", "unique") if k in field})
        if geometry_fields and all(not g.get("crs_wkt") for g in geometry_fields):
            warnings.append(f"Layer {layer.get('name', '[unnamed]')} has no reported CRS WKT")
        layers.append({
            "name": layer.get("name"), "feature_count": layer.get("featureCount"),
            "geometry_fields": geometry_fields, "fields": fields,
            "layer_extent": layer.get("extent"),
            "metadata_domains": sorted((layer.get("metadata") or {}).keys())
                if isinstance(layer.get("metadata"), dict) else [],
        })
    return {
        "driver_short_name": doc.get("driverShortName"), "driver_long_name": doc.get("driverLongName"),
        "description": doc.get("description"), "layer_count": len(layers),
        "layers": layers, "warnings": warnings,
    }


def summarize_raster(doc: dict[str, Any]) -> dict[str, Any]:
    size = doc.get("size")
    crs = doc.get("coordinateSystem") if isinstance(doc.get("coordinateSystem"), dict) else {}
    projjson = crs.get("projjson") if isinstance(crs.get("projjson"), dict) else {}
    bands, warnings = [], []
    for band in doc.get("bands", []) if isinstance(doc.get("bands", []), list) else []:
        if isinstance(band, dict):
            bands.append({k: band.get(k) for k in
                          ("band", "type", "colorInterpretation", "block", "noDataValue",
                           "mask", "overviews", "unit", "scale", "offset") if k in band})
    if not crs:
        warnings.append("Raster has no reported coordinate system")
    if not size or not isinstance(size, list) or len(size) != 2:
        warnings.append("Raster dimensions are missing or malformed")
    if not bands:
        warnings.append("No raster bands were reported")
    return {
        "driver_short_name": doc.get("driverShortName"), "driver_long_name": doc.get("driverLongName"),
        "description": doc.get("description"), "size": size, "geo_transform": doc.get("geoTransform"),
        "crs_wkt": crs.get("wkt"), "crs_name": projjson.get("name"),
        "corner_coordinates": doc.get("cornerCoordinates"), "bands": bands,
        "band_count": len(bands), "warnings": warnings,
    }


def inspect_gis(path: Path, source_id: str, title: str, source_url: str, rights_status: str,
                rights_notes: str, timeout_seconds: int = 120, kind_override: str = "auto") -> dict[str, Any]:
    path = path.expanduser().resolve(strict=True)
    if not path.is_file():
        raise ValueError("Input must be a single GIS data file")
    suffix = path.suffix.lower()
    if kind_override not in {"auto", "vector", "raster"}:
        raise ValueError(f"Unsupported inspection kind: {kind_override}")
    if kind_override == "raster" or (kind_override == "auto" and suffix in RASTER_EXTENSIONS):
        tool, kind = shutil.which("gdalinfo"), "raster"
        command = [tool, "-json", "-nomd", str(path)] if tool else None
    elif kind_override == "vector" or (kind_override == "auto" and suffix in VECTOR_EXTENSIONS):
        tool, kind = shutil.which("ogrinfo"), "vector"
        command = [tool, "-json", "-ro", str(path)] if tool else None
    else:
        raise ValueError(f"Unsupported/ambiguous GIS extension: {suffix or '[none]'}; pass --kind if appropriate")
    if command is None:
        raise FileNotFoundError(f"Required GDAL/OGR tool missing: {'gdalinfo' if kind == 'raster' else 'ogrinfo'}")
    try:
        ver = subprocess.run([tool, "--version"], shell=False, check=False, capture_output=True,
                             text=True, encoding="utf-8", errors="replace", timeout=10)
        version = ver.stdout.strip() or ver.stderr.strip() or "unknown"
    except (OSError, subprocess.TimeoutExpired):
        version = "unknown"
    doc, _, stderr = run_json(command, timeout_seconds)
    summary = summarize_raster(doc) if kind == "raster" else summarize_vector(doc)
    warnings = list(summary.pop("warnings", []))
    if stderr.strip():
        warnings.append("GDAL/OGR emitted stderr warnings; inspect the diagnostic excerpt")
    return {
        "report_version": "1.0.0", "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": {"source_id": source_id, "title": title, "canonical_url": source_url or None,
                   "rights_status": rights_status, "rights_notes": rights_notes,
                   "rights_interpretation": "User-supplied status only; this tool does not determine permissions."},
        "artifact": {"basename": path.name, "bytes": path.stat().st_size, "sha256": sha256_file(path)},
        "inspection": {"kind": kind, "tool": tool, "tool_version": version,
                       "status": "review_warnings" if warnings else "metadata_inspected",
                       "metadata": summary, "warnings": warnings,
                       "diagnostic_stderr_excerpt": stderr.strip()[:2000] if stderr.strip() else None},
        "limitations": [
            "Read-only metadata inspection; no features/pixels were exported or altered.",
            "A successful GDAL/OGR open does not prove scientific suitability, licensing, or completeness.",
            "Feature counts/extents/CRS may be absent or driver-dependent; inspect source metadata.",
            "GeoPackage files may contain vectors or raster tiles; auto mode inspects vectors, and --kind raster can inspect raster tiles.",
            "No prospectivity score or geological interpretation was calculated.",
        ],
    }


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("input", type=Path)
    p.add_argument("--source-id", required=True)
    p.add_argument("--title", default="Unspecified source artifact")
    p.add_argument("--source-url", default="")
    p.add_argument("--rights-status", choices=["verified", "restricted", "unknown", "not_permitted"], default="unknown")
    p.add_argument("--rights-notes", default="Rights not independently reviewed")
    p.add_argument("--timeout-seconds", type=int, default=120)
    p.add_argument("--kind", choices=["auto", "vector", "raster"], default="auto", help="Force GDAL/OGR interface; use raster for raster-tile GeoPackages")
    p.add_argument("--output", type=Path)
    args = p.parse_args(argv)
    if not 1 <= args.timeout_seconds <= 3600:
        p.error("--timeout-seconds must be in [1, 3600]")
    try:
        report = inspect_gis(args.input, args.source_id, args.title, args.source_url,
                             args.rights_status, args.rights_notes, args.timeout_seconds, args.kind)
    except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired) as exc:
        print(f"GIS inspection blocked: {exc}", file=sys.stderr)
        return 2
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 0 if report["inspection"]["status"] == "metadata_inspected" else 1


if __name__ == "__main__":
    raise SystemExit(main())
