use crate::translate::data::codelists::*;
use std::{collections::HashMap, sync::LazyLock};

type Codelist = LazyLock<HashMap<&'static str, &'static str>>;
pub(crate) fn get_codelist_map(
    col_id: &str,
    year: u16,
    target_shp: &str, // This is needed to distinguish A42
) -> Option<&'static Codelist> {
    // Handle special cases
    match (col_id, year) {
        ("A03_006", _) => {
            if target_shp.contains("KINKI") {
                return CODELISTS_MAP.get("A03SectionTypeCdKinki").copied();
            }
            if target_shp.contains("CHUBU") {
                // コード一覧のリンクは cyubu だけどファイル名は CHUBU...
                return CODELISTS_MAP.get("A03SectionTypeCdCyubu").copied();
            }
            if target_shp.contains("SYUTO") {
                return CODELISTS_MAP.get("A03SectionTypeCdSyuto").copied();
            }
        }
        // A42: shapefile が複数入っていて、片方にしかない
        ("A42_005", _) => {
            if target_shp.ends_with("Spacial_Preservation_Area_of_Historic_Landscape.shp") {
                return CODELISTS_MAP.get("A42HistoricalDistrictType").copied();
            }
        }
        ("YoutoCode", _) => {
            if target_shp.contains("youto") {
                return Some(&A55_NORMAL_YOUTO_CODE);
            }
            if target_shp.contains("tkbt") {
                return Some(&A55_SPECIAL_YOUTO_CODE);
            }
        }
        // L01: 用途区分
        ("L01_001" | "L01_003", ..=2023) | ("L01_002" | "L01_005", 2024..) => {
            return CODELISTS_MAP.get("IndexNumL01").copied();
        }
        // L01: 選定状況
        ("L01_007", ..=2019) => return CODELISTS_MAP.get("SelectLandStatusL01V1").copied(),
        ("L01_007", 2020..=2021) => return CODELISTS_MAP.get("SelectLandStatusL01V2").copied(),
        ("L01_008", 2022..=2023) => return CODELISTS_MAP.get("SelectLandStatusL01V2").copied(),
        ("L01_010", 2024..) => return CODELISTS_MAP.get("SelectLandStatusL01V2").copied(),

        // L03-b: 土地利用種
        ("土地利用種", 1976) => return CODELISTS_MAP.get("LandUseCd77").copied(),
        ("土地利用種", 1987) => return CODELISTS_MAP.get("LandUseCd88").copied(),
        ("土地利用種", 1991 | 1997 | 2006) => {
            return CODELISTS_MAP.get("LandUseCdYY").copied();
        }
        ("土地利用種", 2009..) => return CODELISTS_MAP.get("LandUseCd09").copied(),
        ("土地利用種", _) => return None, // これらの年以外はないはず

        // L03-b-c: 土地利用種
        ("L03b_c_002", _) => {
            // TODO: L03b_c_004（都市地域範囲）の値によって代わるが、今のやり方では他のカラムの値にアクセスできない
            //
            // return CODELISTS_MAP.get("LandUseCd09").copied();
            // return CODELISTS_MAP.get("LandUseCd09-u").copied(),
            return None;
        }

        _ => {}
    }

    CODELISTS_MAP.get(col_id).copied()
}

#[rustfmt::skip]
static CODELISTS_MAP: LazyLock<HashMap<&'static str, &'static Codelist>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, &'static Codelist> = HashMap::with_capacity(150);

    map.entry("A03_007").or_insert(&A03_SECTION_CD);
    map.entry("A03SectionTypeCdKinki").or_insert(&A03_SECTION_TYPE_CD_KINKI);
    map.entry("A03SectionTypeCdCyubu").or_insert(&A03_SECTION_TYPE_CD_CYUBU);
    map.entry("A03SectionTypeCdSyuto").or_insert(&A03_SECTION_TYPE_CD_SYUTO);

    // A10~A13
    map.entry("LAYER_NO").or_insert(&A10_LAYER_NO);
    map.entry("AREA_CD").or_insert(&A10_AREA_CD);
    map.entry("IOSIDE_DIV").or_insert(&A10_INSIDE_DIV);

    map.entry("A15_003").or_insert(&AUTHORITY_TYPE);
    map.entry("A15_004").or_insert(&PROTECTION_AREA_TYPE);

    map.entry("A17_009").or_insert(&KASO_CD);

    map.entry("A18_008").or_insert(&SPECIFIC_AIRPORT_SPECIFIED_SITUATION_CD);

    map.entry("A19_009").or_insert(&RITO_CD);
    map.entry("A19_010").or_insert(&SPECIFIC_AIRPORT_SPECIFIED_SITUATION_CD);

    map.entry("A20_008").or_insert(&SPECIFIC_AIRPORT_SPECIFIED_SITUATION_CD);

    map.entry("A21_007").or_insert(&SPECIFIC_AIRPORT_SPECIFIED_SITUATION_CD);

    map.entry("A22_009").or_insert(&HEAVY_SNOW_TYPE_CODE);
    map.entry("A22_050001").or_insert(&POS_SPECIFIC_LEVEL);
    map.entry("A22_100005").or_insert(&AGGREGATE_UNIT_FLAG);
    map.entry("A22_100007").or_insert(&AGGREGATE_UNIT_FLAG);
    map.entry("A22_40009").or_insert(&POS_SPECIFIC_LEVEL);
    map.entry("A22_007").or_insert(&HEAVY_SNOW_TYPE_CODE);
    map.entry("A22_008").or_insert(&SPECIFIC_AIRPORT_SPECIFIED_SITUATION_CD);

    map.entry("A23_009").or_insert(&TOKUSYUDOJYO_CD_V3_0);

    map.entry("A26_005").or_insert(&SEDIMENT_DISASTERS_PRONE_AREA_CD);

    // A28
    map.entry("WHC").or_insert(&WORLD_HERITAGE_CD);

    map.entry("A31_101").or_insert(&WATER_DEPTH_CODE);
    map.entry("A31_201").or_insert(&WATER_DEPTH_CODE);
    map.entry("A31_301").or_insert(&FLOOD_DURATION_CODE);
    map.entry("A31_401").or_insert(&HAZARDOUS_AREA_CLASSIFICATION_CODE);

    map.entry("A33_001").or_insert(&CODE_OF_PHENOMENON);
    map.entry("A33_002").or_insert(&CODE_OF_ZONE_H27);
    map.entry("A33_008").or_insert(&CODE_OF_UN_SPECIFICATION);

    map.entry("A35d_009").or_insert(&LANDSCAPE_DISTRICT_TYPE);

    map.entry("A35e_009").or_insert(&LANDSCAPE_DISTRICT_TYPE);

    map.entry("A35f_007").or_insert(&LANDSCAPE_DISTRICT_TYPE);

    map.entry("A37_300007").or_insert(&AGG_UNIT_FLAG_EMER_TRANS_CD);

    map.entry("A38a_005").or_insert(&SETTING_FLAG);

    map.entry("A39_021").or_insert(&CODE_NONCOMBUSTIBLE_CD);
    map.entry("A39_025").or_insert(&CODE_DESIGNATION_CD);

    map.entry("A42HistoricalDistrictType").or_insert(&A42_HISTORICAL_DISTRICT_TYPE);

    map.entry("A45_006").or_insert(&SHOUHANSHUBAN_CD);
    map.entry("A45_025").or_insert(&RINSHUNOSAIBUN_CD);
    map.entry("A45_026").or_insert(&KINOURUIKEI_CD);
    map.entry("A45_028").or_insert(&HOANRIN_CD);
    map.entry("A45_029").or_insert(&HOANRIN_CD);
    map.entry("A45_030").or_insert(&HOANRIN_CD);
    map.entry("A45_031").or_insert(&HOANRIN_CD);
    map.entry("A45_032").or_insert(&HOGORIN_CD);
    map.entry("A45_033").or_insert(&MIDORINOKAIRO_CD);

    // A55
    map.entry("AreaCode").or_insert(&A55_AREA_CODE);
    map.entry("DistCode").or_insert(&A55_AREA_CODE); // DistCode も AreaCode と同じ
    map.entry("DouroCode").or_insert(&A55_ROAD_AND_PARK_CODE);
    map.entry("ParkCode").or_insert(&A55_ROAD_AND_PARK_CODE);

    map.entry("C02_001").or_insert(&CLASS_HARBOR1_CD);
    map.entry("C02_002").or_insert(&CLASS_HARBOR2_CD);
    map.entry("C02_006").or_insert(&ADMIN_HARBOR_CD);
    map.entry("C02_010").or_insert(&MARITIME_ORG_CD);

    map.entry("C09_004").or_insert(&CLASS_FISH_PORT_CD);
    map.entry("C09_005").or_insert(&FISH_PORT_ADMIN_CD);

    map.entry("C23_002").or_insert(&ADMIN_SEA_LINE_CD);
    map.entry("C23_005").or_insert(&ADMIN_CON_AREA_CD);

    map.entry("C28_003").or_insert(&INSTALL_AIRPORT_CD_V2_3);
    map.entry("C28_006").or_insert(&INSTALL_ADMIN_CD_V2_3);
    map.entry("C28_007").or_insert(&INSTALL_ADMIN_CD_V2_3);
    map.entry("C28_011").or_insert(&REGULAR_FLIGHT_CD);

    map.entry("G04a_005").or_insert(&UNDERSEA);
    map.entry("G04a_007").or_insert(&DIRECTION);
    map.entry("G04a_009").or_insert(&DIRECTION);

    map.entry("G04c_005").or_insert(&UNDERSEA);
    map.entry("G04c_007").or_insert(&DIRECTION);
    map.entry("G04c_009").or_insert(&DIRECTION);

    map.entry("G04d_005").or_insert(&UNDERSEA);
    map.entry("G04d_007").or_insert(&DIRECTION);
    map.entry("G04d_009").or_insert(&DIRECTION);

    map.entry("G08_003").or_insert(&REFERENCE_DATA_CD);

    // L01
    map.entry("IndexNumL01").or_insert(&INDEX_NUM_L01);
    map.entry("SelectLandStatusL01V1").or_insert(&SELECT_LAND_STATUS_L01V1);
    map.entry("SelectLandStatusL01V2").or_insert(&SELECT_LAND_STATUS_L01V2);

    // L03-b
    map.entry("LandUseCd77").or_insert(&LAND_USE_CD_77);
    map.entry("LandUseCd88").or_insert(&LAND_USE_CD_88);
    map.entry("LandUseCdYY").or_insert(&LAND_USE_CD_YY);
    map.entry("LandUseCd09").or_insert(&LAND_USE_CD_09);
    map.entry("LandUseCd09-u").or_insert(&LAND_USE_CD_09_U);

    map.entry("L05_013").or_insert(&USE_DISTRICT);

    map.entry("N02_001").or_insert(&RAILWAY_CLASS_CD);
    map.entry("N02_002").or_insert(&INSTITUTION_TYPE_CD);

    map.entry("N05_001").or_insert(&RAILWAY_CLASS2_CD);
    map.entry("N05_007").or_insert(&RAILWAY_TRANSITION_CD);

    map.entry("N06_005").or_insert(&HIGHWAY_TRANSITION_CD);
    map.entry("N06_008").or_insert(&HIGHWAY_CAT_CD);
    map.entry("N06_009").or_insert(&HIGHWAY_USE_CD);
    map.entry("N06_016").or_insert(&HIGHWAY_TRANSITION_CD);
    map.entry("N06_019").or_insert(&HIGHWAY_CON_CD);

    map.entry("N07_001").or_insert(&BUS_CLASS_CD);

    map.entry("N08_002").or_insert(&AIRPORT_CAT_CD);
    map.entry("N08_004").or_insert(&INSTALL_ADMIN_CD_V2_3);
    map.entry("N08_005").or_insert(&INSTALL_ADMIN_CD_V2_3);
    map.entry("N08_010").or_insert(&REGULAR_FLIGHT_CD);
    map.entry("N08_011").or_insert(&AIR_JET_CD);
    map.entry("N08_013").or_insert(&AIRPORT_USE_CD);
    map.entry("N08_017").or_insert(&AIRPORT_TRANSITION_CD);
    map.entry("N08_021").or_insert(&AIRPORT_USE_CD);

    map.entry("N10_002").or_insert(&URGENT_ROAD_CD);
    map.entry("N10_003").or_insert(&ROAD_CATEGORY_CD);

    map.entry("N11_002").or_insert(&AVIATION_ACT_CD);

    map.entry("P03_0004").or_insert(&UNDER_CONSTRUCTION);
    map.entry("P03_0102").or_insert(&HYDROELECTRIC_POWER_PLANT_TYPE);
    map.entry("P03_0209").or_insert(&PUMPINGUP_TYPE);
    map.entry("P03_0404").or_insert(&THERMAL_POWER_ENGINE);
    map.entry("P03_0602").or_insert(&FURNACE_TYPE);
    map.entry("P03_0901").or_insert(&BIOMASS_TYPE);

    map.entry("P04_001").or_insert(&MED_CLASS_CD);
    map.entry("P04_007").or_insert(&EST_CLASS_CD);

    // P05
    map.entry("PubOfficeCd").or_insert(&PUB_OFFICE_CD);

    map.entry("P07_001").or_insert(&FUEL_STORE_CD);

    map.entry("P11_002").or_insert(&BUS_CLASS_CD);

    map.entry("P12_007").or_insert(&TOURISM_RESOURCE_CATEGORY_CD);

    map.entry("P13_004").or_insert(&CITY_PARK_CD);

    map.entry("P13_009").or_insert(&URBAN_PLANNING_DECIDED);

    map.entry("P14_004").or_insert(&PUB_FAC_MACLASS_CD);
    map.entry("P14_005").or_insert(&PUB_FAC_MICLASS_CD_WF);
    map.entry("P14_008").or_insert(&PUB_FAC_ADMIN_CD);

    map.entry("P15_003").or_insert(&FACILITIES_CLASSIFICATION_CD);
    map.entry("P15_017").or_insert(&INDUSTRIAL_WASTE_DISPOSAL);
    map.entry("P15_018").or_insert(&INDUSTRIAL_WASTE_SPECIAL_TREATMENT);

    map.entry("P16_002").or_insert(&RESEARCH_INSTITUTION_CD);

    map.entry("P17_003").or_insert(&FIREHOUSE_TYPE);

    map.entry("P18_003").or_insert(&POLICE_STATION_CD);

    map.entry("P19_004").or_insert(&NATURALSCENE_CD);
    map.entry("P19_006").or_insert(&NATURALFEATURE_CD);

    map.entry("P21A_003").or_insert(&WATER_SUPPLY_TYPE);

    map.entry("P24_011").or_insert(&REFERECED_FROM_AGRI);

    map.entry("P26_009").or_insert(&BUSINESS_TECH_CD);

    map.entry("P27_002").or_insert(&PUB_FAC_MACLASS_CD);
    map.entry("P27_003").or_insert(&PUB_FAC_MINCLASS_CD);
    map.entry("P27_004").or_insert(&CULTURE_FAC_CD);
    map.entry("P27_007").or_insert(&ADMIN_CODE);

    map.entry("P28_002").or_insert(&PUB_FAC_MACLASS_CD);
    map.entry("P28_003").or_insert(&PUB_FAC_MINCLASS_CD);
    map.entry("P28_007").or_insert(&ADMIN_CODE);

    map.entry("P29_002").or_insert(&PUB_FAC_MACLASS_CD);
    map.entry("P29_003").or_insert(&PUB_FAC_MINCLASS_CD);
    map.entry("P29_004").or_insert(&SCHOOL_CLASS_CD);
    map.entry("P29_007").or_insert(&ADMIN_CODE);

    map.entry("P30_002").or_insert(&PUB_FAC_MACLASS_CD);
    map.entry("P30_003").or_insert(&PUB_FAC_MINCLASS_CD);
    map.entry("P30_004").or_insert(&POST_OFFICE_CD);
    map.entry("P30_007").or_insert(&ADMIN_CODE);

    map.entry("P31_002").or_insert(&DISTRIBUTION_CENTER_CD);
    map.entry("P31_003").or_insert(&DISTRIBUTION_CD);
    map.entry("P31_006").or_insert(&ENTREPRENEUR_CD);

    map.entry("P32_004").or_insert(&LARGE_CLASSIFICATION_CD);
    map.entry("P32_005").or_insert(&SMALL_CLASSIFICATION_CD);
    map.entry("P32_009").or_insert(&POINT_CLASSIFICATION_CD);

    map.entry("P33_004").or_insert(&FACILITY_TYPE_CODE);
    map.entry("P33_014").or_insert(&COMMUNITY_CENTER_TYPE);
    map.entry("P33_041").or_insert(&POINT_CLASSIFICATION_CODE);

    map.entry("P34_002").or_insert(&PUB_OFFICE_CLASS_CD);

    map.entry("S12_004").or_insert(&RAILWAY_CLASS_CD);
    map.entry("S12_005").or_insert(&INSTITUTION_TYPE_CD);
    map.entry("S12_006").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_007").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_010").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_011").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_014").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_015").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_018").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_019").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_022").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_023").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_026").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_027").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_030").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_031").or_insert(&RAILWAY_EXISTENCE_CD);
    map.entry("S12_034").or_insert(&RAILWAY_DUPLICATE_CD);
    map.entry("S12_035").or_insert(&RAILWAY_EXISTENCE_CD);

    map.entry("W01_005").or_insert(&DAM_TYPE_CD);
    map.entry("W01_006").or_insert(&DAMP_URPOSE_CD);
    map.entry("W01_011").or_insert(&DAM_INSTITUTION_CD);
    map.entry("W01_014").or_insert(&LOCATION_ACCURACY_CD);

    map.entry("W05_001").or_insert(&WATER_SYSTEM_CODE_CD);
    map.entry("W05_003").or_insert(&SECTION_TYPE);
    map.entry("W05_005").or_insert(&ORIGINAL_DATA_CODE_CD);

    map
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    fn assert_codelist_label(
        col_id: &str,
        year: u16,
        target_shp: &str,
        code: &str,
        expected_label: &str,
    ) {
        let map = get_codelist_map(col_id, year, target_shp)
            .unwrap_or_else(|| panic!("missing map for {target_shp}"));
        let map = map.deref();
        let actual = map
            .get(code)
            .copied()
            .unwrap_or_else(|| panic!("missing code {code} for {target_shp}"));
        assert_eq!(actual, expected_label);
    }

    #[test]
    fn test_a03() {
        let cases = [
            ("A03-03_KINKI-g_ThreeMajorMetroPlanArea.shp", "既成都市区域"),
            (
                "A03-03_CHUBU-g_ThreeMajorMetroPlanArea.shp",
                "都市整備区域(［保全区域］との重複無し",
            ),
            ("A03-03_SYUTO-g_ThreeMajorMetroPlanArea.shp", "既成市街地"),
        ];

        for (target_shp, expected_label) in cases {
            assert_codelist_label("A03_006", 2024, target_shp, "1", expected_label);
        }
    }

    #[test]
    fn test_a42_special_shapefile() {
        assert_codelist_label(
            "A42_005",
            2024,
            "Spacial_Preservation_Area_of_Historic_Landscape.shp",
            "2",
            "第１種歴史的風土保存地区（明日香村のみ）",
        );
    }
}
