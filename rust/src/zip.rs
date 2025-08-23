use std::io::Seek as _;

use wasm_bindgen::JsValue;
use zip::ZipArchive;

use crate::io::{OpfsFile, UserLocalFile};

pub struct ZipReader {
    zip: ZipArchive<UserLocalFile>,
    shp_filename: String,
    dbf_filename: String,
    shx_filename: String,
    // prj: String // TODO
}

impl ZipReader {
    pub fn new(zip: ZipArchive<UserLocalFile>) -> Result<Self, JsValue> {
        let filenames: Vec<&str> = zip.file_names().collect();

        let shp_filename = find_specific_ext(&filenames, ".shp")?;
        let dbf_filename = find_specific_ext(&filenames, ".dbf")?;
        let shx_filename = find_specific_ext(&filenames, ".shx")?;

        drop(filenames);

        Ok(Self {
            zip,
            shp_filename,
            dbf_filename,
            shx_filename,
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
}

fn find_specific_ext(filenames: &[&str], ext: &str) -> Result<String, JsValue> {
    // TODO: how to handle multiple Shapefiles?
    let filename = match filenames.iter().find(|x| x.ends_with(ext)) {
        Some(filename) => filename.to_string(),
        None => return Err(format!("This ZIP file doesn't contain any {ext} file").into()),
    };

    Ok(filename)
}
