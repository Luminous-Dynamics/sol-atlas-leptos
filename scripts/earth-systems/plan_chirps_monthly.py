#!/usr/bin/env python3
"""Create a read-only CHIRPS v3 monthly index snapshot and acquisition plan.

This tool reads the official directory index only. It never downloads GeoTIFFs,
clips rasters, or marks source artifacts as acquired.
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
from urllib.parse import unquote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

INDEX_URL = (
    "https://data.chc.ucsb.edu/products/CHIRPS/v3.0/"
    "monthly/africa/tifs/"
)
INDEX_HOST = "data.chc.ucsb.edu"
INDEX_PATH = "/products/CHIRPS/v3.0/monthly/africa/tifs/"
MAX_INDEX_BYTES = 16 * 1024 * 1024
DEFAULT_TIMEOUT_SECONDS = 20
FILENAME_RE = re.compile(r"^chirps-v3\.0\.(\d{4})\.(\d{2})\.tif$")


class IndexError(ValueError):
    """Raised when the remote index cannot be interpreted safely."""


class SameOriginRedirectHandler(HTTPRedirectHandler):
    """Reject directory-index redirects to any other URL or path."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        target = urlsplit(newurl)
        if target.scheme != "https" or target.netloc != INDEX_HOST or target.path != INDEX_PATH:
            raise IndexError("directory index attempted to redirect outside the approved HTTPS path")
        return super().redirect_request(req, fp, code, msg, headers, newurl)


class MonthlyIndexParser(html.parser.HTMLParser):
    """Extract only bare monthly filenames from an Apache-style directory page."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.months: dict[str, str] = {}
        self.errors: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag.lower() != "a":
            return
        href = dict(attrs).get("href")
        if not isinstance(href, str) or not href:
            return

        parts = urlsplit(href)
        name = Path(unquote(parts.path)).name
        match = FILENAME_RE.fullmatch(name)
        if not match:
            return

        # The expected archive uses simple sibling filenames. Do not accept
        # external links, absolute paths, query strings, fragments, or traversal.
        if parts.scheme or parts.netloc or parts.query or parts.fragment or parts.path != name:
            self.errors.append(f"unsafe monthly file link: {href!r}")
            return

        year, month = int(match.group(1)), int(match.group(2))
        if year < 1981 or not 1 <= month <= 12:
            self.errors.append(f"invalid year/month in file link: {name}")
            return

        key = f"{year:04d}-{month:02d}"
        if key in self.months:
            self.errors.append(f"duplicate monthly file for {key}")
            return
        self.months[key] = name


def parse_month(value: str) -> str:
    """Validate and normalize a YYYY-MM string."""
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d{2}", value):
        raise argparse.ArgumentTypeError("month must use YYYY-MM format")
    year, month = map(int, value.split("-"))
    if year < 1981 or not 1 <= month <= 12:
        raise argparse.ArgumentTypeError("month must be a valid CHIRPS month from 1981 onward")
    return f"{year:04d}-{month:02d}"


def month_range(start: str, end: str) -> list[str]:
    if start > end:
        raise ValueError("start month must be less than or equal to end month")
    sy, sm = map(int, start.split("-"))
    ey, em = map(int, end.split("-"))
    count = (ey - sy) * 12 + em - sm + 1
    if count > 600:
        raise ValueError("requested range exceeds the 600-month safety cap")
    result: list[str] = []
    year, month = sy, sm
    for _ in range(count):
        result.append(f"{year:04d}-{month:02d}")
        month += 1
        if month == 13:
            year += 1
            month = 1
    return result


def read_index(index_file: Path | None, timeout_seconds: float) -> tuple[bytes, str | None, str]:
    """Return index bytes, retrieval timestamp (if remote), and acquisition mode."""
    if index_file is not None:
        try:
            raw = index_file.read_bytes()
        except OSError as exc:
            raise IndexError(f"cannot read local HTML index: {exc}") from exc
        if len(raw) > MAX_INDEX_BYTES:
            raise IndexError(f"local index exceeds {MAX_INDEX_BYTES} byte limit")
        return raw, None, "local_html_snapshot"

    request = Request(
        INDEX_URL,
        headers={"User-Agent": "Sol-Atlas-CHIRPS-Index-Inventory/1.0"},
        method="GET",
    )
    try:
        opener = build_opener(SameOriginRedirectHandler())
        with opener.open(request, timeout=timeout_seconds) as response:
            final = urlsplit(response.geturl())
            if final.scheme != "https" or final.netloc != INDEX_HOST or final.path != INDEX_PATH:
                raise IndexError("directory index redirected outside the approved HTTPS path")
            content_type = response.headers.get("Content-Type", "").lower()
            if "text/html" not in content_type and "application/xhtml+xml" not in content_type:
                raise IndexError(f"expected an HTML directory index, got {content_type!r}")
            declared_length = response.headers.get("Content-Length")
            if declared_length:
                try:
                    if int(declared_length) > MAX_INDEX_BYTES:
                        raise IndexError(f"remote index exceeds {MAX_INDEX_BYTES} byte limit")
                except ValueError as exc:
                    raise IndexError("invalid Content-Length header") from exc
            raw = response.read(MAX_INDEX_BYTES + 1)
            if len(raw) > MAX_INDEX_BYTES:
                raise IndexError(f"remote index exceeds {MAX_INDEX_BYTES} byte limit")
    except IndexError:
        raise
    except Exception as exc:
        raise IndexError(f"failed to read official CHIRPS index: {exc}") from exc

    retrieved_at = datetime.now(timezone.utc).isoformat(timespec="seconds")
    return raw, retrieved_at, "https_directory_index"


def build_plan(raw: bytes, start: str, end: str, retrieved_at: str | None, mode: str) -> dict:
    if len(raw) > MAX_INDEX_BYTES:
        raise IndexError(f"index exceeds {MAX_INDEX_BYTES} byte limit")
    try:
        document = raw.decode("utf-8-sig")
    except UnicodeDecodeError as exc:
        raise IndexError("index is not valid UTF-8 HTML") from exc

    parser = MonthlyIndexParser()
    try:
        parser.feed(document)
        parser.close()
    except Exception as exc:
        raise IndexError(f"failed to parse directory index: {exc}") from exc
    if parser.errors:
        raise IndexError("; ".join(parser.errors))

    requested = month_range(start, end)
    missing = [month for month in requested if month not in parser.months]
    selected = [
        {
            "month": month,
            "filename": parser.months[month],
            "url": INDEX_URL + parser.months[month],
            "artifact_sha256": None,
            "state": "catalogued_not_downloaded",
        }
        for month in requested
        if month in parser.months
    ]
    return {
        "manifest_version": "1.0.0",
        "dataset": "CHIRPS v3.0 monthly Africa-subset GeoTIFF series",
        "status": "index_inventory_only",
        "source_provider": "Climate Hazards Center, University of California Santa Barbara",
        "source_doi": "https://doi.org/10.15780/G2JQ0P",
        "license": "CC BY 4.0; retain provider attribution and release citation",
        "official_landing_page": "https://www.chc.ucsb.edu/data/chirps3",
        "index_url": INDEX_URL,
        "index_snapshot_mode": mode,
        "index_snapshot_saved": False,
        "index_snapshot_filename": None,
        "index_retrieved_at_utc": retrieved_at,
        "index_sha256": hashlib.sha256(raw).hexdigest(),
        "requested_period": {"start_month": start, "end_month": end},
        "requested_month_count": len(requested),
        "catalogued_month_count": len(selected),
        "missing_months": missing,
        "complete": not missing,
        "files": selected,
        "claim_ceiling": [
            "This plan inventories index links only; it does not prove remote raster bytes are unchanged or valid.",
            "artifact_sha256 is null because no GeoTIFF bytes were downloaded.",
            "Monthly precipitation is a gridded areal estimate, not a local rain-gauge observation.",
            "Clip or aggregate to a frozen study boundary only after the boundary and CRS have been independently checked.",
            "Do not resample the 0.05-degree rainfall grid to 20 m and present it as 20 m information.",
        ],
    }


def remove_created_file(path: Path) -> str:
    """Remove a path created by this invocation and report cleanup failure explicitly."""
    try:
        path.unlink()
    except FileNotFoundError:
        return ""
    except OSError as exc:
        return f"; also could not remove partial artifact {path.name}: {exc}"
    return f"; removed partial artifact {path.name}"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start", required=True, type=parse_month, help="first month, inclusive (YYYY-MM)")
    parser.add_argument("--end", required=True, type=parse_month, help="last month, inclusive (YYYY-MM)")
    parser.add_argument("--index-file", type=Path, help="use a saved HTML directory index instead of the network")
    parser.add_argument("--snapshot-index", type=Path, help="save the exact retrieved HTML index bytes to a new file (never overwrites)")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SECONDS, help="HTTPS index timeout in seconds")
    parser.add_argument("--output", type=Path, help="write the JSON plan to a file; default is stdout")
    args = parser.parse_args(argv)

    if not 0 < args.timeout <= 120:
        parser.error("--timeout must be greater than 0 and no more than 120 seconds")
    if args.output is not None and args.index_file is not None and args.output.resolve() == args.index_file.resolve():
        parser.error("--output must not overwrite --index-file")
    if args.output is not None and args.snapshot_index is not None and args.output.resolve() == args.snapshot_index.resolve():
        parser.error("--output and --snapshot-index must be different paths")
    if args.output is not None and (args.output.exists() or args.output.is_symlink()):
        print("output path already exists; refusing to overwrite", file=sys.stderr)
        return 2
    snapshot_created = False
    try:
        raw, retrieved_at, mode = read_index(args.index_file, args.timeout)
        plan = build_plan(raw, args.start, args.end, retrieved_at, mode)
        if args.snapshot_index is not None:
            try:
                with args.snapshot_index.open("xb") as snapshot:
                    snapshot_created = True
                    written = snapshot.write(raw)
                    if written != len(raw):
                        raise OSError(f"short write: captured {written} of {len(raw)} bytes")
            except FileExistsError as exc:
                raise IndexError(f"snapshot path already exists; refusing to overwrite: {args.snapshot_index.name}") from exc
            except OSError as exc:
                cleanup_note = remove_created_file(args.snapshot_index) if snapshot_created else ""
                raise IndexError(f"cannot save index snapshot: {exc}{cleanup_note}") from exc
            plan["index_snapshot_saved"] = True
            plan["index_snapshot_filename"] = args.snapshot_index.name
    except (IndexError, ValueError) as exc:
        print(f"CHIRPS index planning failed: {exc}", file=sys.stderr)
        return 2

    output = json.dumps(plan, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        sys.stdout.write(output)
    else:
        output_created = False
        try:
            with args.output.open("x", encoding="utf-8") as handle:
                output_created = True
                written = handle.write(output)
                if written != len(output):
                    raise OSError(f"short write: emitted {written} of {len(output)} characters")
        except FileExistsError:
            snapshot_cleanup = (
                remove_created_file(args.snapshot_index)
                if snapshot_created and args.snapshot_index is not None else ""
            )
            print("output path already exists; refusing to overwrite" + snapshot_cleanup, file=sys.stderr)
            return 2
        except OSError as exc:
            output_cleanup = remove_created_file(args.output) if output_created else ""
            snapshot_cleanup = (
                remove_created_file(args.snapshot_index)
                if snapshot_created and args.snapshot_index is not None else ""
            )
            print(f"cannot write plan: {exc}{output_cleanup}{snapshot_cleanup}", file=sys.stderr)
            return 2

    if not plan["complete"]:
        print(
            "CHIRPS index is missing requested months: " + ", ".join(plan["missing_months"]),
            file=sys.stderr,
        )
        return 3
    print(
        f"Inventory only: {plan['catalogued_month_count']} monthly links; "
        "no raster files downloaded.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
