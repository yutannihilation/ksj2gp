use ksj2gp::{convert_shp_inner, decode_cp437cp932_to_utf8};
use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;
use zip::ZipArchive;

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
    let zip = ZipArchive::new(reader).map_err(|e| format!("{e}"))?;

    let shp_files: Vec<String> = zip
        .file_names()
        .filter(|path| path.ends_with(".shp"))
        .map(decode_cp437cp932_to_utf8)
        .collect::<Result<_, _>>()?;

    Ok(shp_files)
}

#[wasm_bindgen]
pub fn convert_shp(
    zip_file: web_sys::File,
    target_shp: &str,
    intermediate_files: IntermediateFiles,
    output_file: web_sys::FileSystemSyncAccessHandle,
    output_format: &str,
) -> Result<(), String> {
    let zip = UserLocalFile::new(zip_file);
    let output_file_opfs = std::io::BufWriter::new(OpfsFile::new(output_file)?);

    convert_shp_inner(
        zip,
        target_shp,
        OpfsFile::new(intermediate_files.shp)?,
        OpfsFile::new(intermediate_files.dbf)?,
        OpfsFile::new(intermediate_files.shx)?,
        output_file_opfs,
        output_format,
    )?;

    Ok(())
}
