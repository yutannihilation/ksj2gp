# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ksj2gp is a browser-based tool that converts Japan's National Land Numerical Information (国土数値情報 / KSJ) Shapefiles into GeoParquet, GeoPackage, and GeoJSON formats. It translates cryptic KSJ attribute names and coded values (e.g., `L001_123`) into human-readable Japanese labels. Runs entirely in-browser via WebAssembly compiled from Rust, with a SvelteKit frontend. The UI is in Japanese.

## Commands

### Frontend (repo root, uses pnpm)

- `pnpm run dev` — Dev server (use `--force` after rebuilding WASM)
- `pnpm run build` — Build static site to `build/`
- `pnpm run check` — Svelte type checking
- `pnpm run lint` — Prettier + ESLint
- `pnpm run format` — Auto-format with Prettier
- `pnpm run test:browser` — Vitest browser tests (Playwright, headless Chromium)

### Rust (from `rust/` directory)

- `cargo test --workspace` — Run all Rust tests
- `cargo build --target wasm32-unknown-unknown -p ksj2gp-web` — Build WASM (debug)
- `cargo build --target wasm32-unknown-unknown --release -p ksj2gp-web` — Build WASM (release)

After building WASM, generate JS bindings from repo root:
```
wasm-bindgen --out-dir npm --typescript --target bundler ./rust/target/wasm32-unknown-unknown/debug/ksj2gp_web.wasm
```

Optional WASM optimization: `wasm-opt ./npm/ksj2gp_web_bg.wasm -O -o tmp.wasm && mv tmp.wasm ./npm/ksj2gp_web_bg.wasm`

## Architecture

### Rust workspace (`rust/`)

Three crates in a Cargo workspace:

- **`ksj2gp`** (library) — Core conversion logic: Shapefile reading, CRS detection, encoding detection, attribute translation, and output writing (GeoParquet/GeoJSON/GeoPackage)
- **`ksj2gp-web`** (cdylib) — WASM bindings via wasm-bindgen (NOT wasm-pack). Exports `list_shp_files` and `convert_shp`. Implements `Read`+`Seek`+`Write` for browser File API (`UserLocalFile`) and OPFS (`OpfsFile`)
- **`ksj2gp-cli`** — Command-line interface using clap

Key Rust modules:
- `builder.rs` — Arrow/GeoArrow schema and array construction
- `crs/` — CRS detection from `.prj` (ESRI WKT) or `KS-META-*.xml`; supports EPSG 4301/4612/6668 only
- `encoding.rs` — CP437↔CP932 conversion for Shift_JIS ZIP filenames (zip crate doesn't support Shift_JIS)
- `transform_coord.rs` — Tokyo Datum→WGS84 via proj4rs (JGD2000/JGD2011 treated as equivalent to WGS84)
- `translate/` — Column name and codelist value translation. `translate/data/` files are **auto-generated** by R scripts in `scripts/`; do not edit manually
- `writer/` — GeoParquet, GeoJSON, GeoPackage output writers
- `zip_reader.rs` — ZIP extraction with CRS and encoding guessing

### SvelteKit frontend (`src/`)

- Static site (adapter-static), base path `/ksj2gp` for GitHub Pages
- **Svelte 5** with runes (`$state`, `$derived`, `$props`, `$bindable`)
- **Tailwind CSS v4** via `@tailwindcss/vite`
- **Bits UI v2** for accessible components (Dialog, Select, Switch)
- WASM consumed as local npm package (`"ksj2gp": "file:npm"` in package.json)

### Worker pattern

Main thread → Web Worker → WASM. The worker (`src/lib/worker.ts`) extracts shapefile components from ZIP into OPFS temp files (because the shapefile reader requires `Read + Seek`), calls WASM `convert_shp`, then returns the OPFS file handle to the main thread for download via `URL.createObjectURL()`.

### Component tests

Tests use `vitest-browser-svelte` with Playwright (Chromium, headless). Some components have `*TestHost.svelte` wrappers for testing bound state.

## Code Style

- Prettier: tabs, single quotes, no trailing commas, 100-char width
- ESLint: typescript-eslint + eslint-plugin-svelte + prettier
- Rust: standard rustfmt
