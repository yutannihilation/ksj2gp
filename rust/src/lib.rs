use wasm_bindgen::prelude::*;
use web_sys::{FileReaderSync, js_sys::Uint8Array};

use crate::io::ReadOnlyFile;

mod io;

thread_local! {
    static FILE_READER_SYNC: FileReaderSync = FileReaderSync::new().unwrap();
}

#[wasm_bindgen]
pub fn read_bytes(file: web_sys::File, start: i32, end: i32) -> Result<(), JsValue> {
    let blob = file.slice_with_i32_and_i32(start, end)?;

    let array_buffer = FILE_READER_SYNC.with(|reader| reader.read_as_array_buffer(&blob).unwrap());
    let u8_array = Uint8Array::new(&array_buffer);

    let len = u8_array.length() as usize;
    let mut bytes = vec![0u8; len];
    u8_array.copy_to(&mut bytes);

    let msg = format!("(start: {start}, end: {end}) -> {bytes:?}");
    web_sys::console::log_1(&msg.into()); // Note: into() works only on `&str`, not on `String`, so & is necessary

    Ok(())
}

#[wasm_bindgen]
pub fn list_files(zip_file: web_sys::File) -> Result<(), JsValue> {
    let reader = ReadOnlyFile::new(zip_file);
    let mut zip = match zip::ZipArchive::new(reader) {
        Ok(zip) => zip,
        Err(e) => return Err(format!("Failed to read ZIP file!: {e:?}").into()),
    };

    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();
        let msg = format!("Filename: {}", file.name());
        web_sys::console::log_1(&msg.into()); // Note: into() works only on `&str`, not on `String`, so & is necessary
    }

    Ok(())
}
