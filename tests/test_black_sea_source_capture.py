import contextlib
import hashlib
import io
import json
import sys
import tempfile
import unittest
from unittest.mock import patch
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
        cls.current_context_path = ROOT / "docs" / "pilots" / "black-sea-current-context-2026-09.json"
        cls.current_context = json.loads(cls.current_context_path.read_text(encoding="utf-8"))

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
            self.assertEqual(params["reportercode"], ["804"])
            self.assertNotIn("reporterCode", params)
            self.assertEqual(task["expected_response_dimensions"]["reporterCode"], 804)
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

        empty = json.dumps({"count": 0, "data": []}).encode()
        empty_result = capture.assess_comtrade_response(empty)
        self.assertEqual(empty_result["state"], "empty_response_needs_availability_review")
        self.assertFalse(empty_result["rows_present"])
        self.assertFalse(empty_result["query_dimensions_verified"])

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

    def test_snapshot_write_fsyncs_and_verifies_exact_readback(self):
        with tempfile.TemporaryDirectory() as temp:
            target = Path(temp) / "source.bin"
            raw = b"immutable-source-bytes\\x00\\x01"
            with patch.object(capture.os, "fsync", wraps=capture.os.fsync) as fsync:
                capture.write_bytes_create_only(target, raw)
            self.assertEqual(target.read_bytes(), raw)
            fsync.assert_called_once()

    def test_snapshot_readback_mismatch_removes_file_and_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            target = Path(temp) / "source.bin"
            raw = b"expected-source-bytes"
            real_read_bytes = Path.read_bytes

            def corrupted_readback(path):
                result = real_read_bytes(path)
                if path == target:
                    return result + b"corruption"
                return result

            with patch.object(Path, "read_bytes", new=corrupted_readback):
                with self.assertRaisesRegex(OSError, "read-back integrity check failed"):
                    capture.write_bytes_create_only(target, raw)
            self.assertFalse(target.exists())

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


    def test_comtrade_response_must_match_requested_dimensions(self):
        row = {
            "reporterCode": 804, "partnerCode": 0, "flowCode": "X",
            "cmdCode": "1001", "period": "202201",
        }
        expected = {
            "reporterCode": 804, "partnerCode": 0, "flowCode": "X",
            "cmdCode": "1001", "period": "202201",
        }
        good = json.dumps({"count": 1, "data": [row]}).encode()
        result = capture.assess_comtrade_response(good, expected_parameters=expected)
        self.assertEqual(result["state"], "response_shape_ok_pending_coverage_review")
        self.assertTrue(result["query_dimensions_verified"])

        wrong = dict(row, cmdCode="1005")
        mismatch = json.dumps({"count": 1, "data": [wrong]}).encode()
        result = capture.assess_comtrade_response(mismatch, expected_parameters=expected)
        self.assertEqual(result["state"], "captured_query_dimension_mismatch")
        self.assertIn("cmdCode", result["dimension_mismatches"])

        missing = json.dumps({"count": 1, "data": [{"period": "202201"}]}).encode()
        result = capture.assess_comtrade_response(missing, expected_parameters=expected)
        self.assertEqual(result["state"], "captured_but_unrecognized_schema")
        self.assertIn("reporterCode", result["missing_query_dimension_fields"])

    def test_default_capture_does_not_run_comtrade_queries(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            snapshot = root / "snapshot"
            calls = []

            def fake_fetcher(url, max_bytes, timeout_seconds):
                calls.append(url)
                is_csv = "food_price_indices_data.csv" in url
                return {
                    "status_code": 200,
                    "final_url": url,
                    "content_type": "text/csv" if is_csv else "text/html",
                    "headers": {},
                    "raw": b"month,value\\n" if is_csv else b"<html></html>",
                    "error": None,
                }

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = capture.main(["--snapshot-dir", str(snapshot)], fetcher=fake_fetcher)
            self.assertEqual(exit_code, 0)
            self.assertEqual(len(calls), 2)
            self.assertTrue(all("comtradeapi.un.org" not in url for url in calls))
            manifest = json.loads((snapshot / "capture-manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["capture_state"], "companion_sources_captured_pending_review")
            self.assertEqual(manifest["plan_request_count"], 170)
            self.assertEqual(manifest["selected_request_count"], 2)
            self.assertEqual(manifest["comtrade_calls_not_selected_by_default"], 168)
            self.assertFalse(manifest["full_plan_captured_and_shape_checked"])
            self.assertFalse(manifest["rights_review_complete"])
            self.assertFalse(manifest["case_qualified_for_analysis"])

    def test_collector_stops_at_first_429_and_marks_remaining_requests_unattempted(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            snapshot = root / "snapshot"
            calls = []

            def fake_fetcher(url, max_bytes, timeout_seconds):
                calls.append(url)
                if "comtradeapi.un.org" in url:
                    return {
                        "status_code": 429,
                        "final_url": url,
                        "content_type": "application/json",
                        "headers": {"Retry-After": "3"},
                        "raw": None,
                        "error": "HTTP 429",
                    }
                is_csv = "food_price_indices_data.csv" in url
                return {
                    "status_code": 200,
                    "final_url": url,
                    "content_type": "text/csv" if is_csv else "text/html",
                    "headers": {},
                    "raw": b"month,value\\n" if is_csv else b"<html></html>",
                    "error": None,
                }

            with patch.object(capture.time, "sleep", return_value=None):
                with contextlib.redirect_stdout(io.StringIO()):
                    exit_code = capture.main(
                        ["--snapshot-dir", str(snapshot), "--include-comtrade"],
                        fetcher=fake_fetcher,
                    )
            self.assertEqual(exit_code, 2)
            self.assertEqual(len(calls), 3)
            manifest = json.loads((snapshot / "capture-manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["capture_state"], "stopped_after_rate_limit")
            self.assertTrue(manifest["rate_limit_hit"])
            self.assertEqual(manifest["stop_reason"], "upstream_rate_limit")
            self.assertEqual(manifest["attempted_request_count"], 3)
            self.assertEqual(manifest["not_attempted_request_count"], 167)
            self.assertFalse(manifest["full_plan_captured_and_shape_checked"])
            api_record = next(item for item in manifest["artifacts"] if item["source_id"] == "UN-COMTRADE")
            self.assertEqual(api_record["status_code"], 429)
            self.assertEqual(api_record["response_headers"]["Retry-After"], "3")
            self.assertTrue(all(
                item["capture_state"] == "not_attempted_after_rate_limit"
                for item in manifest["artifacts"][3:]
            ))

    def test_current_fao_context_is_separate_and_not_source_qualified(self):
        context = self.current_context
        self.assertEqual(context["observation_id"], "black-sea-global-food-market-context-2026-09")
        self.assertEqual(context["status"], "publisher_narrative_discovery_only")
        self.assertEqual(context["source"]["release_date"], "2026-10-02")
        self.assertEqual(context["reference_period"], "2026-09")
        self.assertFalse(context["source"]["exact_source_bytes_acquired"])
        self.assertIsNone(context["source"]["source_sha256"])
        self.assertTrue(context["case_relationship"]["do_not_append_to_historical_corridor_series"])
        self.assertTrue(all(value is False for value in context["validation_gates"].values()))
        metrics = context["publisher_reported_metrics"]
        self.assertEqual(metrics["fao_food_price_index"]["value"], 136.0)
        self.assertEqual(metrics["fao_cereal_price_index"]["change_from_previous_month_percent"], 5.1)
        self.assertEqual(metrics["international_maize_prices"]["change_from_previous_month_percent"], 5.6)
        self.assertIn("Strait of Hormuz", context["publisher_narrative"]["simultaneous_factors_reported"]["summary"])



    def test_empty_comtrade_response_is_saved_but_not_counted_as_qualified_capture(self):
        with tempfile.TemporaryDirectory() as temp:
            snapshot = Path(temp) / "snapshot"
            snapshot.mkdir()
            raw = b'{"count":0,"data":[]}'
            task = {
                "source_id": "UN-COMTRADE", "artifact_id": "202201_hs1001",
                "url": "https://comtradeapi.un.org/public/v1/preview/C/M/HS?reportercode=804&period=202201",
                "filename": "empty-response.json", "max_bytes": 1000,
                "expected_content_types": ["application/json"],
                "response_kind": "comtrade_json",
                "request_parameters": {"reportercode": 804, "partnerCode": 0, "flowCode": "X", "cmdCode": "1001", "period": "202201"},
                "expected_response_dimensions": {"reporterCode": 804, "partnerCode": 0, "flowCode": "X", "cmdCode": "1001", "period": "202201"},
                "record_limit": 500,
            }

            def fake_fetcher(url, max_bytes, timeout_seconds):
                return {"status_code": 200, "final_url": url, "content_type": "application/json",
                        "headers": {}, "raw": raw, "error": None}

            record = capture.capture_one(task, snapshot, 1, fetcher=fake_fetcher)
            self.assertEqual(record["capture_state"], "empty_response_needs_availability_review")
            self.assertIn("not proof of zero trade", record["error"])
            self.assertEqual((snapshot / "empty-response.json").read_bytes(), raw)
            self.assertEqual(record["response_sha256"], hashlib.sha256(raw).hexdigest())



if __name__ == "__main__":
    unittest.main()
