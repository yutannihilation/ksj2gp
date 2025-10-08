use std::path::PathBuf;

use clap::Parser;
use ksj2gp::{Ksj2GpError, convert_shp_inner, encode_utf8_to_cp437cp932, list_shp_files_fs};

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
        &output_format,
    )
}

fn main() {
    let args = Args::parse();
    let target_shp = list_shp_files_fs(args.zip.clone()).unwrap();
    convert_shp_fs(args.zip, &target_shp[0], args.out).unwrap();
}
