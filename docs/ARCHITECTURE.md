# Architecture — Panen-ID (`Season Replay`)

## 1. Repository Layout

```text
panen-id/
├── crates/agro-core/        # Pure-Rust C-ABI WebAssembly kernel (< 35 KB): GDD, FAO-56 ET0, IDW & OLS
├── pipeline/                # Sentinel-2 L2A radiometric offset + SCL masking & seasonal bundle export
├── scripts/                 # Open-Meteo ERA5 archive sync, landform polygon extraction & YoY stability
├── web/                     # Astro 5 + Svelte 5 + MapLibre GL v5 + uPlot + lazy Three.js + WASM bridge
│   ├── public/mock-data/    # MT2 2024 & MT2 2025 real ERA5 + landform datasets
│   └── public/wasm/         # Compiled agro_core.wasm
└── docs/                    # Architecture, scientific methodology & build progress
```

## 2. Core Technical Architecture

- **Static-First Cartographic Workbench**: Built with Astro 5, Svelte 5, MapLibre GL v5, and `uPlot` so initial interactive loads stay compact (`<= 335 kB` gzipped), while `three.js` (`SoilBlock3D.svelte`) is dynamically imported only when inspecting the 3D horizon block.
- **Zero-Glue Rust WebAssembly Engine (`crates/agro-core`)**: Exposes a pure C-ABI compiled to `wasm32-unknown-unknown` (~31 KB) computing modified Growing Degree Days, FAO-56 Penman-Monteith $ET_0$, Inverse Distance Weighting (`IDW`), and OLS regression with an automatic small-sample guard (`n < 30`).
- **Local-First BYOD Workspace (`/lahan-saya`)**: Growers can load local CSV or GeoJSON yield records (`95°–141° E, 6° N–11° S`) for instant in-browser IDW/OLS diagnostics and QGIS-ready exports without transmitting raw farm records to external servers.
- **Clean Data Provenance**: Synthetic yield estimates are explicitly flagged (`is_simulated: true` / `SIMULASI`), while weather timelines use archived Open-Meteo ERA5 daily reanalysis and measured soil texture attributes.
