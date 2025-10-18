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
        // L01: 用途区分
        ("L01_001" | "L01_003", ..=2023) | ("L01_002" | "L01_005", 2024..) => {
            return CODELISTS_MAP.get("IndexNumL01").copied();
        }
        // L01: 選定状況
        ("L01_007", ..=2019) => return CODELISTS_MAP.get("SelectLandStatusL01V1").copied(),
        ("L01_007", 2020..=2021) => return CODELISTS_MAP.get("SelectLandStatusL01V2").copied(),
        ("L01_008", 2022..=2023) => return CODELISTS_MAP.get("SelectLandStatusL01V2").copied(),
        ("L01_010", 2024..) => return CODELISTS_MAP.get("SelectLandStatusL01V2").copied(),
        _ => {}
    }

    CODELISTS_MAP.get(col_id).copied()
}

#[rustfmt::skip]
static CODELISTS_MAP: LazyLock<HashMap<&'static str, &'static Codelist>> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, &'static Codelist> = HashMap::with_capacity(150);

    map.entry("A03_007").or_insert(&A03SectionCd);
    map.entry("A03SectionTypeCdKinki").or_insert(&A03SectionTypeCdKinki);
    map.entry("A03SectionTypeCdCyubu").or_insert(&A03SectionTypeCdCyubu);
    map.entry("A03SectionTypeCdSyuto").or_insert(&A03SectionTypeCdSyuto);

    // A10~A13
    map.entry("LAYER_NO").or_insert(&A10LayerNo);
    map.entry("AREA_CD").or_insert(&A10AreaCode);
    map.entry("IOSIDE_DIV").or_insert(&A10InsideDiv);

    map.entry("A15_003").or_insert(&AuthorityType);
    map.entry("A15_004").or_insert(&ProtectionAreaType);

    map.entry("A17_009").or_insert(&KasoCd);

    map.entry("A18_008").or_insert(&SpecificAirPortSpecifiedSituationCd);

    map.entry("A19_009").or_insert(&RitoCd);
    map.entry("A19_010").or_insert(&SpecificAirPortSpecifiedSituationCd);

    map.entry("A20_008").or_insert(&SpecificAirPortSpecifiedSituationCd);

    map.entry("A21_007").or_insert(&SpecificAirPortSpecifiedSituationCd);

    map.entry("A22_009").or_insert(&HeavySnowTypeCode);
    map.entry("A22_050001").or_insert(&PosSpecificLevel);
    map.entry("A22_100005").or_insert(&AggregateUnitFlag);
    map.entry("A22_100007").or_insert(&AggregateUnitFlag);
    map.entry("A22_40009").or_insert(&PosSpecificLevel);
    map.entry("A22_007").or_insert(&HeavySnowTypeCode);
    map.entry("A22_008").or_insert(&SpecificAirPortSpecifiedSituationCd);

    map.entry("A23_009").or_insert(&TokusyudojyoCdV3_0);

    map.entry("A26_005").or_insert(&SedimentDisastersProneAreaCd);

    // A28
    map.entry("WHC").or_insert(&WorldHeritageCd);

    map.entry("A31_101").or_insert(&WaterDepthCode);
    map.entry("A31_201").or_insert(&WaterDepthCode);
    map.entry("A31_301").or_insert(&FloodDurationCode);
    map.entry("A31_401").or_insert(&HazardousAreaClassificationCode);

    map.entry("A33_001").or_insert(&CodeOfPhenomenon);
    map.entry("A33_002").or_insert(&CodeOfZoneH27);
    map.entry("A33_008").or_insert(&CodeOfUnSpecification);

    map.entry("A35d_009").or_insert(&LandscapeDistrictType);

    map.entry("A35e_009").or_insert(&LandscapeDistrictType);

    map.entry("A35f_007").or_insert(&LandscapeDistrictType);

    map.entry("A37_300007").or_insert(&AggUnitFlagEmerTransCd);

    map.entry("A38a_005").or_insert(&SettingFlag);

    map.entry("A39_021").or_insert(&CodeNoncombustibleCd);
    map.entry("A39_025").or_insert(&CodeDesignationCd);

    map.entry("A42HistoricalDistrictType").or_insert(&A42HistoricalDistrictType);

    map.entry("A45_006").or_insert(&ShouhanshubanCd);
    map.entry("A45_025").or_insert(&RinshunosaibunCd);
    map.entry("A45_026").or_insert(&KinouruikeiCd);
    map.entry("A45_028").or_insert(&HoanrinCd);
    map.entry("A45_029").or_insert(&HoanrinCd);
    map.entry("A45_030").or_insert(&HoanrinCd);
    map.entry("A45_031").or_insert(&HoanrinCd);
    map.entry("A45_032").or_insert(&HogorinCd);
    map.entry("A45_033").or_insert(&midorinokairoCd);

    map.entry("C02_001").or_insert(&ClassHarbor1Cd);
    map.entry("C02_002").or_insert(&ClassHarbor2Cd);
    map.entry("C02_006").or_insert(&AdminHarborCd);
    map.entry("C02_010").or_insert(&MaritimeOrgCd);

    map.entry("C09_004").or_insert(&ClassFishPortCd);
    map.entry("C09_005").or_insert(&FishPortAdminCd);

    map.entry("C23_002").or_insert(&AdminSeaLineCd);

    map.entry("C28_003").or_insert(&InstallAirPortCdV2_3);
    map.entry("C28_006").or_insert(&InstallAdminCdV2_3);
    map.entry("C28_007").or_insert(&InstallAdminCdV2_3);
    map.entry("C28_011").or_insert(&RegularFlightCd);

    map.entry("G04a_005").or_insert(&Undersea);
    map.entry("G04a_007").or_insert(&Direction);
    map.entry("G04a_009").or_insert(&Direction);

    map.entry("G04c_005").or_insert(&Undersea);
    map.entry("G04c_007").or_insert(&Direction);
    map.entry("G04c_009").or_insert(&Direction);

    map.entry("G04d_005").or_insert(&Undersea);
    map.entry("G04d_007").or_insert(&Direction);
    map.entry("G04d_009").or_insert(&Direction);

    map.entry("G08_003").or_insert(&ReferenceDataCd);

    // L01
    map.entry("IndexNumL01").or_insert(&IndexNumL01);
    map.entry("SelectLandStatusL01V1").or_insert(&SelectLandStatusL01V1);
    map.entry("SelectLandStatusL01V2").or_insert(&SelectLandStatusL01V2);

    map.entry("L03b_c_002").or_insert(&LandUseCd09Tweaked);

    map.entry("L05_013").or_insert(&UseDistrict);

    map.entry("N02_001").or_insert(&RailwayClassCd);
    map.entry("N02_002").or_insert(&InstitutionTypeCd);

    map.entry("N05_001").or_insert(&RailwayClass2Cd);
    map.entry("N05_007").or_insert(&RailwayTransitionCd);

    map.entry("N06_005").or_insert(&HighwayTransitionCd);
    map.entry("N06_008").or_insert(&HighwayCatCd);
    map.entry("N06_009").or_insert(&HighwayUseCd);
    map.entry("N06_016").or_insert(&HighwayTransitionCd);
    map.entry("N06_019").or_insert(&HighwayConCd);

    map.entry("N07_001").or_insert(&BusClassCd);

    map.entry("N08_002").or_insert(&AirportCatCdHtml);
    map.entry("N08_004").or_insert(&InstallAdminCdV2_3);
    map.entry("N08_005").or_insert(&InstallAdminCdV2_3);
    map.entry("N08_010").or_insert(&RegularFlightCd);
    map.entry("N08_011").or_insert(&AirJetCd);
    map.entry("N08_013").or_insert(&AirportUseCd);
    map.entry("N08_017").or_insert(&AirportTransitionCd);
    map.entry("N08_021").or_insert(&AirportUseCd);

    map.entry("N10_002").or_insert(&UrgentRoadCd);
    map.entry("N10_003").or_insert(&RoadCategoryCd);

    map.entry("N11_002").or_insert(&AviationActCd);

    map.entry("P03_0004").or_insert(&UnderConstruction);
    map.entry("P03_0102").or_insert(&HydroelectricPowerPlantType);
    map.entry("P03_0209").or_insert(&PumpingupType);
    map.entry("P03_0404").or_insert(&ThermalPowerEngine);
    map.entry("P03_0602").or_insert(&FurnaceType);
    map.entry("P03_0901").or_insert(&BiomassType);

    map.entry("P04_001").or_insert(&MedClassCd);
    map.entry("P04_007").or_insert(&EstClassCd);

    // P05
    map.entry("PubOfficeCd").or_insert(&PubOfficeCd);

    map.entry("P07_001").or_insert(&FuelStoreCd);

    map.entry("P11_002").or_insert(&BusClassCd);

    map.entry("P12_007").or_insert(&TourismResourceCategoryCd);

    map.entry("P13_004").or_insert(&CityParkCd);

    map.entry("P13_009").or_insert(&UrbanPlanningDecided);

    map.entry("P14_004").or_insert(&PubFacMaclassCd);
    map.entry("P14_005").or_insert(&PubFacMiclassCd_wf);
    map.entry("P14_008").or_insert(&PubFacAdminCd);

    map.entry("P15_003").or_insert(&FacilitiesClassificationCd);
    map.entry("P15_017").or_insert(&IndustrialWasteDisposal);
    map.entry("P15_018").or_insert(&IndustrialWasteSpecialTreatment);

    map.entry("P16_002").or_insert(&ResearchInstitutionCd);

    map.entry("P17_003").or_insert(&FirehouseType);

    map.entry("P18_003").or_insert(&PoliceStationCd);

    map.entry("P19_004").or_insert(&NaturalsceneCd);
    map.entry("P19_006").or_insert(&NaturalfeatureCd);

    map.entry("P21A_003").or_insert(&WaterSupplyType);

    map.entry("P24_011").or_insert(&ReferecedFromAgri);

    map.entry("P26_009").or_insert(&BusinessTechCd);

    map.entry("P27_002").or_insert(&PubFacMaclassCd);
    map.entry("P27_003").or_insert(&PubFacMinclassCd);
    map.entry("P27_004").or_insert(&CultureFacCd);
    map.entry("P27_007").or_insert(&AdminCd);

    map.entry("P28_002").or_insert(&PubFacMaclassCd);
    map.entry("P28_003").or_insert(&PubFacMinclassCd);
    map.entry("P28_007").or_insert(&AdminCd);

    map.entry("P29_002").or_insert(&PubFacMaclassCd);
    map.entry("P29_003").or_insert(&PubFacMinclassCd);
    map.entry("P29_004").or_insert(&SchoolClassCd);
    map.entry("P29_007").or_insert(&AdminCd);

    map.entry("P30_002").or_insert(&PubFacMaclassCd);
    map.entry("P30_003").or_insert(&PubFacMinclassCd);
    map.entry("P30_004").or_insert(&PostOfficeCd);
    map.entry("P30_007").or_insert(&AdminCd);

    map.entry("P31_002").or_insert(&DistributionCenterCd);
    map.entry("P31_003").or_insert(&DistributionCd);
    map.entry("P31_006").or_insert(&EntrepreneurCd);

    map.entry("P32_004").or_insert(&LargeClassificationCd);
    map.entry("P32_005").or_insert(&SmallClassificationCd);
    map.entry("P32_009").or_insert(&PointClassificationCd);

    map.entry("P33_004").or_insert(&FacilityTypeCode);
    map.entry("P33_014").or_insert(&CommunityCenterType);
    map.entry("P33_041").or_insert(&PointClassificationCode);

    map.entry("P34_002").or_insert(&PubOfficeClassCd);

    map.entry("S12_004").or_insert(&RailwayClassCd);
    map.entry("S12_005").or_insert(&InstitutionTypeCd);
    map.entry("S12_006").or_insert(&RailwayDuplicateCd);
    map.entry("S12_007").or_insert(&RailwayExistenceCd);
    map.entry("S12_010").or_insert(&RailwayDuplicateCd);
    map.entry("S12_011").or_insert(&RailwayExistenceCd);
    map.entry("S12_014").or_insert(&RailwayDuplicateCd);
    map.entry("S12_015").or_insert(&RailwayExistenceCd);
    map.entry("S12_018").or_insert(&RailwayDuplicateCd);
    map.entry("S12_019").or_insert(&RailwayExistenceCd);
    map.entry("S12_022").or_insert(&RailwayDuplicateCd);
    map.entry("S12_023").or_insert(&RailwayExistenceCd);
    map.entry("S12_026").or_insert(&RailwayDuplicateCd);
    map.entry("S12_027").or_insert(&RailwayExistenceCd);
    map.entry("S12_030").or_insert(&RailwayDuplicateCd);
    map.entry("S12_031").or_insert(&RailwayExistenceCd);
    map.entry("S12_034").or_insert(&RailwayDuplicateCd);
    map.entry("S12_035").or_insert(&RailwayExistenceCd);

    map.entry("W01_005").or_insert(&DamTypeCd);
    map.entry("W01_006").or_insert(&DamPurposeCd);
    map.entry("W01_011").or_insert(&DamInstitutionCd);
    map.entry("W01_014").or_insert(&LocationAccuracyCd);

    map.entry("W05_001").or_insert(&WaterSystemCodeCd);
    map.entry("W05_003").or_insert(&SectionType);
    map.entry("W05_005").or_insert(&OriginalDataCodeCd);

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
