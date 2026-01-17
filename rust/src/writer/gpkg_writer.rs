use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::sync::LazyLock;

use rusqlite_gpkg::{ColumnSpec, ColumnType, Dimension, GeometryType, Gpkg};

use crate::{
    Ksj2GpError, TranslateOptions,
    crs::JapanCrs,
    translate::{get_codelist_map, translate_colnames},
    writer::get_fields_except_geometry,
};

pub(crate) fn write_gpkg<T: Read + Seek, D: Read + Seek, W: Write + Send>(
    reader: &mut shapefile::Reader<T, D>,
    writer: &mut W,
    dbf_fields: &[dbase::FieldInfo],
    crs: JapanCrs,
    translate_options: &TranslateOptions,
) -> Result<(), Ksj2GpError> {
    // TODO: sqlite-wasm-rs supports opfs-sahpool, but I couldn't find out how to use it;
    // sqlite_wasm_vfs::sahpool::install() is async, so it cannot be called in this sync function...
    let gpkg = Gpkg::open_in_memory().map_err(|e| Ksj2GpError::from(format!("{e:?}")))?;
    // TODO: write_gpkg doesn't know the filename
    let layer_name = "layer";

    let shape_type = reader.header().shape_type;
    let geometry_type = geometry_type_from_shape_type(shape_type)?;

    let field_names = get_fields_except_geometry(dbf_fields);
    let (column_specs, codelist_maps) =
        build_column_specs(dbf_fields, translate_options, &field_names)?;

    let (srs_id, srs_name) = match crs {
        JapanCrs::Tokyo => (4301, "Tokyo"),
        JapanCrs::JGD2000 => (4612, "JGD2000"),
        JapanCrs::JGD2011 => (6668, "JGD2011"),
    };

    if srs_id != 4326 {
        gpkg.register_srs(
            srs_name,
            srs_id,
            "EPSG",
            srs_id,
            crs.to_projjson(), // TODO: needs to implement to_wkt()
            srs_name,
        )
        .map_err(|e| Ksj2GpError::from(format!("{e:?}")))?;
    }

    let iter = reader.iter_shapes_and_records();
    let dimension = default_dimension_from_shape_type(shape_type);

    let layer = gpkg
        .create_layer(
            layer_name,
            "geom",
            geometry_type,
            dimension,
            srs_id as u32,
            &column_specs,
        )
        .map_err(|e| Ksj2GpError::from(format!("{e:?}")))?;

    for result in iter {
        let (shape, record) = result?;
        insert_shape_record(
            &layer,
            shape,
            record,
            &field_names,
            &codelist_maps,
            translate_options.translate_contents,
        )?;
    }

    let bytes = gpkg.to_bytes().map_err(|e| format!("{e:?}"))?;
    let mut cursor = std::io::Cursor::new(bytes);
    std::io::copy(&mut cursor, writer).map_err(|e| format!("{e:?}"))?;

    Ok(())
}

fn geometry_type_from_shape_type(
    shape_type: shapefile::ShapeType,
) -> Result<GeometryType, Ksj2GpError> {
    match shape_type {
        shapefile::ShapeType::Point
        | shapefile::ShapeType::PointM
        | shapefile::ShapeType::PointZ => Ok(GeometryType::Point),
        shapefile::ShapeType::Multipoint
        | shapefile::ShapeType::MultipointM
        | shapefile::ShapeType::MultipointZ => Ok(GeometryType::MultiPoint),
        shapefile::ShapeType::Polyline
        | shapefile::ShapeType::PolylineM
        | shapefile::ShapeType::PolylineZ => Ok(GeometryType::MultiLineString),
        shapefile::ShapeType::Polygon
        | shapefile::ShapeType::PolygonM
        | shapefile::ShapeType::PolygonZ => Ok(GeometryType::MultiPolygon),
        shapefile::ShapeType::NullShape | shapefile::ShapeType::Multipatch => {
            Err(format!("Unsupported shape type: {shape_type}").into())
        }
    }
}

fn default_dimension_from_shape_type(shape_type: shapefile::ShapeType) -> Dimension {
    match shape_type {
        shapefile::ShapeType::PointM
        | shapefile::ShapeType::PolylineM
        | shapefile::ShapeType::PolygonM
        | shapefile::ShapeType::MultipointM => Dimension::Xym,
        shapefile::ShapeType::PointZ
        | shapefile::ShapeType::PolylineZ
        | shapefile::ShapeType::PolygonZ
        | shapefile::ShapeType::MultipointZ => Dimension::Xyzm,
        _ => Dimension::Xy,
    }
}

fn build_column_specs(
    dbf_fields: &[dbase::FieldInfo],
    translate_options: &TranslateOptions,
    field_names: &[&str],
) -> Result<
    (
        Vec<ColumnSpec>,
        Vec<Option<&'static LazyLock<HashMap<&'static str, &'static str>>>>,
    ),
    Ksj2GpError,
> {
    let mut column_specs = Vec::with_capacity(dbf_fields.len());
    let mut codelist_maps = Vec::with_capacity(dbf_fields.len());

    for (field, field_name) in dbf_fields.iter().zip(field_names.iter()) {
        let mut column_type = match field.field_type() {
            dbase::FieldType::Numeric
            | dbase::FieldType::Double
            | dbase::FieldType::Currency
            | dbase::FieldType::Float => ColumnType::Double,
            dbase::FieldType::Integer => ColumnType::Integer,
            dbase::FieldType::Logical => ColumnType::Boolean,
            dbase::FieldType::Date => ColumnType::Integer,
            dbase::FieldType::DateTime => ColumnType::Integer,
            dbase::FieldType::Character | dbase::FieldType::Memo => ColumnType::Varchar,
        };

        let codelist_map = if translate_options.translate_contents {
            get_codelist_map(
                field_name,
                translate_options.year,
                &translate_options.target_shp,
            )
        } else {
            None
        };

        if codelist_map.is_some() {
            column_type = ColumnType::Varchar;
        }

        let translated_name = translate_colnames(field.name(), translate_options)?;
        column_specs.push(ColumnSpec {
            name: translated_name,
            column_type,
        });
        codelist_maps.push(codelist_map);
    }

    Ok((column_specs, codelist_maps))
}

fn insert_shape_record(
    layer: &rusqlite_gpkg::GpkgLayer<'_>,
    shape: shapefile::Shape,
    mut record: dbase::Record,
    field_names: &[&str],
    codelist_maps: &[Option<&'static LazyLock<HashMap<&'static str, &'static str>>>],
    translate_contents: bool,
) -> Result<(), Ksj2GpError> {
    let mut values = Vec::with_capacity(field_names.len());

    for (index, field_name) in field_names.iter().enumerate() {
        let value = record
            .remove(field_name)
            .ok_or_else(|| format!("Not found {field_name}"))?;
        let sql_value = field_value_to_sql_value(value, codelist_maps[index], translate_contents);
        values.push(sql_value);
    }

    let params = &values;

    let geometry = geo_types::Geometry::<f64>::try_from(shape)?;
    layer
        .insert(geometry, params)
        .map_err(|e| Ksj2GpError::from(format!("{e:?}")))?;

    Ok(())
}

fn field_value_to_sql_value(
    value: dbase::FieldValue,
    codelist_map: Option<&'static LazyLock<HashMap<&'static str, &'static str>>>,
    translate_contents: bool,
) -> rusqlite_gpkg::Value {
    if translate_contents {
        if let Some(map) = codelist_map {
            if let Some(label) = translate_codelist_value(&value, map) {
                return rusqlite_gpkg::Value::Text(label);
            }
            return rusqlite_gpkg::Value::Null;
        }
    }

    match value {
        dbase::FieldValue::Character(Some(v)) => rusqlite_gpkg::Value::Text(v),
        dbase::FieldValue::Character(None) => rusqlite_gpkg::Value::Null,
        dbase::FieldValue::Memo(v) => rusqlite_gpkg::Value::Text(v),
        dbase::FieldValue::Numeric(Some(v)) => rusqlite_gpkg::Value::Real(v),
        dbase::FieldValue::Numeric(None) => rusqlite_gpkg::Value::Null,
        dbase::FieldValue::Float(Some(v)) => rusqlite_gpkg::Value::Real(v as f64),
        dbase::FieldValue::Float(None) => rusqlite_gpkg::Value::Null,
        dbase::FieldValue::Double(v) => rusqlite_gpkg::Value::Real(v),
        dbase::FieldValue::Currency(v) => rusqlite_gpkg::Value::Real(v),
        dbase::FieldValue::Integer(v) => rusqlite_gpkg::Value::Integer(v as i64),
        dbase::FieldValue::Logical(Some(v)) => rusqlite_gpkg::Value::Integer(if v { 1 } else { 0 }),
        dbase::FieldValue::Logical(None) => rusqlite_gpkg::Value::Null,
        dbase::FieldValue::Date(Some(date)) => {
            rusqlite_gpkg::Value::Integer(date.to_unix_days() as i64)
        }
        dbase::FieldValue::Date(None) => rusqlite_gpkg::Value::Null,
        dbase::FieldValue::DateTime(dt) => rusqlite_gpkg::Value::Integer(dt.to_unix_timestamp()),
    }
}

fn translate_codelist_value(
    value: &dbase::FieldValue,
    map: &'static LazyLock<HashMap<&'static str, &'static str>>,
) -> Option<String> {
    match value {
        dbase::FieldValue::Character(Some(v)) | dbase::FieldValue::Memo(v) => Some(
            map.get(v.as_str())
                .copied()
                .unwrap_or(v.as_str())
                .to_string(),
        ),
        dbase::FieldValue::Character(None)
        | dbase::FieldValue::Numeric(None)
        | dbase::FieldValue::Float(None)
        | dbase::FieldValue::Logical(None)
        | dbase::FieldValue::Date(None) => None,
        dbase::FieldValue::Numeric(Some(v)) | dbase::FieldValue::Double(v) => {
            let code = format!("{v:.0}");
            Some(
                map.get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str())
                    .to_string(),
            )
        }
        dbase::FieldValue::Currency(v) => {
            let code = format!("{v:.0}");
            Some(
                map.get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str())
                    .to_string(),
            )
        }
        dbase::FieldValue::Float(Some(v)) => {
            let code = format!("{v:.0}");
            Some(
                map.get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str())
                    .to_string(),
            )
        }
        dbase::FieldValue::Integer(v) => {
            let code = format!("{v:.0}");
            Some(
                map.get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str())
                    .to_string(),
            )
        }
        dbase::FieldValue::Logical(Some(v)) => {
            let code = if *v { "1" } else { "0" };
            Some(map.get(code).copied().unwrap_or(code).to_string())
        }
        dbase::FieldValue::Date(Some(date)) => {
            let code = date.to_string();
            Some(
                map.get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str())
                    .to_string(),
            )
        }
        dbase::FieldValue::DateTime(dt) => {
            let code = dt.to_unix_timestamp().to_string();
            Some(
                map.get(code.as_str())
                    .copied()
                    .unwrap_or(code.as_str())
                    .to_string(),
            )
        }
    }
}
