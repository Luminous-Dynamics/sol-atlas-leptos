#!/usr/bin/env python3
"""Inventory the official WaPOR v3 catalogue and optional mapset raster records.

This is read-only discovery: it fetches small JSON catalogue responses only and
never downloads raster data. Use --snapshot-dir to retain exact response bytes.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import quote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

API_ORIGIN = "https://data.apps.fao.org"
API_PREFIX = "/gismgr/api/v2/catalog/workspaces/WAPOR-3/"
MAPSETS_URL = API_ORIGIN + API_PREFIX + "mapsets"
MAX_RESPONSE_BYTES = 16 * 1024 * 1024
MAX_PAGES = 100
MAX_RECORDS = 100_000
DEFAULT_TIMEOUT_SECONDS = 20
MAPSET_CODE_RE = re.compile(r"^[A-Za-z0-9._-]{1,128}$")


class WaporCatalogError(ValueError):
    """Raised when the public catalogue response cannot be admitted safely."""


def validate_api_url(url: str) -> None:
    """Require HTTPS to the official FAO host and within the WaPOR v3 API path."""
    try:
        parts = urlsplit(url)
    except ValueError as exc:
        raise WaporCatalogError("malformed URL in catalogue response") from exc
    if (
        parts.scheme != "https"
        or parts.netloc != "data.apps.fao.org"
        or not parts.path.startswith(API_PREFIX)
        or parts.username is not None
        or parts.password is not None
    ):
        raise WaporCatalogError("catalogue URL escaped the approved WaPOR v3 API origin/path")


class SameOriginRedirectHandler(HTTPRedirectHandler):
    """Reject redirects outside the approved FAO WaPOR v3 API prefix."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        validate_api_url(newurl)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def fetch_json(url: str, timeout_seconds: float) -> tuple[dict, bytes, str]:
    validate_api_url(url)
    request = Request(url, headers={"User-Agent": "Sol-Atlas-WaPOR-Catalog-Inventory/1.0"}, method="GET")
    try:
        opener = build_opener(SameOriginRedirectHandler())
        with opener.open(request, timeout=timeout_seconds) as response:
            final_url = response.geturl()
            validate_api_url(final_url)
            content_type = response.headers.get("Content-Type", "").lower()
            if "application/json" not in content_type and "+json" not in content_type:
                raise WaporCatalogError(f"expected JSON from WaPOR API, got {content_type!r}")
            declared_length = response.headers.get("Content-Length")
            if declared_length:
                try:
                    if int(declared_length) > MAX_RESPONSE_BYTES:
                        raise WaporCatalogError(f"API response exceeds {MAX_RESPONSE_BYTES} byte limit")
                except ValueError as exc:
                    raise WaporCatalogError("invalid API Content-Length header") from exc
            raw = response.read(MAX_RESPONSE_BYTES + 1)
            if len(raw) > MAX_RESPONSE_BYTES:
                raise WaporCatalogError(f"API response exceeds {MAX_RESPONSE_BYTES} byte limit")
    except WaporCatalogError:
        raise
    except Exception as exc:
        raise WaporCatalogError(f"failed to query official WaPOR API: {exc}") from exc

    try:
        document = json.loads(raw.decode("utf-8-sig"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise WaporCatalogError("WaPOR API returned invalid UTF-8 JSON") from exc
    if not isinstance(document, dict):
        raise WaporCatalogError("WaPOR API response root must be an object")
    return document, raw, final_url


def paginate(endpoint: str, timeout_seconds: float, fetcher=fetch_json) -> tuple[list[dict], list[dict], list[bytes]]:
    records: list[dict] = []
    page_metadata: list[dict] = []
    page_bytes: list[bytes] = []
    next_url: str | None = endpoint
    visited: set[str] = set()

    while next_url is not None:
        if len(page_metadata) >= MAX_PAGES:
            raise WaporCatalogError(f"catalogue exceeded {MAX_PAGES} pages")
        validate_api_url(next_url)
        if next_url in visited:
            raise WaporCatalogError("pagination loop detected")
        visited.add(next_url)

        document, raw, final_url = fetcher(next_url, timeout_seconds)
        payload = document.get("response")
        if not isinstance(payload, dict):
            raise WaporCatalogError("API response is missing object field 'response'")
        items = payload.get("items")
        if not isinstance(items, list) or any(not isinstance(item, dict) for item in items):
            raise WaporCatalogError("API response 'items' must be an array of objects")
        if len(records) + len(items) > MAX_RECORDS:
            raise WaporCatalogError(f"catalogue exceeded {MAX_RECORDS} records")

        records.extend(items)
        page_metadata.append({
            "request_url": next_url,
            "final_url": final_url,
            "retrieved_at_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "response_sha256": hashlib.sha256(raw).hexdigest(),
            "response_bytes": len(raw),
            "item_count": len(items),
        })
        page_bytes.append(raw)

        links = payload.get("links", [])
        if not isinstance(links, list):
            raise WaporCatalogError("API response 'links' must be an array")
        next_links = [
            item.get("href") for item in links
            if isinstance(item, dict) and item.get("rel") == "next"
        ]
        if len(next_links) > 1:
            raise WaporCatalogError("API response has multiple 'next' links")
        next_url = next_links[0] if next_links else None
        if next_url is not None and not isinstance(next_url, str):
            raise WaporCatalogError("pagination next-link href must be a string")

    return records, page_metadata, page_bytes


def search_text(value: object) -> str:
    return json.dumps(value, sort_keys=True, ensure_ascii=False).casefold()


def validate_mapset_code(code: str) -> str:
    if not MAPSET_CODE_RE.fullmatch(code):
        raise argparse.ArgumentTypeError("mapset code may contain only letters, digits, dot, underscore, and hyphen")
    return code


def save_snapshots(snapshot_dir: Path, groups: list[tuple[str, list[dict], list[bytes]]]) -> list[dict]:
    try:
        snapshot_dir.mkdir(parents=False, exist_ok=False, mode=0o700)
    except FileExistsError as exc:
        raise WaporCatalogError("snapshot directory already exists; refusing to overwrite") from exc
    except OSError as exc:
        raise WaporCatalogError(f"cannot create snapshot directory: {exc}") from exc

    saved: list[dict] = []
    try:
        for label, metadata, payloads in groups:
            for index, (page, raw) in enumerate(zip(metadata, payloads), start=1):
                filename = f"{label}-page-{index:03d}-{page['response_sha256'][:12]}.json"
                path = snapshot_dir / filename
                with path.open("xb") as handle:
                    handle.write(raw)
                saved.append({
                    "filename": filename,
                    "sha256": hashlib.sha256(raw).hexdigest(),
                    "bytes": len(raw),
                })
    except OSError as exc:
        raise WaporCatalogError(f"cannot preserve raw API response: {exc}") from exc
    return saved


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--search", help="case-insensitive substring filter over mapset metadata")
    parser.add_argument("--mapset-code", type=validate_mapset_code, help="also inventory raster records for this exact mapset code")
    parser.add_argument("--snapshot-dir", type=Path, help="save exact paginated API response bytes to a new directory")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SECONDS, help="per-request timeout in seconds")
    parser.add_argument("--output", type=Path, help="write the inventory JSON to a new file; default is stdout")
    args = parser.parse_args(argv)

    if not 0 < args.timeout <= 120:
        parser.error("--timeout must be greater than 0 and no more than 120 seconds")
    if args.output is not None and args.snapshot_dir is not None:
        try:
            if args.output.resolve().is_relative_to(args.snapshot_dir.resolve()):
                parser.error("--output must not be written inside --snapshot-dir")
        except OSError:
            parser.error("cannot resolve --output or --snapshot-dir")

    try:
        mapsets, mapset_pages, mapset_raw = paginate(MAPSETS_URL, args.timeout)
        selected = [
            record for record in mapsets
            if not args.search or args.search.casefold() in search_text(record)
        ]
        raster_records: list[dict] = []
        raster_pages: list[dict] = []
        raster_raw: list[bytes] = []
        if args.mapset_code:
            matches = [item for item in mapsets if item.get("code") == args.mapset_code]
            if len(matches) != 1:
                raise WaporCatalogError(
                    f"requested mapset code must match exactly one catalogue record; found {len(matches)}"
                )
            rasters_url = MAPSETS_URL + "/" + quote(args.mapset_code, safe="._-") + "/rasters"
            raster_records, raster_pages, raster_raw = paginate(rasters_url, args.timeout)

        snapshot_files: list[dict] = []
        if args.snapshot_dir is not None:
            groups = [("mapsets", mapset_pages, mapset_raw)]
            if args.mapset_code:
                safe_label = re.sub(r"[^A-Za-z0-9._-]", "_", args.mapset_code)
                groups.append((f"rasters-{safe_label}", raster_pages, raster_raw))
            snapshot_files = save_snapshots(args.snapshot_dir, groups)
    except (WaporCatalogError, ValueError) as exc:
        print(f"WaPOR catalogue inventory failed: {exc}", file=sys.stderr)
        return 2

    output_document = {
        "manifest_version": "1.0.0",
        "status": "catalogue_inventory_only",
        "dataset": "FAO WaPOR version 3",
        "api_endpoint": MAPSETS_URL,
        "search_filter": args.search,
        "mapset_count_total": len(mapsets),
        "mapset_count_returned": len(selected),
        "mapsets": selected,
        "mapset_pages": mapset_pages,
        "requested_mapset_code": args.mapset_code,
        "raster_count": len(raster_records) if args.mapset_code else None,
        "rasters": raster_records if args.mapset_code else None,
        "raster_pages": raster_pages,
        "snapshot_directory_requested": args.snapshot_dir is not None,
        "raw_response_snapshots": snapshot_files,
        "claim_ceiling": [
            "This tool inventories API metadata only; it does not download raster data.",
            "A catalogue record does not certify the selected layer's scientific validity or legal reuse scope.",
            "Date codes in raster metadata must be interpreted with each mapset's unit, temporal resolution, and final/NRT semantics.",
            "For Baixo Limpopo NPP/AGBP, check the March 2025 correction; for all Level III layers, check the July 2024 re-release history.",
            "A successful response is a catalogue observation at the recorded timestamp, not a promise that an old download URL remains valid.",
        ],
    }
    rendered = json.dumps(output_document, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    if args.output is None:
        sys.stdout.write(rendered)
    else:
        try:
            with args.output.open("x", encoding="utf-8") as handle:
                handle.write(rendered)
        except FileExistsError:
            print("output path already exists; refusing to overwrite", file=sys.stderr)
            return 2
        except OSError as exc:
            print(f"cannot write inventory JSON: {exc}", file=sys.stderr)
            return 2
    print(
        f"Inventory only: {len(selected)} of {len(mapsets)} mapsets; "
        + (f"{len(raster_records)} raster metadata records; " if args.mapset_code else "")
        + "no raster files downloaded.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
