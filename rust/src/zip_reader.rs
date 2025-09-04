use std::io::{Read, Seek as _};

use wasm_bindgen::JsValue;
use zip::ZipArchive;

use crate::io::{OpfsFile, UserLocalFile};

pub struct ZippedShapefileReader {
    zip: ZipArchive<UserLocalFile>,
    shp_filename: String,
    dbf_filename: String,
    shx_filename: String,
    prj_filename: String,
}

impl ZippedShapefileReader {
    pub fn new(zip: ZipArchive<UserLocalFile>, target_shp: &str) -> Result<Self, JsValue> {
        // Sanity checks
        if !target_shp.ends_with(".shp") {
            return Err(format!("Not a Shapefile: {target_shp}").into());
        }

        let (filename_base, _) = target_shp.rsplit_once(".").unwrap();
        let shp_filename = format!("{filename_base}.shp");
        let dbf_filename = format!("{filename_base}.dbf");
        let shx_filename = format!("{filename_base}.shx");
        let prj_filename = format!("{filename_base}.prj");

        // Check if the file actually exists in the ZIP file
        let filenames: Vec<&str> = zip.file_names().collect();

        for f in [
            shp_filename.as_str(),
            dbf_filename.as_str(),
            shx_filename.as_str(),
            prj_filename.as_str(),
        ] {
            if !filenames.contains(&f) {
                return Err(format!("{f} doesn't exist in the ZIP file").into());
            }
        }

        Ok(Self {
            zip,
            shp_filename,
            dbf_filename,
            shx_filename,
            prj_filename,
        })
    }

    fn copy_to(&mut self, dst: &mut OpfsFile, filename: &str) -> Result<(), JsValue> {
        let mut reader = self.zip.by_name(filename).unwrap();

        std::io::copy(&mut reader, dst).map_err(|e| -> JsValue {
            format!("Got an error while extracting {filename} to a OPFS file: {e:?}").into()
        })?;

        dst.rewind()
            .map_err(|e| -> JsValue { format!("Got an error on rewinding file: {e:?}").into() })?;

        Ok(())
    }

    pub fn copy_shp_to(&mut self, dst: &mut OpfsFile) -> Result<(), JsValue> {
        self.copy_to(dst, &self.shp_filename.clone())
    }

    pub fn copy_dbf_to(&mut self, dst: &mut OpfsFile) -> Result<(), JsValue> {
        self.copy_to(dst, &self.dbf_filename.clone())
    }

    pub fn copy_shx_to(&mut self, dst: &mut OpfsFile) -> Result<(), JsValue> {
        self.copy_to(dst, &self.shx_filename.clone())
    }

    pub fn read_prj(&mut self) -> Result<String, JsValue> {
        let mut reader = self.zip.by_name(&self.prj_filename).unwrap();
        let mut wkt = String::new();
        reader.read_to_string(&mut wkt).map_err(|e| -> JsValue {
            format!("Got an error while reading .prj file: {e:?}").into()
        })?;

        Ok(wkt)
    }
}
