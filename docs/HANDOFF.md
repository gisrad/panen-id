# Handoff Sesi — Panen-ID (`gisrad/panen-id`)

- **Tanggal terakhir**: `2026-11-24`
- **Fase aktif**: `P1 & Fondasi Replay 2D — SELESAI & LIVE`
- **Langkah terakhir selesai**:
  - Membangun `web/` (Astro 5 + Svelte 5 + MapLibre GL v5 + `uPlot`) untuk 2 lahan Jawa Tengah (`bompon-magelang` & `klaten-irigasi`, 120 hari GDD/hujan/Tmax, 6 horizon SoilGrids, 9 petak hasil berlabel `SIMULASI`), *CropStageSvg*, *Timeline Scrubber*, tab *Lensa Dangkal vs Dalam*, jembatan ke `/kekeringan/`, dan *TrialOverlay* 5 menit.
  - Deploy ke `panen-id-worker` dan terhubung di `https://agri.transbulent.id/ulang/bompon-magelang/mt2-2025/`.
- **Perintah sesi berikutnya**:
  1. Baca `docs/HANDOFF.md` & `docs/PROGRESS.md`.
  2. Kerjakan **P2** (`crates/agro-core` Rust → WASM + spike evaluasi `geolibre-rust`), lalu tambah Blok 3D `three.js` *lazy-load*.
