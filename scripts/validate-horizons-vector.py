#!/usr/bin/env python3
"""Validate one JPL Horizons GET vector capture without network access.

This validates a response envelope, request identity context, metadata, labelled
columns, and exactly one TDB state row. It does not verify request/response SHA-256
receipts; the capture shell records those bytes before calling this validator.
"""
from __future__ import annotations

import csv
import json
import math
import sys
from pathlib import Path


SUPPORTED_GET_SIGNATURE_VERSIONS = {"1.0", "1.3"}


def fail(message: str) -> "NoReturn":
    raise SystemExit(message)


def body_name(value: str) -> str:
    """Normalize Horizons body names such as 'Mars (499)' or '1 Ceres (1)'."""
    value = value.split("{", 1)[0].split("(", 1)[0].strip()
    first, separator, remainder = value.partition(" ")
    if separator and first.isascii() and first.isdigit():
        return remainder.strip()
    return value


def validate_response(
    response_path: str,
    expected_target: str,
    expected_center: str,
    expected_frame: str,
    expected_plane: str,
    expected_correction: str,
    epoch_text: str,
) -> None:
    raw = Path(response_path).read_bytes()
    try:
        payload = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"invalid Horizons JSON response: {error}")

    signature = payload.get("signature")
    if not isinstance(signature, dict):
        fail("response lacks a JSON signature object")
    source = signature.get("source")
    version = signature.get("version")
    if source != "NASA/JPL Horizons API":
        fail(f"unexpected provider source: {source!r}")
    if version not in SUPPORTED_GET_SIGNATURE_VERSIONS:
        fail(f"unreviewed Horizons GET signature version: {version!r}")
    if payload.get("error"):
        fail(f"Horizons application-level error: {payload['error']!r}")
    result = payload.get("result")
    if not isinstance(result, str):
        fail("response lacks result text")

    lines = result.splitlines()
    start_marker = "$" * 2 + "SOE"
    end_marker = "$" * 2 + "EOE"
    starts = [index for index, line in enumerate(lines) if line.strip() == start_marker]
    ends = [index for index, line in enumerate(lines) if line.strip() == end_marker]
    if len(starts) != 1 or len(ends) != 1 or starts[0] >= ends[0]:
        fail(f"missing or duplicated {start_marker}/{end_marker} markers")

    header = lines[: starts[0]]

    def metadata(label: str) -> str:
        for line in header:
            if ":" in line:
                key, value = line.split(":", 1)
                if key.strip().casefold() == label.casefold():
                    return value.strip()
        fail(f"missing response metadata: {label}")
        raise AssertionError("unreachable")

    actual_target = body_name(metadata("Target body name"))
    actual_center = body_name(metadata("Center body name"))
    if actual_target.casefold() != expected_target.casefold():
        fail(f"target mismatch: expected {expected_target!r}, got {actual_target!r}")
    if actual_center.casefold() != expected_center.casefold():
        fail(f"center mismatch: expected {expected_center!r}, got {actual_center!r}")

    for label, expected in [
        ("Reference frame", expected_frame),
        ("Reference plane", expected_plane),
        ("Aberration corrections", expected_correction),
        ("Output units", "KM-S"),
    ]:
        actual = metadata(label)
        if label == "Reference frame" and expected == "B1950":
            valid = actual.casefold() in {"b1950", "fk4/b1950"}
        else:
            valid = actual.casefold() == expected.casefold()
        if not valid:
            fail(f"{label} mismatch: expected {expected!r}, got {actual!r}")

    column_line = next(
        (line for line in reversed(header) if "JDTDB" in line.upper() and "," in line),
        None,
    )
    if column_line is None:
        fail("missing labelled JDTDB vector-column header")
    columns = [field.strip().upper() for field in next(csv.reader([column_line]))]
    expected_columns = [
        "JDTDB",
        "CALENDAR DATE (TDB)",
        "X",
        "Y",
        "Z",
        "VX",
        "VY",
        "VZ",
    ]
    if columns != expected_columns:
        fail(f"unexpected vector columns: {columns!r}")

    data_lines = [line for line in lines[starts[0] + 1 : ends[0]] if line.strip()]
    if len(data_lines) != 1:
        fail(f"expected one output row, got {len(data_lines)}")
    fields = next(csv.reader([data_lines[0]]))
    if len(fields) != 8:
        fail(f"expected 8 CSV fields, got {len(fields)}")
    try:
        output_epoch = float(fields[0])
        expected_epoch = float(epoch_text)
        components = [float(field) for field in fields[-6:]]
    except ValueError:
        fail("epoch or state components are not numeric")
    if not math.isfinite(output_epoch) or any(
        not math.isfinite(value) for value in components
    ):
        fail("epoch or state components are non-finite")
    if abs(output_epoch - expected_epoch) > 1.0e-8:
        fail(f"epoch mismatch: expected {expected_epoch}, got {output_epoch}")


def main(argv: list[str]) -> int:
    if len(argv) != 8:
        print(
            "Usage: validate-horizons-vector.py RESPONSE_JSON TARGET CENTER "
            "REF_SYSTEM REF_PLANE VEC_CORR EPOCH_JD_TDB",
            file=sys.stderr,
        )
        return 2
    try:
        validate_response(*argv[1:])
    except SystemExit as error:
        message = error.code if isinstance(error.code, str) else "validation failed"
        print(f"Horizons capture validation failed: {message}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
