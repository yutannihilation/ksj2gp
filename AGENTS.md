# Repository Guidelines

## Project Structure & Module Organization

- src/: TypeScript UI and worker code (`main.ts`, `worker.ts`).
- rust/: Rust crate compiled to WebAssembly (`ksj2gp`).
- npm/: Generated bindings from `wasm-bindgen` consumed by Vite.
- dist/: Production build output.
- index.html, vite.config.ts, tsconfig.json: Frontend entry and tooling.
- .github/: CI and repo workflows.

## Build, Test, and Development Commands

- `pnpm dev`: Start the Vite dev server. Use `--force` after rebuilding WASM.
- `pnpm build`: Type-checks (tsc) then `vite build` to `dist/`.
- `pnpm preview`: Serves the production build locally.
- `cd rust && cargo build`: Builds the Rust crate (WASM target required).
- `wasm-bindgen --out-dir npm --typescript --target bundler ./rust/target/wasm32-unknown-unknown/debug/ksj2gp.wasm`: Regenerate JS/TS bindings for the site. Optionally run `wasm-opt -O` on the output.

## Coding Style & Naming Conventions

- TypeScript: 2-space indent, ES modules, camelCase for functions/variables, PascalCase for types. Prefer small, focused modules.
- Files: lowercase short names (e.g., `worker.ts`, `main.ts`).
- Rust: follow rustfmt defaults; small modules under `rust/src/` (`io.rs`, `builder.rs`, etc.). Public APIs should be `snake_case`, types `PascalCase`.

## Testing Guidelines

- No formal test suite yet. For Rust, add unit tests alongside modules (`#[cfg(test)]`) and run with `cargo test`.
- Frontend changes should be verified manually via `pnpm dev` (file pick/drop, successful download, console errors).
- Keep tests fast and deterministic; prefer small, pure functions for logic.

## Commit & Pull Request Guidelines

- Commits are short, imperative subjects (e.g., “Handle .prj (#17)”, “Refactor”). Use `[skip ci]` when appropriate.
- PRs: include a concise description, link related issues/PRs, screenshots of UI changes, and clear test steps.
- Keep changes scoped; separate refactors from behavior changes when possible.

## Security & Configuration Tips

- OPFS: the worker writes to the Origin Private File System; avoid persistent filenames and prefer temp-like names.
- Vite base: `vite.config.ts` sets `base: "/ksj2gp"` for GitHub Pages—keep in sync with deployment path.
- No secrets should live in this repo; environment configuration is not required for local development.
