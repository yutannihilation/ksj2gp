use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
};

use shapefile::{Reader, ShapeReader};

use crate::{
    writer::{write_geojson, write_geoparquet},
    zip_reader::ZippedShapefileReader,
};

pub use crate::error::Ksj2GpError;
pub use encoding::decode_cp437cp932_to_utf8;

mod builder;
mod crs;
mod encoding;
mod error;
mod transform_coord;
mod writer;
mod zip_reader;

pub fn list_shp_files_fs(zip_file: PathBuf) -> Result<Vec<String>, Ksj2GpError> {
    let reader = std::io::BufReader::new(std::fs::File::open(zip_file)?);
    match zip::ZipArchive::new(reader) {
        Ok(zip) => {
            let shp_files = zip
                .file_names()
                .filter(|path| path.ends_with(".shp"))
                .map(decode_cp437cp932_to_utf8)
                .collect::<Result<_, _>>()?;
            Ok(shp_files)
        }
        Err(e) => Err(format!("Failed to read ZIP file!: {e:?}").into()),
    }
}

pub fn convert_shp_inner<RW: Read + Seek + Write, R: Read + Seek, W: Write + Send>(
    zip: R,
    target_shp: &str,
    shp: RW,
    dbf: RW,
    shx: RW,
    mut out: W,
    output_format: &str,
) -> Result<(), Ksj2GpError> {
    let mut zip = match zip::ZipArchive::new(zip) {
        Ok(zip) => ZippedShapefileReader::new(zip, target_shp),
        Err(e) => Err(format!("Failed to read ZIP file!: {e:?}").into()),
    }?;

    let shp_reader = zip.copy_shp_to(shp)?;
    let dbf_reader = zip.copy_dbf_to(dbf)?;
    let shx_reader = zip.copy_shx_to(shx)?;

    let shapefile_reader = ShapeReader::with_shx(shp_reader, shx_reader)?;

    let wkt = zip.read_prj()?;

    let dbase_reader =
        shapefile::dbase::Reader::new_with_encoding(dbf_reader, zip.guess_encoding()?)?;

    let dbf_fields = dbase_reader.fields().to_vec();

    let mut reader = Reader::new(shapefile_reader, dbase_reader);

    match output_format {
        "GeoParquet" => write_geoparquet(&mut reader, &mut out, &dbf_fields, &wkt),
        "GeoJson" => write_geojson(&mut reader, &mut out, &dbf_fields),
        _ => Err(format!("Unsupported format: {output_format}").into()),
    }
}
