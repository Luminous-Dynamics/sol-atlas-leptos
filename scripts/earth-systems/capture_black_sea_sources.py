#!/usr/bin/env python3
"""Bounded local source capture for the Black Sea Grain Initiative research case.

This research helper is read-only: no API key, no repository writes, no uploads,
and no automatic license or scientific approval.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import time
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
    "/public/v1/getDa/C/M/HS",
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
        error_headers = {}
        if exc.headers:
            error_headers = {
                key: exc.headers.get(key)
                for key in ("Date", "Retry-After", "Content-Type")
                if exc.headers.get(key) is not None
            }
        return {
            "status_code": exc.code,
            "final_url": exc.geturl(),
            "content_type": exc.headers.get_content_type().lower() if exc.headers else None,
            "headers": error_headers,
            "raw": None,
            "error": f"HTTP {exc.code}",
        }
    except Exception as exc:
        return {"status_code": None, "final_url": None, "content_type": None, "headers": {}, "raw": None, "error": f"{type(exc).__name__}: {str(exc)[:240]}"}


def assess_comtrade_response(
    raw: bytes,
    record_limit: int = MAX_PREVIEW_RECORDS,
    expected_parameters: dict | None = None,
) -> dict:
    """Assess response envelope and requested dimensions, not measurement meaning."""
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
    if count != len(data) or len(data) >= record_limit:
        return {"state": "captured_but_incomplete", "count": count, "returned_rows": len(data)}

    # Empty data is a distinct review state, not successful query qualification.
    # It may mean no report, no coverage, a dimension mismatch, or a genuine
    # absence. A separate availability/metadata check must adjudicate it.
    if not data:
        return {
            "state": "empty_response_needs_availability_review",
            "count": count,
            "returned_rows": 0,
            "rows_present": False,
            "query_dimensions_verified": False,
        }

    if expected_parameters and data:
        required = ("reporterCode", "partnerCode", "flowCode", "cmdCode", "period")
        missing = sorted(field for field in required if any(field not in row for row in data))
        if missing:
            return {
                "state": "captured_but_unrecognized_schema",
                "count": count,
                "returned_rows": len(data),
                "missing_query_dimension_fields": missing,
            }
        mismatches = {}
        for field in required:
            expected = str(expected_parameters[field])
            observed = sorted({str(row[field]) for row in data})
            if observed != [expected]:
                mismatches[field] = {"expected": expected, "observed": observed[:10]}
        if mismatches:
            return {
                "state": "captured_query_dimension_mismatch",
                "count": count,
                "returned_rows": len(data),
                "dimension_mismatches": mismatches,
            }
    return {
        "state": "response_shape_ok_pending_coverage_review",
        "count": count,
        "returned_rows": len(data),
        "rows_present": bool(data),
        "query_dimensions_verified": bool(expected_parameters and data),
    }



def assess_comtrade_availability(
    raw: bytes,
    expected_parameters: dict,
    record_limit: int = MAX_PREVIEW_RECORDS,
) -> dict:
    """Validate reporter/month metadata without inferring commodity-level coverage."""
    try:
        obj = json.loads(raw.decode("utf-8-sig"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return {"state": "captured_but_unparseable", "count": None, "returned_rows": None}
    if not isinstance(obj, dict):
        return {"state": "captured_but_unrecognized_schema", "count": None, "returned_rows": None}

    data, count = obj.get("data"), obj.get("count")
    valid_count = isinstance(count, int) and not isinstance(count, bool)
    if not isinstance(data, list) or not valid_count or any(not isinstance(row, dict) for row in data):
        return {
            "state": "captured_but_unrecognized_schema",
            "count": count if valid_count else None,
            "returned_rows": len(data) if isinstance(data, list) else None,
        }
    if count != len(data) or len(data) >= record_limit:
        return {"state": "captured_but_incomplete", "count": count, "returned_rows": len(data)}

    if not data:
        return {
            "state": "availability_record_absent_unresolved",
            "count": count,
            "returned_rows": 0,
            "rows_present": False,
            "reporter_period_dimensions_verified": False,
            "commodity_partner_flow_availability_verified": False,
        }

    required = ("reporterCode", "period")
    missing = sorted(field for field in required if any(field not in row for row in data))
    if missing:
        return {
            "state": "availability_dimensions_unverified",
            "count": count,
            "returned_rows": len(data),
            "missing_dimension_fields": missing,
            "commodity_partner_flow_availability_verified": False,
        }

    mismatches = {}
    for field in required:
        expected = str(expected_parameters[field])
        observed = sorted({str(row[field]) for row in data})
        if observed != [expected]:
            mismatches[field] = {"expected": expected, "observed": observed[:10]}
    if mismatches:
        return {
            "state": "availability_dimension_mismatch",
            "count": count,
            "returned_rows": len(data),
            "dimension_mismatches": mismatches,
            "commodity_partner_flow_availability_verified": False,
        }

    return {
        "state": "availability_record_found_reporter_period_only",
        "count": count,
        "returned_rows": len(data),
        "rows_present": True,
        "reporter_period_dimensions_verified": True,
        "commodity_partner_flow_availability_verified": False,
        "interpretation_limit": (
            "This endpoint indicates reporter/month availability only; it does not verify "
            "the requested commodity, partner, or flow."
        ),
    }



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
            "expected_response_dimensions": None,
            "record_limit": int(plan["request_policy"]["public_preview_record_limit"]),
        })
    source, query = plan["source"], plan["query_dimensions"]
    route = source["endpoint_template"].format(typeCode=query["type_code"], freqCode=query["frequency_code"], classificationCode=query["classification_code"])
    availability_route = source["availability_endpoint_template"].format(
        typeCode=query["type_code"], freqCode=query["frequency_code"], classificationCode=query["classification_code"]
    )
    record_limit = int(plan["request_policy"]["public_preview_record_limit"])
    availability_periods = sorted({chunk["period"] for chunk in plan.get("request_chunks", [])})
    for period in availability_periods:
        if not isinstance(period, str) or not re.fullmatch(r"\d{6}", period):
            raise CaptureError("availability periods must use YYYYMM")
        params = {"reportercode": query["reporter_code"], "period": period}
        expected_dimensions = {"reporterCode": query["reporter_code"], "period": period}
        url = source["endpoint_root"].rstrip("/") + availability_route + "?" + urlencode(params, safe=",")
        validate_source_url(url)
        tasks.append({
            "source_id": source["source_id"], "artifact_id": f"availability_{period}", "url": url,
            "filename": safe_filename(f"comtrade-ukraine-monthly-availability-{period}.json"),
            "max_bytes": 16 * 1024 * 1024, "expected_content_types": ["application/json"],
            "response_kind": "comtrade_availability_json", "request_parameters": params,
            "expected_response_dimensions": expected_dimensions, "record_limit": record_limit,
        })
    record_limit = int(plan["request_policy"]["public_preview_record_limit"])
    for chunk in plan.get("request_chunks", []):
        chunk_id, periods = chunk.get("chunk_id", ""), chunk.get("periods")
        commodity_code = chunk.get("commodity_code")
        if not SAFE_CHUNK_RE.fullmatch(chunk_id):
            raise CaptureError("unsafe request chunk identifier")
        if not isinstance(periods, list) or len(periods) != 1 or len(periods) > int(plan["request_policy"]["periods_per_request_target"]):
            raise CaptureError(f"{chunk_id}: public preview requires exactly one month per request")
        if commodity_code not in query["commodity_codes"]:
            raise CaptureError(f"{chunk_id}: commodity is not an admitted planned product")
        if any(not isinstance(p, str) or not re.fullmatch(r"\d{6}", p) for p in periods):
            raise CaptureError(f"{chunk_id}: periods must use YYYYMM")
        # UN Comtrade's maintained public-preview client uses lowercase
        # "reportercode" on the preview endpoint. Response records use
        # "reporterCode"; keep the wire parameter and response schema distinct.
        params = {
            "reportercode": query["reporter_code"], "partnerCode": query["partner_code"],
            "cmdCode": commodity_code, "flowCode": query["flow_code"],
            "period": ",".join(periods),
        }
        expected_dimensions = {
            "reporterCode": query["reporter_code"],
            "partnerCode": query["partner_code"],
            "cmdCode": commodity_code,
            "flowCode": query["flow_code"],
            "period": periods[0],
        }
        url = source["endpoint_root"].rstrip("/") + route + "?" + urlencode(params, safe=",")
        validate_source_url(url)
        tasks.append({
            "source_id": source["source_id"], "artifact_id": chunk_id, "url": url,
            "filename": safe_filename(f"comtrade-ukraine-world-grain-exports-{periods[0]}-hs{commodity_code}.json"),
            "max_bytes": 16 * 1024 * 1024, "expected_content_types": ["application/json"],
            "response_kind": "comtrade_json", "request_parameters": params,
            "expected_response_dimensions": expected_dimensions,
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
    """Create an immutable snapshot and verify durable, exact-byte read-back."""
    created = False
    try:
        with path.open("xb") as stream:
            created = True
            written = stream.write(raw)
            if written != len(raw):
                raise OSError(f"short snapshot write: expected {len(raw)} bytes, wrote {written}")
            stream.flush()
            os.fsync(stream.fileno())
        read_back = path.read_bytes()
        if read_back != raw:
            raise OSError(
                f"snapshot read-back integrity check failed: expected {len(raw)} bytes, "
                f"read {len(read_back)} bytes"
            )
    except OSError as exc:
        if created:
            try:
                path.unlink(missing_ok=True)
            except OSError as cleanup_exc:
                raise OSError(
                    f"{exc}; incomplete snapshot cleanup also failed: {cleanup_exc}"
                ) from exc
        raise


def capture_one(task: dict, snapshot_dir: Path, timeout_seconds: float, fetcher=fetch_response) -> dict:
    record = {
        "source_id": task["source_id"], "artifact_id": task["artifact_id"],
        "request_url": task["url"], "request_parameters": task["request_parameters"],
        "expected_response_dimensions": task.get("expected_response_dimensions"),
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
        assessment = assess_comtrade_response(
            raw,
            int(task["record_limit"]),
            expected_parameters=task.get("expected_response_dimensions"),
        )
        record["response_validation"] = assessment
        if assessment["state"] == "response_shape_ok_pending_coverage_review":
            record["capture_state"] = "captured_pending_metadata_review"
        elif assessment["state"] == "captured_but_incomplete":
            record["capture_state"] = "captured_but_incomplete"
            record["error"] = "count/row mismatch or public-preview ceiling reached"
        else:
            record["capture_state"] = assessment["state"]
            if assessment["state"] == "captured_query_dimension_mismatch":
                record["error"] = "Comtrade response dimensions did not match the requested query"
            elif assessment["state"] == "empty_response_needs_availability_review":
                record["error"] = "Empty response is not proof of zero trade; check source data availability and query dimensions"
            else:
                record["error"] = "Comtrade response envelope or required dimensions could not be validated"
    elif task["response_kind"] == "comtrade_availability_json":
        assessment = assess_comtrade_availability(
            raw,
            expected_parameters=task["expected_response_dimensions"],
            record_limit=int(task["record_limit"]),
        )
        record["response_validation"] = assessment
        if assessment["state"] == "availability_record_found_reporter_period_only":
            record["capture_state"] = "captured_pending_metadata_review"
        elif assessment["state"] == "availability_record_absent_unresolved":
            record["capture_state"] = "availability_record_absent_unresolved"
            record["error"] = (
                "No reporter/month availability record found; this does not establish "
                "commodity-level absence or zero trade"
            )
        else:
            record["capture_state"] = assessment["state"]
            record["error"] = "Comtrade availability response dimensions or schema could not be validated"
    else:
        record["capture_state"] = "captured_pending_metadata_review"
    return record


def _not_attempted_record(task: dict, reason: str) -> dict:
    return {
        "source_id": task["source_id"],
        "artifact_id": task["artifact_id"],
        "request_url": task["url"],
        "request_parameters": task["request_parameters"],
        "expected_response_dimensions": task.get("expected_response_dimensions"),
        "retrieved_at_utc": None,
        "capture_state": "not_attempted_after_rate_limit",
        "status_code": None,
        "final_url": None,
        "content_type": None,
        "response_headers": {},
        "response_bytes": None,
        "response_sha256": None,
        "filename": None,
        "response_validation": None,
        "error": reason,
    }


def main(argv: list[str] | None = None, fetcher=fetch_response) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, default=DEFAULT_PLAN, help="read-only query-plan JSON")
    parser.add_argument("--snapshot-dir", required=True, type=Path, help="new local directory; existing paths are refused")
    parser.add_argument("--timeout-seconds", type=float, default=20.0)
    parser.add_argument(
        "--include-comtrade",
        action="store_true",
        help="explicitly make all 168 public-preview commodity queries",
    )
    parser.add_argument(
        "--include-availability",
        action="store_true",
        help="explicitly make 84 monthly reporter-availability queries; these do not verify commodity, partner, or flow",
    )
    args = parser.parse_args(argv)
    if not 1 <= args.timeout_seconds <= 120:
        parser.error("--timeout-seconds must be in 1..120")
    try:
        plan_bytes = args.plan.read_bytes()
        plan = load_plan(args.plan)
        all_plan_tasks = build_request_tasks(plan)
        selected_api_kinds = set()
        if args.include_comtrade:
            selected_api_kinds.add("comtrade_json")
        if args.include_availability:
            selected_api_kinds.add("comtrade_availability_json")
        tasks = [
            task for task in all_plan_tasks
            if task["response_kind"] not in {"comtrade_json", "comtrade_availability_json"}
            or task["response_kind"] in selected_api_kinds
        ]
        snapshot_dir = create_snapshot_dir(args.snapshot_dir)
    except (CaptureError, OSError, KeyError, TypeError, ValueError) as exc:
        print(f"capture setup failed: {exc}", file=sys.stderr)
        return 2

    artifacts = []
    minimum_interval = float(plan["request_policy"]["minimum_seconds_between_comtrade_requests"])
    last_comtrade_start = None
    rate_limit_hit = False
    stop_reason = None

    for index, task in enumerate(tasks):
        if task["response_kind"] in {"comtrade_json", "comtrade_availability_json"}:
            if last_comtrade_start is not None:
                elapsed = time.monotonic() - last_comtrade_start
                if elapsed < minimum_interval:
                    time.sleep(minimum_interval - elapsed)
            last_comtrade_start = time.monotonic()
        record = capture_one(task, snapshot_dir, args.timeout_seconds, fetcher=fetcher)
        artifacts.append(record)
        error_text = (record.get("error") or "").lower()
        if record.get("status_code") == 429 or "rate limit is exceeded" in error_text:
            rate_limit_hit = True
            stop_reason = "upstream_rate_limit"
            reason = "not attempted: stopped at the first upstream rate-limit response"
            artifacts.extend(_not_attempted_record(remaining, reason) for remaining in tasks[index + 1:])
            break

    ok_states = {"captured_pending_metadata_review"}
    not_attempted_count = sum(item["capture_state"] == "not_attempted_after_rate_limit" for item in artifacts)
    attempted_count = len(artifacts) - not_attempted_count
    selected_complete = (
        not rate_limit_hit
        and len(artifacts) == len(tasks)
        and all(item["capture_state"] in ok_states for item in artifacts)
    )
    full_plan_selected = bool(args.include_comtrade and args.include_availability)
    if rate_limit_hit:
        state = "stopped_after_rate_limit"
    elif not full_plan_selected and selected_complete:
        state = "companion_sources_captured_pending_review"
    elif selected_complete:
        state = "all_selected_sources_captured_pending_review"
    else:
        state = "partial_or_incomplete_capture"

    manifest = {
        "schema_version": "luminous.sol-atlas.source-capture-manifest.v1",
        "plan_id": plan["plan_id"],
        "capture_state": state,
        "captured_at_utc": utc_now(),
        "snapshot_directory": snapshot_dir.name,
        "query_plan_sha256": hashlib.sha256(plan_bytes).hexdigest(),
        "plan_request_count": len(all_plan_tasks),
        "selected_request_count": len(tasks),
        "attempted_request_count": attempted_count,
        "not_attempted_request_count": not_attempted_count,
        "comtrade_calls_requested": bool(args.include_comtrade),
        "comtrade_call_count_in_plan": sum(t["response_kind"] == "comtrade_json" for t in all_plan_tasks),
        "comtrade_calls_not_selected_by_default": 0 if args.include_comtrade else sum(t["response_kind"] == "comtrade_json" for t in all_plan_tasks),
        "availability_calls_requested": bool(args.include_availability),
        "availability_call_count_in_plan": sum(t["response_kind"] == "comtrade_availability_json" for t in all_plan_tasks),
        "availability_calls_not_selected_by_default": 0 if args.include_availability else sum(t["response_kind"] == "comtrade_availability_json" for t in all_plan_tasks),
        "rate_limit_hit": rate_limit_hit,
        "stop_reason": stop_reason,
        "selected_requests_all_captured_and_shape_checked": selected_complete,
        "full_plan_captured_and_shape_checked": full_plan_selected and selected_complete,
        "rights_review_complete": False,
        "case_qualified_for_analysis": False,
        "artifacts": artifacts,
        "limitations": [
            "A response SHA-256 establishes byte identity only, not accuracy, source independence, license or scientific validity.",
            "Comtrade count/row and requested-dimension agreement are transport/schema checks, not proof of complete reporting or comparable measurement.",
            "Empty monthly trade responses remain unknown until data-availability/coverage is reviewed; they are not interpreted as zero trade.",
            "The Comtrade data-availability endpoint is reporter/month metadata only and does not verify the product, partner, or trade flow.",
            "Unexpected content types, HTTP errors, oversized bodies, row truncation, dimension mismatches and malformed responses are not qualified acquisitions.",
            "At the first upstream rate-limit response, no later request is sent in that run; Retry-After, when supplied, is preserved as response metadata.",
            "By default this command captures only the UN page and FAO CSV. Commodity preview calls (168) and reporter/month availability calls (84) are separately opt-in to avoid unexpected request volume.",
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
        "capture_state": manifest["capture_state"],
        "snapshot_directory": str(snapshot_dir),
        "selected_request_count": len(tasks),
        "attempted_request_count": attempted_count,
        "not_attempted_request_count": not_attempted_count,
        "comtrade_calls_requested": bool(args.include_comtrade),
        "availability_calls_requested": bool(args.include_availability),
        "successful_responses_pending_review": sum(item["capture_state"] in (ok_states | {"availability_record_absent_unresolved"}) for item in artifacts),
        "manifest": str(manifest_path),
    }, indent=2))
    return 0 if selected_complete else 2


if __name__ == "__main__":
    raise SystemExit(main())
