"""Unit tests for panen-id Python pipeline (`python -m unittest`)."""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from s2_ndvi import compute_ndvi, mask_scl_clear, smooth_series, to_surface_reflectance
from season_bake import run


class PanenPipelineTests(unittest.TestCase):
    def test_baseline_04_radiometric_offset(self) -> None:
        # Post-2022-01-25 Baseline >= 04.00 shifts DN by +1000, so BOA_ADD_OFFSET = -1000 must apply
        sr_new = to_surface_reflectance(3500.0, "2025-07-15", baseline="05.10")
        sr_old = to_surface_reflectance(3500.0, "2021-07-15", baseline="03.01")
        self.assertAlmostEqual(sr_new, 0.25, places=4)
        self.assertAlmostEqual(sr_old, 0.35, places=4)

    def test_scl_mask_and_ndvi(self) -> None:
        self.assertTrue(mask_scl_clear(4))
        self.assertFalse(mask_scl_clear(9))  # High cloud probability
        self.assertAlmostEqual(compute_ndvi(0.45, 0.05), 0.8, places=4)

    def test_smooth_series_fills_gaps(self) -> None:
        smoothed = smooth_series([0.4, None, None, 0.7])
        self.assertEqual(len(smoothed), 4)
        self.assertGreater(smoothed[1], 0.4)
        self.assertLess(smoothed[2], 0.7)

    def test_season_bake_dry_run(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            res = run(dry_run=True, out_dir=Path(tmp))
            self.assertEqual(res["fields"], 2)
            self.assertTrue((Path(tmp) / "bompon-magelang-mt2-2025.json").exists())
            self.assertTrue((Path(tmp) / "klaten-irigasi-mt2-2025.json").exists())


if __name__ == "__main__":
    unittest.main()
