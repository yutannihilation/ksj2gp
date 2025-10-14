use crate::translate::data::codelists::*;
use std::{collections::HashMap, sync::LazyLock};

use crate::translate::data::colnames::COLNAMES;

macro_rules! register_codelists {
    ($map:ident, $col_id:ident, $id:ident; $($name:ident),+ $(,)?) => {
        match $id {
            $(
                CodelistId::$name => {
                    $map.entry($col_id).or_insert_with(|| {
                        LazyLock::new(|| {
                            let mut inner = HashMap::with_capacity($name.len());
                            for &(code, label) in $name {
                                inner.insert(code, label);
                            }
                            inner
                        })
                    });
                }
            )+
            _ => {}
        }
    };
}

pub(crate) fn get_codelist_map(
    col_id: &str,
    year: u16,
    target_shp: &str, // This is needed to distinguish A42
) -> Option<&'static LazyLock<HashMap<&'static str, &'static str>>> {
    // Handle special cases
    match (col_id, year) {
        ("A03_006", _) => {
            if target_shp.contains("KINKI") {
                return CODELISTS_MAP.get("A03SectionTypeCdKinki");
            }
            if target_shp.contains("CHUBU") {
                // コード一覧のリンクは cyubu だけどファイル名は CHUBU...
                return CODELISTS_MAP.get("A03SectionTypeCdCyubu");
            }
            if target_shp.contains("SYUTO") {
                return CODELISTS_MAP.get("A03SectionTypeCdSyuto");
            }
        }
        // A42: shapefile が複数入っていて、片方にしかない
        ("A42_005", _) => {
            if target_shp.ends_with("Spacial_Preservation_Area_of_Historic_Landscape.shp") {
                return CODELISTS_MAP.get("A42HistoricalDistrictType");
            }
        }
        // L01: 用途区分
        ("L01_001" | "L01_003", ..=2023) | ("L01_002" | "L01_005", 2024..) => {
            return CODELISTS_MAP.get("IndexNumL01");
        }
        // L01: 選定状況
        ("L01_007", ..=2019) => return CODELISTS_MAP.get("SelectLandStatusL01V1"),
        ("L01_007", 2020..=2021) => return CODELISTS_MAP.get("SelectLandStatusL01V2"),
        ("L01_008", 2022..=2023) => return CODELISTS_MAP.get("SelectLandStatusL01V2"),
        ("L01_010", 2024..) => return CODELISTS_MAP.get("SelectLandStatusL01V2"),
        _ => {}
    }

    CODELISTS_MAP.get(col_id)
}

static CODELISTS_MAP: LazyLock<
    HashMap<&'static str, LazyLock<HashMap<&'static str, &'static str>>>,
> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, LazyLock<HashMap<&'static str, &'static str>>> =
        HashMap::with_capacity(150);
    let normal_cases =
        COLNAMES.iter().flat_map(
            |&(col_id, (_, maybe_codelist_id))| match maybe_codelist_id {
                Some(codelist_id) => Some((col_id, codelist_id)),
                None => None,
            },
        );

    // cases that cannot be determined by the column name (e.g. L01_001 is different before and after 2023)
    let special_cases = [
        ("A03SectionTypeCdKinki", CodelistId::A03SectionTypeCdKinki),
        ("A03SectionTypeCdCyubu", CodelistId::A03SectionTypeCdCyubu),
        ("A03SectionTypeCdSyuto", CodelistId::A03SectionTypeCdSyuto),
        ("IndexNumL01", CodelistId::IndexNumL01),
        ("SelectLandStatusL01V1", CodelistId::SelectLandStatusL01V1),
        ("SelectLandStatusL01V2", CodelistId::SelectLandStatusL01V2),
        (
            "A42HistoricalDistrictType",
            CodelistId::A42HistoricalDistrictType,
        ),
    ];

    for (col_id, codelist_id) in normal_cases.chain(special_cases.into_iter()) {
        register_codelists!(
            map, col_id, codelist_id;
            A03SectionCd,
            A03SectionTypeCdKinki,
            A03SectionTypeCdCyubu,
            A03SectionTypeCdSyuto,
            A10AreaCode,
            A10LayerNo,
            A10InsideDiv,
            A42HistoricalDistrictType,
            AdminCd,
            AdminConAreaCd,
            AdminHarborCd,
            AdminSeaLineCd,
            AggregateUnitFlag,
            AggUnitFlagEmerTransCd,
            AgriculturalAreaCd,
            AirJetCd,
            AirportCatCdHtml,
            AirportTransitionCd,
            AirportUseCd,
            AuthorityType,
            AviationActCd,
            BiomassType,
            BusClassCd,
            BusinessTechCd,
            CityParkCd,
            ClassFishPortCd,
            ClassHarbor1Cd,
            ClassHarbor2Cd,
            CodeDesignationCd,
            CodeNoncombustibleCd,
            CodeOfPhenomenon,
            CodeOfUnSpecification,
            CodeOfZoneH27,
            CommunityCenterType,
            CultureFacCd,
            DamInstitutionCd,
            DamPurposeCd,
            DamTypeCd,
            Direction,
            DistributionCd,
            DistributionCenterCd,
            EntrepreneurCd,
            EstClassCd,
            FacilitiesClassificationCd,
            FacilityTypeCode,
            FirehouseType,
            FishPortAdminCd,
            FloodDurationCode,
            ForestAreaCd,
            FuelStoreCd,
            FurnaceType,
            HazardousAreaClassificationCode,
            HeavySnowTypeCode,
            HighwayCatCd,
            HighwayConCd,
            HighwayTransitionCd,
            HighwayUseCd,
            HoanrinCd,
            HogorinCd,
            HydroelectricPowerPlantType,
            IndexNumL02V2_4,
            IndustrialWasteDisposal,
            IndustrialWasteSpecialTreatment,
            InstallAdminCdV2_3,
            InstallAirPortCdV2_3,
            InstitutionTypeCd,
            KasoCd,
            KinouruikeiCd,
            LandscapeDistrictType,
            LandUseCd09U,
            LandUseCd09,
            LandUseCd77,
            LandUseCd88,
            LandUseCdYY,
            LargeClassificationCd,
            LocationAccuracyCd,
            MaritimeOrgCd,
            MedClassCd,
            midorinokairoCd,
            N04FukuinH16,
            N04FukuinH22,
            NaturalfeatureCd,
            NaturalParkAreaCd,
            NaturalsceneCd,
            NatureConservationAreaCd,
            OriginalDataCodeCd,
            PointClassificationCd,
            PointClassificationCode,
            PoliceStationCd,
            PosSpecificLevel,
            PostOfficeCd,
            ProtectionAreaType,
            PTAreaCd,
            PubFacAdminCd,
            PubFacMaclassCd,
            PubFacMiclassCd_wf,
            PubFacMiclassCd,
            PubFacMinclassCd,
            PubOfficeCd,
            PubOfficeClassCd,
            PumpingupType,
            RailwayClass2Cd,
            RailwayClassCd,
            RailwayDuplicateCd,
            RailwayExistenceCd,
            RailwayTransitionCd,
            ReferecedFromAgri,
            ReferenceDataCd,
            RegularFlightCd,
            ResearchInstitutionCd,
            RinshunosaibunCd,
            RitoCd,
            RoadCategoryCd,
            SchoolClassCd,
            SeasideType,
            SectionType,
            SectionCdCyubu,
            SectionCdKinki,
            SectionCdSyuto,
            SectionTypeCdCyubu,
            SectionTypeCdKinki,
            SectionTypeCdSyuto,
            SedimentDisastersProneAreaCd,
            SelectLandStatusL01V1,
            SelectLandStatusL01V2,
            SettingFlag,
            ShouhanshubanCd,
            SmallClassificationCd,
            SpecificAirPortSpecifiedSituationCd,
            SubprefectureNameCd,
            ThermalPowerEngine,
            TokusyudojyoCdV3_0,
            TourismResourceCategoryCd,
            TripGenerationCd,
            LandUseCd09Tweaked,
            UnderConstruction,
            Undersea,
            UrbanPlanningDecided,
            UrbanPlanningAreaCd_2019,
            UrgentRoadCd,
            UseDistrict,
            UseDistrictCd,
            WaterDepthCode,
            WaterSupplyType,
            WaterSystemCodeCd,
            WelfareFacMiclassCdH23,
            WelfareFacMiclassCdH27,
            WorldHeritageCd,
        );
    }
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
}
