# ADR-001: GeoLibre / Rust-in-Browser Map Spike — Decision: NO-GO for Runtime Core

- **Date**: 2026-11-24
- **Status**: Accepted (NO-GO for WebGPU/Rust map rendering core; GO for `agro-core` pure-Rust C-ABI WASM math kernel)
- **Context**: Evaluated whether `panen-id` should adopt a Rust-compiled map renderer (GeoLibre / MapLibre-rs) versus keeping MapLibre GL JS v5 for 2D vector/choropleth rendering alongside a dedicated Rust-WASM scientific compute module (`crates/agro-core`).

## Findings

1. **Ecosystem & Build Footprint on Windows + Cloudflare Workers**:
   - `maplibre-gl` v5 (`^5.6.0`) already provides production-grade WebGL2 vector tiles, GeoJSON fill-extrusion, expression-driven Crameri color ramps, and seamless integration with PMTiles and `uPlot` timelines at ~220 KB gzipped.
   - Adopting an experimental Rust/WASM map renderer (GeoLibre / `maplibre-rs`) requires unstable `wgpu`/WebGL2 canvas bridging, larger WASM payloads (> 1.5 MB uncompressed), and breaks Cloudflare Workers fallback determinism and Safari/mobile compatibility.
2. **Where Rust WASM Excels (`crates/agro-core`)**:
   - Moving agronomic numerical kernels—modified Growing Degree Days (GDD), FAO-56 Penman-Monteith $ET_0$, Inverse Distance Weighting (IDW) grid interpolation, and OLS regression with explicit $n < 30$ statistical sample-size guards—into zero-dependency Rust (`wasm32-unknown-unknown` with `opt-level = "z"` and `lto = true`) produces a compact **< 60 KB** binary loaded lazily via `WebAssembly.instantiateStreaming`.
   - Zero `wasm-bindgen` JS glue is required when exposing a clean C-ABI (`alloc_f64`, `dealloc_f64`, `wasm_*`).

## Decision

- **NO-GO** for GeoLibre / Rust map rendering engine in browser runtime. Keep **MapLibre GL v5** for 2D maps and **lazy `three.js`** for optional 3D Soil Block stratigraphy.
- **GO** for **pure-Rust C-ABI `crates/agro-core`** compiled to `/ulang/wasm/agro_core.wasm` for client-side BYOD plot diagnostics and weather verification.
