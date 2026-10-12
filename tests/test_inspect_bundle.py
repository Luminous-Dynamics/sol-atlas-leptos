import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "mineral-prospectivity"))
from inspect_bundle import inspect_path, safe_member_name


class BundlePreflightTests(unittest.TestCase):
    def make_zip(self, root: Path, members: dict[str, bytes]) -> Path:
        path = root / "sample.zip"
        with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as z:
            for name, data in members.items():
                z.writestr(name, data)
        return path

    def test_rejects_traversal_absolute_and_drive_paths(self):
        for name in ("../outside.txt", "/absolute/file.shp", "C:/drive/file.shp", r"folder\\evil.shp"):
            self.assertFalse(safe_member_name(name), name)
        self.assertTrue(safe_member_name("folder/layer.gpkg"))

    def test_hash_and_geospatial_candidates(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {
                "sources/readme.txt": b"metadata",
                "layers/rocks.gpkg": b"placeholder",
                "layers/stations.csv": b"x,y,value\n1,2,3\n",
            })
            report = inspect_path(p, "TEST", "Test", "https://example.org", "unknown", "test")
            self.assertEqual(len(report["artifact"]["sha256"]), 64)
            self.assertEqual(report["inspection"]["member_count"], 3)
            self.assertEqual(set(report["inspection"]["geospatial_extension_types"]), {".gpkg", ".csv"})
            self.assertEqual(report["inspection"]["payload_crc_status"], "not_requested")
            self.assertEqual(report["inspection"]["preflight_status"], "structurally_inspected")

    def test_warns_for_incomplete_shapefile(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {"rocks/units.shp": b"x", "rocks/units.prj": b"crs"})
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["shapefile_sidecar_checks"][0]["missing_required"], [".dbf", ".shx"])
            self.assertEqual(report["inspection"]["preflight_status"], "review_warnings")

    def test_blocks_unsafe_archive_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {"../escape.shp": b"unsafe", "data/readme.txt": b"ok"})
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["preflight_status"], "blocked_unsafe_paths")
            self.assertEqual(report["inspection"]["unsafe_member_names"], ["../escape.shp"])
            self.assertGreater(report["inspection"]["uncompressed_bytes_total"], 0)

    def test_non_zip_is_not_claimed_validated(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp) / "sample.csv"
            p.write_text("x,y\n1,2\n", encoding="utf-8")
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["container_type"], "single_file")
            self.assertIn("CRS, geometry", " ".join(report["limitations"]))

    def test_crc_is_explicit_and_bounded(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {"data.txt": b"small payload"})
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed", verify_crc=True)
            self.assertEqual(report["inspection"]["payload_crc_status"], "passed")

    def test_malformed_zip_suffix_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp) / "broken.zip"
            p.write_text("not a zip")
            with self.assertRaises(ValueError):
                inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")

    def test_empty_zip_is_not_silent_success(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {})
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["preflight_status"], "review_warnings")


if __name__ == "__main__":
    unittest.main()
