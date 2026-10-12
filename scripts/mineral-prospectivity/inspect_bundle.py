#!/usr/bin/env python3
"""Read-only, bounded preflight for mineral-prospectivity source bundles.

This inventories archive metadata and hashes the source file. It does not
extract members, determine licensing, parse GIS geometry/CRS, or predict ores.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import stat
import sys
import unicodedata
import zipfile
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any

GEO_TYPES = {
    ".shp": "Shapefile; required sidecars must be present",
    ".gpkg": "GeoPackage; validate with GDAL/OGR",
    ".geojson": "GeoJSON", ".json": "JSON; inspect schema",
    ".tif": "TIFF/GeoTIFF; inspect CRS and bands",
    ".tiff": "TIFF/GeoTIFF; inspect CRS and bands",
    ".vrt": "GDAL virtual raster", ".kml": "KML", ".gml": "GML",
    ".csv": "CSV; inspect coordinates and units",
    ".parquet": "Parquet; inspect geometry encoding",
    ".fgb": "FlatGeobuf", ".grd": "Grid candidate; confirm format",
}
MAX_RATIO_WARNING = 200.0
MAX_MEMBER_WARNING_BYTES = 2 * 1024**3
MAX_TOTAL_WARNING_BYTES = 8 * 1024**3
MAX_ENTRIES_WARNING = 250_000


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def safe_member_name(name: str) -> bool:
    """Conservative cross-platform archive path check; never extracts anything."""
    if not name or "\x00" in name or "\\" in name:
        return False
    p = PurePosixPath(name)
    if p.is_absolute() or any(part in {"..", "."} for part in p.parts):
        return False
    if p.parts and (":" in p.parts[0] or p.parts[0].startswith("~")):
        return False
    return True


def inspect_zip(path: Path, verify_crc: bool = False) -> dict[str, Any]:
    members: list[dict[str, Any]] = []
    warnings: list[str] = []
    unsafe: list[str] = []
    duplicate_names: list[str] = []
    portable_collisions: list[dict[str, str]] = []
    seen: set[str] = set()
    portable_seen: dict[str, str] = {}
    extensions: Counter[str] = Counter()
    total_uncompressed = total_compressed = 0

    with zipfile.ZipFile(path, "r") as archive:
        infos = archive.infolist()
        if not infos:
            warnings.append("Archive contains no entries")
        if len(infos) > MAX_ENTRIES_WARNING:
            warnings.append(f"Large central directory ({len(infos)} entries); review memory/metadata costs")
        for info in infos:
            name = info.filename
            # Size totals include unsafe entries too; do not under-report a bundle.
            if not info.is_dir():
                total_uncompressed += info.file_size
                total_compressed += info.compress_size
            if not safe_member_name(name):
                unsafe.append(name)
                continue
            if name in seen:
                duplicate_names.append(name)
            seen.add(name)
            portable_key = unicodedata.normalize("NFKC", name).casefold()
            prior = portable_seen.get(portable_key)
            if prior is not None and prior != name:
                portable_collisions.append({"first": prior, "second": name})
            else:
                portable_seen[portable_key] = name
            if info.is_dir():
                continue
            mode = (info.external_attr >> 16) & 0xFFFF
            is_symlink = stat.S_ISLNK(mode)
            encrypted = bool(info.flag_bits & 1)
            ext = Path(name).suffix.lower()
            extensions[ext or "[no extension]"] += 1
            members.append({
                "path": name, "bytes": info.file_size,
                "compressed_bytes": info.compress_size, "extension": ext,
                "geospatial_candidate": GEO_TYPES.get(ext),
                "encrypted": encrypted, "symlink": is_symlink,
            })
            if encrypted:
                warnings.append(f"Encrypted member not inspected: {name}")
            if is_symlink:
                warnings.append(f"Symlink member blocks future safe extraction planning: {name}")
            if info.file_size > MAX_MEMBER_WARNING_BYTES:
                warnings.append(f"Member exceeds review size threshold ({info.file_size} bytes): {name}")

        ratio = total_uncompressed / total_compressed if total_compressed else None
        if ratio is not None and ratio > MAX_RATIO_WARNING:
            warnings.append(f"High aggregate compression ratio ({ratio:.1f}); review before any decompression")
        if total_uncompressed > MAX_TOTAL_WARNING_BYTES:
            warnings.append(f"Aggregate uncompressed size exceeds review threshold ({total_uncompressed} bytes)")
        if duplicate_names:
            warnings.append("Duplicate archive member names make later extraction ambiguous")
        if portable_collisions:
            warnings.append("Case/Unicode-normalized filename collisions may not round-trip across filesystems")
        if unsafe:
            warnings.append("Unsafe archive paths found; archive must not be extracted before manual review")

    sidecars_by_base: dict[str, set[str]] = {}
    for member in members:
        if member["extension"] in {".shp", ".shx", ".dbf", ".prj", ".cpg"}:
            p = PurePosixPath(member["path"])
            key = str(p.parent / p.stem.casefold())
            sidecars_by_base.setdefault(key, set()).add(member["extension"])
    sidecar_results = []
    for base, suffixes in sorted(sidecars_by_base.items()):
        if ".shp" in suffixes:
            missing = sorted({".shx", ".dbf"} - suffixes)
            sidecar_results.append({"base": base, "present": sorted(suffixes), "missing_required": missing})
            if missing:
                warnings.append(f"Shapefile candidate missing required sidecars {missing}: {base}")

    crc_status = "not_requested"
    if verify_crc:
        # Never invoke the decompression pass on bundles whose metadata already
        # indicates unbounded or oversized expanded content.
        if unsafe or total_uncompressed > MAX_TOTAL_WARNING_BYTES or (ratio is not None and ratio > MAX_RATIO_WARNING):
            crc_status = "skipped_safety_limits"
            warnings.append("CRC/payload scan skipped because archive metadata exceeded safety limits")
        else:
            with zipfile.ZipFile(path, "r") as archive:
                bad_member = archive.testzip()
            if bad_member is None:
                crc_status = "passed"
            else:
                crc_status = "failed"
                warnings.append(f"CRC/data check failed at member: {bad_member}")

    status = "blocked_unsafe_paths" if unsafe else ("review_warnings" if warnings else "structurally_inspected")
    if crc_status == "failed":
        status = "blocked_integrity_failure"
    return {
        "container_type": "zip", "member_count": len(members), "members": members,
        "unsafe_member_names": unsafe,
        "duplicate_member_names": sorted(set(duplicate_names)),
        "portable_filename_collisions": portable_collisions,
        "extensions": dict(sorted(extensions.items())),
        "geospatial_extension_types": sorted({m["extension"] for m in members if m["geospatial_candidate"]}),
        "shapefile_sidecar_checks": sidecar_results,
        "uncompressed_bytes_total": total_uncompressed,
        "compressed_bytes_total": total_compressed, "compression_ratio": ratio,
        "payload_crc_status": crc_status, "warnings": warnings, "preflight_status": status,
    }


def inspect_path(path: Path, source_id: str, title: str, source_url: str,
                 rights_status: str, rights_notes: str, verify_crc: bool = False) -> dict[str, Any]:
    resolved = path.expanduser().resolve(strict=True)
    if not resolved.is_file():
        raise ValueError("Input must be a file; directory ingestion is deferred")
    if zipfile.is_zipfile(resolved):
        try:
            details = inspect_zip(resolved, verify_crc=verify_crc)
        except zipfile.BadZipFile as exc:
            raise ValueError(f"Malformed ZIP container: {exc}") from exc
    elif resolved.suffix.lower() == ".zip":
        raise ValueError("File has a .zip extension but is not a valid ZIP container")
    else:
        suffix = resolved.suffix.lower()
        details = {
            "container_type": "single_file", "members": [], "member_count": 0,
            "unsafe_member_names": [], "duplicate_member_names": [],
            "portable_filename_collisions": [],
            "extensions": {suffix or "[no extension]": 1},
            "geospatial_extension_types": [suffix] if suffix in GEO_TYPES else [],
            "shapefile_sidecar_checks": [], "uncompressed_bytes_total": resolved.stat().st_size,
            "compressed_bytes_total": None, "compression_ratio": None,
            "payload_crc_status": "not_applicable",
            "warnings": ["Not a ZIP archive; file content/schema needs domain-specific inspection"],
            "preflight_status": "review_warnings",
        }
    return {
        "report_version": "1.1.0", "generated_at": datetime.now(timezone.utc).isoformat(),
        "source": {
            "source_id": source_id, "title": title, "canonical_url": source_url or None,
            "rights_status": rights_status, "rights_notes": rights_notes,
            "rights_interpretation": "User-supplied status only; this tool does not determine data-use permissions.",
        },
        "artifact": {"basename": resolved.name, "bytes": resolved.stat().st_size, "sha256": sha256_file(resolved)},
        "inspection": details,
        "limitations": [
            "No archive member was extracted.",
            "CRC/payload validation is performed only when explicitly requested and within safety limits.",
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
    parser.add_argument("--verify-crc", action="store_true", help="Read all ZIP members for CRC checks only within configured size/ratio safety limits")
    parser.add_argument("--output", type=Path, help="JSON report path; stdout if omitted")
    args = parser.parse_args(argv)
    try:
        report = inspect_path(args.input, args.source_id, args.title, args.source_url,
                              args.rights_status, args.rights_notes, args.verify_crc)
    except (OSError, ValueError, zipfile.BadZipFile, zipfile.LargeZipFile) as exc:
        print(f"preflight error: {exc}", file=sys.stderr)
        return 2
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 2 if report["inspection"]["preflight_status"].startswith("blocked_") else 0


if __name__ == "__main__":
    raise SystemExit(main())
