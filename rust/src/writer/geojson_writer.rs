use std::io::{Read, Seek, Write};

use itertools::Itertools as _;

use crate::{
    builder::construct_schema, crs::JapanCrs, error::Ksj2GpError,
    transform_coord::CoordTransformer, translate::TranslateOptions,
    writer::get_fields_except_geometry,
};

// Number of rows to process at once
const CHUNK_SIZE: usize = 2048;

pub(crate) fn write_geojson<T: Read + Seek, D: Read + Seek, W: Write + Send>(
    reader: &mut shapefile::Reader<T, D>,
    writer: &mut W,
    dbf_fields: &[dbase::FieldInfo],
    crs: JapanCrs,
    translate_options: &TranslateOptions,
) -> Result<(), Ksj2GpError> {
    // TODO: include this in FieldsWithGeo
    let transformer = CoordTransformer::new(crs.clone());

    let projjson = crs.to_projjson();
    let crs = geoarrow_schema::Crs::from_projjson(projjson.into());

    let fields_info = construct_schema(dbf_fields, crs, translate_options)?;
    let schema_ref = fields_info.schema_ref.clone();

    // Since shapefile::Record is a HashMap, the iterator of it doesn't maintain
    // the order. So, this column names vector is needed to ensure the consistent
    // order with the schema.
    let field_names = get_fields_except_geometry(dbf_fields);

    let mut geojson_writer = geoarrow_geojson::writer::GeoJsonWriter::new(writer)?;

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

            let geometry = transformer.transform(&shape)?;
            builders.geo_builder.push_geometry(Some(&geometry))?;
        }

        let batch = arrow_array::RecordBatch::try_new(schema_ref.clone(), builders.finish())?;
        geojson_writer.write(&batch)?;
    }

    geojson_writer.finish()?;

    Ok(())
}
