use std::io::{Read, Seek, Write};

use dbase::encoding::EncodingRs;
use zip::ZipArchive;

use crate::{
    crs::{JapanCrs, guess_crs_from_esri_wkt, guess_crs_from_meta_xml},
    error::Ksj2GpError,
};

pub struct ZippedShapefileReader<R: Read + Seek> {
    zip: ZipArchive<R>,
    shp_filename: String,
    dbf_filename: String,
    shx_filename: String,
    prj_filename: String,
    cpg_filename: String,
    meta_xml_filename: Option<String>,
}

impl<R: Read + Seek> ZippedShapefileReader<R> {
    pub fn new(
        zip: ZipArchive<R>,
        target_shp: &str,
        meta_xml_filename: Option<String>,
    ) -> Result<Self, Ksj2GpError> {
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
            meta_xml_filename,
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

    pub fn guess_crs(&mut self) -> Result<JapanCrs, Ksj2GpError> {
        // First, if .prj file exists, try to acquire the CRS from it
        match self.zip.by_name(&self.prj_filename) {
            Ok(mut prj_reader) => {
                let mut wkt = String::new();
                prj_reader.read_to_string(&mut wkt)?;

                if let Ok(crs) = guess_crs_from_esri_wkt(&wkt) {
                    return Ok(crs);
                }
            }
            Err(zip::result::ZipError::FileNotFound) => {} // it's not a rara case when we find no .prj file...
            Err(e) => return Err(e.into()),
        }

        // If no .prj file found, use KS-META file.
        if let Some(meta_xml_filename) = &self.meta_xml_filename {
            match self.zip.by_name(meta_xml_filename) {
                Ok(mut meta_xml_reader) => {
                    let mut meta_xml_content_sjis: Vec<u8> = Vec::new();
                    meta_xml_reader.read_to_end(&mut meta_xml_content_sjis)?;

                    // KS-META XML ファイルは Shift_JIS のはず...
                    let (meta_xml_content, _, error) =
                        encoding_rs::SHIFT_JIS.decode(&meta_xml_content_sjis);
                    if error {
                        return Err("Failed to decode KS-META XML file from CP932".into());
                    }

                    guess_crs_from_meta_xml(&meta_xml_content)
                }
                Err(e) => Err(e.into()),
            }
        } else {
            Err("Failed to detect CRS from .prj or KS-META-".into())
        }
    }

    // cf. https://github.com/EsriJapan/shapefile_info
    pub fn guess_encoding(&mut self) -> Result<EncodingRs, Ksj2GpError> {
        // First, try to guess from LDID (29th byte of dBASE file)
        let mut dbf_reader = self.zip.by_name(&self.dbf_filename).unwrap();
        let mut buf = vec![0u8; 29];
        dbf_reader.read_exact(&mut buf)?;
        if buf[28] == 13 {
            return Ok(EncodingRs::from(dbase::encoding_rs::SHIFT_JIS));
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
            .replace(['-', '_'], "")
            .contains("utf8")
        {
            return Ok(EncodingRs::from(dbase::encoding_rs::UTF_8));
        }

        Ok(EncodingRs::from(dbase::encoding_rs::SHIFT_JIS))
    }
}
