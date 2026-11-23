# Changelog — Panen-ID (`gisrad/panen-id`)

## [0.8.0] - 2026-11-24

### Ditambahkan
- **P8 (Berbagi & Ekspor Subset)**: Permalink berkeadaan (`?tab=...&layer=...&day=...&plot=...`), mode embed `?embed=1`, serta ekspor subset GeoJSON siap buka di QGIS dengan atribusi dan penanda `SIMULASI`.
- **P6 (`LahanSayaPanel.svelte`)**: Analisis CSV/GeoJSON lokal (*local-first*) di memori peramban melalui `agro_core.wasm`, guard statistik `n < 30`, serta unduhan CSV & GeoJSON beratribusi untuk koordinat seluruh Indonesia (`95°–141° BT`, `6° LU–11° LS`).
- **P5 (`pipeline/`)**: Pemrosesan NDVI Sentinel-2 L2A (koreksi radiometrik `BOA_ADD_OFFSET = -1000` untuk Baseline >= 04.00, masking awan SCL, interpolasi & smoothing), `season_bake.py --dry-run`, uploader R2, dan 4 unit test.
- **P3b (`SoilBlock3D.svelte`)**: Visualisasi blok stratigrafi 6 horizon SoilGrids 3D dengan *lazy-load* `three.js` (chunk terpisah).
- **P2 (`crates/agro-core`)**: Kernel C-ABI Rust WebAssembly (`GDD`, `FAO-56 ET0`, `IDW`, `OLS` guard `n < 30`) sebesar `31,881 bytes` dan dokumen keputusan `docs/ADR_GEOLIBRE_SPIKE.md`.

## [0.1.0] - 2026-11-24
- **P0–P1 & P3–P4**: Fondasi Astro 5 + Svelte 5 + MapLibre GL v5 + `uPlot`, mock data DAS Bompon & Klaten 120 hari, *CropStageSvg*, dan jembatan lintas pilar ke `/kekeringan/`.
