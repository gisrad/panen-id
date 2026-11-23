# Progres Pembangunan — Panen-ID

- [x] **P0** — Dokumentasi SSOT awal (`README`, `.gitignore`, `.env.example`, `docs/*`)
- [x] **P1** — Kerangka `web/` (Astro 5 + Svelte 5), `worker/`, i18n/Swiss Sensus & Mock Data Bompon/Klaten
- [ ] **P2** — Crate Rust `crates/agro-core` → WASM + Spike `geolibre-rust`
- [x] **P3 (Fondasi 2D)** — Tampilan Replay (`/ulang/[lahan]/[musim]`): peta MapLibre Crameri, scrubber 120 hari, grafik `uPlot`, SVG tanaman progresif, & 6 horizon SoilGrids *(Blok 3D `three.js` lazy menyusul di P3b/F6)*
- [x] **P4 (Fondasi UI)** — Tab *Lensa Dangkal vs Dalam* & jembatan lintas pilar ke `/kekeringan/[kode_bps]/` *(OLS/RF WASM menyusul di P2/F6)*
- [ ] **P5** — Pipeline Python (`pipeline/`) -> R2
- [ ] **P6** — Ruang Kerja `/lahan-saya` (*local-first*) & Ekspor Subset
- [ ] **P7** — Playwright E2E & pengerasan produksi lanjutan

## Catatan Build P1/P3 (2026-11-24)

- Build statis Astro 5 + Svelte 5 + MapLibre GL v5 + `uPlot` hijau (`PanenWorkbench.js = 321.85 kB` gzip, di bawah anggaran <= 350 kB).
- Ter-deploy privat (`workers_dev = false`) sebagai `panen-id-worker` dan terhubung ke `https://agri.transbulent.id/ulang/` via Cloudflare Service Binding `PANEN`.
