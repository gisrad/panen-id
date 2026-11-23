# Progres Pembangunan — Panen-ID

- [x] **P0** — Dokumentasi SSOT awal (`README`, `.gitignore`, `.env.example`, `docs/*`)
- [x] **P1** — Kerangka `web/` (Astro 5 + Svelte 5), `worker/`, i18n/Swiss Sensus & Mock Data Bompon/Klaten
- [x] **P2** — Crate Rust `crates/agro-core` → WASM (`31.88 KB`, 9 unit tests PASS) + `docs/ADR_GEOLIBRE_SPIKE.md` (NO-GO runtime map)
- [x] **P3 (Fondasi 2D + Blok 3D Lazy)** — Replay 2D + `SoilBlock3D.svelte` (*lazy* `three.js` chunk `192.60 kB` gzip) + 6 horizon SoilGrids
- [x] **P4 (Lensa & Jembatan)** — Tab *Lensa Dangkal vs Dalam* & jembatan lintas pilar ke `/kekeringan/[kode_bps]/`
- [x] **P5** — Pipeline Python (`pipeline/s2_ndvi.py`, `season_bake.py`, `r2_uploader.py`, 4 unit tests PASS + `--dry-run`) -> R2
- [x] **P6** — Ruang Kerja `LahanSayaPanel.svelte` `/lahan-saya` (*local-first* BYOD CSV/GeoJSON, WASM OLS + IDW, guard statistik `n < 30`, ekspor ber-atribusi)
- [ ] **P7** — Playwright E2E lintas-browser & pengerasan produksi lanjutan

## Catatan Build P2–P6 (2026-11-24)

- `crates/agro-core`: 9 Rust `cargo test` PASS; biner C-ABI `web/public/wasm/agro_core.wasm` berukuran `31,881 bytes` (< 60 KB).
- Build `web/` hijau: `PanenWorkbench.D0-PAQue.js = 328.71 kB` gzip (di bawah anggaran <= 350 kB), `three.module.BR4CzLBj.js = 192.60 kB` gzip terpisah (*lazy*).
- `panen-id-worker` Version `b2efed48-6b43-43ac-ab18-94d6ae5e3525` aktif di `https://agri.transbulent.id/ulang/` dan `/ulang/wasm/agro_core.wasm` (HTTP `200`, CSP `'wasm-unsafe-eval'`).
