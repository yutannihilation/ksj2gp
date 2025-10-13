mod codelists;
mod colnames;
mod data;
mod ksj_id;

pub(crate) use codelists::CODELISTS_MAP;
pub(crate) use colnames::translate_colnames;
pub use ksj_id::extract_ksj_id;

pub struct TranslateOptions {
    pub translate_colnames: bool,
    pub translate_contents: bool,
    pub ignore_translation_errors: bool,
    pub ksj_id: String,
    pub year: u16,
}
