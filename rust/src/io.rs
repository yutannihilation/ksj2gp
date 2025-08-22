use web_sys::{FileReaderSync, js_sys::Uint8Array};

// Note: OnceLock cannot be used here.
thread_local! {
    static READER: FileReaderSync = FileReaderSync::new().unwrap();
}

pub struct ReadOnlyFile {
    file: web_sys::File,
    offset: u64,
}

impl ReadOnlyFile {
    pub fn new(file: web_sys::File) -> Self {
        Self { file, offset: 0 }
    }
}

impl std::io::Read for ReadOnlyFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let start = self.offset as f64;
        let end = start + buf.len() as f64;

        let blob = self.file.slice_with_f64_and_f64(start, end).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Some error happened on calling slice()",
            )
        })?;

        let array_buffer = READER.with(|reader| reader.read_as_array_buffer(&blob).unwrap());
        let u8_array = Uint8Array::new(&array_buffer);

        let len = u8_array.length() as u64;
        u8_array.copy_to(buf);

        self.offset = self.offset + len;

        Ok(len as usize)
    }
}

impl std::io::Seek for ReadOnlyFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let size = self.file.size() as i64;
        let new_offset = match pos {
            std::io::SeekFrom::Start(offset) => offset as i64,
            std::io::SeekFrom::End(offset) => self.file.size() as i64 - offset,
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
