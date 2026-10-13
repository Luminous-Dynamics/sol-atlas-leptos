import argparse
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from urllib.request import Request

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from plan_chirps_monthly import INDEX_URL, IndexError, SameOriginRedirectHandler, build_plan, month_range, parse_month


def listing(*hrefs):
    links = "".join(f'<a href="{href}">{href}</a>\n' for href in hrefs)
    return f"<html><body>{links}</body></html>".encode("utf-8")


class ChirpsMonthlyPlanTests(unittest.TestCase):
    def test_parse_month_accepts_valid_month(self):
        self.assertEqual(parse_month("2024-02"), "2024-02")

    def test_parse_month_rejects_invalid_format_or_month(self):
        for value in ("2024-2", "2024-13", "1980-12", "nonsense"):
            with self.subTest(value=value):
                with self.assertRaises(argparse.ArgumentTypeError):
                    parse_month(value)

    def test_month_range_is_inclusive_across_year_boundary(self):
        self.assertEqual(month_range("2024-11", "2025-02"), [
            "2024-11", "2024-12", "2025-01", "2025-02"
        ])

    def test_month_range_rejects_reversed_period(self):
        with self.assertRaises(ValueError):
            month_range("2025-01", "2024-12")

    def test_month_range_has_safety_cap(self):
        with self.assertRaises(ValueError):
            month_range("1981-01", "2032-01")

    def test_complete_plan_is_inventory_only_and_hashes_index(self):
        raw = listing(
            "../", "chirps-v3.0.2024.01.tif",
            "chirps-v3.0.2024.02.tif", "chirps-v3.0.2024.03.tif"
        )
        plan = build_plan(raw, "2024-01", "2024-03", "2026-10-13T00:00:00+00:00", "local_html_snapshot")
        self.assertTrue(plan["complete"])
        self.assertEqual(plan["catalogued_month_count"], 3)
        self.assertEqual(plan["index_sha256"], hashlib.sha256(raw).hexdigest())
        self.assertEqual(plan["index_url"], INDEX_URL)
        self.assertEqual(plan["files"][0]["url"], INDEX_URL + "chirps-v3.0.2024.01.tif")
        self.assertIsNone(plan["files"][0]["artifact_sha256"])
        self.assertEqual(plan["files"][0]["state"], "catalogued_not_downloaded")
        self.assertEqual(plan["status"], "index_inventory_only")

    def test_missing_month_is_explicit_not_silently_skipped(self):
        raw = listing("chirps-v3.0.2024.01.tif", "chirps-v3.0.2024.03.tif")
        plan = build_plan(raw, "2024-01", "2024-03", None, "local_html_snapshot")
        self.assertFalse(plan["complete"])
        self.assertEqual(plan["missing_months"], ["2024-02"])
        self.assertEqual(plan["catalogued_month_count"], 2)

    def test_duplicate_month_fails_closed(self):
        raw = listing("chirps-v3.0.2024.01.tif", "chirps-v3.0.2024.01.tif")
        with self.assertRaises(IndexError):
            build_plan(raw, "2024-01", "2024-01", None, "local_html_snapshot")

    def test_malformed_date_in_matching_filename_fails_closed(self):
        raw = listing("chirps-v3.0.2024.13.tif")
        with self.assertRaises(IndexError):
            build_plan(raw, "2024-01", "2024-01", None, "local_html_snapshot")

    def test_external_or_traversal_matching_links_are_rejected(self):
        raw = listing(
            "../chirps-v3.0.2024.01.tif",
            "https://attacker.example/chirps-v3.0.2024.02.tif",
            "chirps-v3.0.2024.03.tif?download=1"
        )
        with self.assertRaises(IndexError):
            build_plan(raw, "2024-01", "2024-03", None, "local_html_snapshot")

    def test_unrelated_or_preliminary_links_are_not_selected(self):
        raw = listing(
            "../", "chirps-v3.0.2024.01.tif",
            "chirps-v3.0.2024.02.prelim.tif", "readme.txt"
        )
        plan = build_plan(raw, "2024-01", "2024-01", None, "local_html_snapshot")
        self.assertTrue(plan["complete"])
        self.assertEqual(len(plan["files"]), 1)
        self.assertNotIn("prelim", plan["files"][0]["url"])

    def test_large_or_non_utf8_index_is_rejected(self):
        with self.assertRaises(IndexError):
            build_plan(b"\xff\xfe", "2024-01", "2024-01", None, "local_html_snapshot")

    def test_redirect_handler_rejects_external_location_before_following(self):
        handler = SameOriginRedirectHandler()
        request = Request(INDEX_URL)
        with self.assertRaises(IndexError):
            handler.redirect_request(
                request, None, 302, "Found", {}, 
                "https://attacker.example/steal"
            )

    def test_redirect_handler_accepts_only_exact_same_index_path(self):
        handler = SameOriginRedirectHandler()
        request = Request(INDEX_URL)
        result = handler.redirect_request(
            request, None, 302, "Found", {},
            INDEX_URL
        )
        self.assertIsInstance(result, Request)

    def test_cli_saves_exact_index_snapshot_without_overwriting(self):
        from plan_chirps_monthly import main
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "source-index.html"
            saved = root / "captured-index.html"
            plan_out = root / "plan.json"
            raw = listing("chirps-v3.0.2024.01.tif")
            source.write_bytes(raw)
            code = main([
                "--start", "2024-01", "--end", "2024-01",
                "--index-file", str(source),
                "--snapshot-index", str(saved),
                "--output", str(plan_out),
            ])
            self.assertEqual(code, 0)
            self.assertEqual(saved.read_bytes(), raw)
            plan = json.loads(plan_out.read_text(encoding="utf-8"))
            self.assertTrue(plan["index_snapshot_saved"])
            self.assertEqual(plan["index_snapshot_filename"], "captured-index.html")
            self.assertEqual(plan["index_sha256"], hashlib.sha256(raw).hexdigest())
            self.assertEqual(source.read_bytes(), raw)
            self.assertEqual(main([
                "--start", "2024-01", "--end", "2024-01",
                "--index-file", str(source),
                "--snapshot-index", str(saved),
                "--output", str(root / "second-plan.json"),
            ]), 2)

    def test_local_index_file_is_read_only_input(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "index.html"
            raw = listing("chirps-v3.0.2024.01.tif")
            source.write_bytes(raw)
            from plan_chirps_monthly import read_index
            loaded, retrieved_at, mode = read_index(source, 1.0)
            self.assertEqual(loaded, raw)
            self.assertIsNone(retrieved_at)
            self.assertEqual(mode, "local_html_snapshot")
            self.assertEqual(source.read_bytes(), raw)

    def test_plan_claim_ceiling_disallows_false_resolution(self):
        raw = listing("chirps-v3.0.2024.01.tif")
        plan = build_plan(raw, "2024-01", "2024-01", None, "local_html_snapshot")
        claim_ceiling = " ".join(plan["claim_ceiling"]).lower()
        self.assertIn("no geotiff bytes were downloaded", claim_ceiling)
        self.assertIn("20 m information", claim_ceiling)


if __name__ == "__main__":
    unittest.main()
