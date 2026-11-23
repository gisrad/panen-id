"""Sentinel-2 L2A NDVI helpers including Processing Baseline >= 04.00 BOA_ADD_OFFSET."""

from typing import Optional, Sequence

from config import S2_BASELINE_04_CUTOFF, S2_BOA_ADD_OFFSET, S2_SCALE

# Sentinel-2 L2A Scene Classification Layer (SCL) valid clear land/water classes:
# 4 = Vegetation, 5 = Not-vegetated, 6 = Water, 7 = Unclassified
VALID_SCL_CLASSES = {4, 5, 6, 7}


def is_baseline_04_or_newer(scene_date: str, baseline: Optional[str] = None) -> bool:
    if baseline is not None:
        try:
            return float(baseline) >= 4.0
        except ValueError:
            pass
    return scene_date >= S2_BASELINE_04_CUTOFF


def to_surface_reflectance(dn: float, scene_date: str, baseline: Optional[str] = None) -> float:
    """Convert Sentinel-2 L2A integer DN to surface reflectance [0, 1]."""
    adjusted = dn + S2_BOA_ADD_OFFSET if is_baseline_04_or_newer(scene_date, baseline) else dn
    sr = adjusted / S2_SCALE
    return max(0.0, min(1.0, sr))


def mask_scl_clear(scl_code: int) -> bool:
    return scl_code in VALID_SCL_CLASSES


def compute_ndvi(nir_sr: float, red_sr: float) -> float:
    denom = nir_sr + red_sr
    if abs(denom) < 1e-9:
        return 0.0
    return round(max(-1.0, min(1.0, (nir_sr - red_sr) / denom)), 4)


def smooth_series(values: Sequence[Optional[float]], half_window: int = 2) -> list[float]:
    """Linearly interpolate masked gaps then apply a triangular smoothing kernel."""
    n = len(values)
    if n == 0:
        return []

    valid_idx = [i for i, v in enumerate(values) if v is not None]
    if not valid_idx:
        return [0.0] * n

    filled: list[float] = [0.0] * n
    first_i, last_i = valid_idx[0], valid_idx[-1]
    for i in range(0, first_i + 1):
        filled[i] = float(values[first_i])  # type: ignore[arg-type]
    for i in range(last_i, n):
        filled[i] = float(values[last_i])  # type: ignore[arg-type]

    for a, b in zip(valid_idx, valid_idx[1:]):
        va = float(values[a])  # type: ignore[arg-type]
        vb = float(values[b])  # type: ignore[arg-type]
        filled[a] = va
        span = b - a
        for step in range(1, span):
            filled[a + step] = va + (vb - va) * (step / span)
        filled[b] = vb

    smoothed: list[float] = []
    for i in range(n):
        w_sum = 0.0
        v_sum = 0.0
        for d in range(-half_window, half_window + 1):
            j = i + d
            if 0 <= j < n:
                w = float(half_window + 1 - abs(d))
                w_sum += w
                v_sum += w * filled[j]
        smoothed.append(round(v_sum / w_sum, 4))
    return smoothed
