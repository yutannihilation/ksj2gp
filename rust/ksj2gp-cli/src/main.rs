use std::path::PathBuf;

use clap::Parser;
use ksj2gp::{
    Ksj2GpError, TranslateOptions, convert_shp_inner, encode_utf8_to_cp437cp932, extract_ksj_id,
    list_shp_files,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to ZIP file
    zip: PathBuf,

    /// Path to output file
    out: PathBuf,
}

pub fn convert_shp_fs(
    zip_file: PathBuf,
    target_shp: &str,
    output_file: PathBuf,
) -> Result<(), Ksj2GpError> {
    let filename = zip_file.file_name().unwrap().to_string_lossy().to_string();
    let (ksj_id, year) = extract_ksj_id(&filename)?;

    let zip = std::io::BufReader::new(std::fs::File::open(zip_file)?);

    let tmp_shp_file_path = tempfile::NamedTempFile::with_suffix(".shp")?;
    let tmp_dbf_file_path = tempfile::NamedTempFile::with_suffix(".dbf")?;
    let tmp_shx_file_path = tempfile::NamedTempFile::with_suffix(".shx")?;

    let output_format = match output_file.extension() {
        Some(ext) => match ext.to_string_lossy().as_ref() {
            "geojson" => "GeoJson",
            "parquet" => "GeoParquet",
            e => return Err(format!("Unsupported extension: {e}").into()),
        },
        None => return Err(format!("Unsupported format: {}", output_file.display()).into()),
    };

    let output_file = std::io::BufWriter::new(std::fs::File::create(&output_file)?);

    convert_shp_inner(
        zip,
        &encode_utf8_to_cp437cp932(target_shp)?,
        tmp_shp_file_path,
        tmp_dbf_file_path,
        tmp_shx_file_path,
        output_file,
        output_format,
        &TranslateOptions {
            // TODO: pass this option from outside
            translate_colnames: true,
            translate_contents: true,
            ignore_translation_errors: false,
            ksj_id,
            year,
        },
    )
}

fn main() {
    let args = Args::parse();

    let file = std::fs::File::open(args.zip.clone()).unwrap();
    let buf_reader = std::io::BufReader::new(file);
    let target_shp = list_shp_files(buf_reader).unwrap();

    convert_shp_fs(args.zip, &target_shp[0], args.out).unwrap();
}
