# Panen-ID — Musim Ulang (*Season Replay*)

**Observatorium analisis hasil panen berbasis lingkungan untuk Jawa Tengah & Indonesia.**  
Menjawab pertanyaan inti: *"Varietas ini kurang hasil di mana, kapan, dan karena apa?"*

`Panen-ID` merupakan **Pilar 1** dalam ekosistem terfederasi [`agri.transbulent.id`](https://agri.transbulent.id) (melayani rute `/ulang/*`, `/analisis/*`, dan `/lahan-saya` melalui Cloudflare Workers Service Binding `PANEN`), namun dirancang mandiri agar dapat dikembangkan, diuji, dan dibuka sebagai proyek open-source terpisah.

## Fitur & Arsitektur Inti

- **Input Hasil Realistis Indonesia**: mendukung hasil per petak (poligon), titik ubinan metode BPS, catatan panen petani, dan proksi satelit (integral NDVI/EVI) terkalibrasi ubinan — serta berkas *combine harvester* (`CSV`, `Shapefile .zip`, `ISOXML`) sebagai jalur sekunder.
- **Visualisasi Replay & Blok 3D Geofisika**:
  - Peta 2D interaktif dengan **MapLibre GL v5** (colormap Fabio Crameri aman buta warna).
  - Blok 3D (*lazy-load* **three.js**): *heightfield* DEMNAS / LiDAR DAS Bompon bertekstur peta hasil, dengan **dinding samping 6 horizon tanah SoilGrids** (`0–5` s.d. `100–200` cm) diwarnai menurut persentase liat.
  - Grafik cuaca & akumulasi GDD via **uPlot** + ilustrasi tanaman SVG progresif (padi & jagung) yang tersinkron dengan *scrubber* waktu 60 fps.
- **Komputasi Lokal di Browser (`crates/agro-core` Rust WASM + DuckDB-WASM)**: GDD, fenologi, neraca air FAO-56, indeks stres, IDW, *zonal statistics*, OLS interaksi `varietas × zona`, *feature importance* Random Forest, peta stabilitas, dan zona manajemen (*k-means*) — data lahan pribadi di `/lahan-saya` tidak meninggalkan perangkat.
- **Lahan Demo Flagship Jawa Tengah**:
  1. Lahan di **DAS Bompon, Magelang** (memanfaatkan topografi mikro LiDAR internal; data mentah LiDAR tidak dipublikasikan).
  2. Satu **sawah irigasi Jawa Tengah** (dipilih usai verifikasi tutupan lahan Sentinel-2).
  *(Angka hasil panen sintetis wajib diberi label **`SIMULASI`** hingga data ubinan nyata tersedia.)*

## Indeks Dokumentasi (`docs/`)

| Berkas | Isi |
| --- | --- |
| [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) | Struktur repositori, jembatan Service Binding `PANEN` ke `agri.transbulent.id`, kontrak token `agri-tokens` |
| [docs/SCIENCE_AND_DATA.md](./docs/SCIENCE_AND_DATA.md) | Rumus GDD, FAO-56, stres panas/kering, pitfall Sentinel-2 Baseline 04.00, HLS + Sentinel-1 VH, & skema R2 |
| [docs/ROADMAP.md](./docs/ROADMAP.md) | Tahapan build mandiri P0–P7 |
| [docs/PROGRESS.md](./docs/PROGRESS.md) | Daftar periksa progres P0–P7 & rincian atomik P1 |
| [docs/HANDOFF.md](./docs/HANDOFF.md) | Titik serah-terima sesi agar pekerjaan bisa berhenti & lanjut kapan saja |

## Protokol Berhenti & Lanjut

Baca **`docs/HANDOFF.md`** lalu **`docs/PROGRESS.md`** setiap memulai sesi baru. Akhiri setiap sesi dengan build hijau, pembaruan kedua berkas tersebut, dan commit bersih.
