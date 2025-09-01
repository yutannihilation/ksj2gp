# KSJ → GP

> [!WARNING]
> めっちゃ開発中です。

＞＞＞ [ウェブサイト](https://yutannihilation.github.io/ksj2gp/) ＜＜＜

## これは何？

ブラウザに国土数値情報の ZIP ファイルを投げつけると、いい感じに GeoParquet に変換するやつです。

実用性はあまりないですが（ふつうに DuckDB Wasm を使った方がいい）、pure Rust & WebAssembly でどこまでやれるかの実験のためです。

## やりたいこと

- [x] GeoParquet ファイルの出力
- [x] `.prj` ファイルを扱う
  - [x] GeoParquet の `crs` メタデータに入れる
  - [ ] ブラウザ上で座標変換までやってしまう
- [ ] Shift_JIS 以外の文字コード
- [ ] 複数の `.shp` ファイルを含む場合に選べるようにする
- [ ] メタデータをいい感じに紐づける（[過去のデータ](https://github.com/yutannihilation/kokudosuuchi-metadata)）
- [ ] GeoParquet v1.1 を選べるようにする
- [ ] GeoPackage ファイルの出力（turso 待ち？）

## ビルド

```sh
cd rust
cargo build
cd -

wasm-bindgen --out-dir npm --typescript --target bundler ./rust/target/wasm32-unknown-unknown/debug/ksj2gp.wasm
```

Note: needs `vite --force` to reflect the new binary

```sh
pnpm run dev --force
```

To optimize:

```sh
path/to/wasm-opt ./npm/ksj2gp_bg.wasm -O -o tmp.wasm
mv tmp.wasm ./npm/ksj2gp_bg.wasm
```
