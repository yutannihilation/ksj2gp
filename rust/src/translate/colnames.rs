use std::{collections::HashMap, sync::LazyLock};

use crate::{TranslateOptions, error::Ksj2GpError, translate::data::colnames::COLNAMES};

static COLNAMES_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::with_capacity(COLNAMES.len());
    for &(code, name) in COLNAMES {
        map.insert(code, name);
    }
    map
});

pub(crate) fn translate_colnames(
    code: &str,
    translate_options: &TranslateOptions,
) -> Result<String, Ksj2GpError> {
    // No translation
    if !translate_options.translate_colnames {
        return Ok(code.to_string());
    }

    match COLNAMES_MAP.get(code) {
        Some(name) => Ok(name.to_string()),
        None => {
            if translate_options.ignore_translation_errors {
                Ok(code.to_string())
            } else {
                Err(format!("Unknown column name translation: {code}").into())
            }
        }
    }
}

fn translate_colnames_L01(code: &str) -> String {
    // TODO
    code.to_string()
}

fn translate_colnames_L02(code: &str) -> String {
    // TODO
    code.to_string()
}
