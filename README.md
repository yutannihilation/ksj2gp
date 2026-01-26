# ksj2gp

＞＞＞ [ウェブサイト](https://yutannihilation.github.io/ksj2gp/) ＜＜＜

## 概要

ブラウザ上で動く、国土数値情報の Shapefile を GeoParquet / GeoJSON に変換するツールです。
フォーマットの変換だけでなく、国土数値情報のデータは属性名やデータの中身が`L001_123`のようなコードになっていますが、これを人間が読めるラベルに変換します。

まだ開発中ですが、ある程度は動く状態のはずです。もし使ってみてエラーやおかしな挙動を発見されましたら、お気軽に [issues](https://github.com/yutannihilation/ksj2gp/issues) からご連絡ください（日本語で大丈夫です）。

## 仕組み

技術的な話は [`docs/design.md`](./docs/design.md) にまとめています。

## 使用上の注意

- 一時データを作成するために [OPFS](https://developer.mozilla.org/ja/docs/Web/API/File_System_API/Origin_private_file_system) というブラウザのストレージ領域を利用します。
- GeoJSON の座標系は、以下のように扱います。
  - Tokyo Datum は WGS84 に座標変換
  - JGD2011・JGD2000 は無変換

## やりたいこと

### 出力

- [x] GeoParquet
- [x] GeoPackage
- [x] GeoJSON

## 入力

- 文字コード
  - [x] Shift_JIS
  - [x] UTF-8
- 座標系（緯度経度座標系のみ）
  - [x] JGD2011
  - [x] JGD2000
  - [x] 旧日本測地系
- [x] `.prj` ファイルがない場合

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
