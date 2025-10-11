use std::{collections::HashMap, sync::LazyLock};

use crate::translate::data::colnames::COLNAMES;

static COLNAMES_MAP: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(COLNAMES.len());
    for &(code, name) in COLNAMES {
        map.insert(code, name);
    }
    map
});

pub(crate) fn translate_colnames(code: &str) -> String {
    match COLNAMES_MAP.get(code) {
        Some(name) => name.to_string(),
        // If the code is not included in COLNAMES_MAP, return the code as it is.
        // TODO: add logging here.
        None => code.to_string(),
    }
}
