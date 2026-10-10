#!/usr/bin/env python3
"""Offline regression tests for the Horizons capture validator."""
from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[1]
FIXTURE_PATH = ROOT / "sol-atlas-core/tests/fixtures/horizons/mars_ssb_tdb_frame_km_s.json"
VALIDATOR_PATH = Path(__file__).with_name("validate-horizons-vector.py")

spec = importlib.util.spec_from_file_location("validate_horizons_vector", VALIDATOR_PATH)
assert spec is not None and spec.loader is not None
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


class HorizonsCaptureValidatorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))

    def validate(self, payload: dict, target: str = "Mars") -> None:
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "response.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
            validator.validate_response(
                str(path),
                target,
                "Solar System Barycenter",
                "ICRF",
                "FRAME",
                "NONE",
                "2461323.5",
            )

    def test_synthetic_fixture_is_valid(self) -> None:
        self.validate(self.fixture)

    def test_wrong_target_fails_closed(self) -> None:
        with self.assertRaisesRegex(SystemExit, "target mismatch"):
            self.validate(self.fixture, target="Venus")

    def test_missing_exact_horizons_markers_fails_closed(self) -> None:
        payload = json.loads(json.dumps(self.fixture))
        start_marker = "$" * 2 + "SOE"
        payload["result"] = payload["result"].replace(start_marker, "$SOE")
        with self.assertRaisesRegex(SystemExit, "markers"):
            self.validate(payload)

    def test_unknown_signature_version_fails_closed(self) -> None:
        payload = json.loads(json.dumps(self.fixture))
        payload["signature"]["version"] = "99.0"
        with self.assertRaisesRegex(SystemExit, "signature version"):
            self.validate(payload)

    def test_reordered_vector_columns_fail_closed(self) -> None:
        payload = json.loads(json.dumps(self.fixture))
        payload["result"] = payload["result"].replace(
            "JDTDB, Calendar Date (TDB), X, Y, Z, VX, VY, VZ",
            "JDTDB, Calendar Date (TDB), Y, X, Z, VX, VY, VZ",
        )
        with self.assertRaisesRegex(SystemExit, "unexpected vector columns"):
            self.validate(payload)

    def test_wrong_epoch_fails_closed(self) -> None:
        payload = json.loads(json.dumps(self.fixture))
        payload["result"] = payload["result"].replace(
            "2461323.500000000",
            "2461324.500000000",
        )
        with self.assertRaisesRegex(SystemExit, "epoch mismatch"):
            self.validate(payload)

    def test_non_finite_vector_component_fails_closed(self) -> None:
        payload = json.loads(json.dumps(self.fixture))
        payload["result"] = payload["result"].replace(
            "1.782345678901234E+08",
            "NaN",
        )
        with self.assertRaisesRegex(SystemExit, "non-finite"):
            self.validate(payload)


if __name__ == "__main__":
    unittest.main()
