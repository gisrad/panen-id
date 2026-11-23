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
