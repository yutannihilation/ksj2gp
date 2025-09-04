use dbase::encoding::EncodingRs;

// currently, there's no reliable way to guess the encoding of the .dbf file.
pub fn guess_encoding(path: &str) -> EncodingRs {
    if path
        .to_lowercase()
        .replace('-', "")
        .replace('_', "")
        .contains("utf8")
    {
        return EncodingRs::from(dbase::encoding_rs::UTF_8);
    }

    EncodingRs::from(dbase::encoding_rs::SHIFT_JIS)
}
