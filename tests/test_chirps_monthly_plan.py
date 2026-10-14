import argparse
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from urllib.request import Request
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from plan_chirps_monthly import INDEX_URL, MAX_INDEX_BYTES, IndexError, SameOriginRedirectHandler, build_plan, month_range, parse_month


def listing(*hrefs):
    links = "".join(f'<a href="{href}">{href}</a>\n' for href in hrefs)
    return f"<html><body>{links}</body></html>".encode("utf-8")


class PartialWriteFailure:
    """File-like wrapper that leaves partial bytes before raising on write."""

    def __init__(self, handle):
        self.handle = handle

    def __enter__(self):
        self.handle.__enter__()
        return self

    def __exit__(self, exc_type, exc, tb):
        return self.handle.__exit__(exc_type, exc, tb)

    def write(self, value):
        self.handle.write(value[:10])
        raise OSError("simulated write failure")


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

    def test_snapshot_only_captures_index_without_selecting_period(self):
        from plan_chirps_monthly import main

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "source-index.html"
            snapshot = root / "captured-index.html"
            manifest_out = root / "snapshot-manifest.json"
            raw = listing("chirps-v3.0.2026.08.tif")
            source.write_bytes(raw)

            code = main([
                "--snapshot-only",
                "--index-file", str(source),
                "--snapshot-index", str(snapshot),
                "--output", str(manifest_out),
            ])
            self.assertEqual(code, 0)
            self.assertEqual(snapshot.read_bytes(), raw)
            manifest = json.loads(manifest_out.read_text(encoding="utf-8"))
            self.assertEqual(manifest["status"], "index_snapshot_only")
            self.assertIsNone(manifest["requested_period"])
            self.assertFalse(manifest["analysis_period_selected"])
            self.assertEqual(manifest["raster_files_downloaded"], 0)
            self.assertEqual(manifest["index_bytes"], len(raw))
            self.assertEqual(manifest["index_sha256"], hashlib.sha256(raw).hexdigest())
            self.assertEqual(manifest["index_snapshot_filename"], snapshot.name)
            self.assertIsNone(manifest["index_retrieved_at_utc"])

    def test_snapshot_only_manifest_partial_write_removes_manifest_and_snapshot(self):
        from plan_chirps_monthly import main

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "source-index.html"
            snapshot = root / "captured-index.html"
            manifest_out = root / "snapshot-manifest.json"
            source.write_bytes(listing("chirps-v3.0.2026.08.tif"))
            real_open = Path.open

            def fail_output_open(path_obj, mode="r", *args, **kwargs):
                handle = real_open(path_obj, mode, *args, **kwargs)
                if path_obj == manifest_out and mode == "x":
                    return PartialWriteFailure(handle)
                return handle

            with patch.object(Path, "open", new=fail_output_open):
                code = main([
                    "--snapshot-only",
                    "--index-file", str(source),
                    "--snapshot-index", str(snapshot),
                    "--output", str(manifest_out),
                ])

            self.assertEqual(code, 2)
            self.assertFalse(manifest_out.exists())
            self.assertFalse(snapshot.exists())

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

    def test_cli_refuses_to_overwrite_existing_plan(self):
        from plan_chirps_monthly import main
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "index.html"
            output = root / "existing-plan.json"
            source.write_bytes(listing("chirps-v3.0.2024.01.tif"))
            output.write_text("preserve this evidence\\n", encoding="utf-8")
            code = main([
                "--start", "2024-01", "--end", "2024-01",
                "--index-file", str(source), "--output", str(output),
            ])
            self.assertEqual(code, 2)
            self.assertEqual(output.read_text(encoding="utf-8"), "preserve this evidence\\n")

    def test_snapshot_partial_write_is_removed(self):
        from plan_chirps_monthly import main
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "index.html"
            snapshot = root / "snapshot.html"
            source.write_bytes(listing("chirps-v3.0.2024.01.tif"))
            real_open = Path.open

            def fail_snapshot_open(path_obj, mode="r", *args, **kwargs):
                handle = real_open(path_obj, mode, *args, **kwargs)
                if path_obj == snapshot and mode == "xb":
                    return PartialWriteFailure(handle)
                return handle

            with patch.object(Path, "open", new=fail_snapshot_open):
                code = main([
                    "--start", "2024-01", "--end", "2024-01",
                    "--index-file", str(source), "--snapshot-index", str(snapshot),
                ])
            self.assertEqual(code, 2)
            self.assertFalse(snapshot.exists())

    def test_plan_partial_write_removes_plan_and_new_snapshot(self):
        from plan_chirps_monthly import main
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "index.html"
            snapshot = root / "snapshot.html"
            output = root / "plan.json"
            source.write_bytes(listing("chirps-v3.0.2024.01.tif"))
            real_open = Path.open

            def fail_output_open(path_obj, mode="r", *args, **kwargs):
                handle = real_open(path_obj, mode, *args, **kwargs)
                if path_obj == output and mode == "x":
                    return PartialWriteFailure(handle)
                return handle

            with patch.object(Path, "open", new=fail_output_open):
                code = main([
                    "--start", "2024-01", "--end", "2024-01",
                    "--index-file", str(source), "--snapshot-index", str(snapshot),
                    "--output", str(output),
                ])
            self.assertEqual(code, 2)
            self.assertFalse(output.exists())
            self.assertFalse(snapshot.exists())

    def test_output_collision_race_removes_new_snapshot_not_existing_output(self):
        from plan_chirps_monthly import main
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "index.html"
            snapshot = root / "snapshot.html"
            output = root / "plan.json"
            source.write_bytes(listing("chirps-v3.0.2024.01.tif"))
            existing_bytes = b"created by another process"
            real_open = Path.open

            def collide_on_output(path_obj, mode="r", *args, **kwargs):
                if path_obj == output and mode == "x":
                    output.write_bytes(existing_bytes)
                    raise FileExistsError("simulated concurrent creator")
                return real_open(path_obj, mode, *args, **kwargs)

            with patch.object(Path, "open", new=collide_on_output):
                code = main([
                    "--start", "2024-01", "--end", "2024-01",
                    "--index-file", str(source), "--snapshot-index", str(snapshot),
                    "--output", str(output),
                ])
            self.assertEqual(code, 2)
            self.assertEqual(output.read_bytes(), existing_bytes)
            self.assertFalse(snapshot.exists())

    def test_local_index_rejects_oversized_or_nonregular_path_before_reading(self):
        from plan_chirps_monthly import read_index

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            oversized = root / "oversized-index.html"
            oversized.touch()
            with oversized.open("wb") as handle:
                handle.truncate(MAX_INDEX_BYTES + 1)

            # An oversized local file must be rejected from metadata alone.
            # If read_index tries to open/read it, fail this test immediately.
            with patch.object(Path, "open", side_effect=AssertionError("oversized source must not be opened")):
                with self.assertRaisesRegex(IndexError, "exceeds"):
                    read_index(oversized, 1.0)

            with self.assertRaisesRegex(IndexError, "regular file"):
                read_index(root, 1.0)

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
