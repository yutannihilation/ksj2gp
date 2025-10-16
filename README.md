# KSJ → GP

> [!WARNING]
> めっちゃ開発中です。

＞＞＞ [ウェブサイト](https://yutannihilation.github.io/ksj2gp/) ＜＜＜

## これは何？

ブラウザに国土数値情報の ZIP ファイルを投げつけると、いい感じに GeoParquet に変換するやつです。

実用性はあまりないですが（ふつうに DuckDB Wasm を使った方がいい）、pure Rust & WebAssembly でどこまでやれるかの実験のためです。

## やりたいこと

### 出力

- [x] GeoParquet
- [ ] GeoPackage
- [x] GeoJSON

## 入力

- 文字コード
  - [x] Shift_JIS
  - [x] UTF-8
- 座標系（緯度経度座標系のみ）
  - [x] JGD2011
  - [x] JGD2000
  - [ ] 旧日本測地系
- [ ] `.prj` ファイルがない場合

### 変換

※機能はあるけど、カバー率はまだまだです

- [x] 属性名を人間が読めるラベルにする
- [x] データの中身を人間が読めるラベルにする

## ビルド

```sh
cd rust
cargo build --target wasm32-unknown-unknown -p ksj2gp-web
cd -

wasm-bindgen --out-dir npm --typescript --target bundler ./rust/target/wasm32-unknown-unknown/debug/ksj2gp_web.wasm
```

Note: needs `vite --force` to reflect the new binary

```sh
pnpm run dev --force
```

To optimize:

```sh
path/to/wasm-opt ./npm/ksj2gp_web_bg.wasm -O -o tmp.wasm
mv tmp.wasm ./npm/ksj2gp_web_bg.wasm
```
