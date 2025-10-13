use crate::translate::data::codelists::{AdminCd, CodelistId};
use std::{collections::HashMap, sync::LazyLock};

use crate::translate::data::colnames::COLNAMES;

// TODO: in terms of performance, should this be HashMap<&str, HashMap<&str, &str>>?
static CODELISTS_MAP: LazyLock<HashMap<(&'static str, &'static str), &'static str>> =
    LazyLock::new(|| {
        let mut map: HashMap<(&'static str, &'static str), &'static str> =
            HashMap::with_capacity(COLNAMES.len());
        for (col_id, metadata) in COLNAMES {
            if let Some(id) = metadata.1 {
                match id {
                    CodelistId::AdminCd => {
                        for (code, label) in AdminCd {
                            map.insert((col_id, code), label);
                        }
                    }
                    _ => {}
                }
            }
        }
        map
    });

// TODO: return &str to avoid unnecessary allocation
pub(crate) fn translate_codelists(col_id: &str, code: &str) -> String {
    match CODELISTS_MAP.get(&(col_id, code)) {
        Some(label) => label.to_string(),
        None => code.to_string(),
    }
}
