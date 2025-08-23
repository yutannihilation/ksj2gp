use std::io::{Seek, Write};

use wasm_bindgen::prelude::*;
use web_sys::FileReaderSync;

use crate::io::{OpfsFile, UserLocalFile};

mod io;

thread_local! {
    static FILE_READER_SYNC: FileReaderSync = FileReaderSync::new().unwrap();
}

#[wasm_bindgen]
pub fn list_files(
    zip_file: web_sys::File,
    tmp_shp_file: web_sys::FileSystemSyncAccessHandle,
) -> Result<(), JsValue> {
    let reader = UserLocalFile::new(zip_file);
    let mut zip = reader.new_zip_reader()?;

    let shp_filenames: Vec<&str> = zip.file_names().filter(|x| x.ends_with(".shp")).collect();
    if shp_filenames.is_empty() {
        return Err("The ZIP file doesn't contain any .shp file".into());
    }

    let (base_filename, _) = shp_filenames[0].rsplit_once(".").unwrap();
    let shp_filename = format!("{base_filename}.shp");
    let dbf_filename = format!("{base_filename}.dbf");
    let shx_filename = format!("{base_filename}.shx");
    let prj_filename = format!("{base_filename}.prj"); // TODO
    drop(shp_filenames);

    let mut shp_file_reader = zip.by_name(&shp_filename).unwrap();
    let mut shp_file_opfs = OpfsFile::new(tmp_shp_file);

    std::io::copy(&mut shp_file_reader, &mut shp_file_opfs).unwrap();

    shp_file_opfs
        .rewind()
        .map_err(|e| -> JsValue { format!("Got an error on rewinding .shp file: {e:?}").into() })?;

    let mut zip_dbf = reader.new_zip_reader()?;
    let dbf_file = zip_dbf.by_name(&dbf_filename).unwrap();

    let mut zip_shx = reader.new_zip_reader()?;
    let shx_file = zip_shx.by_name(&shx_filename).unwrap();

    let mut shp = geozero::shp::ShpReader::new(shp_file_opfs)
        .map_err(|e| -> JsValue { format!("Got an error on opening .shp file: {e:?}").into() })?;

    // shp.add_dbf_source(dbf_file)
    //     .map_err(|e| -> JsValue { format!("Got an error on add_dbf_source(): {e:?}").into() })?;
    // shp.add_index_source(shx_file)
    //     .map_err(|e| -> JsValue { format!("Got an error on add_index_source(): {e:?}").into() })?;

    // let fields = shp
    //     .dbf_fields()
    //     .map_err(|e| -> JsValue { format!("Got an error on dbf_fields(): {e:?}").into() })?;

    // let msg = format!("{:?}", fields);
    // web_sys::console::log_1(&msg.into()); // Note: into() works only on `&str`, not on `String`, so & is necessary

    Ok(())
}
