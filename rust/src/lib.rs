use std::io::Write;

use geojson::JsonObject;
use geoparquet::writer::{GeoParquetRecordBatchEncoder, GeoParquetWriterOptionsBuilder};
use parquet::arrow::ArrowWriter;
use shapefile::{Reader, ShapeReader};
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;
use zip::ZipArchive;

use crate::{
    builder::construct_schema,
    crs::wild_guess_from_esri_wkt_to_projjson,
    encoding::guess_encoding,
    error::Ksj2GpError,
    io::{OpfsFile, UserLocalFile},
    writer::{write_geojson, write_geoparquet},
};

mod builder;
mod crs;
mod encoding;
mod error;
mod io;
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

#[wasm_bindgen]
pub fn convert_shp_to_geoparquet(
    zip_file: web_sys::File,
    target_shp: &str,
    intermediate_files: IntermediateFiles,
    output_file: web_sys::FileSystemSyncAccessHandle,
) -> Result<(), Ksj2GpError> {
    let reader = UserLocalFile::new(zip_file);
    let mut zip = reader.new_zip_reader(target_shp)?;

    let shp_file_opfs = zip.copy_shp_to_opfs(intermediate_files.shp)?;
    let dbf_file_opfs = zip.copy_dbf_to_opfs(intermediate_files.dbf)?;
    let shx_file_opfs = zip.copy_shx_to_opfs(intermediate_files.shx)?;

    let mut output_file_opfs = std::io::BufWriter::new(OpfsFile::new(output_file)?);

    let shapefile_reader = ShapeReader::with_shx(shp_file_opfs, shx_file_opfs)?;

    let wkt = zip.read_prj()?;

    let dbase_reader =
        shapefile::dbase::Reader::new_with_encoding(dbf_file_opfs, guess_encoding(target_shp))?;

    let dbf_fields = dbase_reader.fields().to_vec();

    let mut reader = Reader::new(shapefile_reader, dbase_reader);

    write_geoparquet(&mut reader, &mut output_file_opfs, &dbf_fields, &wkt)
}

#[wasm_bindgen]
pub fn convert_shp_to_geojson(
    zip_file: web_sys::File,
    target_shp: &str,
    intermediate_files: IntermediateFiles,
    output_file: web_sys::FileSystemSyncAccessHandle,
) -> Result<(), Ksj2GpError> {
    let reader = UserLocalFile::new(zip_file);
    let mut zip = reader.new_zip_reader(target_shp)?;

    let shp_file_opfs = zip.copy_shp_to_opfs(intermediate_files.shp)?;
    let dbf_file_opfs = zip.copy_dbf_to_opfs(intermediate_files.dbf)?;
    let shx_file_opfs = zip.copy_shx_to_opfs(intermediate_files.shx)?;

    let mut output_file_opfs = std::io::BufWriter::new(OpfsFile::new(output_file)?);

    let shapefile_reader = ShapeReader::with_shx(shp_file_opfs, shx_file_opfs)?;

    let wkt = zip.read_prj()?;

    let dbase_reader =
        shapefile::dbase::Reader::new_with_encoding(dbf_file_opfs, guess_encoding(target_shp))?;

    let dbf_fields = dbase_reader.fields().to_vec();

    let mut reader = Reader::new(shapefile_reader, dbase_reader);

    write_geojson(&mut reader, &mut output_file_opfs, &dbf_fields, &wkt)
}
