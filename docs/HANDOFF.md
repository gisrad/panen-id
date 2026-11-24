# Handoff Sesi — Panen-ID (`gisrad/panen-id`)

- **Tanggal terakhir**: `2026-11-24`
- **Fase aktif**: `v1.1 (Multi-Musim MT2 2024 vs 2025 ERA5 + Peta Stabilitas 68 Poligon SHP) — SELESAI & LIVE`
- **Langkah terakhir selesai**:
  - Menambahkan `bompon-mt2-2024.json` (Open-Meteo ERA5 `2024-05-01..2024-08-28`, `590.4 mm`) dan klasifikasi `stability_class` lintas musim (`6 stabil_tinggi`, `27 sensitif_iklim`, `35 rentan_kronis`) via `scripts/build_multiseason_stability.py`.
  - Menambahkan pemilih `MT2 2025` / `MT2 2024 (Kering)`, layer `Stabilitas 24–25`, dan kartu selisih YoY di `PanenWorkbench.svelte`.
  - Mengunggah `v1/season/bompon-real-mt2-2024.json` ke R2, deploy `panen-id-worker` (`6622f5bd-4eee-4500-9c5f-bd4b52094dc4`), dan memverifikasi live HTTPS `200`.
