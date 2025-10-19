ksj2gp を支える技術
=================

## 背景

かつて国土数値情報ダウンロードサービスにまだ API があった時代、私はその API を R から使うための [kokudosuuchi](https://github.com/yutannihilation/kokudosuuchi/) というパッケージを作っていました。
そのパッケージの機能のひとつとして

- カラム名
- コードリスト型の列のコード

を、人間が読めるラベルに変換する関数がありました。
このためにつくった変換データを使って Rust 版が作れないか？というのが ksj2gp を考え始めたきっかけでした。

そこから、せっかく Rust を使うならブラウザで動くものにしてみよう、ということで今のかたちになっています。

## Rust コードの NPM 化

これは細かい話ですが、このレポジトリは wasm-pack を使っていません。
この理由は、ksj2gp の開発を始めたのがちょうど rustwasm が消滅する時期で、wasm-pack の継続性が怪しかったためです。
今はいちおう引き取り手が見つかっている状態ですが、念の為に wasm-pack を使わなくても NPM 化する方法を模索しておこう、ということで今の形になっています。
さいわい、やり方はそんなに難しくありません。Zenn に記事を書いたので詳細はこちらをご参照ください。

https://zenn.dev/yutannihilation/articles/5c795a302f2c51

## WebAssembly でのファイルの読み書き

ファイルシステムへのアクセスというのは WebAssembly の規格外で、ランタイムによって提供されるものです。
これも Zenn に記事を書いたので詳細はこちらをご参照ください。

https://zenn.dev/yutannihilation/articles/afdc292ebf9dbe

https://zenn.dev/yutannihilation/articles/95303dddeb8044

### 読む

ファイルを読むのは、