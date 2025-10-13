use std::{collections::HashMap, sync::LazyLock};

use crate::{
    TranslateOptions,
    error::Ksj2GpError,
    translate::data::colnames::{
        A42_COLNAMES_NORMAL, A42_COLNAMES_SPECIAL, COLNAMES, L01_COLNAMES_1983, L01_COLNAMES_2014,
        L01_COLNAMES_2018, L01_COLNAMES_2022, L01_COLNAMES_2024,
    },
};

static COLNAMES_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::with_capacity(COLNAMES.len());
    for (col_id, name) in COLNAMES {
        map.insert(col_id, name.0);
    }
    map
});

// TODO: return &str to avoid unnecessary allocation
pub(crate) fn translate_colnames(
    col_id: &str,
    translate_options: &TranslateOptions,
) -> Result<String, Ksj2GpError> {
    // No translation
    if !translate_options.translate_colnames {
        return Ok(col_id.to_string());
    }

    // 特殊な処理が必要な ID のものは専用の関数をつくる
    match translate_options.ksj_id.as_str() {
        "A42" => return translate_colnames_a42(col_id, &translate_options.target_shp),
        "L01" => return translate_colnames_l01(col_id, translate_options.year),
        "L02" => unimplemented!(),
        _ => {}
    }

    match COLNAMES_MAP.get(col_id) {
        Some(name) => Ok(name.to_string()),
        None => {
            if translate_options.ignore_translation_errors {
                Ok(col_id.to_string())
            } else {
                Err(format!("Unknown column name translation: {col_id}").into())
            }
        }
    }
}

fn translate_colnames_a42(code: &str, target_shp: &str) -> Result<String, Ksj2GpError> {
    let idx: usize = parse_idx(code)?;

    if target_shp.ends_with("Preservation_Area_of_Historic_Landscape.shp") {
        return Ok(A42_COLNAMES_NORMAL[idx].to_string());
    }

    if target_shp.ends_with("Spacial_Preservation_Area_of_Historic_Landscape.shp") {
        return Ok(A42_COLNAMES_SPECIAL[idx].to_string());
    }

    Err(format!("Unknown shapefile: {target_shp}").into())
}

// 現時点での最新仕様: https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-L01-2025.html
// L01 の難しい点は、カラムの構成が年によって変わる点。
// - 2013年までは32カラム
// - それ以降は「昭和59年から令和6年公示価格」や「昭和60年～令和6年属性移動」の部分が増える
fn translate_colnames_l01(code: &str, year: u16) -> Result<String, Ksj2GpError> {
    let idx: usize = parse_idx(code)?;

    match (year, idx) {
        (_, 0) => panic!("Something is wrong"),
        (..=2013, _) => Ok(L01_COLNAMES_1983[idx - 1].to_string()),
        (2014..=2017, 1..=47) => Ok(L01_COLNAMES_2014[idx - 1].to_string()),
        (2014..=2017, 48..) => {
            let y = (idx - 48) + 1983;
            if y <= year as _ {
                Ok(format!("調査価格_{y}年"))
            } else {
                Ok(format!(
                    "属性移動_{}年",
                    y - (year as usize - 1983) // (year - 1983) までは調査価格なのでその分がすれる
                ))
            }
        }
        (2018..=2021, 1..=55) => Ok(L01_COLNAMES_2018[idx - 1].to_string()),
        (2018..=2021, 56..) => {
            let y = (idx - 56) + 1983;
            if y <= year as _ {
                Ok(format!("調査価格_{y}年"))
            } else {
                Ok(format!(
                    "属性移動_{}年",
                    y - (year as usize - 1983) // (year - 1983) までは調査価格なのでその分がすれる
                ))
            }
        }
        (2022..=2023, 1..=60) => Ok(L01_COLNAMES_2022[idx - 1].to_string()),
        (2022..=2023, 61..) => {
            let y = (idx - 61) + 1983;
            if y <= year as _ {
                Ok(format!("調査価格_{y}年"))
            } else {
                Ok(format!(
                    "属性移動_{}年",
                    y - (year as usize - 1983) // (year - 1983) までは調査価格なのでその分がすれる
                ))
            }
        }
        (2024.., 1..=61) => Ok(L01_COLNAMES_2024[idx - 1].to_string()),
        (2024.., 62..) => {
            let y = (idx - 62) + 1983;
            if y <= year as _ {
                Ok(format!("調査価格_{y}年"))
            } else {
                Ok(format!(
                    "属性移動_{}年",
                    y - (year as usize - 1983) // (year - 1983) までは調査価格なのでその分がすれる
                ))
            }
        }
    }
}

fn parse_idx(code: &str) -> Result<usize, Ksj2GpError> {
    Ok(code[4..7]
        .parse()
        .map_err(|e| -> Ksj2GpError { format!("Failed to parse {code} as int: {e}").into() })?)
}

fn translate_colnames_l02(code: &str) -> String {
    // TODO
    code.to_string()
}
