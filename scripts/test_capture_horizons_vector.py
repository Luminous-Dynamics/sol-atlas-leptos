#!/usr/bin/env python3
"""Offline end-to-end tests for the JPL capture command.

Only curl is mocked. The repository's real shell script, jq JSON receipt generation,
response validator, and independent packet verifier are exercised. No network
request is made.
"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CAPTURE_SCRIPT = ROOT / "scripts/capture-horizons-vector.sh"
VERIFY_SCRIPT = ROOT / "scripts/verify-horizons-capture.py"
FIXTURE = ROOT / "sol-atlas-core/tests/fixtures/horizons/mars_ssb_tdb_frame_km_s.json"

MOCK_CURL = r'''#!/usr/bin/env python3
import os
import sys
from pathlib import Path

args = sys.argv[1:]

def value_after(flag):
    return args[args.index(flag) + 1]

header_path = Path(value_after("--dump-header"))
body_path = Path(value_after("--output"))
status = os.environ.get("MOCK_HTTP_STATUS", "200")
curl_exit = int(os.environ.get("MOCK_CURL_EXIT_CODE", "0"))
reason = {
    "200": "OK",
    "400": "Bad Request",
    "500": "Internal Server Error",
    "503": "Service Unavailable",
}.get(status, "Mock Status")
header_path.write_bytes(
    f"HTTP/2 {status} {reason}\r\ncontent-type: application/json\r\n\r\n".encode("ascii")
)
body_path.write_bytes(Path(os.environ["MOCK_HORIZONS_RESPONSE"]).read_bytes())
sys.stdout.write(status)
raise SystemExit(curl_exit)
'''


@unittest.skipUnless(shutil.which("jq"), "jq is required by the capture script")
class CaptureScriptIntegrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        bin_dir = self.root / "bin"
        bin_dir.mkdir()
        mock_curl = bin_dir / "curl"
        mock_curl.write_text(MOCK_CURL, encoding="utf-8")
        mock_curl.chmod(0o755)

        env = os.environ.copy()
        env["PATH"] = f"{bin_dir}{os.pathsep}{env.get('PATH', '')}"
        env["MOCK_HORIZONS_RESPONSE"] = str(FIXTURE)
        env["HORIZONS_TARGET_ID"] = "mars"
        env["HORIZONS_CENTER_ID"] = "ssb"
        env["HORIZONS_COMMAND"] = "499"
        env["HORIZONS_CENTER"] = "@0"
        env["HORIZONS_EPOCH_JD"] = "2461323.5"
        env["HORIZONS_EXPECTED_TARGET"] = "Mars"
        env["HORIZONS_EXPECTED_CENTER"] = "Solar System Barycenter"
        self.env = env

    def run_capture(self, name: str) -> tuple[subprocess.CompletedProcess[str], Path]:
        destination = self.root / name
        result = subprocess.run(
            [str(CAPTURE_SCRIPT), str(destination)],
            cwd=ROOT,
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        return result, destination

    def test_ambiguous_catalogue_manifest_is_rejected_before_request(self) -> None:
        manifests = [
            (
                "duplicate-json-key",
                b'{"schema_version":1,"schema_version":1,"objects":[]}',
                "duplicate JSON object key: 'schema_version'",
            ),
            (
                "duplicate-catalogue-id",
                json.dumps(
                    {
                        "schema_version": 1,
                        "objects": [
                            {"id": "mars", "name": "Mars", "kind": "planet", "command": "499", "center": "@499"},
                            {"id": "mars", "name": "Mars", "kind": "planet", "command": "499", "center": "@499"},
                        ],
                        "special_centers": [
                            {"id": "ssb", "name": "Solar System Barycenter", "provider_center": "@0"}
                        ],
                    }
                ).encode("utf-8"),
                "duplicate catalogue binding ID: mars",
            ),
        ]
        for case_name, manifest_bytes, expected_error in manifests:
            with self.subTest(case=case_name):
                mini_root = self.root / case_name
                scripts_dir = mini_root / "scripts"
                manifest_path = (
                    mini_root
                    / "sol-atlas-core/tests/fixtures/horizons/catalogue-bindings.json"
                )
                scripts_dir.mkdir(parents=True)
                manifest_path.parent.mkdir(parents=True)
                shutil.copy2(CAPTURE_SCRIPT, scripts_dir / CAPTURE_SCRIPT.name)
                shutil.copy2(
                    ROOT / "scripts/horizons_catalogue_manifest.py",
                    scripts_dir / "horizons_catalogue_manifest.py",
                )
                manifest_path.write_bytes(manifest_bytes)

                result = subprocess.run(
                    [str(scripts_dir / CAPTURE_SCRIPT.name), "--print-identity"],
                    cwd=ROOT,
                    env=self.env,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(expected_error, result.stderr)

    def test_self_centered_request_is_rejected_before_curl(self) -> None:
        self.env["HORIZONS_CENTER_ID"] = "MARS"
        result = subprocess.run(
            [str(CAPTURE_SCRIPT), "--print-identity"],
            cwd=ROOT,
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("target cannot be its own center", result.stderr)

    def test_stable_ids_follow_rust_canonicalization_rules(self) -> None:
        cases = [
            ({"HORIZONS_TARGET_ID": "MARS"}, "catalogue target ID must use canonical spelling"),
            ({"HORIZONS_TARGET_ID": "ssb"}, "ssb is a coordinate center, not a target"),
            ({"HORIZONS_CENTER_ID": "EARTH"}, "catalogue center ID must use canonical spelling"),
            ({"HORIZONS_CENTER_ID": "SSB"}, "ssb center ID must use canonical spelling"),
            ({"HORIZONS_TARGET_ID": "custom-Mars"}, "target and center IDs must be lowercase ASCII slugs"),
        ]
        for overrides, expected_error in cases:
            with self.subTest(overrides=overrides):
                env = self.env.copy()
                env.update(overrides)
                result = subprocess.run(
                    [str(CAPTURE_SCRIPT), "--print-identity"],
                    cwd=ROOT,
                    env=env,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(expected_error, result.stderr)

    def test_successful_mock_capture_verifies_end_to_end(self) -> None:
        result, destination = self.run_capture("success")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name in (
            "request.url",
            "request.identity",
            "response.raw.json",
            "response.headers",
            "capture-receipt.json",
            "capture-metadata.json",
        ):
            self.assertTrue((destination / name).is_file(), name)

        verify = subprocess.run(
            ["python3", str(VERIFY_SCRIPT), str(destination)],
            cwd=ROOT,
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(verify.returncode, 0, verify.stdout + verify.stderr)
        self.assertIn("capture verification: PASS", verify.stdout)

    def test_requested_epoch_metadata_matches_canonical_transmitted_epoch(self) -> None:
        # This decimal is below the f64 resolution at this JD; the transmitted
        # epoch canonicalizes to 2461323.5 and metadata must say the same thing.
        self.env["HORIZONS_EPOCH_JD"] = "2461323.5000000001"
        result, destination = self.run_capture("epoch-canonicalized")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

        url = (destination / "request.url").read_text(encoding="utf-8")
        metadata = json.loads(
            (destination / "capture-metadata.json").read_text(encoding="utf-8")
        )
        self.assertIn("TLIST=%272461323.5%27", url)
        self.assertEqual(metadata["requested_epoch_jd_tdb"], "2461323.5")

        verify = subprocess.run(
            ["python3", str(VERIFY_SCRIPT), str(destination)],
            cwd=ROOT,
            env=self.env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(verify.returncode, 0, verify.stdout + verify.stderr)

    def test_http_error_is_sealed_before_capture_fails(self) -> None:
        self.env["MOCK_HTTP_STATUS"] = "503"
        result, destination = self.run_capture("http-error")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((destination / "capture-receipt.json").is_file())
        self.assertTrue((destination / "response.headers").is_file())

        receipt = json.loads((destination / "capture-receipt.json").read_text(encoding="utf-8"))
        metadata = json.loads((destination / "capture-metadata.json").read_text(encoding="utf-8"))
        self.assertEqual(receipt["receipt_status"], "captured-http-error")
        self.assertEqual(metadata["fixture_status"], "captured-http-error")

        verify = subprocess.run(
            ["python3", str(VERIFY_SCRIPT), str(destination)],
            cwd=ROOT, env=self.env, text=True,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("capture HTTP status must be 200, got 503", verify.stderr)

    def test_transport_failure_is_sealed_and_never_verifiable(self) -> None:
        self.env["MOCK_CURL_EXIT_CODE"] = "28"
        result, destination = self.run_capture("transport-error")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((destination / "capture-receipt.json").is_file())
        self.assertTrue((destination / "response.raw.json").is_file())

        receipt = json.loads((destination / "capture-receipt.json").read_text(encoding="utf-8"))
        metadata = json.loads((destination / "capture-metadata.json").read_text(encoding="utf-8"))
        self.assertEqual(receipt["receipt_status"], "capture-incomplete")
        self.assertEqual(metadata["fixture_status"], "capture-incomplete")
        self.assertEqual(receipt["curl_exit_code"], "28")

        verify = subprocess.run(
            ["python3", str(VERIFY_SCRIPT), str(destination)],
            cwd=ROOT, env=self.env, text=True,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("capture has nonzero curl exit code 28", verify.stderr)


if __name__ == "__main__":
    unittest.main()
