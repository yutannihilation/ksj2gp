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
さいわい、やり方はそんなに難しくありません。詳しくは、Zenn に記事を書いたのでこちらをご参照ください。

https://zenn.dev/yutannihilation/articles/5c795a302f2c51

## WebAssembly でのファイルの読み書き

ファイルシステムへのアクセスというのは WebAssembly の規格外で、ランタイムによって提供されるものです。
これも Zenn に記事を書いたので詳細はこちらをご参照ください。

https://zenn.dev/yutannihilation/articles/afdc292ebf9dbe

https://zenn.dev/yutannihilation/articles/95303dddeb8044

ksj2gp で発生するファイルの読み書きは以下のステップがあります。

1. ZIP ファイルを読む
2. ZIP ファイルの中の Shapefile などを一時ファイルに書き出す
3. 一時ファイルを読む
4. 出力（GeoParquet、GeoJSON）をファイルを書き出す
5. 出力ファイルをユーザーにダウンロードさせる

順にみていきましょう。

### 1. ZIP ファイルを読む

ユーザーから指定されたファイルを読むのは、web worker 上で [`FileReaderSync`](https://developer.mozilla.org/ja/docs/Web/API/FileReaderSync) を使って**同期的に**（重要）行っています。
以下の `struct` を定義して、`Read` と `Seek` を実装しています。
ちなみに、これだけなら [wasm-bindgen-file-reader crate](https://crates.io/crates/wasm-bindgen-file-reader) を使えばよかったんですが、
次のステップで OPFS を読み書きするために似たような実装が必要だったので、これもついでに自分で実装しました。

```rs
pub struct UserLocalFile {
    file: web_sys::File,
    offset: u64,
}
```

これを `zip::ZipArchive::new()` に渡すことで、ZIP ファイルの中身が読めるようになります。

```rs
let mut zip = match zip::ZipArchive::new(user_local_file) {
    Ok(zip) => ...,
    Err(e) => Err(format!("Failed to read ZIP file!: {e:?}").into()),
}?;
```

ひとつ工夫が必要な点は、zip crate が Shift_JIS のデコードをサポートしていないことです。
そのため、ZIP ファイル内のファイル名（ファイルの中身は zip crate がデコードするわけではないので問題ない）をデコードするには、自前で Shift_JIS をデコードする必要があります。
どうやるかというと、zip crate は　UTF-8 ではない文字列は [CP437](https://ja.wikipedia.org/wiki/Code_page_437) として扱います。

https://github.com/zip-rs/zip2/blob/f628624e58764d7f151e674190ff8c4e8786760d/src/read.rs#L1370-L1373

これは文字化けっぽく見えるのですが、通常の（？）文字化けと違って元の情報は失われていません。CP437 は256通りすべての8ビットコードに対応する文字があるためです。
なので、これをバイト列に戻して、それを encoding_rs crate で Shift_JIS としてデコードすればいいわけですが...、ここで問題がひとつあります。
encoding_rs　は CP437 をサポートしていません。

https://github.com/hsivonen/encoding_rs/issues/51

これは、encoding_rs のスコープが [Encoding Standard](https://encoding.spec.whatwg.org/) であるためなので、まあ仕方ないことなのですが、そんなわけで encoding_rs には頼れないので自分で何とかする必要があります。さいわい、256個の `u8` と文字列の対応を書くだけなので、AI に関数を書かせて何とかなりました（コードは [src/encoding.rs](https://github.com/yutannihilation/ksj2gp/rust/src/encoding.rs) にあります）。

### 2. ZIP ファイルの中の Shapefile などを一時ファイルに書き出す

次は、ZIP ファイルから Shapefile などの必要なデータを展開して、それを shapefile crate に渡すわけですが、方法は3つあります。

a. メモリ上にデータを持つ
b. 一時ファイルに書き出す
c. ZIPファイルのまま読む

このうち、c.はうまくいきませんでした。zip crate が返す `ZipFile` は、`Read` はあるんですが `Seek` がなかったためです（暗号化とか圧縮とか考えると、任意の位置に seek できないのはまあそういうものなのかも）。

a. は、国土数値情報のデータは、重くても数百MB程度なのでまあメモリ上に乗るんじゃないかな...？という気はします。
ただ、他の処理でどれくらいメモリを使うかわからないので、今回は b. の一時ファイルに書き出す手にしようと思いました。

ブラウザ上で一時ファイルを作るにはどうするかというと、[OPFS (Origin Private File System)](https://developer.mozilla.org/ja/docs/Web/API/File_System_API/Origin_private_file_system) を使います。
これはブラウザが提供するファイルシステム API で、以下のようなコードでしてした名前のファイルのハンドルを得ることができます。

```ts
async function newSyncAccessHandle(
	opfsRoot: FileSystemDirectoryHandle,
	filename: string
): Promise<FileSystemSyncAccessHandle> {
	const fileHandle = await opfsRoot.getFileHandle(filename, {
		create: true
	});

	return await fileHandle.createSyncAccessHandle();
}
```

ただ、`FileReaderSync` を使う関係上、ウェブワーカーで動かすコードは同期実行なので、そこからこの非同期の関数を呼び出すことはできません
（WebAssembly では同期実行と非同期実行を混ぜる方法がない、というのが私の理解なのですが、あまりよくわかっていないので間違ってたらご指摘ください！）。
そこで、WebAssembly のコードの外側で必要なファイルハンドルをつくって、それを WebAssembly に渡す、ということをしています。

```ts
// 出力用のファイル（ファイルハンドルをあとで使うので newSyncAccessHandle() ではなくて直接やっている）
const outputFileHandle = await opfsRoot.getFileHandle('tmp_output', {
	create: true
});
const outputFile = await outputFileHandle.createSyncAccessHandle();

//　Zipファイルの中身を展開する用のファイル
const shp = await newSyncAccessHandle(opfsRoot, 'tmp.shp');
const dbf = await newSyncAccessHandle(opfsRoot, 'tmp.dbf');
const shx = await newSyncAccessHandle(opfsRoot, 'tmp.shx');

//　IntermediateFiles は Zipファイルの中身を展開する用のファイルをまとめた構造体
const intermediateFiles = new IntermediateFiles(shp, dbf, shx);

try {
	convert_shp(
	  ...
		intermediateFiles,
		outputFile,
    ...
	);
```

（「あれ、`.prj` は...？」と思った方は鋭いです。`.prj`、`.cpg`、あと `KS-META-*.xml` はそんなに重くないので、一時ファイルをつくらずメモリ上に持つように使い分けています）

これを受け取って、Rust 側では `Read` + `Seek` + `Write` を実装した struct を作ります。

```rs
pub struct OpfsFile {
    file: web_sys::FileSystemSyncAccessHandle,
    offset: FileSystemReadWriteOptions,
}
```

この `OpfsFile` に対して、`std::io::copy()` で `ZipFile` の中身を書き出したらこのステップは終了です。

### 3. 一時ファイルを読む

Shepefile を読むのは、

- [shapefile crate](https://docs.rs/shapefile/latest/shapefile/)
- [dbase crate](https://docs.rs/dbase/latest/dbase/)

を使います。これらの reader は `Read` + `Seek` を実装したものであれば対応しているので、ここに先ほどの `OpfsFile` を渡してあげれば OK です。
CRS についてはあとで説明するのでここでは省略します。

```rs
let shapefile_reader = ShapeReader::with_shx(shp_reader, shx_reader)?;

let crs = zip.guess_crs()?;

let dbase_reader =
    shapefile::dbase::Reader::new_with_encoding(dbf_reader, zip.guess_encoding()?)?;
```

### 4. 出力（GeoParquet、GeoJSON）をファイルを書き出す

ここも、writer が `Write` + `Seek` を実装したものであれば対応しているので、先ほどの `OpfsFile` を渡すだけです。

### 5. 出力ファイルをユーザーにダウンロードさせる

書き出したら、web worker からは OPFS ファイルのハンドルと、そのファイル名をメインスレッドに返します。

```ts
convert_shp(
  ...
);
const filename = getOutputFilename(targetShp, outputFormat);
postTypedMessage({ output: { handle: outputFileHandle, filename } });
```

メインスレッドでは、ファイルハンドルを受け取ったら、`URL.createObjectURL()`(https://developer.mozilla.org/ja/docs/Web/API/URL/createObjectURL_static) でファイルをダウンロードできるリンクを作成します。

これを仮の `a` タグに設定してそれをクリックし、ダウンロードを発生させる、というかたちでファイルダウンロードを実現しています。
このあたりは AI 頼みで書いたコードなので、正直ちゃんと理解できていません。。

```ts
worker.onmessage = async (event: MessageEvent<WorkerResponse>) => {
	const data = event.data;

  ...

	const file = await data.output.handle.getFile();
	const url = URL.createObjectURL(file);

	const a = document.createElement('a');
	a.href = url;
	a.download = data.output.filename;
	document.body.appendChild(a);
	a.click();

	setTimeout(() => {
		URL.revokeObjectURL(url);
		a.remove();
		finish();
	}, 600);
};
```
