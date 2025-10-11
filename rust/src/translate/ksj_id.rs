use std::sync::LazyLock;

use regex::Regex;

use crate::Ksj2GpError;

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Z][0-9]{2}[a-z]?[0-9]?(?:-[a-z12])?(?:-[cu])?|m1000|m500)-([0-9]{2})").unwrap()
});

/// Guess KSJ ID and year from the ZIP filename
pub fn extract_ksj_id(filename: &str) -> Result<(String, u16), Ksj2GpError> {
    // Handle mesh first, because these are out of the pattern...

    if filename.starts_with("1km_mesh_suikei_2018") {
        return Ok(("mesh1000h30".to_string(), 2018));
    }

    if filename.starts_with("1km_mesh_2024") {
        return Ok(("mesh1000r6".to_string(), 2024));
    }

    if filename.starts_with("500m_mesh_suikei_2018") {
        return Ok(("mesh500h30".to_string(), 2018));
    }

    if filename.starts_with("500m_mesh_2024") {
        return Ok(("mesh500r6".to_string(), 2018));
    }
    if filename.starts_with("250m_mesh_2024") {
        return Ok(("mesh250r6".to_string(), 2018));
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

    Ok((id.to_string(), year))
}

#[cfg(test)]
mod tests {
    use super::extract_ksj_id;

    #[test]
    fn test_extract_ksj_id() {
        let cases: &[(&str, &str, u16)] = &[
            ("m1000-17_27_GML.zip", "mesh1000", 2017),
            ("m500-17_44_GML.zip", "mesh500", 2017),
            ("1km_mesh_suikei_2018_shape_19.zip", "mesh1000h30", 2018),
            ("1km_mesh_2024_04_GML.zip", "mesh1000r6", 2024),
            ("500m_mesh_suikei_2018_shape_23.zip", "mesh500h30", 2018),
            ("500m_mesh_2024_GML.zip", "mesh500r6", 2018),
            ("250m_mesh_2024_GML.zip", "mesh250r6", 2018),
            ("A09-06_02_GML.zip", "A09", 2006),
            ("A10-06_03_GML.zip", "A10", 2006),
            ("A11-15_31_GML.zip", "A11", 2015),
            ("A12-06_04_GML.zip", "A12", 2006),
            ("A13-06_14_GML.zip", "A13", 2006),
            ("A15-15_12_GML.zip", "A15", 2015),
            ("A16-75_34_GML.zip", "A16", 1975),
            ("A17-901001_20_GML.zip", "A17", 1990),
            ("A18-051001_16_GML.zip", "A18", 2005),
            ("A18s-a-10_GML.zip", "A18s_a", 2010),
            ("A19-651001_45_GML.zip", "A19", 1965),
            ("A19s-05_17_GML.zip", "A19s", 2005),
            ("A19s-a-05_17_GML.zip", "A19s", 2005),
            ("A20-601001_46_GML.zip", "A20", 1960),
            ("A20s-05_46_GML.zip", "A20s", 2005),
            ("A21-001001_13_GML.zip", "A21", 2000),
            ("A21s-10_13_GML.zip", "A21s", 2010),
            ("A22-070402_32_GML.zip", "A22", 2007),
            ("A22-m-14_33_GML.zip", "A22-m", 2014),
            ("A22s-00_19_GML.zip", "A22s", 2000),
            ("A23-051001_46_GML.zip", "A23", 2005),
            ("A24-901001_36_GML.zip", "A24", 1990),
            ("A25-930928_35_GML.zip", "A25", 1993),
            ("A27-21_10_GML.zip", "A27", 2021),
            ("A28-10_GML.zip", "A28", 2010),
            ("A29-11_39_GML.zip", "A29", 2011),
            ("A30a5-11_5438-jgd_GML.zip", "A30a5", 2011),
            ("A30b-11_GML.zip", "A30b", 2011),
            ("A31a-24_81_10_GML.zip", "A31a", 2024),
            ("A31b-24_10_5540_GML.zip", "A31b", 2024),
            ("A32-21_29_GML.zip", "A32", 2021),
            ("A33-24_00_GML.zip", "A33", 2024),
            ("A34-180316_GML.zip", "A34", 2018),
            ("A35a-14_46_GML.zip", "A35a", 2014),
            ("A35b-14_26_GML.zip", "A35b", 2014),
            ("A35c-14_07_GML.zip", "A35c", 2014),
            ("A37-15_39_GML.zip", "A37", 2015),
            ("A38-20_27_GML.zip", "A38", 2020),
            ("A39-15_GML.zip", "A39", 2015),
            ("A40-20_14_GML.zip", "A40", 2020),
            ("A42-18_GML.zip", "A42", 2018),
            ("A43-18_GML.zip", "A43", 2018),
            ("A44-18_GML.zip", "A44", 2018),
            ("A45-19_41_GML.zip", "A45", 2019),
            ("A46-20_04_GML.zip", "A46", 2020),
            ("A47-21_52_GML.zip", "A47", 2021),
            ("A48-21_17_GML.zip", "A48", 2021),
            ("A49-20_13_GML.zip", "A49", 2020),
            ("A50-20_14_GML.zip", "A50", 2020),
            ("A51-24_20_GML.zip", "A51", 2024),
            ("A52-23_25_GML.zip", "A52", 2023),
            ("A53-23-81_GML.zip", "A53", 2023),
            ("A54-23_GML.zip", "A54", 2023),
            ("A55-22_01100_SHP.zip", "A55", 2022),
            ("C02-06_GML.zip", "C02", 2006),
            ("C09-06_GML.zip", "C09", 2006),
            ("C23-06_28_GML.zip", "C23", 2006),
            ("C28-13.zip", "C28", 2013),
            ("G02-12_5238-jgd_GML.zip", "G02", 2012),
            ("G04-a-11_5940-jgd_GML.zip", "G04a", 2011),
            ("G04-c-11_5439-jgd_GML.zip", "G04c", 2011),
            ("G04-d-11_5338-jgd_GML.zip", "G04d", 2011),
            ("G08-15_43_GML.zip", "G08", 2015),
            ("L01-87_05_GML.zip", "L01", 1987),
            ("L02-09_02_GML.zip", "L02", 2009),
            ("L03-a-21_5339-jgd2011_GML.zip", "L03-a", 2021),
            ("L03-b-21_5336-jgd2011_GML.zip", "L03-b", 2021),
            ("L03-b-c-21_5339-jgd2011_GML.zip", "L03-b-c", 2021),
            ("L03-b-u-21_5539-jgd2011_GML.zip", "L03-b-u", 2021),
            // L03-b_r is for raster, so ignore.
            ("L05-1-09_04_GML.zip", "L05-1", 2009),
            ("L05-2-09_04_GML.zip", "L05-2", 2009),
            ("N02-05_GML.zip", "N02", 2005),
            ("N03-170101_32_GML.zip", "N03", 2017),
            ("N04-78_5439-tky_GML.zip", "N04", 1978),
            ("N05-15_GML.zip", "N05", 2015),
            ("N06-16_GML.zip", "N06", 2016),
            ("N07-22_GML.zip", "N07", 2022),
            ("N08-18_GML.zip", "N08", 2018),
            ("N09-12_GML.zip", "N09", 2012),
            ("N10-20_19_GML.zip", "N10", 2020),
            ("N11-13_38.zip", "N11", 2013),
            ("N12-21_37_GML.zip", "N12", 2021),
            ("P02-90_43_GML.zip", "P02", 1990),
            ("P03-13.zip", "P03", 2013),
            ("P04-14_47_GML.zip", "P04", 2014),
            ("P05-10_27_GML.zip", "P05", 2010),
            ("P07-15_29_GML.zip", "P07", 2015),
            ("P09-10_4830-jgd_GML.zip", "P09", 2010),
            ("P11-22_01_SHP.zip", "P11", 2022),
            ("P12-14_17_GML.zip", "P12", 2014),
            ("P13-11_42_GML.zip", "P13", 2011),
            ("P14-15_33_GML.zip", "P14", 2015),
            ("P15-12_28_GML.zip", "P15", 2012),
            ("P16-12_22_GML.zip", "P16", 2012),
            ("P17-12_26_GML.zip", "P17", 2012),
            ("P18-12_30_GML.zip", "P18", 2012),
            ("P19-12_44_GML.zip", "P19", 2012),
            ("P20-12_08_GML.zip", "P20", 2012),
            ("P21-12_39_GML.zip", "P21", 2012),
            ("P22-12_11_GML.zip", "P22", 2012),
            ("P23-12_22_GML.zip", "P23", 2012),
            ("P24-12_GML.zip", "P24", 2012),
            ("P26-13_34.zip", "P26", 2013),
            ("P27-13_21.zip", "P27", 2013),
            ("P28-13_03.zip", "P28", 2013),
            ("P29-13_01.zip", "P29", 2013),
            ("P30-13_37.zip", "P30", 2013),
            ("P31-13_13.zip", "P31", 2013),
            ("P32-14_35_GML.zip", "P32", 2014),
            ("P33-14_44_GML.zip", "P33", 2014),
            ("P34-14_03_GML.zip", "P34", 2014),
            ("P35-18_GML.zip", "P35", 2018),
            ("P36-23_01_SHP.zip", "P36", 2023),
            ("S05-a-12_KINKI_GML.zip", "S05-a", 2012),
            ("S05-b-10_KINKI_GML.zip", "S05-b", 2010),
            ("S05-c-10_SYUTO_GML.zip", "S05-c", 2010),
            ("S05-d-04_GML.zip", "S05-d", 2004),
            ("S10a-14_GML.zip", "S10a", 2014),
            ("S10b-14_GML.zip", "S10b", 2014),
            ("S12-19_GML.zip", "S12", 2019),
            ("W01-05_GML.zip", "W01", 2005),
            ("W05-08_21_GML.zip", "W05", 2008),
            ("W09-05_GML.zip", "W09", 2005),
        ];

        for &(filename, expected_id, expected_year) in cases {
            let (actual_id, actual_year) = extract_ksj_id(filename).unwrap();
            assert_eq!(
                (actual_id.as_str(), actual_year),
                (expected_id, expected_year)
            );
        }
    }
}
