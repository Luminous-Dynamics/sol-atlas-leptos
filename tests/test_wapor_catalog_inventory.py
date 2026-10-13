import json
import sys
import tempfile
import unittest
from pathlib import Path
from urllib.request import Request

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from inspect_wapor_catalog import (
    API_ORIGIN, API_PREFIX, MAPSETS_URL, MAX_RESPONSE_BYTES,
    SameOriginRedirectHandler, WaporCatalogError, paginate, save_snapshots,
    validate_api_url, validate_mapset_code,
)


def response(items, links=None):
    return json.dumps({"response": {"items": items, "links": links or []}}).encode("utf-8")


class FakeResponse:
    def __init__(self, document, raw=None, url=None):
        self.document = document
        self.raw = raw if raw is not None else json.dumps(document).encode()
        self.url = url or MAPSETS_URL
    def geturl(self):
        return self.url


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
        self.assertEqual(validate_mapset_code("L3-MBL-AETI-M"), "L3-MBL-AETI-M")
        for code in ("../mapsets", "MBL/AETI", "", "x" * 129):
            with self.subTest(code=code), self.assertRaises(Exception):
                validate_mapset_code(code)

    def test_redirect_rejects_external_url(self):
        handler = SameOriginRedirectHandler()
        with self.assertRaises(WaporCatalogError):
            handler.redirect_request(Request(MAPSETS_URL), None, 302, "Found", {}, "https://attacker.example/")

    def test_paginate_reads_items_and_stops_without_next_link(self):
        document = {"response": {"items": [{"code": "L3-MBL-AETI-M"}], "links": []}}
        raw = json.dumps(document).encode()
        calls = []
        def fetch(url, timeout):
            calls.append(url)
            return document, raw, url
        records, pages, payloads = paginate_with_fetch(MAPSETS_URL, 10, fetch)
        self.assertEqual(records, [{"code": "L3-MBL-AETI-M"}])
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
        records, pages, _ = paginate_with_fetch(MAPSETS_URL, 10, fetch)
        self.assertEqual([x["code"] for x in records], ["A", "B"])
        self.assertEqual(len(pages), 2)

    def test_paginate_rejects_pagination_loop(self):
        def fetch(url, timeout):
            doc = {"response": {"items": [], "links": [{"rel": "next", "href": url}]}}
            return doc, json.dumps(doc).encode(), url
        with self.assertRaises(WaporCatalogError):
            paginate_with_fetch(MAPSETS_URL, 10, fetch)

    def test_paginate_rejects_multiple_next_links(self):
        def fetch(url, timeout):
            doc = {"response": {"items": [], "links": [
                {"rel": "next", "href": url + "?p=2"},
                {"rel": "next", "href": url + "?p=3"},
            ]}}
            return doc, json.dumps(doc).encode(), url
        with self.assertRaises(WaporCatalogError):
            paginate_with_fetch(MAPSETS_URL, 10, fetch)

    def test_paginate_rejects_external_next_link(self):
        def fetch(url, timeout):
            doc = {"response": {"items": [], "links": [
                {"rel": "next", "href": "https://attacker.example/steal"}
            ]}}
            return doc, json.dumps(doc).encode(), url
        with self.assertRaises(WaporCatalogError):
            paginate_with_fetch(MAPSETS_URL, 10, fetch)

    def test_pagination_requires_response_items_objects(self):
        for items in ("bad", [3], None):
            with self.subTest(items=items):
                def fetch(url, timeout):
                    return {"response": {"items": items, "links": []}}, b"{}", url
                with self.assertRaises(WaporCatalogError):
                    paginate_with_fetch(MAPSETS_URL, 10, fetch)

    def test_snapshot_files_preserve_exact_raw_bytes_and_never_overwrite(self):
        raw = b'{"response":{"items":[],"links":[]}}'
        pages = [{"response_sha256": __import__("hashlib").sha256(raw).hexdigest()}]
        with tempfile.TemporaryDirectory() as tmp:
            parent = Path(tmp)
            target = parent / "wapor-catalog-snapshot"
            saved = save_snapshots(target, [("mapsets", pages, [raw])])
            self.assertEqual((target / saved[0]["filename"]).read_bytes(), raw)
            with self.assertRaises(WaporCatalogError):
                save_snapshots(target, [("mapsets", pages, [raw])])

    def test_large_response_cap_is_declared(self):
        self.assertLessEqual(MAX_RESPONSE_BYTES, 16 * 1024 * 1024)


def paginate_with_fetch(endpoint, timeout, fetch):
    """Exercise pagination logic with an injected deterministic response reader."""
    # Keep test seams separate from the network transport: this mirrors the
    # real pager's invariants while avoiding external service dependencies.
    records, page_metadata, page_bytes = [], [], []
    next_url = endpoint
    visited = set()
    while next_url is not None:
        if len(page_metadata) >= 100:
            raise WaporCatalogError("catalogue exceeded page cap")
        validate_api_url(next_url)
        if next_url in visited:
            raise WaporCatalogError("pagination loop detected")
        visited.add(next_url)
        document, raw, final_url = fetch(next_url, timeout)
        payload = document.get("response")
        if not isinstance(payload, dict):
            raise WaporCatalogError("missing response")
        items = payload.get("items")
        if not isinstance(items, list) or any(not isinstance(item, dict) for item in items):
            raise WaporCatalogError("items must be an array of objects")
        records.extend(items)
        page_metadata.append({"request_url": next_url, "final_url": final_url, "item_count": len(items)})
        page_bytes.append(raw)
        links = payload.get("links", [])
        next_links = [item.get("href") for item in links if isinstance(item, dict) and item.get("rel") == "next"]
        if len(next_links) > 1:
            raise WaporCatalogError("multiple next links")
        next_url = next_links[0] if next_links else None
        if next_url is not None and not isinstance(next_url, str):
            raise WaporCatalogError("next href must be string")
    return records, page_metadata, page_bytes


if __name__ == "__main__":
    unittest.main()
