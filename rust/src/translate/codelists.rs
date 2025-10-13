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

// TODO: probably, it's not ideal to expose this HashMap directly. Can we wrap this as a function like colnames.rs?
pub(crate) static CODELISTS_MAP: LazyLock<
    HashMap<&'static str, LazyLock<HashMap<&'static str, &'static str>>>,
> = LazyLock::new(|| {
    let mut map: HashMap<&'static str, LazyLock<HashMap<&'static str, &'static str>>> =
        HashMap::with_capacity(200); // TODO: choose a nicer number
    for (col_id, metadata) in COLNAMES {
        if let Some(id) = metadata.1 {
            register_codelists!(
                map, col_id, id;
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
                IndexNumL01V1_1,
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
                SelectLandStatusV1_1,
                SelectLandStatusV2_4,
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
    }
    map
});
