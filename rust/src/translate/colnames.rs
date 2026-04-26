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
        map.insert(col_id, name);
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
        "S12" => return translate_colnames_s12(col_id),
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

    if target_shp.ends_with("Spacial_Preservation_Area_of_Historic_Landscape.shp") {
        return Ok(A42_COLNAMES_SPECIAL[idx].to_string());
    }

    if target_shp.ends_with("Preservation_Area_of_Historic_Landscape.shp") {
        return Ok(A42_COLNAMES_NORMAL[idx].to_string());
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

fn translate_colnames_l02(code: &str) -> String {
    // TODO
    code.to_string()
}

fn translate_colnames_s12(code: &str) -> Result<String, Ksj2GpError> {
    // S12_001c が parse_idx() でパースできないので、年が入っていない列名は先に変換する
    let idx: usize = match code {
        "S12_001" => return Ok("駅名".to_string()),
        "S12_001c" => return Ok("駅コード".to_string()),
        "S12_001g" => return Ok("グループコード".to_string()),
        "S12_002" => return Ok("運営会社".to_string()),
        "S12_003" => return Ok("路線名".to_string()),
        "S12_004" => return Ok("鉄道区分".to_string()),
        "S12_005" => return Ok("事業者種別".to_string()),
        _ => parse_idx(code)? - 6, // S12_006 が基準なので6を引く
    };

    match (idx % 4, idx / 4) {
        (0, year_delta) => Ok(format!("重複コード{}", 2011 + year_delta)),
        (1, year_delta) => Ok(format!("データ有無コード{}", 2011 + year_delta)),
        (2, year_delta) => Ok(format!("備考{}", 2011 + year_delta)),
        (3, year_delta) => Ok(format!("乗降客数{}", 2011 + year_delta)),
        (_, _) => unreachable!(),
    }
}

// e.g. "S12_053" -> 53
fn parse_idx(code: &str) -> Result<usize, Ksj2GpError> {
    code[4..7]
        .parse()
        .map_err(|e| -> Ksj2GpError { format!("Failed to parse {code} as int: {e}").into() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translate::data::colnames::{
        A42_COLNAMES_NORMAL, A42_COLNAMES_SPECIAL, L01_COLNAMES_1983, L01_COLNAMES_2014,
        L01_COLNAMES_2018, L01_COLNAMES_2022, L01_COLNAMES_2024,
    };

    fn options(ksj_id: &str, target_shp: &str) -> TranslateOptions {
        TranslateOptions {
            translate_colnames: true,
            translate_contents: false,
            ignore_translation_errors: false,
            ksj_id: ksj_id.to_string(),
            year: 2024,
            target_shp: target_shp.to_string(),
        }
    }

    #[test]
    fn translate_a03_columns() {
        let opts = options("A03", "");
        let cases = [
            ("A03_001", "行政区域コード"),
            ("A03_002", "都道府県名"),
            ("A03_003", "郡市名"),
            ("A03_004", "区町村名"),
            ("A03_005", "陸水等区分"),
            ("A03_006", "区域区分"),
            ("A03_007", "区域コード"),
            ("A03_008", "備考"),
        ];

        for (code, expected) in cases {
            let actual = translate_colnames(code, &opts).unwrap();
            assert_eq!(actual, expected, "code={code}");
        }
    }

    #[test]
    fn translate_a42_normal_columns() {
        let opts = options("A42", "Preservation_Area_of_Historic_Landscape.shp");
        let cases = [
            ("A42_000", A42_COLNAMES_NORMAL[0]),
            ("A42_003", A42_COLNAMES_NORMAL[3]),
            ("A42_008", A42_COLNAMES_NORMAL[8]),
        ];

        for (code, expected) in cases {
            let actual = translate_colnames(code, &opts).unwrap();
            assert_eq!(actual, expected, "code={code}");
        }
    }

    #[test]
    fn translate_a42_special_columns() {
        let opts = options("A42", "Spacial_Preservation_Area_of_Historic_Landscape.shp");
        let cases = [
            ("A42_000", A42_COLNAMES_SPECIAL[0]),
            ("A42_003", A42_COLNAMES_SPECIAL[3]),
            ("A42_009", A42_COLNAMES_SPECIAL[9]),
        ];

        for (code, expected) in cases {
            let actual = translate_colnames(code, &opts).unwrap();
            assert_eq!(actual, expected, "code={code}");
        }
    }

    #[test]
    fn translate_l01_columns() {
        // Boundaries for each year band: last fixed idx → first dynamic (調査価格 y=1983)
        // → dynamic where y == year → first 属性移動 (y=1984).
        let cases: &[(&str, u16, &str)] = &[
            // year ≤ 2013
            ("L01_001", 2013, L01_COLNAMES_1983[0]),
            ("L01_031", 2010, L01_COLNAMES_1983[30]),
            // 2014..=2017: fixed[1..=47], dynamic from idx 48
            ("L01_047", 2014, L01_COLNAMES_2014[46]),
            ("L01_048", 2014, "調査価格_1983年"),
            ("L01_079", 2014, "調査価格_2014年"),
            ("L01_080", 2014, "属性移動_1984年"),
            ("L01_050", 2015, "調査価格_1985年"),
            ("L01_090", 2015, "属性移動_1993年"),
            // 2018..=2021: fixed[1..=55], dynamic from idx 56
            ("L01_055", 2019, L01_COLNAMES_2018[54]),
            ("L01_056", 2018, "調査価格_1983年"),
            ("L01_091", 2018, "調査価格_2018年"),
            ("L01_092", 2018, "属性移動_1984年"),
            ("L01_058", 2020, "調査価格_1985年"),
            ("L01_100", 2018, "属性移動_1992年"),
            // 2022..=2023: fixed[1..=60], dynamic from idx 61
            ("L01_060", 2022, L01_COLNAMES_2022[59]),
            ("L01_061", 2022, "調査価格_1983年"),
            ("L01_100", 2022, "調査価格_2022年"),
            ("L01_101", 2022, "属性移動_1984年"),
            ("L01_063", 2023, "調査価格_1985年"),
            ("L01_120", 2022, "属性移動_2003年"),
            // 2024..: fixed[1..=61], dynamic from idx 62
            ("L01_061", 2024, L01_COLNAMES_2024[60]),
            ("L01_062", 2024, "調査価格_1983年"),
            ("L01_103", 2024, "調査価格_2024年"),
            ("L01_104", 2024, "属性移動_1984年"),
            ("L01_065", 2024, "調査価格_1986年"),
            ("L01_120", 2024, "属性移動_2000年"),
        ];

        for &(code, year, expected) in cases {
            let actual = translate_colnames_l01(code, year).unwrap();
            assert_eq!(actual, expected, "code={code}, year={year}");
        }
    }

    #[test]
    fn translate_s12_columns() {
        let cases = [
            ("S12_001", "駅名"),
            ("S12_001c", "駅コード"),
            ("S12_001g", "グループコード"),
            ("S12_002", "運営会社"),
            ("S12_003", "路線名"),
            ("S12_004", "鉄道区分"),
            ("S12_005", "事業者種別"),
            ("S12_006", "重複コード2011"),
            ("S12_007", "データ有無コード2011"),
            ("S12_008", "備考2011"),
            ("S12_009", "乗降客数2011"),
            ("S12_058", "重複コード2024"),
            ("S12_059", "データ有無コード2024"),
            ("S12_060", "備考2024"),
            ("S12_061", "乗降客数2024"),
        ];

        for (code, expected) in cases {
            let actual = translate_colnames_s12(code).unwrap();
            assert_eq!(actual, expected, "code={code}");
        }
    }
}
