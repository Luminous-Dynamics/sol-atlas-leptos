#!/usr/bin/env python3
"""Offline verifier for a saved JPL Horizons vector capture packet.

This rechecks the exact saved bytes, semantic request identity, recorded digests,
and the response schema. It performs no network requests and never edits evidence.
Hash agreement demonstrates byte consistency, not provider or transport authenticity.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path
from typing import Any
from urllib.parse import quote


class CaptureVerificationError(ValueError):
    """A saved capture packet is incomplete or internally inconsistent."""


SCRIPT_DIR = Path(__file__).resolve().parent
VALIDATOR_PATH = SCRIPT_DIR / "validate-horizons-vector.py"
_spec = importlib.util.spec_from_file_location("validate_horizons_vector", VALIDATOR_PATH)
if _spec is None or _spec.loader is None:
    raise RuntimeError(f"cannot load response validator: {VALIDATOR_PATH}")
_validator = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_validator)


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    try:
        obj = json.loads(path.read_bytes())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise CaptureVerificationError(f"cannot read valid JSON from {path.name}: {error}") from error
    if not isinstance(obj, dict):
        raise CaptureVerificationError(f"{path.name} must contain a JSON object")
    return obj


def required_string(obj: dict[str, Any], key: str, source: str) -> str:
    value = obj.get(key)
    if not isinstance(value, str) or not value:
        raise CaptureVerificationError(f"{source}.{key} must be a non-empty string")
    return value


def expected_request_identity(metadata: dict[str, Any], url_bytes: bytes) -> bytes:
    try:
        url = url_bytes.decode("utf-8")
    except UnicodeDecodeError as error:
        raise CaptureVerificationError("request.url must be UTF-8") from error

    def enc(value: str) -> str:
        return quote(value, safe="-_.~")

    try:
        target_id = required_string(metadata, "requested_target_id", "metadata")
        center_id = required_string(metadata, "requested_center_id", "metadata")
        target_name = required_string(metadata, "expected_target_name", "metadata")
        center_name = required_string(metadata, "expected_center_name", "metadata")
    except CaptureVerificationError:
        raise

    identity = (
        "SOL-ATLAS-HORIZONS-REQUEST-V1\n"
        f"target_id={enc(target_id)}\n"
        f"center_id={enc(center_id)}\n"
        f"expected_target_name={enc(target_name)}\n"
        f"expected_center_name={enc(center_name)}\n"
        f"transport_bytes={len(url_bytes)}\n"
        f"{url}"
    )
    return identity.encode("utf-8")


def assert_equal(actual: Any, expected: Any, label: str) -> None:
    if actual != expected:
        raise CaptureVerificationError(
            f"{label} mismatch: recorded={actual!r}, verified={expected!r}"
        )


def verify_capture(directory: Path) -> dict[str, str]:
    """Verify a capture directory and return the computed SHA-256 digests."""
    if not directory.is_dir():
        raise CaptureVerificationError(f"capture directory not found: {directory}")

    paths = {
        "url": directory / "request.url",
        "identity": directory / "request.identity",
        "response": directory / "response.raw.json",
        "receipt": directory / "capture-receipt.json",
        "metadata": directory / "capture-metadata.json",
    }
    for label, path in paths.items():
        if not path.is_file():
            raise CaptureVerificationError(f"missing {label} file: {path.name}")

    url_bytes = paths["url"].read_bytes()
    identity_bytes = paths["identity"].read_bytes()
    response_bytes = paths["response"].read_bytes()
    receipt = read_json(paths["receipt"])
    metadata = read_json(paths["metadata"])

    if receipt.get("receipt_status") != "captured-unreviewed":
        raise CaptureVerificationError("receipt status is not captured-unreviewed")
    if metadata.get("fixture_status") != "captured-not-yet-reviewed":
        raise CaptureVerificationError("metadata status is not captured-not-yet-reviewed")

    recomputed_identity = expected_request_identity(metadata, url_bytes)
    assert_equal(identity_bytes, recomputed_identity, "request.identity bytes")

    digests = {
        "canonical_request_sha256": sha256_hex(identity_bytes),
        "canonical_url_sha256": sha256_hex(url_bytes),
        "raw_response_sha256": sha256_hex(response_bytes),
    }
    for label, digest in digests.items():
        assert_equal(receipt.get(label), digest, f"receipt.{label}")
        assert_equal(metadata.get(label), digest, f"metadata.{label}")

    retrieved_at = required_string(metadata, "retrieved_at_utc", "metadata")
    assert_equal(receipt.get("retrieved_at_utc"), retrieved_at, "retrieved_at_utc")
    assert_equal(
        receipt.get("reported_provider_source"),
        metadata.get("provider"),
        "provider source",
    )
    assert_equal(
        receipt.get("reported_api_version"),
        metadata.get("api_signature_version"),
        "API signature version",
    )

    required_string(metadata, "requested_target", "metadata")
    required_string(metadata, "requested_center", "metadata")
    required_string(metadata, "expected_target_name", "metadata")
    required_string(metadata, "expected_center_name", "metadata")

    try:
        _validator.validate_response(
            str(paths["response"]),
            metadata["expected_target_name"],
            metadata["expected_center_name"],
            metadata["reference_system"],
            metadata["reference_plane"],
            metadata["vector_correction"],
            metadata["requested_epoch_jd_tdb"],
        )
    except (KeyError, SystemExit) as error:
        detail = error.code if isinstance(error, SystemExit) else str(error)
        raise CaptureVerificationError(f"response schema validation failed: {detail}") from error

    return digests


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("Usage: verify-horizons-capture.py CAPTURE_DIRECTORY", file=sys.stderr)
        return 2
    try:
        digests = verify_capture(Path(argv[1]))
    except (CaptureVerificationError, OSError) as error:
        print(f"capture verification failed: {error}", file=sys.stderr)
        return 1

    print("capture verification: PASS (byte consistency and schema only)")
    for label, digest in digests.items():
        print(f"{label}: {digest}")
    print("Authenticity status: not established by hashes alone")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
