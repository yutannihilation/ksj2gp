mod geojson_writer;
mod geoparquet_writer;

pub(crate) use geojson_writer::write_geojson;
pub(crate) use geoparquet_writer::write_geoparquet;

fn get_fields_except_geometry(x: &[dbase::FieldInfo]) -> Vec<&str> {
    let (_last, fields_except_geometry) = x.split_last().unwrap();
    fields_except_geometry.iter().map(|f| f.name()).collect()
}
