use dbase::{FieldInfo, FieldType, FieldValue, encoding::EncodingRs};
use geoarrow_array::{GeoArrowArray, builder::WkbBuilder};
use geoarrow_schema::{GeoArrowType, WkbType};
use geoparquet::writer::{GeoParquetRecordBatchEncoder, GeoParquetWriterOptionsBuilder};
use itertools::Itertools;
use parquet::arrow::ArrowWriter;
use shapefile::{Reader, ShapeReader};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;

use crate::io::{OpfsFile, UserLocalFile};

mod io;
mod zip;

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
    // prj: web_sys::FileSystemSyncAccessHandle, // TODO
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
pub fn list_files(
    zip_file: web_sys::File,
    intermediate_files: IntermediateFiles,
    output_file: web_sys::FileSystemSyncAccessHandle,
) -> Result<(), JsValue> {
    let reader = UserLocalFile::new(zip_file);
    let mut zip = reader.new_zip_reader()?;

    let mut shp_file_opfs = OpfsFile::new(intermediate_files.shp)?;
    zip.copy_shp_to(&mut shp_file_opfs)?;

    let mut shx_file_opfs = OpfsFile::new(intermediate_files.shx)?;
    zip.copy_shx_to(&mut shx_file_opfs)?;

    let mut dbf_file_opfs = OpfsFile::new(intermediate_files.dbf)?;
    zip.copy_dbf_to(&mut dbf_file_opfs)?;

    let output_file_opfs = OpfsFile::new(output_file)?;

    let shapefile_reader =
        ShapeReader::with_shx(shp_file_opfs, shx_file_opfs).map_err(|e| -> JsValue {
            format!("Got an error on Reading .shp and .shx files: {e:?}").into()
        })?;

    let dbase_reader = shapefile::dbase::Reader::new_with_encoding(
        dbf_file_opfs,
        EncodingRs::from(encoding_rs::SHIFT_JIS), // TODO: read UTF-8
    )
    .map_err(|e| -> JsValue { format!("Got an error on Reading a .dbf file: {e:?}").into() })?;

    let dbf_fields = dbase_reader.fields().to_vec();
    let fields_info = infer_schema(&dbf_fields);
    let schema_ref = fields_info.schema_ref.clone();

    for f in &fields_info.non_geo_fields {
        web_sys::console::log_1(&format!("field: {f:?}").into());
    }
    web_sys::console::log_1(&format!("geometry: {:?}", &fields_info.geo_arrow_type).into());

    let mut reader = Reader::new(shapefile_reader, dbase_reader);

    let options = GeoParquetWriterOptionsBuilder::default()
        // .set_crs_transform(Box::new(todo!())) // TODO
        .set_encoding(geoparquet::writer::GeoParquetWriterEncoding::WKB)
        // .set_column_encoding(
        //     "geometry".to_string(),
        //     geoparquet::writer::GeoParquetWriterEncoding::GeoArrow,
        // )
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

    const CHUNK_SIZE: usize = 2048;
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

struct FieldsWithGeo {
    schema_ref: arrow_schema::SchemaRef,
    non_geo_fields: Vec<Arc<arrow_schema::Field>>,
    geo_arrow_type: geoarrow_schema::GeoArrowType,
}

#[derive(Debug)]
enum NonGeoArrayBuilder {
    // PrimitiveArray
    Float64(arrow_array::builder::Float64Builder),
    Float32(arrow_array::builder::Float32Builder),
    Int32(arrow_array::builder::Int32Builder),
    Boolean(arrow_array::builder::BooleanBuilder),
    // Non-primitives
    Utf8(arrow_array::builder::StringBuilder),
    Date32(arrow_array::builder::Date32Builder),
    // TODO
    // Timestamp(arrow_array::builder::TimestampMillisecondBuilder)
}

impl NonGeoArrayBuilder {
    fn push(&mut self, value: FieldValue) {
        // web_sys::console::log_1(&format!("pushing field: {value:?} to {self:?}").into());

        match (self, value) {
            (NonGeoArrayBuilder::Float64(primitive_builder), FieldValue::Numeric(v)) => {
                if let Some(v) = v {
                    primitive_builder.append_value(v);
                } else {
                    primitive_builder.append_null();
                }
            }
            (
                NonGeoArrayBuilder::Float64(primitive_builder),
                FieldValue::Double(v) | FieldValue::Currency(v),
            ) => {
                primitive_builder.append_value(v);
            }

            (NonGeoArrayBuilder::Float32(primitive_builder), FieldValue::Float(v)) => {
                if let Some(v) = v {
                    primitive_builder.append_value(v);
                } else {
                    primitive_builder.append_null();
                }
            }
            (NonGeoArrayBuilder::Int32(primitive_builder), FieldValue::Integer(v)) => {
                primitive_builder.append_value(v);
            }
            (NonGeoArrayBuilder::Boolean(boolean_builder), FieldValue::Logical(v)) => {
                if let Some(v) = v {
                    boolean_builder.append_value(v);
                } else {
                    boolean_builder.append_null();
                }
            }
            (NonGeoArrayBuilder::Utf8(generic_byte_builder), FieldValue::Character(v)) => {
                if let Some(v) = v {
                    generic_byte_builder.append_value(v);
                } else {
                    generic_byte_builder.append_null();
                }
            }
            (NonGeoArrayBuilder::Utf8(generic_byte_builder), FieldValue::Memo(v)) => {
                generic_byte_builder.append_value(v);
            }
            (NonGeoArrayBuilder::Date32(primitive_builder), FieldValue::Date(v)) => {
                if let Some(v) = v {
                    primitive_builder.append_value(v.to_unix_days());
                } else {
                    primitive_builder.append_null();
                }
            }
            // type mismatch means something is wrong...
            (_, _) => unreachable!(),
        }
    }

    fn finish(&mut self) -> arrow_array::ArrayRef {
        // Note: primitive_builder implements its own finish() that returns
        // PrimitiveArray. So, in order to make it return ArrayRef, it needs
        // to be the qualified form...
        match self {
            NonGeoArrayBuilder::Float64(primitive_builder) => {
                arrow_array::builder::ArrayBuilder::finish(primitive_builder)
            }
            NonGeoArrayBuilder::Float32(primitive_builder) => {
                arrow_array::builder::ArrayBuilder::finish(primitive_builder)
            }
            NonGeoArrayBuilder::Int32(primitive_builder) => {
                arrow_array::builder::ArrayBuilder::finish(primitive_builder)
            }
            NonGeoArrayBuilder::Boolean(primitive_builder) => {
                arrow_array::builder::ArrayBuilder::finish(primitive_builder)
            }
            NonGeoArrayBuilder::Utf8(primitive_builder) => {
                arrow_array::builder::ArrayBuilder::finish(primitive_builder)
            }
            NonGeoArrayBuilder::Date32(primitive_builder) => {
                arrow_array::builder::ArrayBuilder::finish(primitive_builder)
            }
        }
    }
}

struct ArrayBuilderWithGeo {
    builders: Vec<NonGeoArrayBuilder>,
    geo_builder: WkbBuilder<i32>,
}

impl FieldsWithGeo {
    fn create_builders(&self, capacity: usize) -> ArrayBuilderWithGeo {
        web_sys::console::log_1(&"creating builders".into());

        let builders: Vec<NonGeoArrayBuilder> = self
            .non_geo_fields
            .iter()
            .map(|f| match f.data_type() {
                arrow_schema::DataType::Float64 => NonGeoArrayBuilder::Float64(
                    arrow_array::builder::Float64Builder::with_capacity(capacity),
                ),
                arrow_schema::DataType::Float32 => NonGeoArrayBuilder::Float32(
                    arrow_array::builder::Float32Builder::with_capacity(capacity),
                ),
                arrow_schema::DataType::Int32 => NonGeoArrayBuilder::Int32(
                    arrow_array::builder::Int32Builder::with_capacity(capacity),
                ),

                arrow_schema::DataType::Boolean => NonGeoArrayBuilder::Boolean(
                    arrow_array::builder::BooleanBuilder::with_capacity(capacity),
                ),
                arrow_schema::DataType::Utf8 => NonGeoArrayBuilder::Utf8(
                    // TODO: not sure what's the best value to multiply for data_capacity
                    arrow_array::builder::StringBuilder::with_capacity(capacity, capacity * 8),
                ),
                arrow_schema::DataType::Date32 => NonGeoArrayBuilder::Date32(
                    arrow_array::builder::Date32Builder::with_capacity(capacity),
                ),
                // arrow_schema::DataType::Timestamp(time_unit, _) => todo!(),
                _ => unreachable!(),
            })
            .collect();

        web_sys::console::log_1(&"created builders".into());

        let geo_builder = WkbBuilder::new(WkbType::new(Default::default()));

        web_sys::console::log_1(&"created geo_builders".into());

        ArrayBuilderWithGeo {
            builders,
            geo_builder,
        }
    }
}

impl ArrayBuilderWithGeo {
    fn finish(mut self) -> Vec<arrow_array::ArrayRef> {
        let mut result: Vec<_> = self.builders.iter_mut().map(|b| b.finish()).collect();
        result.push(self.geo_builder.finish().into_array_ref());
        result
    }
}

// This function is derived from geoarrow-rs's old code, which is licensed under MIT/Apache
//
// https://github.com/geoarrow/geoarrow-rs/blob/06e1d615134b249eb5fee39020673c8659978d18/rust/geoarrow-old/src/io/shapefile/reader.rs#L385-L411
fn infer_schema(fields: &[FieldInfo]) -> FieldsWithGeo {
    let mut non_geo_fields = Vec::with_capacity(fields.len());

    for field in fields {
        let name = field.name().to_string();
        let field = match field.field_type() {
            FieldType::Numeric | FieldType::Double | FieldType::Currency => {
                arrow_schema::Field::new(name, arrow_schema::DataType::Float64, true)
            }
            FieldType::Character | FieldType::Memo => {
                arrow_schema::Field::new(name, arrow_schema::DataType::Utf8, true)
            }
            FieldType::Float => {
                arrow_schema::Field::new(name, arrow_schema::DataType::Float32, true)
            }
            FieldType::Integer => {
                arrow_schema::Field::new(name, arrow_schema::DataType::Int32, true)
            }
            FieldType::Logical => {
                arrow_schema::Field::new(name, arrow_schema::DataType::Boolean, true)
            }
            FieldType::Date => arrow_schema::Field::new(name, arrow_schema::DataType::Date32, true),
            // TODO
            FieldType::DateTime => unimplemented!(),
            // FieldType::DateTime => arrow_schema::Field::new(
            //     name,
            //     // The dbase DateTime only stores data at second precision, but we currently build
            //     // millisecond arrays, because that's our existing code path
            //     arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None),
            //     true,
            // ),
        };
        non_geo_fields.push(Arc::new(field));
    }

    let geo_arrow_type = GeoArrowType::Wkb(WkbType::new(Default::default()));

    let mut fields = non_geo_fields.clone();
    fields.push(Arc::new(geo_arrow_type.to_field("geometry", true)));
    let schema_ref = Arc::new(arrow_schema::Schema::new(fields));

    FieldsWithGeo {
        schema_ref,
        non_geo_fields,
        geo_arrow_type,
    }
}
