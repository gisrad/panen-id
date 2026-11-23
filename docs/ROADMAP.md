# Roadmap Build — Panen-ID (`P0–P7`)

Selaras dengan fase terpadu `F3`, `F5`, `F6`, `F7b`, `F8`, dan `F10` pada dokumen payung `D:\transbulent.id\docs\agri\ROADMAP.md`.

| Fase | Sinkron Payung | Hasil Utama | Kriteria Selesai |
| --- | --- | --- | --- |
| **P0** | `F0` | Inisialisasi SSOT `panen-id` (`README`, `docs/*`, `.gitignore`, `.env.example`) | 8 berkas terdokumentasi dan ter-push ke `gisrad/panen-id` |
| **P1** | `F3` | Scaffold `web/` (Astro 5 + Svelte 5) + `worker/` + Dok Melayang + i18n (`id`/`en`/`ja`/`zh`) + dataset mock realistis DAS Bompon & sawah irigasi Jateng | Build hijau, peta dasar terbuka di `127.0.0.1:4327`, label `SIMULASI` jelas |
| **P2** | `F5` | Crate Rust `crates/agro-core` → WASM (GDD, fenologi, FAO-56, stres, IDW, zonal, OLS/RF) + spike evaluasi `geolibre-rust` | Unit test cocok dengan contoh baku FAO-56; keputusan *go/no-go* `geolibre-rust` tercatat |
| **P3** | `F6` | Tampilan Replay (`/ulang/[lahan]/[musim]`): peta 2D MapLibre + *scrubber* 60 fps + grafik `uPlot` + SVG tanaman progresif + Blok 3D (`three.js` lazy, 6 horizon SoilGrids) | JS peta 2D <= 350 KB gzip; sinkronisasi scrubber mulus; 3D ter-lazy-load |
| **P4** | `F6` | Tampilan Analisis (`/analisis/[lahan]`): Lensa Dangkal vs Dalam, stabilitas, zona manajemen, guard $n < 30$, insight template, + jembatan lintas pilar dari `/kekeringan` | Angka cocok terhadap notebook referensi Python |
| **P5** | `F7b` | Pipeline Python (`pipeline/`): Open-Meteo, CHIRPS, SoilGrids, DEMNAS/LiDAR Bompon (internal), S2/HLS (dengan koreksi `BOA_ADD_OFFSET = -1000`), S1 VH -> R2 | Skrip *reproducible*, uji offset Baseline 04.00 lulus |
| **P6** | `F8` | Ruang kerja `/lahan-saya` (*local-first* IndexedDB + DuckDB-WASM), unggah CSV/GeoJSON/SHP zip, ekspor subset (CSV/GeoJSON/GeoParquet/PNG) & antrean NDVI Cloud Run Job | Uji 3 format unggahan lulus; file ekspor valid di QGIS |
| **P7** | `F10` | Deploy `panen-id-worker` (`workers_dev = false`) + integrasi penuh Service Binding `PANEN` ke `agri-gateway` + Playwright E2E | Seluruh alur teruji pada `https://agri.transbulent.id/ulang/...` |
