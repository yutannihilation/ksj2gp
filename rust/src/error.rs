#[derive(Debug, Clone)]
pub struct Ksj2GpError(String);

impl std::fmt::Display for Ksj2GpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::error::Error for Ksj2GpError {}

impl From<Ksj2GpError> for String {
    fn from(value: Ksj2GpError) -> Self {
        value.0
    }
}

impl From<zip::result::ZipError> for Ksj2GpError {
    fn from(value: zip::result::ZipError) -> Self {
        Self(format!("zip error: {value:?}"))
    }
}

impl From<std::io::Error> for Ksj2GpError {
    fn from(value: std::io::Error) -> Self {
        Self(format!("IO error: {value:?}"))
    }
}

impl From<shapefile::Error> for Ksj2GpError {
    fn from(value: shapefile::Error) -> Self {
        Self(format!("shapefile error: {value:?}"))
    }
}

impl From<dbase::Error> for Ksj2GpError {
    fn from(value: dbase::Error) -> Self {
        Self(format!("dbase error: {value:?}"))
    }
}

impl From<geoarrow_schema::error::GeoArrowError> for Ksj2GpError {
    fn from(value: geoarrow_schema::error::GeoArrowError) -> Self {
        Self(format!("geoarrow error: {value:?}"))
    }
}

impl From<arrow_schema::ArrowError> for Ksj2GpError {
    fn from(value: arrow_schema::ArrowError) -> Self {
        Self(format!("arrow error: {value:?}"))
    }
}

impl From<parquet::errors::ParquetError> for Ksj2GpError {
    fn from(value: parquet::errors::ParquetError) -> Self {
        Self(format!("parquet error: {value:?}"))
    }
}

impl From<proj4rs::errors::Error> for Ksj2GpError {
    fn from(value: proj4rs::errors::Error) -> Self {
        Self(format!("proj4rs error: {value:?}"))
    }
}

impl From<&str> for Ksj2GpError {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for Ksj2GpError {
    fn from(value: String) -> Self {
        Self(value)
    }
}
