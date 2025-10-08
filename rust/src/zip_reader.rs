use std::io::{Read, Seek, Write};

use dbase::encoding::EncodingRs;
use zip::ZipArchive;

use crate::error::Ksj2GpError;

pub struct ZippedShapefileReader<R: Read + Seek> {
    zip: ZipArchive<R>,
    shp_filename: String,
    dbf_filename: String,
    shx_filename: String,
    prj_filename: String,
    cpg_filename: String,
}

impl<R: Read + Seek> ZippedShapefileReader<R> {
    pub fn new(zip: ZipArchive<R>, target_shp: &str) -> Result<Self, Ksj2GpError> {
        // Sanity checks
        if !target_shp.ends_with(".shp") {
            return Err(format!("Not a Shapefile: {target_shp}").into());
        }

        let (filename_base, _) = target_shp.rsplit_once(".").unwrap();
        let shp_filename = format!("{filename_base}.shp");
        let dbf_filename = format!("{filename_base}.dbf");
        let shx_filename = format!("{filename_base}.shx");
        let prj_filename = format!("{filename_base}.prj");
        let cpg_filename = format!("{filename_base}.cpg");

        // Check if the file actually exists in the ZIP file
        let filenames: Vec<&str> = zip.file_names().collect();

        // Note: .prj and .cpg are optional, so don't need to check here
        for f in [
            shp_filename.as_str(),
            dbf_filename.as_str(),
            shx_filename.as_str(),
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
            cpg_filename,
        })
    }

    /// Write to the `dst` and return the BufReader fo it
    fn copy_to<RW: Write + Seek + Read>(
        &mut self,
        mut dst: RW,
        filename: &str,
    ) -> Result<std::io::BufReader<RW>, Ksj2GpError> {
        let reader = self.zip.by_name(filename).unwrap();

        std::io::copy(
            &mut std::io::BufReader::new(reader),
            &mut std::io::BufWriter::new(&mut dst),
        )?;

        dst.rewind()?;

        Ok(std::io::BufReader::new(dst))
    }

    pub fn copy_shp_to<RW: Write + Seek + Read>(
        &mut self,
        dst: RW,
    ) -> Result<std::io::BufReader<RW>, Ksj2GpError> {
        self.copy_to(dst, &self.shp_filename.clone())
    }

    pub fn copy_dbf_to<RW: Write + Seek + Read>(
        &mut self,
        dst: RW,
    ) -> Result<std::io::BufReader<RW>, Ksj2GpError> {
        self.copy_to(dst, &self.dbf_filename.clone())
    }

    pub fn copy_shx_to<RW: Write + Seek + Read>(
        &mut self,
        dst: RW,
    ) -> Result<std::io::BufReader<RW>, Ksj2GpError> {
        self.copy_to(dst, &self.shx_filename.clone())
    }

    pub fn read_prj(&mut self) -> Result<Option<String>, Ksj2GpError> {
        let mut reader = match self.zip.by_name(&self.prj_filename) {
            Ok(reader) => reader,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut wkt = String::new();
        reader.read_to_string(&mut wkt)?;

        Ok(Some(wkt))
    }

    // cf. https://github.com/EsriJapan/shapefile_info
    pub fn guess_encoding(&mut self) -> Result<EncodingRs, Ksj2GpError> {
        // First, try to guess from LDID (29th byte of dBASE file)
        let mut dbf_reader = self.zip.by_name(&self.dbf_filename).unwrap();
        let mut buf = vec![0u8; 29];
        dbf_reader.read(&mut buf)?;
        match buf[28] {
            13 => return Ok(EncodingRs::from(dbase::encoding_rs::SHIFT_JIS)),
            _ => {}
        }
        drop(dbf_reader);

        // Next, check .cpg file
        match self.zip.by_name(&self.cpg_filename) {
            Ok(mut reader) => {
                let mut cpg = String::new();
                reader.read_to_string(&mut cpg)?;

                match cpg.as_str() {
                    "UTF-8" => return Ok(EncodingRs::from(dbase::encoding_rs::UTF_8)),
                    "CP932" => return Ok(EncodingRs::from(dbase::encoding_rs::SHIFT_JIS)),
                    _ => {
                        return Err(format!("Unknown encoding is found in .cpg file: {cpg}").into());
                    }
                }
            }
            Err(zip::result::ZipError::FileNotFound) => {} // If ZIP file doesn't contain .cpg file, use other heuristics...
            Err(e) => return Err(e.into()),
        }

        // In case of no LDID and no .cpg file, try to wild guess from the path...
        // If file path contains some characters like "utf-8", it's probably UTF-8
        if self
            .shp_filename
            .to_lowercase()
            .replace('-', "")
            .replace('_', "")
            .contains("utf8")
        {
            return Ok(EncodingRs::from(dbase::encoding_rs::UTF_8));
        }

        Ok(EncodingRs::from(dbase::encoding_rs::SHIFT_JIS))
    }
}
