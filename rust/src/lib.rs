use std::io::{Seek, Write};

use geozero::geo_types::{GeoFeatureWriter, GeoWriter};
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;

use crate::io::{OpfsFile, UserLocalFile};

mod io;
mod zip;

thread_local! {
    static FILE_READER_SYNC: FileReaderSync = FileReaderSync::new().unwrap();
}

// ShpReader requires Read and Seek, but files in a Zip archive cannot be Seek (only Read).
// So, these files are necessary for temporarily extracting the file.
#[wasm_bindgen]
pub struct IntermediateFiles {
    shp: web_sys::FileSystemSyncAccessHandle,
    dbf: web_sys::FileSystemSyncAccessHandle,
    shx: web_sys::FileSystemSyncAccessHandle,
    // prj: web_sys::FileSystemSyncAccessHandle, // TODO
}

#[wasm_bindgen]
impl IntermediateFiles {
    #[wasm_bindgen(constructor)]
    pub fn new(
        shp: web_sys::FileSystemSyncAccessHandle,
        dbf: web_sys::FileSystemSyncAccessHandle,
        shx: web_sys::FileSystemSyncAccessHandle,
    ) -> Self {
        Self { shp, dbf, shx }
    }
}

#[wasm_bindgen]
pub fn list_files(
    zip_file: web_sys::File,
    intermediate_files: IntermediateFiles,
) -> Result<(), JsValue> {
    let reader = UserLocalFile::new(zip_file);
    let mut zip = reader.new_zip_reader()?;

    let mut shp_file_opfs = OpfsFile::new(intermediate_files.shp);
    zip.copy_shp_to(&mut shp_file_opfs)?;

    let mut shp = geozero::shp::ShpReader::new(shp_file_opfs)
        .map_err(|e| -> JsValue { format!("Got an error on opening .shp file: {e:?}").into() })?;

    let mut shx_file_opfs = OpfsFile::new(intermediate_files.shx);
    zip.copy_shx_to(&mut shx_file_opfs)?;

    shp.add_index_source(shx_file_opfs)
        .map_err(|e| -> JsValue { format!("Got an error on add_index_source(): {e:?}").into() })?;

    let mut dbf_file_opfs = OpfsFile::new(intermediate_files.dbf);
    zip.copy_dbf_to(&mut dbf_file_opfs)?;

    shp.add_dbf_source(dbf_file_opfs)
        .map_err(|e| -> JsValue { format!("Got an error on add_dbf_source(): {e:?}").into() })?;

    let fields = shp
        .dbf_fields()
        .map_err(|e| -> JsValue { format!("Got an error on dbf_fields(): {e:?}").into() })?;

    let msg = format!("{:?}", fields);
    web_sys::console::log_1(&msg.into()); // Note: into() works only on `&str`, not on `String`, so & is necessary

    // let geo = sh

    for f in &shp.dbf_fields().unwrap() {
        // f.
    }

    Ok(())
}
