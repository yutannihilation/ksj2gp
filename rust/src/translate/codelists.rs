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
        // L01: 用途区分
        ("L01_001" | "L01_003", ..=2023) | ("L01_002" | "L01_005", 2024..) => {
            return CODELISTS_MAP.get("IndexNumL01");
        }
        // L01: 選定状況
        ("L01_007", ..=2019) => return CODELISTS_MAP.get("SelectLandStatusL01V1"),
        ("L01_007", 2020..=2021) => return CODELISTS_MAP.get("SelectLandStatusL01V2"),
        ("L01_008", 2022..=2023) => return CODELISTS_MAP.get("SelectLandStatusL01V2"),
        ("L01_010", 2024..) => return CODELISTS_MAP.get("SelectLandStatusL01V2"),
        // A42: shapefile が複数入っていて、片方にしかない
        ("A42_005", _) => {
            if target_shp.ends_with("Spacial_Preservation_Area_of_Historic_Landscape.shp") {
                return CODELISTS_MAP.get("A42HistoricalDistrictType");
            }
        }
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
            A10AreaCode,
            A10LayerNo,
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
            authority_type,
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
            landscape_district_type,
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
            SectionCdTweaked,
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
