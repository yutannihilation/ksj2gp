use std::str::FromStr;

const EPSG4612: &str = include_str!("epsg4612.json");
const EPSG6668: &str = include_str!("epsg6668.json");

pub fn wild_guess_from_esri_wkt_to_projjson(wkt: &str) -> Result<serde_json::Value, String> {
    if wkt.contains("GCS_JGD_2011") {
        let parsed = serde_json::Value::from_str(EPSG6668).unwrap();
        return Ok(parsed);
    }

    if wkt.contains("GCS_JGD_2000") {
        let parsed = serde_json::Value::from_str(EPSG4612).unwrap();
        return Ok(parsed);
    }

    Err(format!(
        "Failed to identify CRS from ESRI WKT in the .prj file: {wkt}"
    ))
}
