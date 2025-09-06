use std::io::{BufWriter, Read, Seek, Write};

use arrow_schema::SchemaRef;
use geojson::JsonObject;
use wasm_bindgen::JsValue;

use crate::{error::Ksj2GpError, io::OpfsFile};

pub(crate) fn write_geojson<T: Read + Seek, D: Read + Seek>(
    reader: &mut shapefile::Reader<T, D>,
    writer: &mut BufWriter<OpfsFile>,
    schema_ref: SchemaRef,
    _wkt: &str, // TODO: Use this wkt to reproject the coordinates to WGS84 by proj4wkt and proj4rs.
) -> Result<(), Ksj2GpError> {
    // Since shapefile::Record is a HashMap, the iterator of it doesn't maintain
    // the order. So, this column names vector is needed to ensure the consistent
    // order with the schema.
    let (_last, fields_except_geometry) = schema_ref.fields().split_last().unwrap();
    let field_names: Vec<String> = fields_except_geometry
        .iter()
        .map(|f| f.name().to_string())
        .collect();

    let mut features: Vec<geojson::Feature> = Vec::new();
    for result in reader.iter_shapes_and_records() {
        let (shape, mut record) = result.unwrap();

        // Convert dBASE record to GeoJSON properties without serde dependency
        let mut properties: JsonObject = JsonObject::new();
        for field_name in &field_names {
            let value = record
                .remove(field_name)
                .ok_or_else(|| -> JsValue { format!("Not found {field_name}").into() })?;
            properties.insert(field_name.to_string(), dbase_field_to_json_value(value));
        }

        let geometry_geo_types = geo_types::Geometry::<f64>::try_from(shape)?;
        let geometry: geojson::Geometry = (&geometry_geo_types).into();

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

fn dbase_field_to_json_value(x: dbase::FieldValue) -> geojson::JsonValue {
    match x {
        // String
        dbase::FieldValue::Character(x) => x.into(),
        dbase::FieldValue::Memo(x) => x.into(),
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
