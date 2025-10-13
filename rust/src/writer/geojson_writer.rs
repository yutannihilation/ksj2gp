use std::io::{Read, Seek, Write};

use geojson::JsonObject;

use crate::{
    error::Ksj2GpError,
    transform_coord::CoordTransformer,
    translate::{TranslateOptions, translate_codelists, translate_colnames},
    writer::get_fields_except_geometry,
};

pub(crate) fn write_geojson<T: Read + Seek, D: Read + Seek, W: Write + Send>(
    reader: &mut shapefile::Reader<T, D>,
    writer: &mut W,
    dbf_fields: &[dbase::FieldInfo],
    translate_options: &TranslateOptions,
) -> Result<(), Ksj2GpError> {
    let transformer = CoordTransformer::new();

    // Since shapefile::Record is a HashMap, the iterator of it doesn't maintain
    // the order. So, this column names vector is needed to ensure the consistent
    // order with the schema.
    let field_names = get_fields_except_geometry(dbf_fields);

    let mut features: Vec<geojson::Feature> = Vec::new();
    for result in reader.iter_shapes_and_records() {
        let (shape, mut record) = result.unwrap();

        // Convert dBASE record to GeoJSON properties without serde dependency
        let mut properties: JsonObject = JsonObject::new();
        for field_name in &field_names {
            let value = record
                .remove(field_name)
                .ok_or_else(|| format!("Not found {field_name}"))?;

            let translated_field_name = translate_colnames(field_name, translate_options)?;

            properties.insert(
                translated_field_name,
                dbase_field_to_json_value(value, field_name),
            );
        }

        let geometry = geojson::Geometry::new(transformer.transform_to_geojson(&shape)?);

        features.push(geojson::Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        });
    }

    let geojson: geojson::GeoJson = geojson::FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
    .into();

    // TODO: implement From<geojson::Error>
    let geojson_str = geojson.to_string_pretty().unwrap();

    writer.write_all(geojson_str.as_bytes())?;
    writer.flush()?;

    Ok(())
}

fn dbase_field_to_json_value(x: dbase::FieldValue, field_name: &str) -> geojson::JsonValue {
    match x {
        // TODO: I'm assuming all fields to translate is string, not number.
        // TODO: Is this too costly to always try to get() on the HashMap? Should I place some filter before actually calling translate_codelists()?

        // String
        dbase::FieldValue::Character(Some(x)) => translate_codelists(field_name, &x).into(),
        dbase::FieldValue::Character(None) => geojson::JsonValue::Null,
        dbase::FieldValue::Memo(x) => translate_codelists(field_name, &x).into(),
        // Number
        dbase::FieldValue::Numeric(x) => x.into(),
        dbase::FieldValue::Float(x) => x.into(),
        dbase::FieldValue::Integer(x) => x.into(),
        dbase::FieldValue::Double(x) => x.into(),
        dbase::FieldValue::Currency(x) => x.into(),
        // Boolean
        dbase::FieldValue::Logical(x) => x.into(),
        // Date
        dbase::FieldValue::Date(Some(x)) => {
            format!("{}-{}-{}", x.year(), x.month(), x.day()).into()
        }
        dbase::FieldValue::Date(None) => geojson::JsonValue::Null,
        dbase::FieldValue::DateTime(x) => {
            let date = x.date();
            let time = x.time();
            format!(
                "{}-{}-{} {}:{}:{}",
                date.year(),
                date.month(),
                date.day(),
                time.hours(),
                time.minutes(),
                time.seconds()
            )
            .into()
        }
    }
}
