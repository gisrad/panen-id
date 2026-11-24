"""Fetch real 2025 daily weather from Open-Meteo ERA5 archive for DAS Bompon & Klaten bundles."""

import json
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

MOCK_DIR = Path(r"C:\Users\anggi\.gemini\antigravity\scratch\panen-id\web\public\mock-data")
TARGETS = [
    ("bompon-mt2-2025.json", -7.5549, 110.0665),
    ("klaten-mt2-2025.json", -7.622, 110.675),
]
START_DATE = "2025-05-01"
END_DATE = "2025-08-28"


def stage_from_gdd(gdd_cum: float) -> str:
    if gdd_cum < 120:
        return "VE"
    if gdd_cum < 500:
        return "tillering"
    if gdd_cum < 900:
        return "pi"
    if gdd_cum < 1200:
        return "heading"
    if gdd_cum < 1550:
        return "grain_fill"
    return "maturity"


def fetch_openmeteo_daily(lat: float, lon: float) -> dict[str, Any]:
    params = urllib.parse.urlencode(
        {
            "latitude": lat,
            "longitude": lon,
            "start_date": START_DATE,
            "end_date": END_DATE,
            "daily": "temperature_2m_max,temperature_2m_min,precipitation_sum,et0_fao_evapotranspiration",
            "timezone": "Asia/Jakarta",
        }
    )
    url = f"https://archive-api.open-meteo.com/v1/archive?{params}"
    req = urllib.request.Request(url, headers={"User-Agent": "AgriTransbulent/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        if resp.status != 200:
            raise RuntimeError(f"Open-Meteo HTTP {resp.status}")
        return json.loads(resp.read().decode("utf-8"))


def detect_dry_windows(dates: list[str], rain: list[float], et0: list[float]) -> list[dict[str, str]]:
    n = len(dates)
    dry_flags: list[bool] = []
    for i in range(n):
        w_start = max(0, i - 6)
        roll7_rain = sum(rain[w_start : i + 1])
        dry_flags.append(roll7_rain < 10.0 and et0[i] > rain[i])

    windows: list[dict[str, str]] = []
    start_i: int | None = None
    for i, is_dry in enumerate(dry_flags):
        if is_dry and start_i is None:
            start_i = i
        elif not is_dry and start_i is not None:
            if i - start_i >= 4:
                windows.append({"start": dates[start_i], "end": dates[i - 1], "kind": "dry_spell_era5"})
            start_i = None
    if start_i is not None and n - start_i >= 4:
        windows.append({"start": dates[start_i], "end": dates[-1], "kind": "dry_spell_era5"})
    return windows


def main() -> None:
    for filename, lat, lon in TARGETS:
        path = MOCK_DIR / filename
        bundle = json.loads(path.read_text(encoding="utf-8"))
        om = fetch_openmeteo_daily(lat, lon)
        daily = om["daily"]

        dates = list(daily["time"])
        tmax = [round(float(v or 30.0), 1) for v in daily["temperature_2m_max"]]
        tmin = [round(float(v or 22.0), 1) for v in daily["temperature_2m_min"]]
        rain = [round(float(v or 0.0), 1) for v in daily["precipitation_sum"]]
        et0 = [round(float(v or 3.5), 2) for v in daily["et0_fao_evapotranspiration"]]
        if len(dates) != 120:
            raise RuntimeError(f"Expected 120 days, got {len(dates)} for {filename}")

        gdd_cum: list[float] = []
        stages: list[str] = []
        acc = 0.0
        for hi, lo in zip(tmax, tmin):
            hi_c = min(35.0, max(10.0, hi))
            lo_c = min(35.0, max(10.0, lo))
            acc += max(0.0, 0.5 * (hi_c + lo_c) - 10.0)
            gdd_cum.append(round(acc, 1))
            stages.append(stage_from_gdd(acc))

        bundle["dates"] = dates
        bundle["tmax_c"] = tmax
        bundle["tmin_c"] = tmin
        bundle["rain_mm"] = rain
        bundle["et0_mm"] = et0
        bundle["gdd_cum"] = gdd_cum
        bundle["stage_code"] = stages
        bundle["stress_windows"] = detect_dry_windows(dates, rain, et0)
        bundle["weather_source"] = f"Open-Meteo ERA5 Reanalysis ({START_DATE} s.d. {END_DATE})"

        path.write_text(json.dumps(bundle, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
        print(
            f"UPDATED {filename}: days={len(dates)} rain_total={sum(rain):.1f}mm "
            f"gdd_final={gdd_cum[-1]} stress_windows={len(bundle['stress_windows'])} "
            f"plots={len(bundle['plots']['features'])}"
        )


if __name__ == "__main__":
    main()
