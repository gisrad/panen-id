# Build Progress — Panen-ID

- [x] **Pure-Rust C-ABI WASM (`crates/agro-core`)**: 9/9 `cargo test` passing; `agro_core.wasm` (`31,881 bytes`) live.
- [x] **Real Landform Units & 3D Horizon Block**: 68 DAS Bompon landform polygons with measured `Clay`/`Silt`/`Sand`/permeability/landslide attributes + lazy `three.js` 3D stratigraphy viewer (`192.60 kB` gzip split chunk).
- [x] **Multi-Season Real ERA5 (`MT2 2024` vs `2025`)**: Real Open-Meteo ERA5 daily weather for both seasons, multi-season stability classification (`stabil_tinggi`, `sensitif_iklim`, `rentan_kronis`), hover tooltips, and interactive What-If simulator.
- [x] **Local-First BYOD & QGIS Exports**: In-browser CSV/GeoJSON analyzer with `n < 30` guard, stateful permalinks, embed mode (`?embed=1`), 4-language UI (`ID | EN | JA | ZH`), and QGIS-ready GeoJSON exports.
- [x] **Sentinel-2 Python Pipeline (`pipeline/`)**: Verified `unittest` suite (4/4 passing) with Sentinel-2 Processing Baseline >= 04.00 `BOA_ADD_OFFSET = -1000` correction & offline `--dry-run`.
