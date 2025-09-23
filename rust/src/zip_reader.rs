use std::io::{Read, Seek as _};

use dbase::encoding::EncodingRs;
use zip::ZipArchive;

use crate::{
    error::Ksj2GpError,
    io::{OpfsFile, UserLocalFile},
};

pub struct ZippedShapefileReader {
    zip: ZipArchive<UserLocalFile>,
    shp_filename: String,
    dbf_filename: String,
    shx_filename: String,
    prj_filename: String,
    cpg_filename: String,
}

impl ZippedShapefileReader {
    pub fn new(zip: ZipArchive<UserLocalFile>, target_shp: &str) -> Result<Self, Ksj2GpError> {
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

    fn copy_to_opfs(
        &mut self,
        dst: web_sys::FileSystemSyncAccessHandle,
        filename: &str,
    ) -> Result<std::io::BufReader<OpfsFile>, Ksj2GpError> {
        let mut opfs = OpfsFile::new(dst)?;
        let reader = self.zip.by_name(filename).unwrap();

        std::io::copy(
            &mut std::io::BufReader::new(reader),
            &mut std::io::BufWriter::new(&mut opfs),
        )?;

        opfs.rewind()?;

        Ok(std::io::BufReader::new(opfs))
    }

    pub fn copy_shp_to_opfs(
        &mut self,
        dst: web_sys::FileSystemSyncAccessHandle,
    ) -> Result<std::io::BufReader<OpfsFile>, Ksj2GpError> {
        self.copy_to_opfs(dst, &self.shp_filename.clone())
    }

    pub fn copy_dbf_to_opfs(
        &mut self,
        dst: web_sys::FileSystemSyncAccessHandle,
    ) -> Result<std::io::BufReader<OpfsFile>, Ksj2GpError> {
        self.copy_to_opfs(dst, &self.dbf_filename.clone())
    }

    pub fn copy_shx_to_opfs(
        &mut self,
        dst: web_sys::FileSystemSyncAccessHandle,
    ) -> Result<std::io::BufReader<OpfsFile>, Ksj2GpError> {
        self.copy_to_opfs(dst, &self.shx_filename.clone())
    }

    pub fn read_prj(&mut self) -> Result<String, Ksj2GpError> {
        let mut reader = self.zip.by_name(&self.prj_filename).unwrap();
        let mut wkt = String::new();
        reader.read_to_string(&mut wkt)?;

        Ok(wkt)
    }

    pub fn guess_encoding(&mut self) -> Result<EncodingRs, Ksj2GpError> {
        match self.zip.by_name(&self.prj_filename) {
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
