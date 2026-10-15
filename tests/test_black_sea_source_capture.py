import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
import capture_black_sea_sources as capture


class BlackSeaSourceCaptureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.plan_path = ROOT / "docs" / "pilots" / "black-sea-grain-trade-query-plan.json"
        cls.plan = json.loads(cls.plan_path.read_text(encoding="utf-8"))

    def test_plan_builds_two_companion_assets_and_168_single_product_month_requests(self):
        tasks = capture.build_request_tasks(self.plan)
        self.assertEqual(len(tasks), 170)
        self.assertEqual(sum(t["response_kind"] == "comtrade_json" for t in tasks), 168)
        self.assertEqual(len({t["filename"] for t in tasks}), len(tasks))
        comtrade = [t for t in tasks if t["response_kind"] == "comtrade_json"]
        expected = [
            (f"{year}{month:02d}", code)
            for year in range(2017, 2024)
            for month in range(1, 13)
            for code in ("1001", "1005")
        ]
        observed = []
        for task, (period, code) in zip(comtrade, expected):
            parsed, params = urlsplit(task["url"]), parse_qs(urlsplit(task["url"]).query)
            self.assertEqual(parsed.scheme, "https")
            self.assertEqual(parsed.hostname, "comtradeapi.un.org")
            self.assertEqual(params["reporterCode"], ["804"])
            self.assertEqual(params["partnerCode"], ["0"])
            self.assertEqual(params["flowCode"], ["X"])
            self.assertEqual(params["cmdCode"], [code])
            self.assertEqual(params["period"], [period])
            self.assertNotIn("subscription-key", parsed.query.lower())
            observed.append((params["period"][0], params["cmdCode"][0]))
        self.assertEqual(observed, expected)
        self.assertEqual(self.plan["request_policy"]["minimum_seconds_between_comtrade_requests"], 1.0)
        self.assertIs(self.plan["request_policy"]["no_parallel_comtrade_requests"], True)

    def test_url_validation_accepts_reviewed_sources_and_rejects_redirect_escape(self):
        allowed = (
            "https://www.un.org/en/node/207702",
            "https://www.fao.org/media/docs/worldfoodsituationlibraries/wfs-library/food_price_indices_data.csv?download=true&sfvrsn=523ebd2a_84",
            "https://comtradeapi.un.org/public/v1/preview/C/M/HS?reporterCode=804&period=202201",
        )
        denied = (
            "http://www.un.org/en/node/207702",
            "https://evil.example/en/node/207702",
            "https://comtradeapi.un.org/public/v1/get/C/M/HS",
            "https://comtradeapi.un.org/public/v1/preview/C/M/HS?subscription-key=secret",
            "https://www.fao.org/media/../private.csv",
        )
        for url in allowed:
            with self.subTest(url=url):
                capture.validate_source_url(url)
        for url in denied:
            with self.subTest(url=url):
                with self.assertRaises(capture.CaptureError):
                    capture.validate_source_url(url)

    def test_comtrade_envelope_rejects_count_mismatch_and_preview_ceiling(self):
        valid = json.dumps({"count": 2, "data": [{"period": "202201"}, {"period": "202202"}]}).encode()
        result = capture.assess_comtrade_response(valid)
        self.assertEqual(result["state"], "response_shape_ok_pending_coverage_review")
        self.assertEqual((result["count"], result["returned_rows"]), (2, 2))
        mismatch = json.dumps({"count": 3, "data": [{"period": "202201"}]}).encode()
        self.assertEqual(capture.assess_comtrade_response(mismatch)["state"], "captured_but_incomplete")
        ceiling = json.dumps({"count": 500, "data": [{} for _ in range(500)]}).encode()
        self.assertEqual(capture.assess_comtrade_response(ceiling)["state"], "captured_but_incomplete")
        malformed = b'{"count":2,"data":"not-an-array"}'
        self.assertEqual(capture.assess_comtrade_response(malformed)["state"], "captured_but_unrecognized_schema")

    def test_exact_response_bytes_are_hashed_and_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            snapshot = root / "snapshot"
            snapshot.mkdir()
            raw = b"month,index\n2022-01,100.0\n"
            task = {
                "source_id": "FAO-FFPI", "artifact_id": "test-csv",
                "url": "https://www.fao.org/media/docs/worldfoodsituationlibraries/wfs-library/food_price_indices_data.csv?download=true",
                "filename": "test.csv", "max_bytes": 1000, "expected_content_types": ["text/csv"],
                "response_kind": "csv", "request_parameters": None,
            }
            def fake_fetcher(url, max_bytes, timeout_seconds):
                return {"status_code": 200, "final_url": url, "content_type": "text/csv",
                        "headers": {"ETag": '"test"'}, "raw": raw, "error": None}
            record = capture.capture_one(task, snapshot, 1, fetcher=fake_fetcher)
            self.assertEqual(record["capture_state"], "captured_pending_metadata_review")
            self.assertEqual((snapshot / "test.csv").read_bytes(), raw)
            self.assertEqual(record["response_bytes"], len(raw))
            self.assertEqual(record["response_sha256"], hashlib.sha256(raw).hexdigest())
            second = capture.capture_one(task, snapshot, 1, fetcher=fake_fetcher)
            self.assertEqual(second["capture_state"], "storage_failed")
            self.assertEqual((snapshot / "test.csv").read_bytes(), raw)

    def test_snapshot_directory_must_be_new(self):
        with tempfile.TemporaryDirectory() as temp:
            existing = Path(temp) / "snapshot"
            existing.mkdir()
            with self.assertRaises(capture.CaptureError):
                capture.create_snapshot_dir(existing)
            with self.assertRaises(capture.CaptureError):
                capture.create_snapshot_dir(Path(temp) / "missing" / "snapshot")

    def test_plan_does_not_claim_hashes_or_rights_review(self):
        self.assertEqual(self.plan["execution_state"], "not_run")
        self.assertFalse(self.plan["source"]["raw_response_bytes_acquired"])
        self.assertIsNone(self.plan["source"]["source_sha256"])
        self.assertEqual(self.plan["control_design"]["selected_controls"], [])
        self.assertEqual(len(self.plan["companion_artifacts"]), 2)


if __name__ == "__main__":
    unittest.main()
