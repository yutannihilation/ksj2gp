use dbase::encoding::EncodingRs;
use shapefile::{Reader, ShapeReader};
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

    let mut shx_file_opfs = OpfsFile::new(intermediate_files.shx);
    zip.copy_shx_to(&mut shx_file_opfs)?;

    let mut dbf_file_opfs = OpfsFile::new(intermediate_files.dbf);
    zip.copy_dbf_to(&mut dbf_file_opfs)?;

    let shape_reader =
        ShapeReader::with_shx(shp_file_opfs, shx_file_opfs).map_err(|e| -> JsValue {
            format!("Got an error on Reading .shp and .shx files: {e:?}").into()
        })?;

    let dbase_reader = shapefile::dbase::Reader::new_with_encoding(
        dbf_file_opfs,
        EncodingRs::from(encoding_rs::SHIFT_JIS), // TODO: read UTF-8
    )
    .map_err(|e| -> JsValue { format!("Got an error on Reading a .dbf file: {e:?}").into() })?;

    let mut reader = Reader::new(shape_reader, dbase_reader);

    for result in reader.iter_shapes_and_records().take(10) {
        let (shape, record) = result.unwrap();
        let geometry = geo_types::Geometry::<f64>::try_from(shape);

        web_sys::console::log_1(&format!("Shape: {geometry:?}, records: ").into());
        for (name, value) in record {
            web_sys::console::log_1(&format!("\t{}: {:?}, ", name, value).into());
        }
    }

    Ok(())
}
