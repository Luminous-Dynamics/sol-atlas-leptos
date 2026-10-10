#!/usr/bin/env python3
"""Offline tests for request/response evidence-packet verification."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
FIXTURE_PATH = ROOT / "sol-atlas-core/tests/fixtures/horizons/mars_ssb_tdb_frame_km_s.json"
VERIFIER_PATH = Path(__file__).with_name("verify-horizons-capture.py")

spec = importlib.util.spec_from_file_location("verify_horizons_capture", VERIFIER_PATH)
assert spec is not None and spec.loader is not None
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def default_url() -> bytes:
    return (
        "https://ssd.jpl.nasa.gov/api/horizons.api?"
        "COMMAND=%27499%27&CENTER=%27%400%27&CSV_FORMAT=%27YES%27&"
        "EPHEM_TYPE=%27VECTORS%27&MAKE_EPHEM=%27YES%27&OBJ_DATA=%27YES%27&"
        "OUT_UNITS=%27KM-S%27&REF_PLANE=%27FRAME%27&REF_SYSTEM=%27ICRF%27&"
        "TIME_TYPE=%27TDB%27&TLIST=%272461323.5%27&TLIST_TYPE=%27JD%27&"
        "VEC_CORR=%27NONE%27&VEC_LABELS=%27YES%27&VEC_TABLE=%272%27&format=json"
    ).encode("utf-8")


def canonical_identity(url: bytes) -> bytes:
    # Independent expectation for the default operator capture identity.
    return (
        "SOL-ATLAS-HORIZONS-REQUEST-V1\n"
        "target_id=mars\n"
        "center_id=ssb\n"
        "expected_target_name=Mars\n"
        "expected_center_name=Solar%20System%20Barycenter\n"
        f"transport_bytes={len(url)}\n"
        + url.decode("utf-8")
    ).encode("utf-8")


class HorizonsCapturePacketTests(unittest.TestCase):
    def make_capture(self, folder: Path) -> Path:
        folder.mkdir()
        url = default_url()
        identity = canonical_identity(url)
        response = FIXTURE_PATH.read_bytes()
        metadata = {
            "fixture_status": "captured-not-yet-reviewed",
            "provider": "NASA/JPL Horizons API",
            "api_signature_version": "1.3",
            "requested_target_id": "mars",
            "requested_center_id": "ssb",
            "requested_target": "499",
            "requested_center": "@0",
            "expected_target_name": "Mars",
            "expected_center_name": "Solar System Barycenter",
            "requested_epoch_jd_tdb": "2461323.5",
            "reference_system": "ICRF",
            "reference_plane": "FRAME",
            "vector_correction": "NONE",
            "output_units": "KM-S",
            "retrieved_at_utc": "2026-10-10T18:00:00Z",
            "canonical_request_sha256": sha256(identity),
            "canonical_url_sha256": sha256(url),
            "raw_response_sha256": sha256(response),
        }
        receipt = {
            "receipt_status": "captured-unreviewed",
            "canonical_request_sha256": sha256(identity),
            "canonical_url_sha256": sha256(url),
            "raw_response_sha256": sha256(response),
            "retrieved_at_utc": "2026-10-10T18:00:00Z",
            "reported_provider_source": "NASA/JPL Horizons API",
            "reported_api_version": "1.3",
        }
        (folder / "request.url").write_bytes(url)
        (folder / "request.identity").write_bytes(identity)
        (folder / "response.raw.json").write_bytes(response)
        (folder / "capture-metadata.json").write_text(json.dumps(metadata), encoding="utf-8")
        (folder / "capture-receipt.json").write_text(json.dumps(receipt), encoding="utf-8")
        return folder

    def test_valid_synthetic_packet_passes_byte_checks_and_schema(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            digests = verifier.verify_capture(capture)
            self.assertEqual(set(digests), {
                "canonical_request_sha256",
                "canonical_url_sha256",
                "raw_response_sha256",
            })
            self.assertEqual(digests["canonical_url_sha256"], sha256(default_url()))

    def test_request_url_parameter_order_must_be_canonical(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            metadata = json.loads(
                (capture / "capture-metadata.json").read_text(encoding="utf-8")
            )
            url = default_url()
            base, query = url.split(b"?", 1)
            reordered = base + b"?" + b"&".join(reversed(query.split(b"&")))
            with self.assertRaisesRegex(
                verifier.CaptureVerificationError,
                "canonical request URL parameter order mismatch",
            ):
                verifier.validate_url_metadata(reordered, metadata)

    def test_identity_format_is_length_delimited_and_has_no_trailing_newline(self) -> None:
        url = default_url()
        identity = canonical_identity(url)
        self.assertTrue(identity.startswith(b"SOL-ATLAS-HORIZONS-REQUEST-V1\n"))
        self.assertIn(f"transport_bytes={len(url)}\n".encode(), identity)
        self.assertFalse(identity.endswith(b"\n"))
        self.assertIn(b"expected_center_name=Solar%20System%20Barycenter\n", identity)

    def test_tampered_url_fails_even_when_response_is_unchanged(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            (capture / "request.url").write_bytes(default_url() + b"#tampered")
            with self.assertRaisesRegex(verifier.CaptureVerificationError, "request.identity bytes mismatch"):
                verifier.verify_capture(capture)

    def test_tampered_raw_response_fails_digest_check(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            response_path = capture / "response.raw.json"
            response_path.write_bytes(response_path.read_bytes() + b" ")
            with self.assertRaisesRegex(verifier.CaptureVerificationError, "raw_response_sha256 mismatch"):
                verifier.verify_capture(capture)

    def test_semantic_metadata_change_fails_identity_check(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            path = capture / "capture-metadata.json"
            metadata = json.loads(path.read_text(encoding="utf-8"))
            metadata["requested_target_id"] = "venus"
            path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(verifier.CaptureVerificationError, "request.identity bytes mismatch"):
                verifier.verify_capture(capture)

    def test_consistently_rehashed_but_invalid_response_fails_schema(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            raw_path = capture / "response.raw.json"
            payload = json.loads(raw_path.read_bytes())
            payload["result"] = payload["result"].replace("1.782345678901234E+08", "NaN")
            raw = json.dumps(payload).encode("utf-8")
            raw_path.write_bytes(raw)

            digest = sha256(raw)
            for name, fields in [
                ("capture-metadata.json", ["raw_response_sha256"]),
                ("capture-receipt.json", ["raw_response_sha256"]),
            ]:
                path = capture / name
                obj = json.loads(path.read_text(encoding="utf-8"))
                for field in fields:
                    obj[field] = digest
                path.write_text(json.dumps(obj), encoding="utf-8")

            with self.assertRaisesRegex(verifier.CaptureVerificationError, "response schema validation failed"):
                verifier.verify_capture(capture)

    def rehash_identity_for_metadata(self, capture: Path, metadata: dict) -> None:
        url = (capture / "request.url").read_bytes()
        identity = verifier.expected_request_identity(metadata, url)
        (capture / "request.identity").write_bytes(identity)
        identity_digest = sha256(identity)
        metadata["canonical_request_sha256"] = identity_digest
        (capture / "capture-metadata.json").write_text(json.dumps(metadata), encoding="utf-8")
        receipt_path = capture / "capture-receipt.json"
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipt["canonical_request_sha256"] = identity_digest
        receipt_path.write_text(json.dumps(receipt), encoding="utf-8")

    def test_coordinate_metadata_must_match_the_exact_url_parameters(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            metadata_path = capture / "capture-metadata.json"
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            metadata["reference_plane"] = "ECLIPTIC"
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(
                verifier.CaptureVerificationError,
                "request.url REF_PLANE versus metadata mismatch",
            ):
                verifier.verify_capture(capture)

    def test_epoch_metadata_must_match_tlist_even_if_hash_fields_remain_consistent(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            metadata_path = capture / "capture-metadata.json"
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            metadata["requested_epoch_jd_tdb"] = "2461324.5"
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            with self.assertRaisesRegex(
                verifier.CaptureVerificationError,
                "request.url TLIST epoch does not match metadata epoch",
            ):
                verifier.verify_capture(capture)

    def test_relabelled_provider_metadata_fails_against_response_signature(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            metadata_path = capture / "capture-metadata.json"
            receipt_path = capture / "capture-receipt.json"
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            metadata["provider"] = "claimed alternate provider"
            receipt["reported_provider_source"] = "claimed alternate provider"
            metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
            receipt_path.write_text(json.dumps(receipt), encoding="utf-8")
            with self.assertRaisesRegex(
                verifier.CaptureVerificationError,
                "metadata provider versus response signature mismatch",
            ):
                verifier.verify_capture(capture)

    def test_rehashed_known_target_relabeling_still_fails_binding_check(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            metadata_path = capture / "capture-metadata.json"
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            metadata["requested_target_id"] = "venus"
            self.rehash_identity_for_metadata(capture, metadata)
            with self.assertRaisesRegex(
                verifier.CaptureVerificationError,
                "catalogue target binding mismatch for venus",
            ):
                verifier.verify_capture(capture)

    def test_rehashed_aggregate_layer_still_fails_point_target_check(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            metadata_path = capture / "capture-metadata.json"
            metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
            metadata["requested_target_id"] = "comets"
            self.rehash_identity_for_metadata(capture, metadata)
            with self.assertRaisesRegex(
                verifier.CaptureVerificationError,
                "aggregate catalogue layer cannot be queried as a point target",
            ):
                verifier.verify_capture(capture)

    def test_missing_packet_member_fails_closed(self) -> None:
        with TemporaryDirectory() as temporary:
            capture = self.make_capture(Path(temporary) / "capture")
            (capture / "request.identity").unlink()
            with self.assertRaisesRegex(verifier.CaptureVerificationError, "missing identity file"):
                verifier.verify_capture(capture)


if __name__ == "__main__":
    unittest.main()
