import json
import sys
import tempfile
import hashlib
import unittest
from pathlib import Path
from urllib.request import Request
import argparse
from io import StringIO
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
import inspect_wapor_catalog as wapor_module
from inspect_wapor_catalog import (
    API_ORIGIN, API_PREFIX, MAPSETS_URL, MAX_RESPONSE_BYTES,
    MAX_TOTAL_RESPONSE_BYTES, SameOriginRedirectHandler, WaporCatalogError, cleanup_snapshot_after_output_failure, paginate, save_snapshots,
    validate_api_url, validate_mapset_code,
)


class PartialWriteFailure:
    """File-like wrapper that leaves a prefix behind before failing a write."""

    def __init__(self, handle):
        self.handle = handle

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, traceback):
        self.handle.close()
        return False

    def write(self, value):
        self.handle.write(value[:8])
        raise OSError("simulated partial write")


class BrokenStdout:
    def write(self, value):
        raise OSError("simulated broken pipe")


class WaporCatalogTests(unittest.TestCase):
    def test_valid_api_url_requires_official_https_origin_and_prefix(self):
        validate_api_url(MAPSETS_URL)
        deeply_encoded_parent = ".."
        for _ in range(8):
            deeply_encoded_parent = deeply_encoded_parent.replace("%", "%25").replace(".", "%2e")
        for url in (
            "http://data.apps.fao.org" + API_PREFIX + "mapsets",
            "https://attacker.example" + API_PREFIX + "mapsets",
            API_ORIGIN + "/other/path",
            "https://user@data.apps.fao.org" + API_PREFIX + "mapsets",
            API_ORIGIN + API_PREFIX + "../private",
            API_ORIGIN + API_PREFIX + "%2e%2e/private",
            API_ORIGIN + API_PREFIX + "%252e%252e/private",
            API_ORIGIN + API_PREFIX + "mapsets%2f..%2fprivate",
            API_ORIGIN + API_PREFIX + r"mapsets\..\private",
            API_ORIGIN + API_PREFIX + "mapsets/%2e%2e/../../private",
            API_ORIGIN + API_PREFIX + "mapsets\\\\..\\\\private",
        ):
            with self.subTest(url=url), self.assertRaises(WaporCatalogError):
                validate_api_url(url)
        with self.assertRaises(WaporCatalogError):
            validate_api_url(API_ORIGIN + API_PREFIX + deeply_encoded_parent + "/private")

    def test_mapset_code_is_restricted_to_safe_path_characters(self):
        self.assertEqual(validate_mapset_code("EXAMPLE-MAPSET-AETI-M"), "EXAMPLE-MAPSET-AETI-M")
        for code in ("../mapsets", "MBL/AETI", "", "x" * 129):
            with self.subTest(code=code), self.assertRaises(argparse.ArgumentTypeError):
                validate_mapset_code(code)

    def test_redirect_rejects_external_url(self):
        handler = SameOriginRedirectHandler()
        with self.assertRaises(WaporCatalogError):
            handler.redirect_request(Request(MAPSETS_URL), None, 302, "Found", {}, "https://attacker.example/")

    def test_paginate_reads_items_and_stops_without_next_link(self):
        document = {"response": {"items": [{"code": "EXAMPLE-MAPSET-AETI-M"}], "links": []}}
        raw = json.dumps(document).encode()
        calls = []
        def fetch(url, timeout):
            calls.append(url)
            return document, raw, url
        records, pages, payloads = paginate(MAPSETS_URL, 10, fetcher=fetch)
        self.assertEqual(records, [{"code": "EXAMPLE-MAPSET-AETI-M"}])
        self.assertEqual(len(pages), 1)
        self.assertEqual(payloads, [raw])
        self.assertEqual(calls, [MAPSETS_URL])

    def test_paginate_follows_same_api_origin_next_link(self):
        second = MAPSETS_URL + "?page=2"
        docs = {
            MAPSETS_URL: {"response": {"items": [{"code": "A"}], "links": [{"rel": "next", "href": second}]}},
            second: {"response": {"items": [{"code": "B"}], "links": []}},
        }
        def fetch(url, timeout):
            doc = docs[url]
            return doc, json.dumps(doc).encode(), url
        records, pages, _ = paginate(MAPSETS_URL, 10, fetcher=fetch)
        self.assertEqual([x["code"] for x in records], ["A", "B"])
        self.assertEqual(len(pages), 2)

    def test_paginate_resolves_relative_next_link_within_approved_path(self):
        relative_next = "?page=2"
        second = MAPSETS_URL + relative_next
        docs = {
            MAPSETS_URL: {"response": {"items": [{"code": "A"}], "links": [{"rel": "next", "href": relative_next}]}},
            second: {"response": {"items": [{"code": "B"}], "links": []}},
        }
        def fetch(url, timeout):
            doc = docs[url]
            return doc, json.dumps(doc).encode(), url
        records, pages, _ = paginate(MAPSETS_URL, 10, fetcher=fetch)
        self.assertEqual([item["code"] for item in records], ["A", "B"])
        self.assertEqual(len(pages), 2)

    def test_paginate_rejects_pagination_loop(self):
        def fetch(url, timeout):
            doc = {"response": {"items": [], "links": [{"rel": "next", "href": url}]}}
            return doc, json.dumps(doc).encode(), url
        with self.assertRaises(WaporCatalogError):
            paginate(MAPSETS_URL, 10, fetcher=fetch)

    def test_paginate_rejects_multiple_next_links(self):
        def fetch(url, timeout):
            doc = {"response": {"items": [], "links": [
                {"rel": "next", "href": url + "?p=2"},
                {"rel": "next", "href": url + "?p=3"},
            ]}}
            return doc, json.dumps(doc).encode(), url
        with self.assertRaises(WaporCatalogError):
            paginate(MAPSETS_URL, 10, fetcher=fetch)

    def test_paginate_rejects_external_next_link(self):
        def fetch(url, timeout):
            doc = {"response": {"items": [], "links": [
                {"rel": "next", "href": "https://attacker.example/steal"}
            ]}}
            return doc, json.dumps(doc).encode(), url
        with self.assertRaises(WaporCatalogError):
            paginate(MAPSETS_URL, 10, fetcher=fetch)

    def test_pagination_requires_response_items_objects(self):
        for items in ("bad", [3], None):
            with self.subTest(items=items):
                def fetch(url, timeout):
                    return {"response": {"items": items, "links": []}}, b"{}", url
                with self.assertRaises(WaporCatalogError):
                    paginate(MAPSETS_URL, 10, fetcher=fetch)

    def test_snapshot_files_preserve_exact_raw_bytes_and_never_overwrite(self):
        raw = b'{"response":{"items":[],"links":[]}}'
        pages = [{"response_sha256": hashlib.sha256(raw).hexdigest()}]
        with tempfile.TemporaryDirectory() as tmp:
            parent = Path(tmp)
            target = parent / "wapor-catalog-snapshot"
            saved = save_snapshots(target, [("mapsets", pages, [raw])])
            self.assertEqual((target / saved[0]["filename"]).read_bytes(), raw)
            with self.assertRaises(WaporCatalogError):
                save_snapshots(target, [("mapsets", pages, [raw])])

    def test_snapshot_rejects_manifest_mismatch_before_creating_directory(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            cases = [
                ("count-mismatch", [("mapsets", [], [b"orphan bytes"])]),
                ("digest-mismatch", [("mapsets", [{"response_sha256": hashlib.sha256(b"other bytes").hexdigest()}], [b"actual bytes"])]),
                ("unsafe-label", [("../escape", [], [])]),
            ]
            for name, groups in cases:
                with self.subTest(name=name):
                    target = root / name
                    with self.assertRaises(WaporCatalogError):
                        save_snapshots(target, groups)
                    self.assertFalse(target.exists())

    def test_snapshot_write_failure_removes_partial_directory(self):
        raw = b'{"response":{"items":[],"links":[]}}'
        pages = [{"response_sha256": hashlib.sha256(raw).hexdigest()}]
        with tempfile.TemporaryDirectory() as temp_dir:
            target = Path(temp_dir) / "partial-snapshot"
            with patch.object(Path, "open", side_effect=OSError("simulated write failure")):
                with self.assertRaisesRegex(WaporCatalogError, "cannot preserve raw API response"):
                    save_snapshots(target, [("mapsets", pages, [raw])])
            self.assertFalse(target.exists())

    def test_snapshot_cleanup_failure_is_reported(self):
        raw = b'{"response":{"items":[],"links":[]}}'
        pages = [{"response_sha256": hashlib.sha256(raw).hexdigest()}]
        with tempfile.TemporaryDirectory() as temp_dir:
            target = Path(temp_dir) / "partial-snapshot"
            with patch.object(Path, "open", side_effect=OSError("simulated write failure")):
                with patch.object(wapor_module.shutil, "rmtree", side_effect=OSError("simulated cleanup failure")):
                    with self.assertRaisesRegex(WaporCatalogError, "also failed to remove partial snapshot"):
                        save_snapshots(target, [("mapsets", pages, [raw])])
            self.assertTrue(target.exists())

    def test_manifest_failure_removes_only_new_snapshot_and_reports_cleanup_failure(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            target = Path(temp_dir) / "snapshot"
            target.mkdir()
            (target / "page.json").write_text("raw response", encoding="utf-8")
            note = cleanup_snapshot_after_output_failure(target, [{"filename": "page.json"}])
            self.assertIn("removed newly created snapshot directory", note)
            self.assertFalse(target.exists())

            target.mkdir()
            with patch.object(wapor_module.shutil, "rmtree", side_effect=OSError("simulated cleanup failure")):
                note = cleanup_snapshot_after_output_failure(target, [{"filename": "page.json"}])
            self.assertIn("failed to remove newly created snapshot directory", note)
            self.assertTrue(target.exists())

    def test_existing_output_is_rejected_without_network_or_overwrite(self):
        from inspect_wapor_catalog import main
        with tempfile.TemporaryDirectory() as temp_dir:
            output = Path(temp_dir) / "inventory.json"
            output.write_text("preserve this evidence\\n", encoding="utf-8")
            self.assertEqual(main(["--output", str(output)]), 2)
            self.assertEqual(output.read_text(encoding="utf-8"), "preserve this evidence\\n")

    def test_partial_inventory_write_removes_output_and_new_snapshot(self):
        from inspect_wapor_catalog import main

        raw = b'{"response":{"items":[],"links":[]}}'
        page = {"response_sha256": hashlib.sha256(raw).hexdigest()}
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            output = root / "inventory.json"
            snapshot = root / "snapshot"

            real_open = Path.open

            def fail_output_open(path_obj, mode="r", *args, **kwargs):
                handle = real_open(path_obj, mode, *args, **kwargs)
                if path_obj == output and mode == "x":
                    return PartialWriteFailure(handle)
                return handle

            with patch.object(wapor_module, "paginate", return_value=([{"code": "A"}], [page], [raw])):
                with patch.object(Path, "open", new=fail_output_open):
                    with patch.object(sys, "stderr", StringIO()) as stderr:
                        code = main(["--snapshot-dir", str(snapshot), "--output", str(output)])

            self.assertEqual(code, 2)
            self.assertFalse(output.exists())
            self.assertFalse(snapshot.exists())
            self.assertIn("removed partial artifact inventory.json", stderr.getvalue())
            self.assertIn("removed newly created snapshot directory", stderr.getvalue())

    def test_broken_stdout_removes_new_snapshot(self):
        from inspect_wapor_catalog import main

        raw = b'{"response":{"items":[],"links":[]}}'
        page = {"response_sha256": hashlib.sha256(raw).hexdigest()}
        with tempfile.TemporaryDirectory() as temp_dir:
            snapshot = Path(temp_dir) / "snapshot"
            with patch.object(wapor_module, "paginate", return_value=([{"code": "A"}], [page], [raw])):
                with patch.object(sys, "stdout", BrokenStdout()):
                    with patch.object(sys, "stderr", StringIO()) as stderr:
                        code = main(["--snapshot-dir", str(snapshot)])

            self.assertEqual(code, 2)
            self.assertFalse(snapshot.exists())
            self.assertIn("cannot write inventory JSON to stdout", stderr.getvalue())
            self.assertIn("removed newly created snapshot directory", stderr.getvalue())

    def test_pagination_rejects_aggregate_response_byte_cap(self):
        document = {"response": {"items": [], "links": []}}
        raw = b'{"response":{"items":[],"links":[]}}'
        def fetch(url, timeout):
            return document, raw, url
        with patch.object(wapor_module, "MAX_TOTAL_RESPONSE_BYTES", len(raw) - 1):
            with self.assertRaisesRegex(WaporCatalogError, "total response byte limit"):
                paginate(MAPSETS_URL, 10, fetcher=fetch)

    def test_large_response_caps_are_declared(self):
        self.assertLessEqual(MAX_RESPONSE_BYTES, 16 * 1024 * 1024)
        self.assertLessEqual(MAX_TOTAL_RESPONSE_BYTES, 64 * 1024 * 1024)



if __name__ == "__main__":
    unittest.main()
