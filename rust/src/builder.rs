use geoarrow_array::GeoArrowArray as _;
use geoarrow_schema::WkbType;

use dbase::FieldType;

use dbase::FieldInfo;

use geoarrow_array::builder::WkbBuilder;

use dbase::FieldValue;

use geoarrow_schema::GeoArrowType;

use std::sync::Arc;

pub(crate) struct FieldsWithGeo {
    pub(crate) schema_ref: arrow_schema::SchemaRef,
    pub(crate) non_geo_fields: Vec<Arc<arrow_schema::Field>>,
    pub(crate) geoarrow_type: geoarrow_schema::GeoArrowType,
}

#[derive(Debug)]
pub(crate) enum NonGeoArrayBuilder {
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
    pub(crate) fn push(&mut self, value: FieldValue) {
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

    pub(crate) fn finish(&mut self) -> arrow_array::ArrayRef {
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

pub(crate) struct ArrayBuilderWithGeo {
    pub(crate) builders: Vec<NonGeoArrayBuilder>,
    pub(crate) geo_builder: WkbBuilder<i32>,
}

impl FieldsWithGeo {
    pub(crate) fn create_builders(&self, capacity: usize) -> ArrayBuilderWithGeo {
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

        // Use the same GeoArrow type (with CRS metadata) as in the schema
        let geo_builder = match &self.geoarrow_type {
            GeoArrowType::Wkb(wkb_type) => WkbBuilder::new(wkb_type.clone()),
            _ => unreachable!(),
        };

        ArrayBuilderWithGeo {
            builders,
            geo_builder,
        }
    }
}

impl ArrayBuilderWithGeo {
    pub(crate) fn finish(mut self) -> Vec<arrow_array::ArrayRef> {
        let mut result: Vec<_> = self.builders.iter_mut().map(|b| b.finish()).collect();
        result.push(self.geo_builder.finish().into_array_ref());
        result
    }
}

// This function is derived from geoarrow-rs's old code, which is licensed under MIT/Apache
//
// https://github.com/geoarrow/geoarrow-rs/blob/06e1d615134b249eb5fee39020673c8659978d18/rust/geoarrow-old/src/io/shapefile/reader.rs#L385-L411
pub(crate) fn construct_schema(fields: &[FieldInfo], crs: geoarrow_schema::Crs) -> FieldsWithGeo {
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

    let geoarrow_metadata = geoarrow_schema::Metadata::new(crs, None);
    let geoarrow_type = GeoArrowType::Wkb(WkbType::new(geoarrow_metadata.into()));
    let geo_field = geoarrow_type.to_field("geometry", true);

    let mut fields = non_geo_fields.clone();
    fields.push(Arc::new(geo_field));
    let schema_ref = Arc::new(arrow_schema::Schema::new(fields));

    FieldsWithGeo {
        schema_ref,
        non_geo_fields,
        geoarrow_type,
    }
}
