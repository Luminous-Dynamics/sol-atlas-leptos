#!/usr/bin/env python3
"""Create a local SHA-256 receipt for an explicitly catalogued lunar asset.

This helper does not download data and does not authenticate the server origin.
It records the identity of local bytes against a reviewed catalogue URL while
making those limitations explicit. It is intentionally standard-library only.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CATALOG = ROOT / "docs/data/lunar_geospatial_source_catalog.v1.json"
RECEIPT_SCHEMA = "org.luminousdynamics.sol-atlas.lunar-asset-receipt/v1"
CHUNK_SIZE = 1024 * 1024


class ReceiptError(ValueError):
    """The requested local asset cannot be safely bound to the source catalogue."""


def _sha256_file(path: Path) -> tuple[str, int, int, int]:
    """Hash one opened regular file and reject path/descriptor changes during capture."""
    try:
        path_before = path.lstat()
    except OSError as exc:
        raise ReceiptError(f"cannot inspect local input path: {exc}") from exc
    if stat.S_ISLNK(path_before.st_mode):
        raise ReceiptError("symbolic-link inputs are not accepted; hash the reviewed regular file directly")
    if not stat.S_ISREG(path_before.st_mode):
        raise ReceiptError("local input must be a regular file, not a directory or special file")

    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        file_descriptor = os.open(path, flags)
    except OSError as exc:
        raise ReceiptError(f"cannot securely open local input for hashing: {exc}") from exc

    def fingerprint(info: os.stat_result) -> tuple[int, int, int, int, int]:
        return (
            info.st_dev,
            info.st_ino,
            info.st_size,
            info.st_mtime_ns,
            info.st_ctime_ns,
        )

    with os.fdopen(file_descriptor, "rb") as handle:
        before = os.fstat(handle.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise ReceiptError("opened input is not a regular file")
        if (path_before.st_dev, path_before.st_ino) != (before.st_dev, before.st_ino):
            raise ReceiptError("file path changed before hashing; retry with a stable input")

        digest = hashlib.sha256()
        byte_count = 0
        while chunk := handle.read(CHUNK_SIZE):
            digest.update(chunk)
            byte_count += len(chunk)

        after = os.fstat(handle.fileno())
        try:
            path_after = path.lstat()
        except OSError as exc:
            raise ReceiptError(f"file path changed while hashing: {exc}") from exc
        if stat.S_ISLNK(path_after.st_mode) or not stat.S_ISREG(path_after.st_mode):
            raise ReceiptError("file path changed type while hashing; retry with a stable input")
        if fingerprint(before) != fingerprint(after) or fingerprint(after) != fingerprint(path_after):
            raise ReceiptError("file changed or was replaced while hashing; retry with a stable input")
        if byte_count <= 0:
            raise ReceiptError("empty input files cannot receive a valid asset receipt")
        if byte_count != after.st_size:
            raise ReceiptError("bytes read do not match the final file size; retry with a stable input")

    return digest.hexdigest(), byte_count, after.st_mtime_ns, after.st_ino

def build_receipt(
    catalog: dict[str, Any],
    catalog_path: Path,
    asset_path: Path,
    asset_url: str,
    *,
    captured_utc: str | None = None,
) -> dict[str, Any]:
    """Return a receipt after matching the exact URL and hashing local bytes."""
    if not isinstance(catalog, dict):
        raise ReceiptError("source catalog must be a JSON object")
    try:
        catalog_bytes = catalog_path.read_bytes()
        on_disk_catalog = json.loads(catalog_bytes)
    except (OSError, json.JSONDecodeError) as exc:
        raise ReceiptError(f"cannot read the reviewed source catalog: {exc}") from exc
    if on_disk_catalog != catalog:
        raise ReceiptError("source catalog changed after parsing; reload it before recording a receipt")
    matches: list[tuple[dict[str, Any], dict[str, Any]]] = []
    for source in catalog.get("sources", []):
        if not isinstance(source, dict):
            continue
        for asset in source.get("sample_direct_assets", []):
            if isinstance(asset, dict) and asset.get("url") == asset_url:
                matches.append((source, asset))
    if len(matches) != 1:
        raise ReceiptError(
            "asset URL must match exactly one sample_direct_assets entry in the reviewed catalog"
        )

    source, asset = matches[0]
    state = asset.get("verification_state", "")
    if not isinstance(state, str) or not state:
        raise ReceiptError("catalog asset has no explicit verification_state")
    if "404" in state.lower():
        raise ReceiptError(
            "catalog records this URL as returning 404; use a listed fallback asset instead"
        )
    if not asset_path.exists() or not asset_path.is_file():
        raise ReceiptError("local input path must be an existing regular file")

    asset_hash, byte_count, mtime_ns, inode = _sha256_file(asset_path)
    catalog_hash = hashlib.sha256(catalog_bytes).hexdigest()
    try:
        catalog_display_path = catalog_path.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        catalog_display_path = catalog_path.name
    if captured_utc is None:
        captured_utc = datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    try:
        timestamp = datetime.fromisoformat(captured_utc.replace("Z", "+00:00"))
    except ValueError as exc:
        raise ReceiptError("captured_utc must be an ISO-8601 timestamp") from exc
    if timestamp.tzinfo is None:
        raise ReceiptError("captured_utc must include a timezone")
    captured_utc = timestamp.astimezone(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")

    return {
        "schema": RECEIPT_SCHEMA,
        "recorded_utc": captured_utc,
        "source_catalog": {
            "path": catalog_display_path,
            "sha256": catalog_hash,
            "schema": catalog.get("schema"),
        },
        "source": {
            "id": source.get("id"),
            "title": source.get("title"),
            "page_url": source.get("url"),
        },
        "asset": {
            "title": asset.get("title"),
            "url": asset_url,
            "asset_type": asset.get("asset_type"),
            "catalog_verification_state_before_receipt": state,
            "local_file_name": asset_path.name,
            "byte_count": byte_count,
            "sha256": asset_hash,
            "observed_mtime_ns_during_capture": mtime_ns,
            "observed_inode_during_capture": inode,
        },
        "verification_boundary": {
            "local_bytes_hashed": True,
            "url_origin_authenticated": False,
            "pds_label_parsed": False,
            "raster_metadata_verified": False,
            "coordinate_frame_verified": False,
            "product_semantics_verified": False,
            "site_metrics_computed": False,
            "statement": (
                "SHA-256 identifies the local file bytes only. This receipt does not prove "
                "that the bytes came from the asserted URL, that a PDS label was interpreted, "
                "or that raster calibration, coordinate reference, units, uncertainty, or "
                "site-level measurements have been verified."
            ),
        },
    }


def _atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            newline="\n",
            dir=path.parent,
            prefix=f".{path.name}.",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temp_name = handle.name
            json.dump(payload, handle, indent=2, ensure_ascii=False)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, path)
    except OSError:
        if temp_name:
            try:
                os.unlink(temp_name)
            except OSError:
                pass
        raise


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset-url", required=True, help="Exact URL listed in sample_direct_assets")
    parser.add_argument("--file", required=True, type=Path, help="Already-downloaded local asset")
    parser.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG, help="Reviewed catalog JSON")
    parser.add_argument("--output", type=Path, help="Write a receipt file atomically instead of stdout")
    parser.add_argument("--captured-utc", help="Optional fixed ISO-8601 time for reproducible tests")
    args = parser.parse_args(argv)

    try:
        catalog_bytes = args.catalog.read_bytes()
        catalog = json.loads(catalog_bytes)
        receipt = build_receipt(
            catalog,
            args.catalog,
            args.file,
            args.asset_url,
            captured_utc=args.captured_utc,
        )
        if args.output:
            if args.output.resolve() == args.file.resolve():
                raise ReceiptError("receipt output path must not overwrite the source asset")
            _atomic_write_json(args.output, receipt)
            print(f"Receipt written: {args.output}")
        else:
            print(json.dumps(receipt, indent=2, ensure_ascii=False))
    except (OSError, json.JSONDecodeError, ReceiptError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
