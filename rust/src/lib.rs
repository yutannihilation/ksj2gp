use geoparquet::writer::{GeoParquetRecordBatchEncoder, GeoParquetWriterOptionsBuilder};
use itertools::Itertools;
use parquet::arrow::ArrowWriter;
use shapefile::{Reader, ShapeReader};
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;
use zip::ZipArchive;

use crate::{
    builder::construct_schema,
    crs::wild_guess_from_esri_wkt_to_projjson,
    encoding::guess_encoding,
    io::{OpfsFile, UserLocalFile},
};

mod builder;
mod crs;
mod encoding;
mod io;
mod zip_reader;

// Number of rows to process at once
const CHUNK_SIZE: usize = 2048;

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
pub fn list_shp_files(zip_file: web_sys::File) -> Result<Vec<String>, JsValue> {
    let reader = UserLocalFile::new(zip_file);
    let zip = ZipArchive::new(reader)
        .map_err(|e| -> JsValue { format!("Failed to inspect the ZIP file: {e:?}").into() })?;

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
) -> Result<(), JsValue> {
    let reader = UserLocalFile::new(zip_file);
    let mut zip = reader.new_zip_reader(target_shp)?;

    let mut shp_file_opfs = OpfsFile::new(intermediate_files.shp)?;
    zip.copy_shp_to(&mut shp_file_opfs)?;

    let mut shx_file_opfs = OpfsFile::new(intermediate_files.shx)?;
    zip.copy_shx_to(&mut shx_file_opfs)?;

    let mut dbf_file_opfs = OpfsFile::new(intermediate_files.dbf)?;
    zip.copy_dbf_to(&mut dbf_file_opfs)?;

    let output_file_opfs = OpfsFile::new(output_file)?;

    let shapefile_reader =
        ShapeReader::with_shx(shp_file_opfs, shx_file_opfs).map_err(|e| -> JsValue {
            format!("Got an error on reading .shp and .shx files: {e:?}").into()
        })?;

    let wkt = zip.read_prj()?;
    let projjson = wild_guess_from_esri_wkt_to_projjson(&wkt)?;
    let crs = geoarrow_schema::Crs::from_projjson(projjson);

    web_sys::console::log_1(&format!("CRS: {crs:?}").into());

    let dbase_reader =
        shapefile::dbase::Reader::new_with_encoding(dbf_file_opfs, guess_encoding(target_shp))
            .map_err(|e| -> JsValue {
                format!("Got an error on Reading a .dbf file: {e:?}").into()
            })?;

    let dbf_fields = dbase_reader.fields().to_vec();
    let fields_info = construct_schema(&dbf_fields, crs);
    let schema_ref = fields_info.schema_ref.clone();

    for f in &fields_info.non_geo_fields {
        web_sys::console::log_1(&format!("field: {f:?}").into());
    }
    web_sys::console::log_1(&format!("geometry: {:?}", &fields_info.geoarrow_type).into());

    let mut reader = Reader::new(shapefile_reader, dbase_reader);

    let options = GeoParquetWriterOptionsBuilder::default()
        .set_encoding(geoparquet::writer::GeoParquetWriterEncoding::WKB)
        .set_generate_covering(true)
        .build();
    let mut gpq_encoder =
        GeoParquetRecordBatchEncoder::try_new(&schema_ref, &options).map_err(|e| -> JsValue {
            format!("Got an error on creating GeoParquetRecordBatchEncoder: {e:?}").into()
        })?;

    let mut parquet_writer =
        ArrowWriter::try_new(output_file_opfs, gpq_encoder.target_schema(), None).map_err(
            |e| -> JsValue { format!("Got an error on creating ArrowWriter {e:?}").into() },
        )?;

    web_sys::console::log_1(&"writing geoparquet".into());

    // Since shapefile::Record is a HashMap, the iterator of it doesn't maintain
    // the order. So, this column names vector is needed to ensure the consistent
    // order with the schema.
    let (_last, fields_except_geometry) = schema_ref.fields().split_last().unwrap();
    let field_names: Vec<String> = fields_except_geometry
        .iter()
        .map(|f| f.name().to_string())
        .collect();

    for chunk in &reader.iter_shapes_and_records().chunks(CHUNK_SIZE) {
        let mut builders = fields_info.create_builders(CHUNK_SIZE);

        for result in chunk {
            let (shape, mut record) = result.unwrap();

            for (i, field_name) in field_names.iter().enumerate() {
                let value = record
                    .remove(field_name)
                    .ok_or_else(|| -> JsValue { format!("Not found {field_name}").into() })?;
                builders.builders[i].push(value);
            }

            let geometry = geo_types::Geometry::<f64>::try_from(shape).map_err(|e| -> JsValue {
                format!("Got an error on converting shape to geometry: {e:?}").into()
            })?;
            builders
                .geo_builder
                .push_geometry(Some(&geometry))
                .map_err(|e| -> JsValue {
                    format!("Got an error on pushing a geometry {geometry:?} to WkbBuilder: {e:?}")
                        .into()
                })?;
        }

        let batch = arrow_array::RecordBatch::try_new(schema_ref.clone(), builders.finish())
            .map_err(|e| -> JsValue {
                format!("Got an error on creating a RecordBatch: {e:?}").into()
            })?;
        let encoded_batch = gpq_encoder
            .encode_record_batch(&batch)
            .map_err(|e| -> JsValue { format!("Failed to encode_record_batch(): {e:?}").into() })?;

        parquet_writer
            .write(&encoded_batch)
            .map_err(|e| -> JsValue {
                format!("Failed to write() on parquet writer: {e:?}").into()
            })?;

        parquet_writer.flush().map_err(|e| -> JsValue {
            format!("Failed to flush() on parquet writer: {e:?}").into()
        })?;
    }

    web_sys::console::log_1(&"writing geoparquet metadata".into());

    let kv_metadata = gpq_encoder.into_keyvalue().unwrap();
    parquet_writer.append_key_value_metadata(kv_metadata);
    parquet_writer
        .finish()
        .map_err(|e| -> JsValue { format!("Failed to finish parquet_writer: {e:?}").into() })?;

    Ok(())
}
