use dbase::{FieldInfo, FieldType, encoding::EncodingRs};
use geoarrow_schema::{
    Dimension, GeoArrowType, MultiLineStringType, MultiPointType, MultiPolygonType, PointType,
    PolygonType,
};
use geoparquet::writer::{GeoParquetRecordBatchEncoder, GeoParquetWriterOptionsBuilder};
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

    let mut shp_file_opfs = OpfsFile::new(intermediate_files.shp);
    zip.copy_shp_to(&mut shp_file_opfs)?;

    let mut shx_file_opfs = OpfsFile::new(intermediate_files.shx);
    zip.copy_shx_to(&mut shx_file_opfs)?;

    let mut dbf_file_opfs = OpfsFile::new(intermediate_files.dbf);
    zip.copy_dbf_to(&mut dbf_file_opfs)?;

    let output_file_opfs = OpfsFile::new(output_file);

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
    let geometry_type = shapefile_reader.header().shape_type;
    let schema = infer_schema(&dbf_fields, geometry_type);

    for f in schema.fields() {
        web_sys::console::log_1(&format!("field: {f:?}").into());
    }

    let mut reader = Reader::new(shapefile_reader, dbase_reader);

    let options = GeoParquetWriterOptionsBuilder::default()
        // .set_crs_transform(Box::new(todo!())) // TODO
        .set_encoding(geoparquet::writer::GeoParquetWriterEncoding::GeoArrow)
        .set_generate_covering(true)
        .build();
    let mut gpq_encoder =
        GeoParquetRecordBatchEncoder::try_new(&schema, &options).map_err(|e| -> JsValue {
            format!("Got an error on creating GeoParquetRecordBatchEncoder: {e:?}").into()
        })?;

    let mut parquet_writer =
        ArrowWriter::try_new(output_file_opfs, gpq_encoder.target_schema(), None).map_err(
            |e| -> JsValue { format!("Got an error on creating ArrowWriter {e:?}").into() },
        )?;

    for result in reader.iter_shapes_and_records().take(10) {
        let (shape, record) = result.unwrap();
        let geometry = geo_types::Geometry::<f64>::try_from(shape);

        web_sys::console::log_1(&format!("Shape: {geometry:?}, records: ").into());
        for (name, value) in record {
            web_sys::console::log_1(&format!("\t{}: {:?}, ", name, value).into());
        }

        // TODO
        // let encoded_batch = gpq_encoder.encode_record_batch(&batch).unwrap();
        // parquet_writer.write(&encoded_batch).unwrap();
    }

    let kv_metadata = gpq_encoder.into_keyvalue().unwrap();
    parquet_writer.append_key_value_metadata(kv_metadata);
    parquet_writer
        .finish()
        .map_err(|e| -> JsValue { format!("Failed to finish parquet_writer: {e:?}").into() })?;

    Ok(())
}

// This function is derived from geoarrow-rs's old code, which is licensed under MIT/Apache
//
// https://github.com/geoarrow/geoarrow-rs/blob/06e1d615134b249eb5fee39020673c8659978d18/rust/geoarrow-old/src/io/shapefile/reader.rs#L385-L411
fn infer_schema(
    fields: &[FieldInfo],
    geometry_type: shapefile::ShapeType,
) -> arrow_schema::SchemaRef {
    let mut out_fields = Vec::with_capacity(fields.len());

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
            FieldType::DateTime => arrow_schema::Field::new(
                name,
                // The dbase DateTime only stores data at second precision, but we currently build
                // millisecond arrays, because that's our existing code path
                arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None),
                true,
            ),
        };
        out_fields.push(Arc::new(field));
    }

    let geometry_field = match geometry_type {
        shapefile::ShapeType::NullShape => unimplemented!(),

        shapefile::ShapeType::Point => {
            GeoArrowType::Point(PointType::new(Dimension::XY, Default::default()))
        }
        shapefile::ShapeType::PointZ => {
            GeoArrowType::Point(PointType::new(Dimension::XYZ, Default::default()))
        }

        shapefile::ShapeType::Polyline => GeoArrowType::MultiLineString(MultiLineStringType::new(
            Dimension::XY,
            Default::default(),
        )),
        shapefile::ShapeType::PolylineZ => GeoArrowType::MultiLineString(MultiLineStringType::new(
            Dimension::XYZ,
            Default::default(),
        )),

        shapefile::ShapeType::Polygon => {
            GeoArrowType::MultiPolygon(MultiPolygonType::new(Dimension::XY, Default::default()))
        }
        shapefile::ShapeType::PolygonZ => {
            GeoArrowType::MultiPolygon(MultiPolygonType::new(Dimension::XYZ, Default::default()))
        }

        shapefile::ShapeType::Multipoint => {
            GeoArrowType::MultiPoint(MultiPointType::new(Dimension::XY, Default::default()))
        }
        shapefile::ShapeType::MultipointZ => {
            GeoArrowType::MultiPoint(MultiPointType::new(Dimension::XY, Default::default()))
        }

        shapefile::ShapeType::PointM => unimplemented!(),
        shapefile::ShapeType::PolylineM => unimplemented!(),
        shapefile::ShapeType::PolygonM => unimplemented!(),
        shapefile::ShapeType::MultipointM => unimplemented!(),
        shapefile::ShapeType::Multipatch => unimplemented!(),
    };

    out_fields.push(Arc::new(geometry_field.to_field("geometry", true)));

    Arc::new(arrow_schema::Schema::new(out_fields))
}
