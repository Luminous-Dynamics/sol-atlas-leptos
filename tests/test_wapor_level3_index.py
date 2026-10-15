import hashlib
import sys
import unittest
from urllib.request import Request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from inspect_wapor_level3_index import (
    INDEX_HOST,
    INDEX_PATH,
    INDEX_URL,
    IndexError,
    MosaicsetIndexParser,
    SameIndexRedirectHandler,
    build_inventory,
    prefix_for,
    validate_index_url,
)


def listing(*hrefs):
    links = "".join(f'<a href="{href}">{href}</a>\n' for href in hrefs)
    return f"<html><body>{links}</body></html>".encode("utf-8")


class WaporLevel3IndexTests(unittest.TestCase):
    def test_prefix_is_derived_only_from_valid_level3_mapset_code(self):
        self.assertEqual(prefix_for("L3-AETI-M"), "DATA/WAPOR-3/MOSAICSET/L3-AETI-M")
        for value in ("L2-AETI-M", "../L3-AETI-M", "L3-AETI-M/MBL", "L3-AETI-MM"):
            with self.subTest(value=value), self.assertRaises(IndexError):
                prefix_for(value)

    def test_index_url_requires_exact_host_path_and_prefix(self):
        prefix = prefix_for("L3-AETI-M")
        valid = INDEX_URL + "?prefix=DATA%2FWAPOR-3%2FMOSAICSET%2FL3-AETI-M"
        validate_index_url(valid, prefix)
        for url in (
            "http://gismgr.fao.org" + INDEX_PATH + "?prefix=" + prefix,
            "https://attacker.example" + INDEX_PATH + "?prefix=" + prefix,
            "https://gismgr.fao.org/elsewhere?prefix=" + prefix,
            valid + "&prefix=DATA%2Fother",
            valid + "#fragment",
            "https://user@gismgr.fao.org" + INDEX_PATH + "?prefix=DATA%2FWAPOR-3%2FMOSAICSET%2FL3-AETI-M",
        ):
            with self.subTest(url=url), self.assertRaises(IndexError):
                validate_index_url(url, prefix)

    def test_redirect_handler_rejects_external_and_wrong_prefix_targets(self):
        prefix = prefix_for("L3-AETI-M")
        handler = SameIndexRedirectHandler(prefix)
        request = Request(INDEX_URL)
        with self.assertRaises(IndexError):
            handler.redirect_request(request, None, 302, "Found", {}, "https://attacker.example/steal")
        with self.assertRaises(IndexError):
            handler.redirect_request(
                request, None, 302, "Found", {},
                INDEX_URL + "?prefix=DATA%2FWAPOR-3%2FMOSAICSET%2FL3-E-A",
            )

    def test_inventory_separates_requested_area_rasters_from_sidecars(self):
        raw = listing(
            "../",
            "WAPOR-3.L3-AETI-M.MBL.2024-01.tif",
            "WAPOR-3.L3-AETI-M.MBL.2024-01.json",
            "WAPOR-3.L3-AETI-M.MBL.2024-02.tif",
            "WAPOR-3.L3-AETI-M.AWA.2024-01.tif",
            "WAPOR-3.L3-E-A.MBL.2024.tif",
            "readme.txt",
        )
        result = build_inventory(raw, "L3-AETI-M", "MBL", "2026-10-15T00:00:00+00:00", "local_html_snapshot")
        self.assertEqual(result["status"], "level3_mosaicset_index_inventory_only")
        self.assertEqual(result["index_sha256"], hashlib.sha256(raw).hexdigest())
        self.assertEqual(result["index_bytes"], len(raw))
        self.assertEqual(result["area_codes_observed"], ["AWA", "MBL"])
        self.assertEqual(result["requested_area_raster_count"], 2)
        self.assertEqual(result["requested_area_sidecar_count"], 1)
        self.assertEqual(result["raster_files_downloaded"], 0)
        self.assertEqual(result["sidecar_json_files_downloaded"], 0)
        self.assertEqual(
            [x["period_token"] for x in result["requested_area_files"] if x["kind"] == "raster"],
            ["2024-01", "2024-02"],
        )
        self.assertEqual(
            result["requested_area_files"][0]["url"],
            "https://gismgr.fao.org/DATA/WAPOR-3/MOSAICSET/L3-AETI-M/WAPOR-3.L3-AETI-M.MBL.2024-01.json",
        )

    def test_inventory_does_not_claim_area_data_when_no_files_match(self):
        raw = listing("WAPOR-3.L3-AETI-M.AWA.2024-01.tif")
        result = build_inventory(raw, "L3-AETI-M", "MBL", None, "local_html_snapshot")
        self.assertEqual(result["requested_area_raster_count"], 0)
        self.assertNotIn("MBL", result["area_codes_observed"])
        self.assertEqual(result["status"], "level3_mosaicset_index_inventory_only")

    def test_same_origin_root_relative_file_link_is_admitted(self):
        raw = listing("/DATA/WAPOR-3/MOSAICSET/L3-AETI-M/WAPOR-3.L3-AETI-M.MBL.2024-01.tif")
        result = build_inventory(raw, "L3-AETI-M", "MBL", None, "local_html_snapshot")
        self.assertEqual(result["requested_area_raster_count"], 1)
        self.assertEqual(
            result["requested_area_files"][0]["url"],
            "https://gismgr.fao.org/DATA/WAPOR-3/MOSAICSET/L3-AETI-M/WAPOR-3.L3-AETI-M.MBL.2024-01.tif",
        )

    def test_relative_traversal_file_link_fails_closed(self):
        raw = listing("../L3-AETI-M/WAPOR-3.L3-AETI-M.MBL.2024-01.tif")
        with self.assertRaisesRegex(IndexError, "unsafe matching Level III file link"):
            build_inventory(raw, "L3-AETI-M", "MBL", None, "local_html_snapshot")

    def test_unsafe_matching_link_fails_closed(self):
        raw = listing(
            "https://attacker.example/WAPOR-3.L3-AETI-M.MBL.2024-01.tif",
        )
        with self.assertRaisesRegex(IndexError, "unsafe matching Level III file link"):
            build_inventory(raw, "L3-AETI-M", "MBL", None, "local_html_snapshot")

    def test_duplicate_matching_filename_fails_closed(self):
        raw = listing(
            "WAPOR-3.L3-AETI-M.MBL.2024-01.tif",
            "WAPOR-3.L3-AETI-M.MBL.2024-01.tif",
        )
        with self.assertRaisesRegex(IndexError, "duplicate directory entry"):
            build_inventory(raw, "L3-AETI-M", "MBL", None, "local_html_snapshot")

    def test_unknown_period_token_is_preserved_without_normalization(self):
        raw = listing("WAPOR-3.L3-AETI-M.MBL.2024-01-D1.tif")
        result = build_inventory(raw, "L3-AETI-M", "MBL", None, "local_html_snapshot")
        self.assertEqual(result["requested_area_files"][0]["period_token"], "2024-01-D1")

    def test_non_utf8_index_is_rejected(self):
        with self.assertRaisesRegex(IndexError, "not valid UTF-8"):
            build_inventory(b"\xff\xfe", "L3-AETI-M", "MBL", None, "local_html_snapshot")


if __name__ == "__main__":
    unittest.main()
