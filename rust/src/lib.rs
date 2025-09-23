use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
};

use shapefile::{Reader, ShapeReader};
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;
use zip::ZipArchive;

use crate::{
    io::{OpfsFile, UserLocalFile},
    writer::{write_geojson, write_geoparquet},
    zip_reader::ZippedShapefileReader,
};

pub use crate::error::Ksj2GpError;

mod builder;
mod crs;
mod error;
mod io;
mod transform_coord;
mod writer;
mod zip_reader;

thread_local! {
    static FILE_READER_SYNC: FileReaderSync = FileReaderSync::new().unwrap();
}

// ShpReader requires Read and Seek, but files in a Zip archive cannot be Seek (only Read).
// So, these files are necessary for temporarily extracting the file.
#[wasm_bindgen]
pub struct IntermediateFiles {
    shp: web_sys::FileSystemSyncAccessHandle,
    dbf: web_sys::FileSystemSyncAccessHandle,
    shx: web_sys::FileSystemSyncAccessHandle,
}

#[wasm_bindgen]
impl IntermediateFiles {
    #[wasm_bindgen(constructor)]
    pub fn new(
        shp: web_sys::FileSystemSyncAccessHandle,
        dbf: web_sys::FileSystemSyncAccessHandle,
        shx: web_sys::FileSystemSyncAccessHandle,
    ) -> Self {
        Self { shp, dbf, shx }
    }
}

#[wasm_bindgen]
pub fn list_shp_files(zip_file: web_sys::File) -> Result<Vec<String>, Ksj2GpError> {
    let reader = UserLocalFile::new(zip_file);
    let zip = ZipArchive::new(reader)?;

    let shp_files: Vec<String> = zip
        .file_names()
        .filter(|path| path.ends_with(".shp"))
        .map(|path| path.to_string())
        .collect();

    Ok(shp_files)
}

pub fn list_shp_files_fs(zip_file: PathBuf) -> Result<Vec<String>, Ksj2GpError> {
    let reader = std::io::BufReader::new(std::fs::File::open(zip_file)?);
    match zip::ZipArchive::new(reader) {
        Ok(zip) => {
            let shp_files = zip
                .file_names()
                .filter(|path| path.ends_with(".shp"))
                .map(|path| path.to_string())
                .collect();
            Ok(shp_files)
        }
        Err(e) => Err(format!("Failed to read ZIP file!: {e:?}").into()),
    }
}

#[wasm_bindgen]
pub fn convert_shp(
    zip_file: web_sys::File,
    target_shp: &str,
    intermediate_files: IntermediateFiles,
    output_file: web_sys::FileSystemSyncAccessHandle,
    output_format: &str,
) -> Result<(), Ksj2GpError> {
    let zip = UserLocalFile::new(zip_file);
    let output_file_opfs = std::io::BufWriter::new(OpfsFile::new(output_file)?);

    convert_shp_inner(
        zip,
        target_shp,
        OpfsFile::new(intermediate_files.shp)?,
        OpfsFile::new(intermediate_files.dbf)?,
        OpfsFile::new(intermediate_files.shx)?,
        output_file_opfs,
        output_format,
    )
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
        "GeoJson" => write_geojson(&mut reader, &mut out, &dbf_fields, &wkt),
        _ => Err(format!("Unsupported format: {output_format}").into()),
    }
}
