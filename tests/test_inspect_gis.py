import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "mineral-prospectivity"))
import inspect_gis


class FakeResult:
    def __init__(self, stdout="", stderr="", returncode=0):
        self.stdout, self.stderr, self.returncode = stdout, stderr, returncode


class GisMetadataTests(unittest.TestCase):
    def test_vector_summary_preserves_fields_crs_and_extent(self):
        doc = {"description":"x.gpkg","driverShortName":"GPKG","layers":[{
            "name":"geology","featureCount":10,
            "geometryFields":[{"name":"geom","type":"MultiPolygon","extent":[1,2,3,4],
                               "coordinateSystem":{"wkt":"WKT","projjson":{"name":"WGS 84"}}}],
            "fields":[{"name":"unit","type":"String"}]}]}
        s = inspect_gis.summarize_vector(doc)
        self.assertEqual(s["layer_count"], 1)
        self.assertEqual(s["layers"][0]["feature_count"], 10)
        self.assertEqual(s["layers"][0]["geometry_fields"][0]["crs_name"], "WGS 84")
        self.assertEqual(s["layers"][0]["fields"][0]["name"], "unit")

    def test_raster_summary_warns_missing_crs(self):
        s = inspect_gis.summarize_raster({"size":[100,200],"bands":[{"band":1,"type":"Float32"}]})
        self.assertIn("Raster has no reported coordinate system", s["warnings"])
        self.assertEqual(s["band_count"], 1)

    def test_uses_read_only_ogr_and_json(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "test.gpkg"
            path.write_bytes(b"placeholder; command output is mocked")
            calls = []
            def fake_run(cmd, **kwargs):
                calls.append(cmd)
                if cmd[-1] == "--version":
                    return FakeResult("GDAL 3.10")
                return FakeResult(json.dumps({"description":str(path),"driverShortName":"GPKG",
                                              "layers":[{"name":"one","geometryFields":[],"fields":[]}]}))
            with patch.object(inspect_gis.shutil, "which", return_value="/usr/bin/ogrinfo"), \
                 patch.object(inspect_gis.subprocess, "run", side_effect=fake_run):
                report = inspect_gis.inspect_gis(path, "T", "test", "", "unknown", "unreviewed")
            self.assertEqual(report["inspection"]["status"], "metadata_inspected")
            self.assertIn("-ro", calls[1])
            self.assertIn("-json", calls[1])
            self.assertNotIn("-update", calls[1])
            self.assertNotIn("-features", calls[1])

    def test_blocks_missing_gdal_tool(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "t.tif"
            path.write_bytes(b"x")
            with patch.object(inspect_gis.shutil, "which", return_value=None):
                with self.assertRaises(FileNotFoundError):
                    inspect_gis.inspect_gis(path, "T", "test", "", "unknown", "unreviewed")

    def test_command_failure_is_explicit(self):
        with patch.object(inspect_gis.subprocess, "run", return_value=FakeResult("", "no datasource", 1)):
            with self.assertRaisesRegex(RuntimeError, "command exited 1"):
                inspect_gis.run_json(["ogrinfo", "-json", "-ro", "bad.gpkg"], 5)

    def test_timeout_is_not_swallowed(self):
        import subprocess
        with patch.object(inspect_gis.subprocess, "run", side_effect=subprocess.TimeoutExpired("ogrinfo", 5)):
            with self.assertRaises(subprocess.TimeoutExpired):
                inspect_gis.run_json(["ogrinfo", "-json", "-ro", "slow.gpkg"], 5)


if __name__ == "__main__":
    unittest.main()
