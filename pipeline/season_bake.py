"""Seasonal replay data baker combining GDD, weather, and Sentinel-2 NDVI."""

import argparse
import math
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from config import FIELDS
from r2_uploader import upload_season_artifacts
from s2_ndvi import compute_ndvi, mask_scl_clear, smooth_series, to_surface_reflectance


def bake_field_bundle(field: dict[str, Any], days: int = 120, dry_run: bool = True) -> dict[str, Any]:
    start = date(2025, 5, 1)
    dates: list[str] = []
    tmax_c: list[float] = []
    tmin_c: list[float] = []
    rain_mm: list[float] = []
    gdd_cum: list[float] = []
    ndvi_raw: list[float | None] = []

    acc_gdd = 0.0
    is_klaten = field["field_id"] == "klaten-irigasi"

    for i in range(days):
        d = start + timedelta(days=i)
        d_str = d.isoformat()
        dates.append(d_str)

        tmax = round(31.0 + 1.8 * math.sin(i / 14.0), 1)
        tmin = round(22.2 + 1.1 * math.cos(i / 18.0), 1)
        tmax_c.append(tmax)
        tmin_c.append(tmin)

        daily_gdd = max(0.0, 0.5 * (min(35.0, max(10.0, tmax)) + min(35.0, max(10.0, tmin))) - 10.0)
        acc_gdd += daily_gdd
        gdd_cum.append(round(acc_gdd, 1))

        rain = 0.0 if (55 <= i <= 76) else round(max(0.0, 12.0 + 10.0 * math.sin(i / 5.0)), 1)
        rain_mm.append(rain)

        # Simulate Sentinel-2 5-day revisit + SCL cloud masking on Baseline 04.00 DN (+1000 offset)
        if i % 5 == 0:
            scl = 9 if (i % 25 == 10) else 4
            if mask_scl_clear(scl):
                veg_curve = math.exp(-((i - 72) ** 2) / (2 * 28.0**2))
                nir_dn = int(1000 + (1800 + (2600 if is_klaten else 2200) * veg_curve))
                red_dn = int(1000 + (1100 - 650 * veg_curve))
                nir_sr = to_surface_reflectance(nir_dn, d_str, baseline="05.10")
                red_sr = to_surface_reflectance(red_dn, d_str, baseline="05.10")
                ndvi_raw.append(compute_ndvi(nir_sr, red_sr))
            else:
                ndvi_raw.append(None)
        else:
            ndvi_raw.append(None)

    ndvi_smooth = smooth_series(ndvi_raw)
    return {
        **field,
        "is_simulated": dry_run,
        "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "dates": dates,
        "tmax_c": tmax_c,
        "tmin_c": tmin_c,
        "rain_mm": rain_mm,
        "gdd_cum": gdd_cum,
        "ndvi_s2_hls": ndvi_smooth,
    }


def run(dry_run: bool = False, out_dir: Path | None = None) -> dict[str, Any]:
    artifacts: dict[str, Any] = {}
    for field in FIELDS:
        bundle = bake_field_bundle(field, dry_run=dry_run)
        artifacts[f"{field['field_id']}-{field['season_id']}.json"] = bundle

    target_dir = out_dir or (Path(__file__).resolve().parent / "out")
    keys = upload_season_artifacts(artifacts, out_dir=target_dir, dry_run=dry_run)
    return {"fields": len(FIELDS), "keys": keys, "out_dir": str(target_dir)}


def main() -> None:
    parser = argparse.ArgumentParser(description="Bake seasonal replay bundles for panen-id.")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--out-dir", type=Path, default=None)
    args = parser.parse_args()
    res = run(dry_run=args.dry_run, out_dir=args.out_dir)
    print(f"[panen-pipeline] OK dry_run={args.dry_run} fields={res['fields']} keys={len(res['keys'])}")


if __name__ == "__main__":
    main()
