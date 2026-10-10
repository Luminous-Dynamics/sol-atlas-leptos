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
