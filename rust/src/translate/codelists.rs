use crate::translate::data::codelists::{A10_area_code, AdminCd, CodelistId};
use std::{collections::HashMap, sync::LazyLock};

use crate::translate::data::colnames::COLNAMES;

// TODO: probably, it's not ideal to expose this HashMap directly. Can we wrap this as a function like colnames.rs?
pub(crate) static CODELISTS_MAP: LazyLock<
    HashMap<&'static str, LazyLock<HashMap<&'static str, &'static str>>>,
> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, LazyLock<HashMap<&'static str, &'static str>>> =
        HashMap::with_capacity(200); // TODO: choose a nicer number
    for (col_id, metadata) in COLNAMES {
        if let Some(id) = metadata.1 {
            match id {
                // TODO use macro to reduce lines of code
                CodelistId::AdminCd => {
                    map.entry(col_id).or_insert_with(|| {
                        LazyLock::new(|| {
                            let mut inner = HashMap::with_capacity(AdminCd.len());
                            for &(code, label) in AdminCd {
                                inner.insert(code, label);
                            }
                            inner
                        })
                    });
                }
                CodelistId::A10_area_code => {
                    map.entry(col_id).or_insert_with(|| {
                        LazyLock::new(|| {
                            let mut inner = HashMap::with_capacity(A10_area_code.len());
                            for &(code, label) in A10_area_code {
                                inner.insert(code, label);
                            }
                            inner
                        })
                    });
                }
                _ => {}
            }
        }
    }
    map
});
