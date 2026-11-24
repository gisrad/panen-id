# Roadmap — Panen-ID

| Stage | Milestone | Status |
| --- | --- | --- |
| **1. Season Replay Foundation** | Astro 5 + Svelte 5 + MapLibre GL v5 + `uPlot`, progressive crop phenology SVG, 120-day daily scrubber, and Shallow vs Deep lens analysis | Completed |
| **2. Rust WASM `agro-core`** | Zero-dependency C-ABI WASM module (`GDD`, `FAO-56 ET0`, `IDW`, `OLS` with `n < 30` statistical guard) & GeoLibre spike evaluation (`ADR_GEOLIBRE_SPIKE.md`) | Completed |
| **3. Real Geomorphology & 3D Block** | 68 real DAS Bompon landform polygons (`Clay`/`Silt`/`Sand`, permeability, landslide classes) + lazy-loaded Three.js 6-horizon 3D stratigraphic block | Completed |
| **4. Real ERA5 & Multi-Season Stability** | Open-Meteo ERA5 `MT2 2024` (dry) vs `MT2 2025` (wet) YoY yield comparison, `Stabilitas 24–25` map layer, interactive What-If simulator & BYOD CSV/GeoJSON workbench | Completed |
| **5. Multi-Commodity Expansion** | 5-crop Rust WASM phenology engine (`Padi`, `Jagung`, `Kedelai`, `Bawang Merah`, `Tebu`) with crop-specific `Tbase`/`Tupper` GDD accumulation | Completed |
