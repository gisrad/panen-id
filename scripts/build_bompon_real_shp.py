"""Replace synthetic Bompon plot grid with 68 real geomorphic landform polygons from landform_bompon.shp."""

import json
import math
from pathlib import Path
from typing import Any

import geopandas as gpd

SHP_PATH = Path(r"D:\GIS_DATA\bompon\raw\legacy\landform_bompon.shp")
BUNDLE_PATH = Path(r"C:\Users\anggi\.gemini\antigravity\scratch\panen-id\web\public\mock-data\bompon-mt2-2025.json")


def round_coords(obj: Any, precision: int = 5) -> Any:
    if isinstance(obj, float):
        return round(obj, precision)
    if isinstance(obj, (list, tuple)):
        return [round_coords(item, precision) for item in obj]
    if isinstance(obj, dict):
        return {k: round_coords(v, precision) for k, v in obj.items()}
    return obj


def main() -> None:
    df_utm = gpd.read_file(SHP_PATH)
    df_utm["area_m2"] = df_utm.geometry.area
    sub_utm = df_utm[df_utm["area_m2"] >= 5000].copy().sort_values("area_m2", ascending=False).reset_index(drop=True)
    if len(sub_utm) != 68:
        raise RuntimeError(f"Expected 68 polygons >= 5000 m2, got {len(sub_utm)}")

    invalid = ~sub_utm.geometry.is_valid
    if invalid.any():
        sub_utm.loc[invalid, "geometry"] = sub_utm.loc[invalid, "geometry"].buffer(0)

    sub_wgs = sub_utm.to_crs(epsg=4326)
    sub_wgs["geometry"] = sub_wgs.geometry.simplify(0.00005, preserve_topology=True)
    sub_wgs = sub_wgs[~sub_wgs.geometry.is_empty].copy()

    bundle = json.loads(BUNDLE_PATH.read_text(encoding="utf-8"))
    bundle["center"] = [110.0665, -7.5549]
    bundle["name"] = "DAS Bompon — 68 Unit Bentuk Lahan Riil (SHP)"

    features: list[dict[str, Any]] = []
    for idx, row in sub_wgs.iterrows():
        area_m2 = float(sub_utm.loc[idx, "area_m2"])
        area_ha = round(area_m2 / 10000.0, 2)
        clay = round(float(row["Clay"]), 1)
        silt = round(float(row["Silt"]), 1)
        sand = round(float(row["Sand"]), 1)
        perm = round(float(row["LPermea"]), 2)
        longsor = str(row["Longsor"] or "L1").strip()
        unit = str(row["Unit"] or "Unit Lahan").strip()
        morf = str(row["Morfologi"] or "").strip()

        # Physically consistent yield & drainage response from measured Clay, permeability, and landslide hazard
        high_clay_penalty = max(0.0, (clay - 40.0) * 0.095)
        hazard_penalty = 0.35 if longsor in ("L4", "L5") else (0.15 if longsor == "L3" else 0.0)
        yield_val = round(max(3.6, min(7.6, 7.2 - high_clay_penalty - hazard_penalty + 0.12 * math.sin(idx))), 2)
        ndvi_peak = round(max(0.54, min(0.86, 0.56 + (yield_val - 3.6) * 0.07)), 3)

        if clay >= 52:
            drainage = "Buruk (Liat Berat / Lapisan Padat)"
        elif perm < 0.8:
            drainage = "Sedang–Terhambat (Permeabilitas Rendah)"
        else:
            drainage = "Baik–Sedang (Koluvial / Gembur)"

        plot_id = f"BP-{idx + 1:02d}"
        shallow_note = (
            f"Varietas {bundle['variety']} menghasilkan {yield_val} t/ha pada unit {unit}. "
            "Pembacaan dangkal mengira selisih hasil dipicu semata oleh performa benih."
        )
        deep_note = (
            f"Unit riil {unit} ({morf}, {area_ha} ha) bertesktur Liat {clay}% / Debu {silt}% / Pasir {sand}%, "
            f"permeabilitas {perm} mm/jam, kelas longsor {longsor}. Saat jendela kering pra-heading, "
            f"zona liat tinggi ({clay}%) menahan air terlalu erat atau terhambat drainase ({drainage})."
        )

        features.append(
            {
                "type": "Feature",
                "properties": {
                    "plot_id": plot_id,
                    "zone": unit,
                    "morphology": morf,
                    "area_ha": area_ha,
                    "yield_t_ha": yield_val,
                    "clay_pct": clay,
                    "silt_pct": silt,
                    "sand_pct": sand,
                    "permeability": perm,
                    "landslide_class": longsor,
                    "ndvi_peak": ndvi_peak,
                    "drainage": drainage,
                    "shallow_note": shallow_note,
                    "deep_note": deep_note,
                },
                "geometry": round_coords(row.geometry.__geo_interface__, 5),
            }
        )

    bundle["plots"] = {"type": "FeatureCollection", "features": features}
    BUNDLE_PATH.write_text(json.dumps(bundle, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print(f"BOMPON_SHP_BUILT features={len(features)} size_kb={BUNDLE_PATH.stat().st_size / 1024:.1f}")


if __name__ == "__main__":
    main()
