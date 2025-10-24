# ksj2gp を支える技術

## 背景

かつて国土数値情報ダウンロードサービスにまだ API があった時代、私はその API を R から使うための [kokudosuuchi](https://github.com/yutannihilation/kokudosuuchi/) というパッケージを作っていました。
そのパッケージの機能のひとつとして

- カラム名
- コードリスト型の列のコード

を、人間が読めるラベルに変換する関数がありました。
このためにつくった変換データを使って Rust 版が作れないか？というのが ksj2gp を考え始めたきっかけでした。

そこから、せっかく Rust を使うならブラウザで動くものにしてみよう、ということで今のかたちになっています。

## 入出力

ksj2gp は GDAL を使っていません。入力も出力も Rust crate でやっています。それぞれ以下の crate を使っています。

- 入力
  - [shapefile crate](https://crates.io/crates/shapefile)
  - [dbase crate](https://crates.io/crates/dbase)
- 出力
  - [geoarrow-geojson crate](https://crates.io/crates/geoarrow-geojson)
  - [geoparquet crate](https://crates.io/crates/geoparquet)

GeoPackage の出力もサポートしたいんですが、今のところ使える crate がなさそうなので断念しています。
[Turso](https://github.com/tursodatabase/turso) で SQLite 拡張が動くようになれば（GeoPackage は rtree 拡張が必要）やってみたいなと思っています。

ちなみに、こうした GIS データ形式の変換としては [geozero crate](https://crates.io/crates/geozero) が有名だと思いますが、なぜ使わなかったのかは...、忘れました。
たしか、文字コードの対応とかで geozero の API よりも細かい部分を操作する必要があったのと、単純に使い方わからなかったとかだった気がします。

## 文字コード

`.dbf` ファイルの文字コードの推定方法はいくつかあります。
まず、以下が ESRI Japan による説明です。

https://github.com/EsriJapan/shapefile_info

ksj2gp では、これに従ってまずは以下で文字コードを決められるか試します。

- `.dbf` ファイルの 29 バイト目が `13` の場合は Shift_JIS
- `.cpg` ファイルが存在する場合
  - 中身が `CP932` なら Shift_JIS
  - 中身が `UTF-8` なら UTF-8

これでも決まらない場合、以下を試します。最近のデータだと UTF-8 と Shift_JIS が両方提供されていて、
それぞれ `UTF-8`、`Shift_JIS` というフォルダに入っていたりするので、そこから推測を試みています。

- Shapefile のパスに `UTF-8` が含まれていれば UTF-8

以上の3つのどれにも当てはまらない場合は、`Shift_JIS` と推測します。

## CRS、座標変換

国土数値情報のデータは、`.prj` ファイルがあったりなかったりします。
ksj2gp では、

- `.prj` ファイルがある場合、そこから CRS を推測する
- ない場合、`KS-META-*.xml` ファイルから CRS を推測する

という作戦でやっています。

### 座標変換

話の順序で座標変換をどうやっているか（あるいはやっていないのか）について先に話します。

まず、これがブラウザ上でなければ、信頼と安心の proj を使って座標変換をすればいいところですが、proj-sys crate は WebAssembly では使えません。

https://github.com/georust/proj/issues/115

proj に頼れないとなると、私の知識では正しい座標変換を実装できるか自信がないので、なるべく座標変換を避けるようにしています。
具体的にはこうです。

- サポートするのは JGD2011、JGD2000、Tokyo の地理座標系（EPSG 6668, 4326、4301）のみ
- GeoParquet では座標変換はせず、CRS を設定するだけ
- GeoJSON の場合のみ、WGS84 への変換を行う

1点目は、他の CRS のデータもサポートできればうれしい（国土数値情報以外のデータにも簡易的に使えるかも）、という気持ちはあるのですが、ひとまず国土数値情報でよくあるデータだけを念頭に考えることにしました。

WGS84 への変換は、以下の方針で行います。

- JGD2011、JGD2000 は変換せずそのままの座標を使う
- Tokyo は [proj4rs crate](https://github.com/3liz/proj4rs) を使って変換する

JGD2000・JGD2011 から　WGS84 への変換が無変換でいいのかについては、ちゃんと理解できていないのですが、いろいろな考え方があるようで悩みました。
が、まあ、そういうのをちゃんとやりたい人はこんなブラウザ上で動く謎のツールは使わないだろう、ということで深く悩まない事にしました！

proj4rs crate は、[proj4js](https://github.com/proj4js/proj4js) と同等の機能を pure Rust で提供することを目指して開発されているライブラリです。
pure Rust なので WebAssembly で動作します。

コードはこんな感じです。proj4rs は、地理座標系の場合は単位が度ではなくラジアンなので、`to_radians()` で変換してから渡して、また `to_degrees()` で戻す必要がある点に注意してください。

```rs
impl CoordTransformer {
    fn transform_single_point(
        &self,
        point: &shapefile::Point,
    ) -> Result<geojson::Position, Ksj2GpError> {
        match self.src {
            JapanCrs::Tokyo => {
                let mut pt = (point.x.to_radians(), point.y.to_radians());
                proj4rs::transform::transform(&PROJ4STRING_TOKYO, &PROJ4STRING_WGS84, &mut pt)?;
                Ok(vec![pt.0.to_degrees(), pt.1.to_degrees()])
            }
            // JGD2000, JGD2011 から WGS84 は無変換とする
            JapanCrs::JGD2000 | JapanCrs::JGD2011 => Ok(vec![point.x, point.y]),
        }
    }
}
```

さて、このコードに出てくる変数名を見れば気付くかと思いますが、CRS は proj4 string の形で渡す必要があります。
最近のは WKT や projjson 形式になっているので、ここをどうするか、というのが次の話になります。

### CRS の推測

#### `.prj` ファイルがある場合

`.prj` ファイルがある場合はこれを使えばいいんですが、中に入っているのは proj4 string ではなく WKT です。
ひとことに「WKT」と言っても色々あり（well-known とはいったい...）、ESRI WKT という形式になっています。

proj4js は WKT を proj4 string に変換する [proj4wkt crate](https://github.com/3liz/proj4wkt-rs) を提供しているんですが、これが対応しているのは OGR WKT1　と OGR WKT2 です。
互換性についてよくわからないのですが、ESRI 版の WKT が読めるか不安だったので、これは使わないことにしました。

というのは、対応する　CRS は3つだけに絞ったので、まあ使わなくてもなんとかなるだろう、という読みもあります。具体的にはこうです。
`JapanCrs` は自分で定義している `enum` で、これを使って、上に挙げたコードの中で適切な proj4 string を選ぶようにしています。

```rs
pub fn guess_crs_from_esri_wkt(wkt: &str) -> Result<JapanCrs, Ksj2GpError> {
    if wkt.contains("GCS_JGD_2011") {
        return Ok(JapanCrs::JGD2011);
    }

    if wkt.contains("GCS_JGD_2000") {
        return Ok(JapanCrs::JGD2000);
    }

    if wkt.contains("GCS_Tokyo") {
        return Ok(JapanCrs::Tokyo);
    }

    Err(format!("Failed to identify CRS from ESRI WKT in the .prj file: {wkt}").into())
}
```

#### `.prj` ファイルがない場合

`.prj` ファイルがない場合は `KS-META-*.xml` を使います。
これを読むためだけに XML パーサーを入れるのが嫌だったので、正規表現でがんばります。

JMP 2.0 の解説書（<https://www.gsi.go.jp/common/000259951.pdf>）の 5.1.2 によると、CRS は `referenceSystemIdentifier` に指定されていて、
以下のフォーマットになっているらしいです。

```
[原子]＋[半角スペース]＋[半角スラッシュ（"/"）]＋[半角スペース]＋[座標]
```

「原子」には測地原子（Geodetic Datum）か鉛直原子（Vertical Datum）が入り、国土数値情報の場合はここに「JGD2011」「JGD2000」「TD」が入っています。
具体的にはこうです。

```xml
<referenceSystemInfo>
    <MD_ReferenceSystem>
        <referenceSystemIdentifier>
            <code>JGD2011 / (B, L)</code>
        </referenceSystemIdentifier>
    </MD_ReferenceSystem>
</referenceSystemInfo>
```

ちなみに、CRS が複数ある場合はカンマ区切りで指定できるみたいですが、ksj2gp はそこまでは対応していません。

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
これはブラウザが提供するファイルシステム API です。以下のようなコードで、指定した名前のファイルのハンドルを得ることができます。

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

ちなみにこの非同期の API は `wasm-bindgen-futures` を使うと Rust コードからも呼ぶことができます。
ただ、非同期の API から得られる値は `JsValue` になっていて、型の情報が落ちてしまっているのでちょっと書きづらいです。
迷ったんですが、ここは TypeScript 側で書いた方がきれいかなあ、ということで TypeScript 側でやることにしました。正解はまだよくわかりません。

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

```rs
let shapefile_reader = ShapeReader::with_shx(shp_reader, shx_reader)?;

let crs = zip.guess_crs()?;

let dbase_reader =
    shapefile::dbase::Reader::new_with_encoding(dbf_reader, zip.guess_encoding()?)?;
```

### 4. 出力（GeoParquet、GeoJSON）をファイルを書き出す

writer が `Write` + `Seek` を実装したものであれば対応しているので、先ほどと同じく `OpfsFile` を渡すだけです。

ここも、一時ファイルを使わずにメモリ上に出力するという手もありますが、（特に GeoJSON の場合は）サイズがそこそこ大きいのでいったんファイルに書き出すのが効率的だと判断しました。
ただ、OPFS は OPFS で[ブラウザのストレージ容量制限](https://developer.mozilla.org/ja/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria)があるので、万能ではありません。環境によっては制限に引っ掛かるかもしれません。

（ちなみに、OPFS にすでにファイルはあるのに、ブラウザの仕組み上、ユーザーはそれを改めてダウンロードしないといけないというのはもどかしい点です。これが CLI 等であれば直接ファイルを書けるのに、このやり方では二重に容量を食うことになります）

### 5. 出力ファイルをユーザーにダウンロードさせる

書き出したら、web worker からは OPFS ファイルのハンドルと、そのファイル名をメインスレッドに返します。

```ts
convert_shp(
  ...
);
const filename = getOutputFilename(targetShp, outputFormat);
postTypedMessage({ output: { handle: outputFileHandle, filename } });
```

メインスレッドでは、ファイルハンドルを受け取ったら、[`URL.createObjectURL()`](https://developer.mozilla.org/ja/docs/Web/API/URL/createObjectURL_static) でファイルをダウンロードできるリンクを作成します。

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
