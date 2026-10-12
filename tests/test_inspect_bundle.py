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
        self.assertFalse(safe_member_name("../outside.txt"))
        self.assertFalse(safe_member_name("/absolute/file.shp"))
        self.assertFalse(safe_member_name("C:/drive/file.shp"))
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
            self.assertFalse(report["inspection"]["payload_crc_checked"])
            self.assertEqual(report["inspection"]["preflight_status"], "structurally_inspected")

    def test_warns_for_incomplete_shapefile(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {"rocks/units.shp": b"x", "rocks/units.prj": b"crs"})
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            check = report["inspection"]["shapefile_sidecar_checks"][0]
            self.assertEqual(check["missing_required"], [".dbf", ".shx"])
            self.assertEqual(report["inspection"]["preflight_status"], "review_warnings")

    def test_blocks_unsafe_archive_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            p = self.make_zip(Path(temp), {"../escape.shp": b"unsafe", "data/readme.txt": b"ok"})
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["preflight_status"], "blocked_unsafe_paths")
            self.assertEqual(report["inspection"]["unsafe_member_names"], ["../escape.shp"])

    def test_non_zip_is_not_claimed_validated(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp) / "sample.csv"
            p.write_text("x,y\n1,2\n", encoding="utf-8")
            report = inspect_path(p, "TEST", "Test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["container_type"], "single_file")
            self.assertIn("CRS, geometry", " ".join(report["limitations"]))


if __name__ == "__main__":
    unittest.main()
