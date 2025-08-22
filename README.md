```sh
cd rust
cargo build
cd -

wasm-bindgen --out-dir npm --typescript --target bundler ./rust/target/wasm32-unknown-unknown/debug/ksj2gp.wasm
```

Note: needs `vite --force` to reflect the new binary

```sh
cd www
pnpm run dev --force
```

To optimize:

```sh
path/to/wasm-opt ./npm/ksj2gp_bg.wasm -O -o tmp.wasm
mv tmp.wasm ./npm/ksj2gp_bg.wasm
```
