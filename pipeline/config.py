"""Configuration for panen-id seasonal replay bake & Sentinel-2 NDVI processing."""

import os
from typing import Any

S2_SCALE = 10000.0
S2_BOA_ADD_OFFSET = -1000.0
S2_BASELINE_04_CUTOFF = "2022-01-25"

R2_BUCKET = os.environ.get("R2_BUCKET", "season-replay-data")
R2_ENDPOINT = os.environ.get("R2_ENDPOINT", "")
R2_PREFIX = os.environ.get("SEASON_R2_PREFIX", "v1/season")

FIELDS: list[dict[str, Any]] = [
    {
        "field_id": "bompon-magelang",
        "name": "DAS Bompon (Lahan Berlereng)",
        "regency": "Kab. Magelang",
        "linked_kode_bps": "3308",
        "season_id": "mt2-2025",
        "crop": "Padi Sawah Teras",
        "variety": "Inpari 32",
        "center": [110.078, -7.548],
    },
    {
        "field_id": "klaten-irigasi",
        "name": "Delanggu (Irigasi Teknis)",
        "regency": "Kab. Klaten",
        "linked_kode_bps": "3310",
        "season_id": "mt2-2025",
        "crop": "Padi Irigasi",
        "variety": "Rojolele Srinuk",
        "center": [110.675, -7.622],
    },
]
