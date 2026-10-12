#!/usr/bin/env python3
"""Independently verify the frozen synthetic economic-intelligence corpus.

Uses only the Python standard library and integer decimal arithmetic. This is
an oracle for the frozen fixture contract, not a UN Comtrade client and not an
estimate of illicit financial flows.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = (
    ROOT
    / "sol-atlas-core"
    / "tests"
    / "fixtures"
    / "economic_intelligence_trade_mirror_v1.json"
)
COMTRADE_PREVIEW_FIXTURE = (
    ROOT
    / "sol-atlas-core"
    / "tests"
    / "fixtures"
    / "synthetic_un_comtrade_preview_v1.json"
)
COMTRADE_ENVELOPE_KEYS = {"elapsedTime", "count", "data", "error"}
COMTRADE_REQUIRED_ROW_KEYS = {
    "typeCode",
    "freqCode",
    "period",
    "reporterCode",
    "partnerCode",
    "flowCode",
    "cmdCode",
    "classificationCode",
    "isAggrAbove",
    "partner2Code",
    "motCode",
    "customsCode",
    "primaryValue",
    "isReported",
    "legacyEstimationFlag",
}
MAX_I128 = (1 << 127) - 1
MIN_I128 = -(1 << 127)
DECIMAL_TOKEN = re.compile(r"-?[0-9]+(?:\.[0-9]{1,18})?\Z")

TOP_LEVEL_KEYS = {"corpus_id", "schema_version", "fixture_kind", "note", "cases"}
CASE_KEYS = {
    "case_id",
    "exporter_value",
    "importer_value",
    "exporter_currency",
    "importer_currency",
    "exporter_value_unit",
    "importer_value_unit",
    "exporter_reporter_kind",
    "exporter_partner_kind",
    "importer_reporter_kind",
    "importer_partner_kind",
    "exporter_geography_code_system",
    "importer_geography_code_system",
    "expected_status",
    "expected_signed_difference",
    "expected_absolute_difference",
}


def parse_decimal_token(token: Any) -> tuple[int, int] | None:
    """Return (coefficient, scale), rejecting noncanonical/unsupported tokens."""
    if not isinstance(token, str) or DECIMAL_TOKEN.fullmatch(token) is None:
        return None

    negative = token.startswith("-")
    unsigned = token[1:] if negative else token
    if "." in unsigned:
        whole, fraction = unsigned.split(".", 1)
    else:
        whole, fraction = unsigned, ""

    coefficient = int(whole + fraction)
    # The Rust parser first parses a positive i128 magnitude, then applies the
    # sign with checked_neg; match that representable input domain exactly.
    if coefficient > MAX_I128:
        return None
    if negative:
        coefficient = -coefficient
    if not MIN_I128 <= coefficient <= MAX_I128:
        return None
    return coefficient, len(fraction)


def rescale(value: tuple[int, int], target_scale: int) -> int | None:
    coefficient, scale = value
    if target_scale < scale:
        return None
    result = coefficient * (10 ** (target_scale - scale))
    if not MIN_I128 <= result <= MAX_I128:
        return None
    return result


def format_scaled(value: int, scale: int) -> str:
    negative = value < 0
    digits = str(abs(value))
    if scale == 0:
        result = digits
    elif len(digits) <= scale:
        result = "0." + ("0" * (scale - len(digits))) + digits
    else:
        result = digits[:-scale] + "." + digits[-scale:]

    if "." in result:
        result = result.rstrip("0").rstrip(".")
    if negative and result != "0":
        result = "-" + result
    return result


def evaluate(case: dict[str, Any]) -> tuple[str, str | None, str | None]:
    """Mirror the documented disposition order using independent Python math."""
    if case.get("exporter_reporter_kind", "reporting_area") != "reporting_area":
        return "aggregate_party_not_supported", None, None
    if case.get("exporter_partner_kind", "reporting_area") != "reporting_area":
        return "aggregate_party_not_supported", None, None
    if case.get("importer_reporter_kind", "reporting_area") != "reporting_area":
        return "aggregate_party_not_supported", None, None
    if case.get("importer_partner_kind", "reporting_area") != "reporting_area":
        return "aggregate_party_not_supported", None, None
    if case.get("exporter_geography_code_system", "TEST_ALPHA3") != case.get(
        "importer_geography_code_system", "TEST_ALPHA3"
    ):
        return "geography_code_system_mismatch", None, None
    if case.get("exporter_currency", "USD") != case.get("importer_currency", "USD"):
        return "currency_mismatch", None, None
    if case.get("exporter_value_unit", "USD") != case.get("importer_value_unit", "USD"):
        return "value_unit_mismatch", None, None

    exporter_token = case.get("exporter_value")
    importer_token = case.get("importer_value")
    if exporter_token is None:
        return "missing_exporter_value", None, None
    if importer_token is None:
        return "missing_importer_value", None, None

    exporter = parse_decimal_token(exporter_token)
    importer = parse_decimal_token(importer_token)
    if exporter is None:
        return "invalid_exporter_value_format", None, None
    if importer is None:
        return "invalid_importer_value_format", None, None
    if exporter[0] <= 0:
        return "non_positive_exporter_value", None, None
    if importer[0] <= 0:
        return "non_positive_importer_value", None, None

    scale = max(exporter[1], importer[1])
    exporter_scaled = rescale(exporter, scale)
    importer_scaled = rescale(importer, scale)
    if exporter_scaled is None or importer_scaled is None:
        return "comparison_arithmetic_overflow", None, None

    signed = importer_scaled - exporter_scaled
    if not MIN_I128 <= signed <= MAX_I128:
        return "comparison_arithmetic_overflow", None, None
    absolute = abs(signed)
    if absolute > MAX_I128:
        return "comparison_arithmetic_overflow", None, None

    return "compared", format_scaled(signed, scale), format_scaled(absolute, scale)


def verify_comtrade_preview_fixture() -> list[str]:
    """Validate the source-shaped payload without treating it as real-world data."""
    try:
        response = json.loads(COMTRADE_PREVIEW_FIXTURE.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"Comtrade preview fixture cannot be loaded: {error}"]

    if not isinstance(response, dict) or set(response) != COMTRADE_ENVELOPE_KEYS:
        return ["Comtrade preview fixture has an unexpected envelope schema"]
    records = response.get("data")
    if not isinstance(records, list) or response.get("count") != len(records):
        return ["Comtrade preview fixture count/data length mismatch"]
    if len(records) != 1:
        return ["Comtrade preview fixture must contain exactly one synthetic row"]

    row = records[0]
    if not isinstance(row, dict) or not COMTRADE_REQUIRED_ROW_KEYS.issubset(row):
        return ["Comtrade preview fixture is missing required row fields"]
    if "Synthetic fixture" not in row.get("reporterDesc", ""):
        return ["Comtrade preview fixture reporter must be explicitly labelled synthetic"]
    if "Synthetic fixture" not in row.get("partnerDesc", ""):
        return ["Comtrade preview fixture partner must be explicitly labelled synthetic"]
    if row.get("primaryValue") != 9007199254740993:
        return ["Comtrade preview fixture lost the exact large integer primaryValue"]
    if row.get("flowCode") != "X" or row.get("period") != "2025":
        return ["Comtrade preview fixture core query dimensions drifted"]
    return []


def main() -> int:
    comtrade_errors = verify_comtrade_preview_fixture()
    if comtrade_errors:
        for error in comtrade_errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1

    try:
        corpus = json.loads(FIXTURE.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        print(f"FAIL: cannot load fixture: {error}", file=sys.stderr)
        return 1

    if not isinstance(corpus, dict) or set(corpus) != TOP_LEVEL_KEYS:
        print("FAIL: unexpected fixture top-level schema", file=sys.stderr)
        return 1
    if corpus["corpus_id"] != "economic-intelligence-trade-mirror-v1":
        print("FAIL: unexpected corpus identity", file=sys.stderr)
        return 1
    if corpus["schema_version"] != 1 or corpus["fixture_kind"] != "synthetic":
        print("FAIL: unsupported or non-synthetic corpus", file=sys.stderr)
        return 1
    if "not UN Comtrade records" not in corpus["note"]:
        print("FAIL: fixture must be labelled synthetic", file=sys.stderr)
        return 1
    if not isinstance(corpus["cases"], list) or not corpus["cases"]:
        print("FAIL: fixture corpus has no cases", file=sys.stderr)
        return 1

    seen: set[str] = set()
    failures: list[str] = []
    for index, case in enumerate(corpus["cases"]):
        if not isinstance(case, dict) or not CASE_KEYS.issuperset(case):
            failures.append(f"case[{index}]: malformed or unknown fields")
            continue
        case_id = case.get("case_id")
        if not isinstance(case_id, str) or not case_id or case_id in seen:
            failures.append(f"case[{index}]: missing or duplicate case_id")
            continue
        seen.add(case_id)

        actual_status, actual_signed, actual_absolute = evaluate(case)
        expected_status = case.get("expected_status")
        expected_signed = case.get("expected_signed_difference")
        expected_absolute = case.get("expected_absolute_difference")
        if (actual_status, actual_signed, actual_absolute) != (
            expected_status,
            expected_signed,
            expected_absolute,
        ):
            failures.append(
                f"{case_id}: expected "
                f"({expected_status}, {expected_signed!r}, {expected_absolute!r}), got "
                f"({actual_status}, {actual_signed!r}, {actual_absolute!r})"
            )

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        print(
            f"FAILED: {len(failures)} issue(s) across {len(corpus['cases'])} cases",
            file=sys.stderr,
        )
        return 1

    print(
        f"PASS: {len(corpus['cases'])}/{len(corpus['cases'])} frozen synthetic "
        "trade-mirror reference cases; exact decimal/status oracle agrees."
    )
    print("PASS: source-shaped UN Comtrade preview fixture; synthetic-only, exact primaryValue retained.")
    print("SCOPE: fixture validation only; not live data, source qualification, or IFF estimation.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
