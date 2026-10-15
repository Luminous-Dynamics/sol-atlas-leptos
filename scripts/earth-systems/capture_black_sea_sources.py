#!/usr/bin/env python3
"""Bounded local source capture for the Black Sea Grain Initiative research case.

This research helper is read-only: no API key, no repository writes, no uploads,
and no automatic license or scientific approval.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import parse_qsl, urlencode, unquote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_PLAN = ROOT / "docs/pilots/black-sea-grain-trade-query-plan.json"
USER_AGENT = "Sol-Atlas-Black-Sea-Source-Capture/1.0"
MAX_PREVIEW_RECORDS = 500
ALLOWED_HOSTS = {"un.org", "www.un.org", "fao.org", "www.fao.org", "comtradeapi.un.org"}
ALLOWED_PATHS = {
    "/en/node/207702",
    "/media/docs/worldfoodsituationlibraries/wfs-library/food_price_indices_data.csv",
    "/public/v1/preview/C/M/HS",
}
FORBIDDEN_QUERY_KEYS = {
    "key", "apikey", "api_key", "token", "access_token", "subscription-key",
    "subscription_key", "authorization",
}
SAFE_FILENAME_RE = re.compile(r"^[a-z0-9][a-z0-9._-]{0,127}$")
SAFE_CHUNK_RE = re.compile(r"^[A-Za-z0-9_-]{1,64}$")


class CaptureError(ValueError):
    """Raised when a plan or output path cannot be safely admitted."""


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def validate_source_url(url: str) -> None:
    """Allow only the three reviewed HTTPS paths; reject credentials and traversal."""
    try:
        parts = urlsplit(url)
        path = unquote(parts.path)
        keys = {key.lower() for key, _ in parse_qsl(parts.query, keep_blank_values=True)}
        port = parts.port
        traversal = any(seg in {".", ".."} for seg in path.replace("\\", "/").split("/"))
    except (TypeError, ValueError) as exc:
        raise CaptureError("malformed source URL") from exc
    if (parts.scheme != "https" or parts.hostname not in ALLOWED_HOSTS
            or port not in (None, 443) or parts.username is not None or parts.password is not None
            or parts.fragment or traversal or path not in ALLOWED_PATHS
            or keys.intersection(FORBIDDEN_QUERY_KEYS)):
        raise CaptureError("URL is outside the approved HTTPS source allowlist")


class ApprovedRedirectHandler(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        validate_source_url(newurl)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def fetch_response(url: str, max_bytes: int, timeout_seconds: float) -> dict:
    """Fetch one bounded response; never save a truncated body as raw evidence."""
    validate_source_url(url)
    request = Request(url, headers={"User-Agent": USER_AGENT, "Accept": "*/*"}, method="GET")
    try:
        opener = build_opener(ApprovedRedirectHandler())
        with opener.open(request, timeout=timeout_seconds) as response:
            final_url = response.geturl()
            validate_source_url(final_url)
            headers = response.headers
            meta = {key: headers.get(key) for key in ("Date", "ETag", "Last-Modified", "Content-Language") if headers.get(key) is not None}
            declared = headers.get("Content-Length")
            if declared and int(declared) > max_bytes:
                return {"status_code": response.status, "final_url": final_url, "content_type": headers.get_content_type().lower(), "headers": meta, "raw": None, "error": f"declared body exceeds {max_bytes} bytes"}
            raw = response.read(max_bytes + 1)
            if len(raw) > max_bytes:
                return {"status_code": response.status, "final_url": final_url, "content_type": headers.get_content_type().lower(), "headers": meta, "raw": None, "error": f"response body exceeds {max_bytes} bytes"}
            return {"status_code": response.status, "final_url": final_url, "content_type": headers.get_content_type().lower(), "headers": meta, "raw": raw, "error": None}
    except HTTPError as exc:
        return {"status_code": exc.code, "final_url": exc.geturl(), "content_type": exc.headers.get_content_type().lower() if exc.headers else None, "headers": {}, "raw": None, "error": f"HTTP {exc.code}"}
    except Exception as exc:
        return {"status_code": None, "final_url": None, "content_type": None, "headers": {}, "raw": None, "error": f"{type(exc).__name__}: {str(exc)[:240]}"}


def assess_comtrade_response(raw: bytes, record_limit: int = MAX_PREVIEW_RECORDS) -> dict:
    """Assess the API envelope only, without interpreting quantities or trade values."""
    try:
        obj = json.loads(raw.decode("utf-8-sig"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return {"state": "captured_but_unparseable", "count": None, "returned_rows": None}
    if not isinstance(obj, dict):
        return {"state": "captured_but_unrecognized_schema", "count": None, "returned_rows": None}
    data, count = obj.get("data"), obj.get("count")
    valid_count = isinstance(count, int) and not isinstance(count, bool)
    if not isinstance(data, list) or not valid_count or any(not isinstance(row, dict) for row in data):
        return {"state": "captured_but_unrecognized_schema", "count": count if valid_count else None, "returned_rows": len(data) if isinstance(data, list) else None}
    state = "response_shape_ok_pending_coverage_review"
    if count != len(data) or len(data) >= record_limit:
        state = "captured_but_incomplete"
    return {"state": state, "count": count, "returned_rows": len(data)}


def load_plan(path: Path) -> dict:
    try:
        obj = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise CaptureError(f"could not read a valid query plan: {exc}") from exc
    if not isinstance(obj, dict) or obj.get("schema_version") != "luminous.sol-atlas.un-comtrade-query-plan.v1" or obj.get("status") != "query_plan_only" or obj.get("execution_state") != "not_run":
        raise CaptureError("plan is not an unexecuted Black Sea query plan")
    return obj


def safe_filename(value: str) -> str:
    if not isinstance(value, str) or not SAFE_FILENAME_RE.fullmatch(value):
        raise CaptureError("unsafe output filename in plan")
    return value


def build_request_tasks(plan: dict) -> list[dict]:
    tasks = []
    for item in plan.get("companion_artifacts", []):
        validate_source_url(item["url"])
        tasks.append({
            "source_id": item["source_id"], "artifact_id": item["artifact_id"], "url": item["url"],
            "filename": safe_filename(item["filename"]), "max_bytes": int(item["max_bytes"]),
            "expected_content_types": [x.lower() for x in item["expected_content_types"]],
            "response_kind": item["response_kind"], "request_parameters": None,
            "record_limit": int(plan["request_policy"]["public_preview_record_limit"]),
        })
    source, query = plan["source"], plan["query_dimensions"]
    route = source["endpoint_template"].format(typeCode=query["type_code"], freqCode=query["frequency_code"], classificationCode=query["classification_code"])
    record_limit = int(plan["request_policy"]["public_preview_record_limit"])
    for chunk in plan.get("request_chunks", []):
        chunk_id, periods = chunk.get("chunk_id", ""), chunk.get("periods")
        if not SAFE_CHUNK_RE.fullmatch(chunk_id):
            raise CaptureError("unsafe request chunk identifier")
        if not isinstance(periods, list) or not periods or len(periods) > int(plan["request_policy"]["periods_per_request_target"]):
            raise CaptureError(f"{chunk_id}: invalid monthly request chunk size")
        if any(not isinstance(p, str) or not re.fullmatch(r"\d{6}", p) for p in periods):
            raise CaptureError(f"{chunk_id}: periods must use YYYYMM")
        params = {
            "reporterCode": query["reporter_code"], "partnerCode": query["partner_code"],
            "cmdCode": ",".join(query["commodity_codes"]), "flowCode": query["flow_code"],
            "period": ",".join(periods),
        }
        url = source["endpoint_root"].rstrip("/") + route + "?" + urlencode(params, safe=",")
        validate_source_url(url)
        tasks.append({
            "source_id": source["source_id"], "artifact_id": chunk_id, "url": url,
            "filename": safe_filename(f"comtrade-ukraine-world-grain-exports-{chunk_id}.json"),
            "max_bytes": 16 * 1024 * 1024, "expected_content_types": ["application/json"],
            "response_kind": "comtrade_json", "request_parameters": params,
            "record_limit": record_limit,
        })
    names = [task["filename"] for task in tasks]
    if len(names) != len(set(names)):
        raise CaptureError("plan has duplicate filenames")
    return tasks


def create_snapshot_dir(path: Path) -> Path:
    if path.exists() or path.is_symlink():
        raise CaptureError(f"snapshot directory already exists: {path}")
    if not path.parent.is_dir():
        raise CaptureError("snapshot parent directory must already exist")
    path.mkdir()
    return path


def write_bytes_create_only(path: Path, raw: bytes) -> None:
    created = False
    try:
        with path.open("xb") as stream:
            created = True
            stream.write(raw)
            stream.flush()
    except OSError:
        if created:
            try:
                path.unlink(missing_ok=True)
            except OSError:
                pass
        raise


def capture_one(task: dict, snapshot_dir: Path, timeout_seconds: float, fetcher=fetch_response) -> dict:
    record = {
        "source_id": task["source_id"], "artifact_id": task["artifact_id"],
        "request_url": task["url"], "request_parameters": task["request_parameters"],
        "retrieved_at_utc": utc_now(), "capture_state": "request_failed", "status_code": None,
        "final_url": None, "content_type": None, "response_headers": {}, "response_bytes": None,
        "response_sha256": None, "filename": None, "response_validation": None, "error": None,
    }
    try:
        response = fetcher(task["url"], task["max_bytes"], timeout_seconds)
    except Exception as exc:
        record["error"] = f"{type(exc).__name__}: {str(exc)[:240]}"
        return record
    for key, response_key in (("status_code", "status_code"), ("final_url", "final_url"), ("content_type", "content_type"), ("response_headers", "headers")):
        record[key] = response.get(response_key) or ({} if key == "response_headers" else None)
    if response.get("error"):
        record["capture_state"] = "request_failed_or_size_rejected"
        record["error"] = response["error"]
        return record
    raw = response.get("raw")
    if not isinstance(raw, bytes):
        record["error"] = "response body was not available as complete raw bytes"
        return record
    if not isinstance(record["status_code"], int) or not 200 <= record["status_code"] < 300:
        record["error"] = f"unexpected HTTP status: {record['status_code']}"
        return record
    try:
        write_bytes_create_only(snapshot_dir / task["filename"], raw)
    except OSError as exc:
        record["capture_state"] = "storage_failed"
        record["error"] = f"{type(exc).__name__}: {str(exc)[:240]}"
        return record
    record["filename"] = task["filename"]
    record["response_bytes"] = len(raw)
    record["response_sha256"] = hashlib.sha256(raw).hexdigest()
    mime = (record["content_type"] or "").lower().split(";", 1)[0].strip()
    if mime not in task["expected_content_types"]:
        record["capture_state"] = "captured_unqualified_response"
        record["error"] = "unexpected response content type"
        return record
    if task["response_kind"] == "comtrade_json":
        assessment = assess_comtrade_response(raw, int(task["record_limit"]))
        record["response_validation"] = assessment
        if assessment["state"] == "response_shape_ok_pending_coverage_review":
            record["capture_state"] = "captured_pending_metadata_review"
        elif assessment["state"] == "captured_but_incomplete":
            record["capture_state"] = "captured_but_incomplete"
            record["error"] = "count/row mismatch or public-preview ceiling reached"
        else:
            record["capture_state"] = assessment["state"]
            record["error"] = "Comtrade response envelope could not be validated"
    else:
        record["capture_state"] = "captured_pending_metadata_review"
    return record


def main(argv: list[str] | None = None, fetcher=fetch_response) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, default=DEFAULT_PLAN, help="read-only query-plan JSON")
    parser.add_argument("--snapshot-dir", required=True, type=Path, help="new local directory; existing paths are refused")
    parser.add_argument("--timeout-seconds", type=float, default=20.0)
    args = parser.parse_args(argv)
    if not 1 <= args.timeout_seconds <= 120:
        parser.error("--timeout-seconds must be in 1..120")
    try:
        plan_bytes = args.plan.read_bytes()
        plan = load_plan(args.plan)
        tasks = build_request_tasks(plan)
        snapshot_dir = create_snapshot_dir(args.snapshot_dir)
    except (CaptureError, OSError, KeyError, TypeError, ValueError) as exc:
        print(f"capture setup failed: {exc}", file=sys.stderr)
        return 2
    artifacts = [capture_one(task, snapshot_dir, args.timeout_seconds, fetcher=fetcher) for task in tasks]
    ok_states = {"captured_pending_metadata_review"}
    complete = all(item["capture_state"] in ok_states for item in artifacts)
    manifest = {
        "schema_version": "luminous.sol-atlas.source-capture-manifest.v1",
        "plan_id": plan["plan_id"],
        "capture_state": "responses_captured_pending_review" if complete else "partial_or_incomplete_capture",
        "captured_at_utc": utc_now(), "snapshot_directory": snapshot_dir.name,
        "query_plan_sha256": hashlib.sha256(plan_bytes).hexdigest(),
        "request_count": len(tasks),
        "successful_responses_pending_review": sum(item["capture_state"] in ok_states for item in artifacts),
        "all_requests_captured_and_shape_checked": complete,
        "rights_review_complete": False, "case_qualified_for_analysis": False,
        "artifacts": artifacts,
        "limitations": [
            "A response SHA-256 establishes byte identity only, not accuracy, source independence, license or scientific validity.",
            "Comtrade count/row agreement is a transport check, not proof of complete reporting or comparable measurement.",
            "Unexpected content types, HTTP errors, oversized bodies, row truncation and malformed responses are not qualified acquisitions.",
            "Files remain in the caller-selected local directory; this helper never commits or uploads source data.",
        ],
    }
    manifest_path = snapshot_dir / "capture-manifest.json"
    try:
        write_bytes_create_only(manifest_path, (json.dumps(manifest, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))
    except OSError as exc:
        print(f"failed to write manifest: {exc}; any captured files remain for manual recovery", file=sys.stderr)
        return 2
    print(json.dumps({
        "capture_state": manifest["capture_state"], "snapshot_directory": str(snapshot_dir),
        "request_count": len(tasks), "successful_responses_pending_review": manifest["successful_responses_pending_review"],
        "manifest": str(manifest_path),
    }, indent=2))
    return 0 if complete else 2


if __name__ == "__main__":
    raise SystemExit(main())
