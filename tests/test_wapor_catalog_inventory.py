import json
import sys
import tempfile
import hashlib
import unittest
from pathlib import Path
from urllib.request import Request
import argparse

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from inspect_wapor_catalog import (
    API_ORIGIN, API_PREFIX, MAPSETS_URL, MAX_RESPONSE_BYTES,
    SameOriginRedirectHandler, WaporCatalogError, paginate, save_snapshots,
    validate_api_url, validate_mapset_code,
)


class WaporCatalogTests(unittest.TestCase):
    def test_valid_api_url_requires_official_https_origin_and_prefix(self):
        validate_api_url(MAPSETS_URL)
        for url in (
            "http://data.apps.fao.org" + API_PREFIX + "mapsets",
            "https://attacker.example" + API_PREFIX + "mapsets",
            API_ORIGIN + "/other/path",
            "https://user@data.apps.fao.org" + API_PREFIX + "mapsets",
        ):
            with self.subTest(url=url), self.assertRaises(WaporCatalogError):
                validate_api_url(url)

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

    def test_large_response_cap_is_declared(self):
        self.assertLessEqual(MAX_RESPONSE_BYTES, 16 * 1024 * 1024)



if __name__ == "__main__":
    unittest.main()
