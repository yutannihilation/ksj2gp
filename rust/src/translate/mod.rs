use std::sync::LazyLock;

use regex::Regex;

mod colnames;
mod data;

pub(crate) use colnames::translate_colnames;

use crate::Ksj2GpError;

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Z][0-9]{2}[a-z]?[0-9]?(?:-[a-z12])?(?:-[cu])?|m1000|m500)-([0-9]{2})").unwrap()
});

/// Guess KSJ ID and year from the ZIP filename
pub(crate) fn extract_ksj_id(filename: &str) -> Result<(&str, u16), Ksj2GpError> {
    // Handle mesh first, because these are out of the pattern...

    if filename.starts_with("1km_mesh_suikei_2018") {
        return Ok(("mesh1000h30", 2018));
    }

    if filename.starts_with("1km_mesh_2024") {
        return Ok(("mesh1000r6", 2024));
    }

    if filename.starts_with("500m_mesh_suikei_2018") {
        return Ok(("mesh500h30", 2018));
    }

    if filename.starts_with("500m_mesh_2024") {
        return Ok(("mesh500r6", 2018));
    }
    if filename.starts_with("250m_mesh_2024") {
        return Ok(("mesh250r6", 2018));
    }

    // Otherwise, use the regex pattern

    let (_, [id_raw, year_str]) = match RE.captures(filename) {
        Some(c) => c.extract(),
        None => return Err(format!("Failed to detect KSJ id from filename: {filename}").into()),
    };

    let year_2digits = year_str
        .parse::<u16>()
        .map_err(|e| -> Ksj2GpError { format!("Failed to parse year: {e}").into() })?;

    // もっといい感じの解決策が 2040 年までに現れてこのレポジトリが不要になることを祈りつつ...
    let year = if year_2digits >= 40 {
        year_2digits + 1900
    } else {
        year_2digits + 2000
    };

    let id = match id_raw {
        "m1000" => "mesh1000",
        "m500" => "mesh500",
        "A18s-a" => "A18s_a",
        "A19s-a" => "A19s",
        "G04-a" => "G04a",
        "G04-c" => "G04c",
        "G04-d" => "G04d",
        _ => id_raw,
    };

    Ok((id, year))
}

#[cfg(test)]
mod tests {
    use super::extract_ksj_id;

    #[test]
    fn test_extract_ksj_id() {
        assert_eq!(
            extract_ksj_id("m1000-17_27_GML.zip").unwrap(),
            ("mesh1000", 2017)
        );
        assert_eq!(
            extract_ksj_id("m500-17_44_GML.zip").unwrap(),
            ("mesh500", 2017)
        );
        assert_eq!(
            extract_ksj_id("1km_mesh_suikei_2018_shape_19.zip").unwrap(),
            ("mesh1000h30", 2018)
        );
        assert_eq!(
            extract_ksj_id("1km_mesh_2024_04_GML.zip").unwrap(),
            ("mesh1000r6", 2024)
        );
        assert_eq!(
            extract_ksj_id("500m_mesh_suikei_2018_shape_23.zip").unwrap(),
            ("mesh500h30", 2018)
        );
        assert_eq!(
            extract_ksj_id("500m_mesh_2024_GML.zip").unwrap(),
            ("mesh500r6", 2018)
        );
        assert_eq!(
            extract_ksj_id("250m_mesh_2024_GML.zip").unwrap(),
            ("mesh250r6", 2018)
        );
        assert_eq!(extract_ksj_id("A09-06_02_GML.zip").unwrap(), ("A09", 2006));
        assert_eq!(extract_ksj_id("A10-06_03_GML.zip").unwrap(), ("A10", 2006));
        assert_eq!(extract_ksj_id("A11-15_31_GML.zip").unwrap(), ("A11", 2015));
        assert_eq!(extract_ksj_id("A12-06_04_GML.zip").unwrap(), ("A12", 2006));
        assert_eq!(extract_ksj_id("A13-06_14_GML.zip").unwrap(), ("A13", 2006));
        assert_eq!(extract_ksj_id("A15-15_12_GML.zip").unwrap(), ("A15", 2015));
        assert_eq!(extract_ksj_id("A16-75_34_GML.zip").unwrap(), ("A16", 1975));
        assert_eq!(
            extract_ksj_id("A17-901001_20_GML.zip").unwrap(),
            ("A17", 1990)
        );
        assert_eq!(
            extract_ksj_id("A18-051001_16_GML.zip").unwrap(),
            ("A18", 2005)
        );
        assert_eq!(
            extract_ksj_id("A18s-a-10_GML.zip").unwrap(),
            ("A18s_a", 2010)
        );
        assert_eq!(
            extract_ksj_id("A19-651001_45_GML.zip").unwrap(),
            ("A19", 1965)
        );
        assert_eq!(
            extract_ksj_id("A19s-05_17_GML.zip").unwrap(),
            ("A19s", 2005)
        );
        assert_eq!(
            extract_ksj_id("A19s-a-05_17_GML.zip").unwrap(),
            ("A19s", 2005)
        ); // A19s_a is same as A19s
        assert_eq!(
            extract_ksj_id("A20-601001_46_GML.zip").unwrap(),
            ("A20", 1960)
        );
        assert_eq!(
            extract_ksj_id("A20s-05_46_GML.zip").unwrap(),
            ("A20s", 2005)
        );
        assert_eq!(
            extract_ksj_id("A21-001001_13_GML.zip").unwrap(),
            ("A21", 2000)
        );
        assert_eq!(
            extract_ksj_id("A21s-10_13_GML.zip").unwrap(),
            ("A21s", 2010)
        );
        assert_eq!(
            extract_ksj_id("A22-070402_32_GML.zip").unwrap(),
            ("A22", 2007)
        );
        assert_eq!(
            extract_ksj_id("A22-m-14_33_GML.zip").unwrap(),
            ("A22-m", 2014)
        );
        assert_eq!(
            extract_ksj_id("A22s-00_19_GML.zip").unwrap(),
            ("A22s", 2000)
        );
        assert_eq!(
            extract_ksj_id("A23-051001_46_GML.zip").unwrap(),
            ("A23", 2005)
        );
        assert_eq!(
            extract_ksj_id("A24-901001_36_GML.zip").unwrap(),
            ("A24", 1990)
        );
        assert_eq!(
            extract_ksj_id("A25-930928_35_GML.zip").unwrap(),
            ("A25", 1993)
        );
        assert_eq!(extract_ksj_id("A27-21_10_GML.zip").unwrap(), ("A27", 2021));
        assert_eq!(extract_ksj_id("A28-10_GML.zip").unwrap(), ("A28", 2010));
        assert_eq!(extract_ksj_id("A29-11_39_GML.zip").unwrap(), ("A29", 2011));
        assert_eq!(
            extract_ksj_id("A30a5-11_5438-jgd_GML.zip").unwrap(),
            ("A30a5", 2011)
        );
        assert_eq!(extract_ksj_id("A30b-11_GML.zip").unwrap(), ("A30b", 2011));
        assert_eq!(
            extract_ksj_id("A31a-24_81_10_GML.zip").unwrap(),
            ("A31a", 2024)
        );
        assert_eq!(
            extract_ksj_id("A31b-24_10_5540_GML.zip").unwrap(),
            ("A31b", 2024)
        );
        assert_eq!(extract_ksj_id("A32-21_29_GML.zip").unwrap(), ("A32", 2021));
        assert_eq!(extract_ksj_id("A33-24_00_GML.zip").unwrap(), ("A33", 2024));
        assert_eq!(extract_ksj_id("A34-180316_GML.zip").unwrap(), ("A34", 2018));
        assert_eq!(
            extract_ksj_id("A35a-14_46_GML.zip").unwrap(),
            ("A35a", 2014)
        );
        assert_eq!(
            extract_ksj_id("A35b-14_26_GML.zip").unwrap(),
            ("A35b", 2014)
        );
        assert_eq!(
            extract_ksj_id("A35c-14_07_GML.zip").unwrap(),
            ("A35c", 2014)
        );
        assert_eq!(extract_ksj_id("A37-15_39_GML.zip").unwrap(), ("A37", 2015));
        assert_eq!(extract_ksj_id("A38-20_27_GML.zip").unwrap(), ("A38", 2020));
        assert_eq!(extract_ksj_id("A39-15_GML.zip").unwrap(), ("A39", 2015));
        assert_eq!(extract_ksj_id("A40-20_14_GML.zip").unwrap(), ("A40", 2020));
        assert_eq!(extract_ksj_id("A42-18_GML.zip").unwrap(), ("A42", 2018));
        assert_eq!(extract_ksj_id("A43-18_GML.zip").unwrap(), ("A43", 2018));
        assert_eq!(extract_ksj_id("A44-18_GML.zip").unwrap(), ("A44", 2018));
        assert_eq!(extract_ksj_id("A45-19_41_GML.zip").unwrap(), ("A45", 2019));
        assert_eq!(extract_ksj_id("A46-20_04_GML.zip").unwrap(), ("A46", 2020));
        assert_eq!(extract_ksj_id("A47-21_52_GML.zip").unwrap(), ("A47", 2021));
        assert_eq!(extract_ksj_id("A48-21_17_GML.zip").unwrap(), ("A48", 2021));
        assert_eq!(extract_ksj_id("A49-20_13_GML.zip").unwrap(), ("A49", 2020));
        assert_eq!(extract_ksj_id("A50-20_14_GML.zip").unwrap(), ("A50", 2020));
        assert_eq!(extract_ksj_id("A51-24_20_GML.zip").unwrap(), ("A51", 2024));
        assert_eq!(extract_ksj_id("A52-23_25_GML.zip").unwrap(), ("A52", 2023));
        assert_eq!(extract_ksj_id("A53-23-81_GML.zip").unwrap(), ("A53", 2023));
        assert_eq!(extract_ksj_id("A54-23_GML.zip").unwrap(), ("A54", 2023));
        assert_eq!(
            extract_ksj_id("A55-22_01100_SHP.zip").unwrap(),
            ("A55", 2022)
        );
        assert_eq!(extract_ksj_id("C02-06_GML.zip").unwrap(), ("C02", 2006));
        assert_eq!(extract_ksj_id("C09-06_GML.zip").unwrap(), ("C09", 2006));
        assert_eq!(extract_ksj_id("C23-06_28_GML.zip").unwrap(), ("C23", 2006));
        assert_eq!(extract_ksj_id("C28-13.zip").unwrap(), ("C28", 2013));
        assert_eq!(
            extract_ksj_id("G02-12_5238-jgd_GML.zip").unwrap(),
            ("G02", 2012)
        );
        assert_eq!(
            extract_ksj_id("G04-a-11_5940-jgd_GML.zip").unwrap(),
            ("G04a", 2011)
        );
        assert_eq!(
            extract_ksj_id("G04-c-11_5439-jgd_GML.zip").unwrap(),
            ("G04c", 2011)
        );
        assert_eq!(
            extract_ksj_id("G04-d-11_5338-jgd_GML.zip").unwrap(),
            ("G04d", 2011)
        );
        assert_eq!(extract_ksj_id("G08-15_43_GML.zip").unwrap(), ("G08", 2015));
        assert_eq!(extract_ksj_id("L01-87_05_GML.zip").unwrap(), ("L01", 1987));
        assert_eq!(extract_ksj_id("L02-09_02_GML.zip").unwrap(), ("L02", 2009));
        assert_eq!(
            extract_ksj_id("L03-a-21_5339-jgd2011_GML.zip").unwrap(),
            ("L03-a", 2021)
        );
        assert_eq!(
            extract_ksj_id("L03-b-21_5336-jgd2011_GML.zip").unwrap(),
            ("L03-b", 2021)
        );
        assert_eq!(
            extract_ksj_id("L03-b-c-21_5339-jgd2011_GML.zip").unwrap(),
            ("L03-b-c", 2021)
        );
        assert_eq!(
            extract_ksj_id("L03-b-u-21_5539-jgd2011_GML.zip").unwrap(),
            ("L03-b-u", 2021)
        );
        // L03-b_r is for raster, so ignore.
        assert_eq!(
            extract_ksj_id("L05-1-09_04_GML.zip").unwrap(),
            ("L05-1", 2009)
        );
        assert_eq!(
            extract_ksj_id("L05-2-09_04_GML.zip").unwrap(),
            ("L05-2", 2009)
        );
        assert_eq!(extract_ksj_id("N02-05_GML.zip").unwrap(), ("N02", 2005));
        assert_eq!(
            extract_ksj_id("N03-170101_32_GML.zip").unwrap(),
            ("N03", 2017)
        );
        assert_eq!(
            extract_ksj_id("N04-78_5439-tky_GML.zip").unwrap(),
            ("N04", 1978)
        );
        assert_eq!(extract_ksj_id("N05-15_GML.zip").unwrap(), ("N05", 2015));
        assert_eq!(extract_ksj_id("N06-16_GML.zip").unwrap(), ("N06", 2016));
        assert_eq!(extract_ksj_id("N07-22_GML.zip").unwrap(), ("N07", 2022));
        assert_eq!(extract_ksj_id("N08-18_GML.zip").unwrap(), ("N08", 2018));
        assert_eq!(extract_ksj_id("N09-12_GML.zip").unwrap(), ("N09", 2012));
        assert_eq!(extract_ksj_id("N10-20_19_GML.zip").unwrap(), ("N10", 2020));
        assert_eq!(extract_ksj_id("N11-13_38.zip").unwrap(), ("N11", 2013));
        assert_eq!(extract_ksj_id("N12-21_37_GML.zip").unwrap(), ("N12", 2021));
        assert_eq!(extract_ksj_id("P02-90_43_GML.zip").unwrap(), ("P02", 1990));
        assert_eq!(extract_ksj_id("P03-13.zip").unwrap(), ("P03", 2013));
        assert_eq!(extract_ksj_id("P04-14_47_GML.zip").unwrap(), ("P04", 2014));
        assert_eq!(extract_ksj_id("P05-10_27_GML.zip").unwrap(), ("P05", 2010));
        assert_eq!(extract_ksj_id("P07-15_29_GML.zip").unwrap(), ("P07", 2015));
        assert_eq!(
            extract_ksj_id("P09-10_4830-jgd_GML.zip").unwrap(),
            ("P09", 2010)
        );
        assert_eq!(extract_ksj_id("P11-22_01_SHP.zip").unwrap(), ("P11", 2022));
        assert_eq!(extract_ksj_id("P12-14_17_GML.zip").unwrap(), ("P12", 2014));
        assert_eq!(extract_ksj_id("P13-11_42_GML.zip").unwrap(), ("P13", 2011));
        assert_eq!(extract_ksj_id("P14-15_33_GML.zip").unwrap(), ("P14", 2015));
        assert_eq!(extract_ksj_id("P15-12_28_GML.zip").unwrap(), ("P15", 2012));
        assert_eq!(extract_ksj_id("P16-12_22_GML.zip").unwrap(), ("P16", 2012));
        assert_eq!(extract_ksj_id("P17-12_26_GML.zip").unwrap(), ("P17", 2012));
        assert_eq!(extract_ksj_id("P18-12_30_GML.zip").unwrap(), ("P18", 2012));
        assert_eq!(extract_ksj_id("P19-12_44_GML.zip").unwrap(), ("P19", 2012));
        assert_eq!(extract_ksj_id("P20-12_08_GML.zip").unwrap(), ("P20", 2012));
        assert_eq!(extract_ksj_id("P21-12_39_GML.zip").unwrap(), ("P21", 2012));
        assert_eq!(extract_ksj_id("P22-12_11_GML.zip").unwrap(), ("P22", 2012));
        assert_eq!(extract_ksj_id("P23-12_22_GML.zip").unwrap(), ("P23", 2012));
        assert_eq!(extract_ksj_id("P24-12_GML.zip").unwrap(), ("P24", 2012));
        assert_eq!(extract_ksj_id("P26-13_34.zip").unwrap(), ("P26", 2013));
        assert_eq!(extract_ksj_id("P27-13_21.zip").unwrap(), ("P27", 2013));
        assert_eq!(extract_ksj_id("P28-13_03.zip").unwrap(), ("P28", 2013));
        assert_eq!(extract_ksj_id("P29-13_01.zip").unwrap(), ("P29", 2013));
        assert_eq!(extract_ksj_id("P30-13_37.zip").unwrap(), ("P30", 2013));
        assert_eq!(extract_ksj_id("P31-13_13.zip").unwrap(), ("P31", 2013));
        assert_eq!(extract_ksj_id("P32-14_35_GML.zip").unwrap(), ("P32", 2014));
        assert_eq!(extract_ksj_id("P33-14_44_GML.zip").unwrap(), ("P33", 2014));
        assert_eq!(extract_ksj_id("P34-14_03_GML.zip").unwrap(), ("P34", 2014));
        assert_eq!(extract_ksj_id("P35-18_GML.zip").unwrap(), ("P35", 2018));
        assert_eq!(extract_ksj_id("P36-23_01_SHP.zip").unwrap(), ("P36", 2023));
        assert_eq!(
            extract_ksj_id("S05-a-12_KINKI_GML.zip").unwrap(),
            ("S05-a", 2012)
        );
        assert_eq!(
            extract_ksj_id("S05-b-10_KINKI_GML.zip").unwrap(),
            ("S05-b", 2010)
        );
        assert_eq!(
            extract_ksj_id("S05-c-10_SYUTO_GML.zip").unwrap(),
            ("S05-c", 2010)
        );
        assert_eq!(extract_ksj_id("S05-d-04_GML.zip").unwrap(), ("S05-d", 2004));
        assert_eq!(extract_ksj_id("S10a-14_GML.zip").unwrap(), ("S10a", 2014));
        assert_eq!(extract_ksj_id("S10b-14_GML.zip").unwrap(), ("S10b", 2014));
        assert_eq!(extract_ksj_id("S12-19_GML.zip").unwrap(), ("S12", 2019));
        assert_eq!(extract_ksj_id("W01-05_GML.zip").unwrap(), ("W01", 2005));
        assert_eq!(extract_ksj_id("W05-08_21_GML.zip").unwrap(), ("W05", 2008));
        assert_eq!(extract_ksj_id("W09-05_GML.zip").unwrap(), ("W09", 2005));
    }
}
