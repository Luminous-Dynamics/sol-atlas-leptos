#!/usr/bin/env python3
"""Read-only ZIP/source preflight for mineral-prospectivity data; stdlib only.

Does not extract archive members, certify licensing, parse spatial metadata, or
calculate prospectivity. CRC/payload validation is deliberately not automatic:
reading every expanded member can be costly for very large or adversarial ZIPs.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import stat
import sys
import zipfile
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any

GEO_TYPES = {
    ".shp": "Shapefile (requires sidecars)", ".gpkg": "GeoPackage",
    ".geojson": "GeoJSON", ".json": "JSON; inspect schema",
    ".tif": "TIFF/GeoTIFF; inspect CRS and bands", ".tiff": "TIFF/GeoTIFF; inspect CRS and bands",
    ".vrt": "GDAL virtual raster", ".kml": "KML", ".gml": "GML",
    ".csv": "CSV; inspect coordinates and units", ".parquet": "Parquet; inspect geometry encoding",
    ".fgb": "FlatGeobuf", ".grd": "Grid candidate; confirm format",
}
MAX_RATIO_WARNING = 1000.0
MAX_MEMBER_WARNING_BYTES = 10 * 1024**3


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def safe_member_name(name: str) -> bool:
    if not name or "\x00" in name or "\\" in name:
        return False
    p = PurePosixPath(name)
    return not (p.is_absolute() or any(part == ".." for part in p.parts)
                or (p.parts and ":" in p.parts[0]))


def inspect_zip(path: Path) -> dict[str, Any]:
    members, warnings, unsafe, duplicates = [], [], [], []
    seen, extensions = set(), Counter()
    uncompressed = compressed = 0
    with zipfile.ZipFile(path, "r") as archive:
        for info in archive.infolist():
            name = info.filename
            if not safe_member_name(name):
                unsafe.append(name)
                continue
            if name in seen:
                duplicates.append(name)
            seen.add(name)
            if info.is_dir():
                continue
            mode = (info.external_attr >> 16) & 0xFFFF
            symlink = stat.S_ISLNK(mode)
            encrypted = bool(info.flag_bits & 1)
            suffix = Path(name).suffix.lower()
            extensions[suffix or "[no extension]"] += 1
            uncompressed += info.file_size
            compressed += info.compress_size
            members.append({
                "path": name, "bytes": info.file_size,
                "compressed_bytes": info.compress_size, "extension": suffix,
                "geospatial_candidate": GEO_TYPES.get(suffix),
                "encrypted": encrypted, "symlink": symlink,
            })
            if encrypted:
                warnings.append(f"Encrypted member not inspected: {name}")
            if symlink:
                warnings.append(f"Symlink member must not be followed during extraction: {name}")
            if info.file_size > MAX_MEMBER_WARNING_BYTES:
                warnings.append(f"Very large uncompressed member ({info.file_size} bytes): {name}")
    ratio = uncompressed / compressed if compressed else None
    if ratio is not None and ratio > MAX_RATIO_WARNING:
        warnings.append(f"High aggregate compression ratio ({ratio:.1f}); review before extraction")
    if duplicates:
        warnings.append("Duplicate archive member names make extraction ambiguous")
    if unsafe:
        warnings.append("Unsafe archive paths found; block extraction until reviewed")

    detected = sorted({m["extension"] for m in members if m["geospatial_candidate"]})
    by_base: dict[str, set[str]] = {}
    for m in members:
        if m["extension"] in {".shp", ".shx", ".dbf", ".prj", ".cpg"}:
            p = PurePosixPath(m["path"])
            by_base.setdefault(str(p.parent / p.stem.lower()), set()).add(m["extension"])
    sidecars = []
    for base, suffixes in sorted(by_base.items()):
        if ".shp" in suffixes:
            missing = sorted({".shx", ".dbf"} - suffixes)
            sidecars.append({"base": base, "present": sorted(suffixes), "missing_required": missing})
            if missing:
                warnings.append(f"Shapefile candidate missing required sidecars {missing}: {base}")

    return {
        "container_type": "zip", "members": members, "member_count": len(members),
        "unsafe_member_names": unsafe, "duplicate_member_names": sorted(set(duplicates)),
        "extensions": dict(sorted(extensions.items())), "geospatial_extension_types": detected,
        "shapefile_sidecar_checks": sidecars, "uncompressed_bytes_total": uncompressed,
        "compressed_bytes_total": compressed, "compression_ratio": ratio,
        "payload_crc_checked": False,
        "warnings": warnings,
        "preflight_status": "blocked_unsafe_paths" if unsafe else ("review_warnings" if warnings else "structurally_inspected"),
    }


def inspect_path(path: Path, source_id: str, title: str, source_url: str,
                 rights_status: str, rights_notes: str) -> dict[str, Any]:
    resolved = path.expanduser().resolve(strict=True)
    if not resolved.is_file():
        raise ValueError("Input must be a file; directory ingestion is deferred")
    if zipfile.is_zipfile(resolved):
        try:
            details = inspect_zip(resolved)
        except zipfile.BadZipFile as exc:
            raise ValueError(f"Malformed ZIP container: {exc}") from exc
    else:
        suffix = resolved.suffix.lower()
        details = {
            "container_type": "single_file", "members": [], "member_count": 0,
            "unsafe_member_names": [], "duplicate_member_names": [],
            "extensions": {suffix or "[no extension]": 1},
            "geospatial_extension_types": [suffix] if suffix in GEO_TYPES else [],
            "shapefile_sidecar_checks": [], "uncompressed_bytes_total": resolved.stat().st_size,
            "compressed_bytes_total": None, "compression_ratio": None,
            "payload_crc_checked": None,
            "warnings": ["Not a ZIP archive; file content/schema needs domain-specific inspection"],
            "preflight_status": "review_warnings",
        }
    return {
        "report_version": "1.0.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": {
            "source_id": source_id, "title": title, "canonical_url": source_url or None,
            "rights_status": rights_status, "rights_notes": rights_notes,
            "rights_interpretation": "User-supplied status only; this tool does not determine data-use permissions.",
        },
        "artifact": {"basename": resolved.name, "bytes": resolved.stat().st_size,
                     "sha256": sha256_file(resolved)},
        "inspection": details,
        "limitations": [
            "No archive member was extracted.",
            "CRCs/payloads were not fully read or validated.",
            "CRS, geometry, attributes, units and spatial coverage were not parsed.",
            "Filename extensions only suggest format; they do not prove valid geospatial content.",
            "No geological inference or prospectivity score was calculated.",
        ],
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--source-id", required=True)
    parser.add_argument("--title", default="Unspecified source artifact")
    parser.add_argument("--source-url", default="")
    parser.add_argument("--rights-status", choices=["verified", "restricted", "unknown", "not_permitted"], default="unknown")
    parser.add_argument("--rights-notes", default="Rights not independently reviewed")
    parser.add_argument("--output", type=Path, help="JSON report path; stdout if omitted")
    args = parser.parse_args(argv)
    try:
        report = inspect_path(args.input, args.source_id, args.title, args.source_url,
                              args.rights_status, args.rights_notes)
    except (OSError, ValueError, zipfile.LargeZipFile) as exc:
        print(f"preflight error: {exc}", file=sys.stderr)
        return 2
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 2 if report["inspection"]["preflight_status"] == "blocked_unsafe_paths" else 0


if __name__ == "__main__":
    raise SystemExit(main())
