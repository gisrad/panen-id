# Progres Pembangunan — Panen-ID

- [x] **P0** — Dokumentasi SSOT awal (`README`, `.gitignore`, `.env.example`, `docs/*`)
- [x] **P1** — Kerangka `web/` (Astro 5 + Svelte 5), `worker/`, i18n/Swiss Sensus & Mock Data Bompon/Klaten
- [x] **P2** — Crate Rust `crates/agro-core` → WASM (`31.88 KB`, 9 unit tests PASS) + `docs/ADR_GEOLIBRE_SPIKE.md`
- [x] **P3 (Fondasi 2D + Blok 3D Lazy)** — Replay 2D + `SoilBlock3D.svelte` (*lazy* `three.js` chunk `192.60 kB` gzip) + 6 horizon SoilGrids
- [x] **P4 (Lensa & Jembatan)** — Tab *Lensa Dangkal vs Dalam* & jembatan lintas pilar ke `/kekeringan/[kode_bps]/`
- [x] **P5** — Pipeline Python (`pipeline/s2_ndvi.py`, `season_bake.py`, `r2_uploader.py`, 4 unit tests PASS + `--dry-run`) + seed R2 (`v1/season/*`)
- [x] **P6** — Ruang Kerja `LahanSayaPanel.svelte` `/lahan-saya` (*local-first* BYOD CSV/GeoJSON, WASM OLS + IDW, guard statistik `n < 30`, koordinat se-Indonesia)
- [x] **P7 & P8 (Berbagi, Embed, QGIS GeoJSON & Rilis)** — Permalink berkeadaan (`?tab=&layer=&day=&plot=`), mode `?embed=1`, ekspor GeoJSON siap QGIS, `CITATION.cff`, `CHANGELOG.md`

## Catatan Build Final v0.8.0 (2026-11-24)

- `PanenWorkbench.oy9pVYQq.js = 329.78 kB` gzip (di bawah anggaran <= 350 kB); `three.module.BR4CzLBj.js = 192.60 kB` gzip terpisah (*lazy*).
- `panen-id-worker` Version `c47db3a1-6b38-4eb7-801f-dea140ceff33` aktif di `https://agri.transbulent.id/ulang/`, `/ulang/?embed=1` (`frame-ancestors 'self' https://*.transbulent.id`), dan `/ulang/wasm/agro_core.wasm`.

## Update v0.9.0 — SHP Bentuk Lahan Riil DAS Bompon & i18n 4 Bahasa (2026-11-24)

- Skrip `scripts/build_bompon_real_shp.py` mengekstrak `68` unit poligon bentuk lahan riil (`area >= 5.000 m²`, mencakup 97,6% dari `294,71 ha` DAS Bompon) dari `D:\GIS_DATA\bompon\raw\legacy\landform_bompon.shp` (`EPSG:32749` -> `EPSG:4326`), mempertahankan pengukuran riil `Clay`, `Silt`, `Sand`, `LPermea`, dan `Longsor` (`bompon-mt2-2025.json` = `128.0 KB`).
- `PanenWorkbench.svelte` kini memiliki pemilih bahasa sinkron (`ID | EN | JA | ZH` via `localStorage.agri_locale`), zoom adaptif `14.1` untuk DAS Bompon, serta tampilan parameter tanah riil pada kartu petak terpilih (`PanenWorkbench.BUoKUHHt.js` = `331.82 kB` gzip; lazy `three.module.BR4CzLBj.js` = `192.60 kB` gzip).
- Deploy `panen-id-worker` Version `18e9fc7a-dc69-4d17-baed-5b8451b60772` & unggahan R2 `v1/season/bompon-real-mt2-2025.json` terverifikasi live HTTP `200`.

## Update v1.0 — Open-Meteo ERA5 2025, Hover Tooltips & Simulator What-If (2026-11-24)

- Skrip `scripts/fetch_real_weather_openmeteo.py` menarik deret harian riil Open-Meteo ERA5 Reanalysis (`2025-05-01` s.d. `2025-08-28`, 120 hari `tmax`, `tmin`, `rain_mm`, `et0_fao_evapotranspiration`, GDD kumulatif, fenologi, dan jendela stres kering) untuk `bompon-mt2-2025.json` (`1065.8 mm`, `GDD 1656.9`, `68` unit SHP) dan `klaten-mt2-2025.json` (`534.1 mm`, `GDD 2076.2`, `1` jendela stres kering).
- `PanenWorkbench.svelte` dilengkapi hover tooltip petak real-time, sumber cuaca ERA5, dan **Simulator What-If Musim Depan** (`shiftDays` ±21 hari + perbaikan drainase zona liat `0–100%`). Build `PanenWorkbench.BzWYl3YA.js` = `332.85 kB` gzip; `panen-id-worker` Version `f8d6013e-6f48-4cf5-bd0c-89c1ae381734`.
