use ksj2gp::{TranslateOptions, convert_shp_inner, encode_utf8_to_cp437cp932, extract_ksj_id};
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;

use crate::io::{OpfsFile, UserLocalFile};

mod io;

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
pub fn list_shp_files(zip_file: web_sys::File) -> Result<Vec<String>, String> {
    let reader = UserLocalFile::new(zip_file);
    ksj2gp::list_shp_files(reader).map_err(|e| format!("{e}"))
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn convert_shp(
    zip_file: web_sys::File,
    target_shp: &str,
    intermediate_files: IntermediateFiles,
    output_file: web_sys::FileSystemSyncAccessHandle,
    output_format: &str,
    translate_colnames: bool,
    translate_contents: bool,
    ignore_translation_errors: bool,
) -> Result<(), String> {
    let filename = zip_file.name();
    let (ksj_id, year) = extract_ksj_id(&filename)?;

    let zip = UserLocalFile::new(zip_file);
    let output_file_opfs = std::io::BufWriter::new(OpfsFile::new(output_file)?);

    convert_shp_inner(
        zip,
        &encode_utf8_to_cp437cp932(target_shp)?,
        OpfsFile::new(intermediate_files.shp)?,
        OpfsFile::new(intermediate_files.dbf)?,
        OpfsFile::new(intermediate_files.shx)?,
        output_file_opfs,
        output_format,
        TranslateOptions {
            translate_colnames,
            translate_contents,
            ignore_translation_errors,
            ksj_id,
            year,
            target_shp: target_shp.to_string(),
        },
    )?;

    Ok(())
}
