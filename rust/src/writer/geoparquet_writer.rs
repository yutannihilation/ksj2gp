use std::io::{Read, Seek, Write};

use geoparquet::writer::{GeoParquetRecordBatchEncoder, GeoParquetWriterOptionsBuilder};
use itertools::Itertools;
use parquet::arrow::ArrowWriter;

use crate::{
    builder::construct_schema, crs::wild_guess_from_esri_wkt_to_projjson, error::Ksj2GpError,
    writer::get_fields_except_geometry,
};

// Number of rows to process at once
const CHUNK_SIZE: usize = 2048;

pub(crate) fn write_geoparquet<T: Read + Seek, D: Read + Seek, W: Write + Send>(
    reader: &mut shapefile::Reader<T, D>,
    writer: &mut W,
    dbf_fields: &[dbase::FieldInfo],
    wkt: &Option<String>,
    use_readable_colnames: bool,
    use_readable_contents: bool,
) -> Result<(), Ksj2GpError> {
    let projjson = match wkt {
        Some(wkt) => wild_guess_from_esri_wkt_to_projjson(wkt)?,
        // TODO: if .prj is not found, guess from other information
        None => return Err(format!(".prj not found").into()),
    };
    let crs = geoarrow_schema::Crs::from_projjson(projjson);

    let fields_info = construct_schema(dbf_fields, crs, use_readable_colnames)?;
    let schema_ref = fields_info.schema_ref.clone();

    // Since shapefile::Record is a HashMap, the iterator of it doesn't maintain
    // the order. So, this column names vector is needed to ensure the consistent
    // order with the schema.
    let field_names = get_fields_except_geometry(dbf_fields);

    let options = GeoParquetWriterOptionsBuilder::default()
        .set_encoding(geoparquet::writer::GeoParquetWriterEncoding::WKB)
        .set_generate_covering(true)
        .build();
    let mut gpq_encoder = GeoParquetRecordBatchEncoder::try_new(&schema_ref, &options)?;

    let mut parquet_writer = ArrowWriter::try_new(writer, gpq_encoder.target_schema(), None)?;

    for chunk in &reader.iter_shapes_and_records().chunks(CHUNK_SIZE) {
        let mut builders = fields_info.create_builders(CHUNK_SIZE);

        for result in chunk {
            let (shape, mut record) = result.unwrap();

            for (i, field_name) in field_names.iter().enumerate() {
                let value = record
                    .remove(field_name)
                    .ok_or_else(|| format!("Not found {field_name}"))?;
                builders.builders[i].push(value);
            }

            let geometry = geo_types::Geometry::<f64>::try_from(shape)?;
            builders.geo_builder.push_geometry(Some(&geometry))?;
        }

        let batch = arrow_array::RecordBatch::try_new(schema_ref.clone(), builders.finish())?;
        let encoded_batch = gpq_encoder.encode_record_batch(&batch)?;

        parquet_writer.write(&encoded_batch)?;
        parquet_writer.flush()?;
    }

    let kv_metadata = gpq_encoder.into_keyvalue().unwrap();
    parquet_writer.append_key_value_metadata(kv_metadata);
    parquet_writer.finish()?;

    Ok(())
}
