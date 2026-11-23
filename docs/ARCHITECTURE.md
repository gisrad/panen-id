# Arsitektur — Panen-ID (Musim Ulang)

## 1. Struktur Direktori Repositori

```text
panen-id/
├── web/                          # Astro 5 + Svelte 5 + MapLibre GL v5 + three.js (lazy) + uPlot
│   ├── public/data/mock/         # Dataset simulasi Jateng & DAS Bompon untuk pengembangan lokal
│   ├── src/pages/                # /ulang/[lahan]/[musim], /analisis/[lahan], /lahan-saya
│   └── src/components/           # Island peta 2D, Blok 3D, scrubber, uPlot, SVG tanaman
├── crates/
│   └── agro-core/                # Rust -> wasm32-unknown-unknown via wasm-bindgen
│       ├── src/                  # gdd, phenology, fao56, stress, idw, zonal, stats (OLS/RF/kmeans)
│       └── tests/                # Unit tests terhadap contoh baku FAO-56
├── pipeline/                     # Python 3.11+ offline bake (Open-Meteo, CHIRPS, SoilGrids, LiDAR, S2/HLS, S1)
├── worker/                       # Cloudflare Worker panen-id-worker (Static Assets + Binding PANEN)
├── docs/                         # SSOT lokal panen-id
├── .env.example
└── .gitignore
```

## 2. Integrasi ke `agri.transbulent.id` & Mode Jalan Mandiri

- **Produksi (`Service Binding PANEN`)**: `panen-id-worker` di-deploy tanpa rute publik langsung (`workers_dev = false`, `preview_urls = false`). Seluruh permintaan `/ulang/*`, `/analisis/*`, dan `/lahan-saya` masuk melalui `workers/agri-gateway` di `D:\transbulent.id` sehingga gerbang pratinjau 5 menit (`overlay`) dan sesi Google OAuth `__Host-agri_session` otomatis berlaku.
- **Pengembangan Mandiri (Lokal)**: `web/` berjalan di `http://127.0.0.1:4327` memakai mock JSON lokal tanpa memerlukan kredensial R2 maupun gateway.

## 3. Kontrak Sistem Desain Bersama (`agri-tokens`)

- **Tema Default**: **Malam Sawah** (`#13110E` / `#1C1915` / `#2A251F`, garis `#3A332B`) untuk inspeksi peta hasil & Blok 3D; mendukung switch ke **Kertas Beras** (`#F3EEE2` / `#1C1A16`).
- **Tipografi**: `Newsreader` (judul), `Schibsted Grotesk` (UI), `IBM Plex Mono` (`tabular-nums` untuk koordinat, GDD, dan hasil panen), serta system font stack untuk `ja`/`zh`.
- **Colormap Ilmiah (Crameri, Aman Buta Warna)**: Hasil panen = `batlow`; Anomali hasil/suhu = `vik`; NDVI = `bamako`; % Liat = `lajolla`; Hujan = `oslo`.
- **Dok Melayang & Anggaran Performa**: berbagi spesifikasi Dok Melayang (pil 56 px, drag/snap, auto-hide scroll, pintasan `1`–`5` & `/`, Mode Ringkas, bottom sheet 3 snap di mobile). JS peta 2D <= **350 KB gzip**; `three.js` dimuat *lazy* hanya ketika mode Blok 3D diaktifkan.
