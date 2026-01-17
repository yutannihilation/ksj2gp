mod geojson_writer;
mod geoparquet_writer;
mod gpkg_writer;

pub(crate) use geojson_writer::write_geojson;
pub(crate) use geoparquet_writer::write_geoparquet;
pub(crate) use gpkg_writer::write_gpkg;

// dBASE fields doesn't include the geometry column
fn get_fields_except_geometry(x: &[dbase::FieldInfo]) -> Vec<&str> {
    x.iter().map(|f| f.name()).collect()
}
