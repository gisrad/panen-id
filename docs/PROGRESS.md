# Progres Pembangunan — Panen-ID

- [x] **P0** — Dokumentasi SSOT awal (`README`, `.gitignore`, `.env.example`, `docs/*`)
- [ ] **P1** — Kerangka `web/` (Astro 5 + Svelte 5), `worker/`, Dok Melayang, i18n & Mock Data Bompon/Jateng
- [ ] **P2** — Crate Rust `crates/agro-core` → WASM + Spike `geolibre-rust`
- [ ] **P3** — Tampilan Replay (`/ulang/[lahan]/[musim]` + Blok 3D SoilGrids)
- [ ] **P4** — Tampilan Analisis (`/analisis/[lahan]` + Jembatan Lintas Pilar)
- [ ] **P5** — Pipeline Python (`pipeline/`) -> R2
- [ ] **P6** — Ruang Kerja `/lahan-saya` (*local-first*) & Ekspor Subset
- [ ] **P7** — Service Binding `PANEN` & Playwright E2E

## Sub-Checklist Atomik P1

- [ ] Inisialisasi `web/` (Astro 5 + Svelte 5 + TypeScript + `maplibre-gl` v5 + `uplot`)
- [ ] Sinkronkan token tema *Malam Sawah* / *Kertas Beras* & font self-hosted sesuai `D:\transbulent.id\docs\agri\DESIGN_SYSTEM.md`
- [ ] Buat mock JSON realistis untuk `bompon-magelang` dan `sawah-jateng` (wajib `is_simulated: true`)
- [ ] Buat `worker/` (`panen-id-worker`, `workers_dev = false`, `preview_urls = false`)
- [ ] Verifikasi `npm run build` di `web/` hijau
