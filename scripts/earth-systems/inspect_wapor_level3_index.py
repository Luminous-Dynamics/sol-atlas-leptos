#!/usr/bin/env python3
"""Capture and inventory a WaPOR v3 Level III MOSAICSET directory index.

This tool captures public HTML metadata only. It does not download raster or
sidecar JSON files, and does not assert scientific validity or reuse rights.
"""
from __future__ import annotations

import argparse
import hashlib
import html.parser
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import parse_qs, urlencode, unquote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

INDEX_URL = "https://gismgr.fao.org/DATA/WAPOR-3/MOSAICSET/index.html"
INDEX_HOST = "gismgr.fao.org"
INDEX_PATH = "/DATA/WAPOR-3/MOSAICSET/index.html"
MAX_INDEX_BYTES = 16 * 1024 * 1024
DEFAULT_TIMEOUT_SECONDS = 20
MAPSET_CODE_RE = re.compile(r"^L3-[A-Z0-9]+-[A-Z]$")
AREA_CODE_RE = re.compile(r"^[A-Z0-9]{3,4}$")
SAFE_FILENAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,255}$")


class IndexError(ValueError):
    """Raised when the directory index cannot be interpreted safely."""


def prefix_for(mapset_code: str) -> str:
    if not MAPSET_CODE_RE.fullmatch(mapset_code):
        raise IndexError("mapset code must be a Level III code such as L3-AETI-M")
    return f"DATA/WAPOR-3/MOSAICSET/{mapset_code}"


def validate_index_url(url: str, prefix: str) -> None:
    """Require the exact official HTTPS index endpoint and requested prefix."""
    try:
        parts = urlsplit(url)
        query = parse_qs(parts.query, keep_blank_values=True, strict_parsing=True)
    except (TypeError, ValueError) as exc:
        raise IndexError("malformed MOSAICSET index URL") from exc
    if (
        parts.scheme != "https"
        or parts.netloc != INDEX_HOST
        or parts.path != INDEX_PATH
        or parts.username is not None
        or parts.password is not None
        or parts.fragment
        or query != {"prefix": [prefix]}
    ):
        raise IndexError("MOSAICSET index URL escaped the approved HTTPS origin/path/prefix")


class SameIndexRedirectHandler(HTTPRedirectHandler):
    """Allow redirects only to the same index and exact prefix."""

    def __init__(self, prefix: str) -> None:
        super().__init__()
        self.prefix = prefix

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        validate_index_url(newurl, self.prefix)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


class MosaicsetIndexParser(html.parser.HTMLParser):
    """Extract mapset files from one server-rendered directory listing."""

    def __init__(self, mapset_code: str, requested_area: str) -> None:
        super().__init__(convert_charrefs=True)
        self.mapset_code = mapset_code
        self.requested_area = requested_area
        self.files: dict[str, dict] = {}
        self.errors: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag.lower() != "a":
            return
        href = dict(attrs).get("href")
        if not isinstance(href, str) or not href:
            return

        try:
            parts = urlsplit(href)
        except ValueError:
            if f"WAPOR-3.{self.mapset_code}." in href:
                self.errors.append(f"malformed matching file link: {href!r}")
            return

        filename = unquote(parts.path)
        if "/" in filename:
            filename = filename.rsplit("/", 1)[-1]
        product_prefix = f"WAPOR-3.{self.mapset_code}."
        if not filename.startswith(product_prefix):
            return
        if (
            parts.scheme or parts.netloc or parts.query or parts.fragment
            or parts.path != filename
            or not SAFE_FILENAME_RE.fullmatch(filename)
            or filename in {".", ".."}
        ):
            self.errors.append(f"unsafe matching Level III file link: {href!r}")
            return
        fields = filename.split(".")
        # Expected family: WAPOR-3.<mapset>.<area>.<period>.<extension>.
        # Period spelling is retained verbatim because it varies by temporal
        # resolution and must not be normalized from guesses.
        if len(fields) < 5 or fields[0] != "WAPOR-3" or fields[1] != self.mapset_code:
            self.errors.append(f"unexpected matching Level III filename: {filename!r}")
            return
        area_code = fields[2]
        extension = fields[-1].lower()
        if not AREA_CODE_RE.fullmatch(area_code) or extension not in {"tif", "json"}:
            self.errors.append(f"unexpected area code or file type in {filename!r}")
            return
        if filename in self.files:
            self.errors.append(f"duplicate directory entry for {filename!r}")
            return
        self.files[filename] = {
            "filename": filename,
            "area_code": area_code,
            "period_token": ".".join(fields[3:-1]),
            "kind": "raster" if extension == "tif" else "sidecar_metadata",
            "url": "https://gismgr.fao.org/" + prefix_for(self.mapset_code) + "/" + filename,
        }


def read_index(
    index_file: Path | None, timeout_seconds: float, prefix: str
) -> tuple[bytes, str | None, str]:
    if index_file is not None:
        try:
            if not index_file.is_file():
                raise IndexError("local index path must be a regular file")
            if index_file.stat().st_size > MAX_INDEX_BYTES:
                raise IndexError(f"local index exceeds {MAX_INDEX_BYTES} byte limit")
            with index_file.open("rb") as handle:
                raw = handle.read(MAX_INDEX_BYTES + 1)
        except IndexError:
            raise
        except OSError as exc:
            raise IndexError(f"cannot read local MOSAICSET index: {exc}") from exc
        if len(raw) > MAX_INDEX_BYTES:
            raise IndexError(f"local index exceeds {MAX_INDEX_BYTES} byte limit")
        return raw, None, "local_html_snapshot"

    url = INDEX_URL + "?" + urlencode({"prefix": prefix})
    validate_index_url(url, prefix)
    request = Request(
        url,
        headers={"User-Agent": "Sol-Atlas-WaPOR-L3-Index-Inventory/1.0"},
        method="GET",
    )
    try:
        opener = build_opener(SameIndexRedirectHandler(prefix))
        with opener.open(request, timeout=timeout_seconds) as response:
            final_url = response.geturl()
            validate_index_url(final_url, prefix)
            content_type = response.headers.get("Content-Type", "").lower()
            if "text/html" not in content_type and "application/xhtml+xml" not in content_type:
                raise IndexError(f"expected HTML directory listing, got {content_type!r}")
            declared_length = response.headers.get("Content-Length")
            if declared_length:
                try:
                    if int(declared_length) > MAX_INDEX_BYTES:
                        raise IndexError(f"remote index exceeds {MAX_INDEX_BYTES} byte limit")
                except ValueError as exc:
                    raise IndexError("invalid index Content-Length header") from exc
            raw = response.read(MAX_INDEX_BYTES + 1)
            if len(raw) > MAX_INDEX_BYTES:
                raise IndexError(f"remote index exceeds {MAX_INDEX_BYTES} byte limit")
    except IndexError:
        raise
    except Exception as exc:
        raise IndexError(f"failed to retrieve official WaPOR MOSAICSET index: {exc}") from exc
    retrieved_at = datetime.now(timezone.utc).isoformat(timespec="seconds")
    return raw, retrieved_at, "https_mosaicset_directory_index"


def build_inventory(
    raw: bytes,
    mapset_code: str,
    area_code: str,
    retrieved_at: str | None,
    mode: str,
) -> dict:
    if len(raw) > MAX_INDEX_BYTES:
        raise IndexError(f"index exceeds {MAX_INDEX_BYTES} byte limit")
    prefix_for(mapset_code)
    if not AREA_CODE_RE.fullmatch(area_code):
        raise IndexError("area code must be 3 or 4 uppercase alphanumeric characters")
    try:
        document = raw.decode("utf-8-sig")
    except UnicodeDecodeError as exc:
        raise IndexError("MOSAICSET index is not valid UTF-8 HTML") from exc

    parser = MosaicsetIndexParser(mapset_code, area_code)
    try:
        parser.feed(document)
        parser.close()
    except Exception as exc:
        raise IndexError(f"failed to parse MOSAICSET directory listing: {exc}") from exc
    if parser.errors:
        raise IndexError("; ".join(parser.errors))

    all_files = sorted(parser.files.values(), key=lambda item: item["filename"])
    area_files = [item for item in all_files if item["area_code"] == area_code]
    raster_files = [item for item in area_files if item["kind"] == "raster"]
    sidecars = [item for item in area_files if item["kind"] == "sidecar_metadata"]
    per_area: dict[str, dict[str, int]] = {}
    for item in all_files:
        counts = per_area.setdefault(item["area_code"], {"raster_files": 0, "sidecar_files": 0})
        counts["raster_files" if item["kind"] == "raster" else "sidecar_files"] += 1

    return {
        "manifest_version": "1.0.0",
        "status": "level3_mosaicset_index_inventory_only",
        "source_provider": "FAO WaPOR version 3",
        "source_url": INDEX_URL + "?" + urlencode({"prefix": prefix_for(mapset_code)}),
        "source_prefix": prefix_for(mapset_code),
        "index_snapshot_mode": mode,
        "index_snapshot_saved": False,
        "index_snapshot_filename": None,
        "index_retrieved_at_utc": retrieved_at,
        "index_sha256": hashlib.sha256(raw).hexdigest(),
        "index_bytes": len(raw),
        "mapset_code": mapset_code,
        "requested_area_code": area_code,
        "directory_entry_count": len(all_files),
        "area_codes_observed": sorted(per_area),
        "file_counts_by_area": per_area,
        "requested_area_raster_count": len(raster_files),
        "requested_area_sidecar_count": len(sidecars),
        "requested_area_files": area_files,
        "raster_files_downloaded": 0,
        "sidecar_json_files_downloaded": 0,
        "claim_ceiling": [
            "This manifest inventories a directory index only; it does not download raster or sidecar metadata bytes.",
            "An indexed filename does not prove that the file is retrievable, valid, final, or legally reusable.",
            "A directory timestamp or period token is not a substitute for raster metadata about temporal support, units, scaling, quality, or correction vintage.",
            "The selected area code identifies the intended Level III region only when matched to authoritative FAO metadata.",
            "SHA-256 identifies exact index bytes, not the scientific truth of the data.",
        ],
    }


def remove_created_file(path: Path) -> str:
    try:
        path.unlink()
    except FileNotFoundError:
        return ""
    except OSError as exc:
        return f"; also could not remove partial artifact {path.name}: {exc}"
    return f"; removed partial artifact {path.name}"


def save_snapshot(raw: bytes, path: Path) -> None:
    created = False
    try:
        with path.open("xb") as handle:
            created = True
            written = handle.write(raw)
            if written != len(raw):
                raise OSError(f"short write: captured {written} of {len(raw)} bytes")
    except FileExistsError as exc:
        raise IndexError(f"snapshot path already exists; refusing to overwrite: {path.name}") from exc
    except OSError as exc:
        cleanup = remove_created_file(path) if created else ""
        raise IndexError(f"cannot save exact index bytes: {exc}{cleanup}") from exc


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mapset-code", default="L3-AETI-M", help="exact Level III mapset code")
    parser.add_argument("--area-code", default="MBL", help="exact Level III area code (FAO lists MBL for Baixo Limpopo)")
    parser.add_argument("--index-file", type=Path, help="inspect a saved HTML index instead of the network")
    parser.add_argument("--snapshot-index", type=Path, help="save exact index bytes to a new file")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SECONDS, help="HTTPS timeout in seconds")
    parser.add_argument("--output", type=Path, help="write JSON inventory to a new file; default is stdout")
    args = parser.parse_args(argv)

    if not 0 < args.timeout <= 120:
        parser.error("--timeout must be greater than 0 and no more than 120 seconds")
    if args.output is not None and (args.output.exists() or args.output.is_symlink()):
        print("output path already exists; refusing to overwrite", file=sys.stderr)
        return 2
    if args.snapshot_index is not None and (args.snapshot_index.exists() or args.snapshot_index.is_symlink()):
        print("snapshot path already exists; refusing to overwrite", file=sys.stderr)
        return 2
    if args.output is not None and args.snapshot_index is not None:
        try:
            if args.output.resolve() == args.snapshot_index.resolve():
                parser.error("--output and --snapshot-index must be different paths")
        except OSError:
            parser.error("cannot resolve --output or --snapshot-index")

    try:
        prefix = prefix_for(args.mapset_code)
        raw, retrieved_at, mode = read_index(args.index_file, args.timeout, prefix)
        inventory = build_inventory(raw, args.mapset_code, args.area_code, retrieved_at, mode)
        if args.snapshot_index is not None:
            save_snapshot(raw, args.snapshot_index)
            inventory["index_snapshot_saved"] = True
            inventory["index_snapshot_filename"] = args.snapshot_index.name
    except (IndexError, ValueError) as exc:
        print(f"WaPOR Level III inventory failed: {exc}", file=sys.stderr)
        return 2

    rendered = json.dumps(inventory, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    if args.output is None:
        try:
            written = sys.stdout.write(rendered)
            if written != len(rendered):
                raise OSError("short stdout write")
        except OSError as exc:
            cleanup = (
                remove_created_file(args.snapshot_index)
                if args.snapshot_index is not None and inventory["index_snapshot_saved"] else ""
            )
            print(f"cannot write inventory to stdout: {exc}{cleanup}", file=sys.stderr)
            return 2
    else:
        created = False
        try:
            with args.output.open("x", encoding="utf-8") as handle:
                created = True
                written = handle.write(rendered)
                if written != len(rendered):
                    raise OSError(f"short write: emitted {written} of {len(rendered)} characters")
        except FileExistsError:
            cleanup = (
                remove_created_file(args.snapshot_index)
                if args.snapshot_index is not None and inventory["index_snapshot_saved"] else ""
            )
            print("output path already exists; refusing to overwrite" + cleanup, file=sys.stderr)
            return 2
        except OSError as exc:
            output_cleanup = remove_created_file(args.output) if created else ""
            snapshot_cleanup = (
                remove_created_file(args.snapshot_index)
                if args.snapshot_index is not None and inventory["index_snapshot_saved"] else ""
            )
            print(f"cannot write inventory JSON: {exc}{output_cleanup}{snapshot_cleanup}", file=sys.stderr)
            return 2

    print(
        f"Inventory only: {inventory['directory_entry_count']} Level III directory entries; "
        f"{len(inventory['area_codes_observed'])} area codes; "
        "no raster or sidecar bytes downloaded.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
