use std::{str::FromStr, sync::LazyLock};

use regex::Regex;

use crate::Ksj2GpError;

const EPSG4301: &str = include_str!("epsg4301.json");
const EPSG4612: &str = include_str!("epsg4612.json");
const EPSG6668: &str = include_str!("epsg6668.json");

pub enum JapanCrs {
    Tokyo,
    JGD2000,
    JGD2011,
    // JGD2024,
}

impl JapanCrs {
    pub fn to_projjson(&self) -> &'static str {
        match self {
            JapanCrs::Tokyo => EPSG4301,
            JapanCrs::JGD2000 => EPSG4612,
            JapanCrs::JGD2011 => EPSG6668,
            // JapanCrs::JGD2024 => todo!(),
        }
    }
}

pub fn guess_crs_from_esri_wkt(wkt: &str) -> Result<JapanCrs, Ksj2GpError> {
    if wkt.contains("GCS_JGD_2011") {
        return Ok(JapanCrs::JGD2011);
    }

    if wkt.contains("GCS_JGD_2000") {
        return Ok(JapanCrs::JGD2000);
    }

    if wkt.contains("GCS_Tokyo") {
        return Ok(JapanCrs::Tokyo);
    }

    Err(format!("Failed to identify CRS from ESRI WKT in the .prj file: {wkt}").into())
}

static RE: LazyLock<Regex> = LazyLock::new(|| {
    // (?m): enable multiline mode
    Regex::new(r"(?m)<extentReferenceSystem>.*(JGD2011|JGD2000|TD).*</extentReferenceSystem>")
        .unwrap()
});

pub fn guess_crs_from_meta_xml(meta_xml_content: &str) -> Result<JapanCrs, Ksj2GpError> {
    if let Some(c) = RE.captures(meta_xml_content) {
        let (_, [crs]) = c.extract();
        match crs {
            "JGD2011" => Ok(JapanCrs::JGD2011),
            "JGD2000" => Ok(JapanCrs::JGD2000),
            "TD" => Ok(JapanCrs::Tokyo),
            _ => Err(format!("Unexpected regex match: {crs}").into()),
        }
    } else {
        Err("Failed to identify CRS from Meta XML".into())
    }
}
