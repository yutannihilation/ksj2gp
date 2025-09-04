use wasm_bindgen::JsValue;
use web_sys::{FileReaderSync, FileSystemReadWriteOptions, js_sys::Uint8Array};

use crate::zip_reader::ZipReader;

// Note: OnceLock cannot be used here.
thread_local! {
    static READER: FileReaderSync = FileReaderSync::new().unwrap();
}

pub struct UserLocalFile {
    file: web_sys::File,
    offset: u64,
}

impl UserLocalFile {
    pub fn new(file: web_sys::File) -> Self {
        Self { file, offset: 0 }
    }

    pub fn new_zip_reader(&self) -> Result<ZipReader, JsValue> {
        let reader = Self {
            file: self.file.clone(),
            offset: 0,
        };
        match zip::ZipArchive::new(reader) {
            Ok(zip) => ZipReader::new(zip),
            Err(e) => Err(format!("Failed to read ZIP file!: {e:?}").into()),
        }
    }
}

impl std::io::Read for UserLocalFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let start = self.offset as f64;
        let end = start + buf.len() as f64;

        let blob = self
            .file
            .slice_with_f64_and_f64(start, end)
            .map_err(convert_js_error_to_io_error)?;

        let array_buffer = READER.with(|reader| reader.read_as_array_buffer(&blob).unwrap());
        let u8_array = Uint8Array::new(&array_buffer);

        let len = u8_array.length() as u64;
        u8_array.copy_to(buf);

        self.offset = self.offset + len;

        Ok(len as usize)
    }
}

impl std::io::Seek for UserLocalFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let size = self.file.size() as i64;
        let new_offset = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::End(offset) => size as i64 - offset,
            std::io::SeekFrom::Current(offset) => self.offset as i64 + offset,
        };

        // The document (https://doc.rust-lang.org/beta/std/io/trait.Seek.html#tymethod.seek) says:
        //
        // > Seeking to a negative offset is considered an error.
        if new_offset < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid offset",
            ));
        }

        // Make sure the offset is within the range of the file.
        self.offset = std::cmp::min(new_offset, size) as u64;

        Ok(self.offset)
    }
}

pub struct OpfsFile {
    file: web_sys::FileSystemSyncAccessHandle,
    offset: FileSystemReadWriteOptions,
}

// Note: FileSystemSyncAccessHandle is not Send in a strict sense, but this is
// required for ArrowWriter. Probably marking as Send won't be a problem as
// long as the program is executed on a single thread, but I'm not fully sure...
unsafe impl std::marker::Send for OpfsFile {}

impl OpfsFile {
    pub fn new(file: web_sys::FileSystemSyncAccessHandle) -> Result<Self, JsValue> {
        // currently, the same name of file is repeatedly used, so it needs to be truncated first.
        file.truncate_with_u32(0)?;

        Ok(Self {
            file,
            offset: FileSystemReadWriteOptions::new(),
        })
    }
}

impl std::io::Write for OpfsFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = self
            .file
            .write_with_u8_array_and_options(buf, &self.offset)
            .map_err(convert_js_error_to_io_error)? as u64;

        self.offset
            .set_at(self.offset.get_at().unwrap_or(0f64) + size as f64);

        Ok(size as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush().map_err(convert_js_error_to_io_error)?;
        Ok(())
    }
}

impl std::io::Read for OpfsFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self
            .file
            .read_with_u8_array_and_options(buf, &self.offset)
            .map_err(convert_js_error_to_io_error)? as u64;

        self.offset
            .set_at(self.offset.get_at().unwrap_or(0f64) + size as f64);

        Ok(size as usize)
    }
}

impl std::io::Seek for OpfsFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let size = self.file.get_size().map_err(convert_js_error_to_io_error)? as i64;
        let new_offset = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::End(offset) => size as i64 - offset,
            std::io::SeekFrom::Current(offset) => {
                self.offset.get_at().unwrap_or(0f64) as i64 + offset
            }
        };

        // The document (https://doc.rust-lang.org/beta/std/io/trait.Seek.html#tymethod.seek) says:
        //
        // > Seeking to a negative offset is considered an error.
        if new_offset < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid offset",
            ));
        }

        // Make sure the offset is within the range of the file.
        let new_offset = std::cmp::min(new_offset, size) as u64;
        self.offset.set_at(new_offset as f64);

        Ok(new_offset)
    }
}

impl Drop for OpfsFile {
    fn drop(&mut self) {
        self.file.close();
    }
}

fn convert_js_error_to_io_error(e: JsValue) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Other,
        format!(
            "Some error happened on JS API: {}",
            e.as_string().unwrap_or("<undisplayable>".to_string())
        ),
    )
}
