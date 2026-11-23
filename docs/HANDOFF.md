# Handoff Sesi — Panen-ID (`gisrad/panen-id`)

- **Tanggal terakhir**: `2026-11-24`
- **Fase aktif**: `v0.8.0 (P0–P8) — SELESAI & LIVE`
- **Langkah terakhir selesai**:
  - Menambahkan permalink berkeadaan, mode `?embed=1`, ekspor subset GeoJSON siap QGIS, dukungan BYOD seluruh koordinat Indonesia (`95°–141° BT, 6° LU–11° LS`), serta `CITATION.cff` & `CHANGELOG.md`.
  - Mengunggah bundel seed `v1/season/bompon-magelang-mt2-2025.json` dan `v1/season/klaten-irigasi-mt2-2025.json` ke bucket R2 `agri-transbulent-data`.
  - Deploy `panen-id-worker` (`c47db3a1-6b38-4eb7-801f-dea140ceff33`) dan verifikasi live HTTPS `200` (`embed=1` CSP, WASM `31881` bytes).
- **Ekspansi selanjutnya**:
  - Perluasan model fenologi multi-komoditas (kedelai, hortikultura, tebu, lahan rawa) & partisi R2 nasional 38 provinsi sesuai `ROADMAP.md` di `transbulent.id`.
