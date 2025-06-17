mod common;
use crate::common::TestContext;
use ::common::types::{JReitBuildingIdAndCorporationId, TransactionCategory};
use pretty_assertions::assert_eq;
use rest::api::buildings::export_csv_v2::fetch_buildings_for_csv;
use rest::api::buildings::export_csv_v2::{
    InitialAcquisition, JReitAppraisal, JReitBuildingForCsv, LatestFinancial, LatestTransaction,
    SecondTermBeforeFinancial,
};
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use sql_entities::cities;
use sql_entities::j_reit_buildings;
use sql_entities::j_reit_corporations;
use sql_entities::j_reit_transactions;
use sql_entities::prefectures;
use sql_entities::wards;

const TEST_CITY_ID_MARUNOUCHI: i64 = 1;
const TEST_PREFECTURE_ID_TOKYO: i64 = 1;
const TEST_WARD_ID_CHIYODA: i64 = 1;

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_fetch_buildings_for_csv() -> anyhow::Result<()> {
    let TestContext { db } = TestContext::new().await?;

    let j_reit_building_id_1 = "test_id_1".to_string();
    let j_reit_building_id_2 = "test_id_2".to_string();

    prefectures::ActiveModel {
        id: Set(TEST_PREFECTURE_ID_TOKYO),
        name: Set("東京都".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    wards::ActiveModel {
        id: Set(TEST_WARD_ID_CHIYODA),
        prefecture_id: Set(TEST_PREFECTURE_ID_TOKYO),
        name: Set("千代田区".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    cities::ActiveModel {
        id: Set(TEST_CITY_ID_MARUNOUCHI),
        ward_id: Set(TEST_WARD_ID_CHIYODA),
        name: Set(Some("丸の内".to_string())),
        latitude: Set(35.0),
        longitude: Set(139.0),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_corporations::ActiveModel {
        id: Set("corp1".to_string()),
        name: Set("テスト法人1".to_string()),
        snowflake_deleted: Set(0),
        is_delisted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_corporations::ActiveModel {
        id: Set("corp2".to_string()),
        name: Set("テスト法人2".to_string()),
        snowflake_deleted: Set(0),
        is_delisted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_buildings::ActiveModel {
        id: Set(j_reit_building_id_1.clone()),
        name: Set("テストビル1".into()),
        is_office: Set(1),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        snowflake_deleted: Set(0),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        completed_year: Set(Some(2020)),
        land: Set(Some(1000.0)),
        gross_floor_area: Set(Some(10000.0)),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_buildings::ActiveModel {
        id: Set(j_reit_building_id_2.clone()),
        name: Set("テストビル2".into()),
        is_office: Set(0),
        is_residential: Set(1),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        snowflake_deleted: Set(0),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.1),
        longitude: Set(139.1),
        completed_year: Set(Some(2010)),
        land: Set(Some(500.0)),
        gross_floor_area: Set(Some(5000.0)),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_appraisals::ActiveModel {
        id: Set("appraisal1".to_string()),
        appraisal_date: Set(Some(chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap())),
        appraisal_price: Set(Some(1300000000)),
        cap_rate: Set(Some(4.5)),
        terminal_cap_rate: Set(Some(5.0)),
        discount_rate: Set(Some(6.0)),
        appraisal_company: Set(Some("テスト鑑定会社".to_string())),
        net_operating_income: Set(Some(58500000)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("transaction1".to_string()),
        j_reit_building_id: Set(j_reit_building_id_1.clone()),
        j_reit_corporation_id: Set("corp1".to_string()),
        j_reit_appraisal_id: Set(Some("appraisal1".to_string())),
        combined_transaction_id: Set(format!("{}-{}", j_reit_building_id_1, "corp1")),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        transaction_price: Set(Some(1000000000)),
        transaction_partner: Set(Some("テスト売主".to_string())),
        trustee: Set(Some("テスト信託銀行".to_string())),
        property_manager: Set(Some("テストPM会社".to_string())),
        pml_assessment_company: Set(Some("テストPM調査会社".to_string())),
        leasable_area: Set(Some(1000.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("transaction2".to_string()),
        j_reit_building_id: Set(j_reit_building_id_2.clone()),
        j_reit_corporation_id: Set("corp2".to_string()),
        combined_transaction_id: Set(format!("{}-{}", j_reit_building_id_2, "corp2")),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()),
        transaction_price: Set(Some(500000000)),
        transaction_partner: Set(Some("テスト売主2".to_string())),
        trustee: Set(Some("テスト信託銀行2".to_string())),
        property_manager: Set(Some("テストPM会社2".to_string())),
        pml_assessment_company: Set(Some("テストPM調査会社2".to_string())),
        leasable_area: Set(Some(500.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("latest_transaction1".to_string()),
        j_reit_building_id: Set(j_reit_building_id_1.clone()),
        j_reit_corporation_id: Set("corp1".to_string()),
        combined_transaction_id: Set(format!("{}-{}", j_reit_building_id_1, "corp1")),
        leasable_area: Set(Some(1000.0)),
        total_leasable_area: Set(Some(1000.0)),
        transaction_category: Set(TransactionCategory::PartialTransfer as i8),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        transaction_price: Set(Some(1200000000)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        transaction_partner: Set(Some("テスト譲渡先".to_string())),
        leasable_units: Set(Some(100)),
        building_ownership_ratio: Set(Some(50.0)),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_id_mappings::ActiveModel {
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_1".to_string()),
        j_reit_building_id: Set(j_reit_building_id_1.clone()),
        j_reit_corporation_id: Set("corp1".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_id_mappings::ActiveModel {
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_2".to_string()),
        j_reit_building_id: Set(j_reit_building_id_2.clone()),
        j_reit_corporation_id: Set("corp2".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_financials::ActiveModel {
        id: Set("financial1".to_string()),
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_1".to_string()),
        fiscal_period_start_date: Set(chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()),
        fiscal_period_end_date: Set(chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()),
        fiscal_period_operating_day: Set(365),
        net_operating_income: Set(Some(60000000)),
        appraisal_price: Set(Some(1350000000)),
        cap_rate: Set(Some(4.4)),
        terminal_cap_rate: Set(Some(4.9)),
        discount_rate: Set(Some(5.9)),
        snowflake_deleted: Set(0),
        occupancy_rate: Set(Some(0.95)),
        number_of_tenants: Set(Some(10)),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_financials::ActiveModel {
        id: Set("financial2".to_string()),
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_2".to_string()),
        fiscal_period_start_date: Set(chrono::NaiveDate::from_ymd_opt(2022, 4, 1).unwrap()),
        fiscal_period_end_date: Set(chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap()),
        fiscal_period_operating_day: Set(365),
        net_operating_income: Set(Some(55000000)),
        appraisal_price: Set(Some(1250000000)),
        cap_rate: Set(Some(4.4)),
        terminal_cap_rate: Set(Some(4.9)),
        discount_rate: Set(Some(5.9)),
        snowflake_deleted: Set(0),
        occupancy_rate: Set(Some(0.90)),
        number_of_tenants: Set(Some(8)),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_financials::ActiveModel {
        id: Set("financial3".to_string()),
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_1".to_string()),
        fiscal_period_start_date: Set(chrono::NaiveDate::from_ymd_opt(2022, 4, 1).unwrap()),
        fiscal_period_end_date: Set(chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap()),
        fiscal_period_operating_day: Set(365),
        net_operating_income: Set(Some(11000000)),
        appraisal_price: Set(Some(1200000000)),
        cap_rate: Set(Some(4.5)),
        terminal_cap_rate: Set(Some(5.0)),
        discount_rate: Set(Some(5.5)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_financials::ActiveModel {
        id: Set("financial4".to_string()),
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_1".to_string()),
        fiscal_period_start_date: Set(chrono::NaiveDate::from_ymd_opt(2021, 4, 1).unwrap()),
        fiscal_period_end_date: Set(chrono::NaiveDate::from_ymd_opt(2022, 3, 31).unwrap()),
        fiscal_period_operating_day: Set(365),
        net_operating_income: Set(Some(6000000)),
        appraisal_price: Set(Some(1150000000)),
        cap_rate: Set(Some(4.6)),
        terminal_cap_rate: Set(Some(5.1)),
        discount_rate: Set(Some(5.6)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let ids = vec![
        JReitBuildingIdAndCorporationId {
            building_id: j_reit_building_id_1.clone(),
            corporation_id: "corp1".to_string(),
        },
        JReitBuildingIdAndCorporationId {
            building_id: j_reit_building_id_2.clone(),
            corporation_id: "corp2".to_string(),
        },
    ];

    let buildings = fetch_buildings_for_csv(&db, &ids).await?;

    assert_eq!(buildings.len(), 2);

    let first_building = &buildings[0];

    assert_eq!(
        first_building,
        &JReitBuildingForCsv {
            j_reit_building: j_reit_buildings::Model {
                id: "test_id_1".to_string(),
                name: "テストビル1".to_string(),
                is_office: 1,
                is_residential: 0,
                is_hotel: 0,
                is_logistic: 0,
                is_retail: 0,
                is_health_care: 0,
                is_other: 0,
                office_building_id: None,
                residential_building_id: None,
                address: None,
                city_id: TEST_CITY_ID_MARUNOUCHI,
                latitude: 35.0,
                longitude: 139.0,
                nearest_station: None,
                completed_year: Some(2020),
                completed_month: None,
                gross_floor_area: Some(10000.0),
                basement: None,
                groundfloor: None,
                structure: None,
                floor_plan: None,
                land: Some(1000.0),
                building_coverage_ratio: None,
                floor_area_ratio: None,
                snowflake_deleted: 0,
            },
            corporation_id: "corp1".to_string(),
            corporation_name: "テスト法人1".to_string(),
            is_delisted: 0,
            initial_acquisition: Some(InitialAcquisition {
                initial_acquisition_transaction_date: Some(
                    chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()
                ),
                initial_acquisition_transaction_price: Some(1000000000),
                initial_acquisition_transaction_partner: Some("テスト売主".to_string()),
                initial_acquisition_trustee: Some("テスト信託銀行".to_string()),
                initial_acquisition_property_manager: Some("テストPM会社".to_string()),
                initial_acquisition_pml_assessment_company: Some("テストPM調査会社".to_string()),
                initial_acquisition_leasable_area: Some(1000.0),
            }),
            j_reit_appraisal: Some(JReitAppraisal {
                j_reit_appraisal_appraisal_price: Some(1300000000),
                j_reit_appraisal_cap_rate: Some(4.5),
                j_reit_appraisal_terminal_cap_rate: Some(5.0),
                j_reit_appraisal_discount_rate: Some(6.0),
                j_reit_appraisal_appraisal_company: Some("テスト鑑定会社".to_string()),
                j_reit_appraisal_net_operating_income: Some(58500000),
            }),
            latest_transaction: Some(LatestTransaction {
                transaction_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                transaction_price: Some(1200000000),
                total_leasable_area: Some(1000.0),
                transaction_category: Some(TransactionCategory::PartialTransfer as i8),
                transaction_partner: Some("テスト譲渡先".to_string()),
                leasable_units: Some(100),
                building_ownership_ratio: Some(50.0),
            }),
            latest_financial: Some(LatestFinancial {
                fiscal_period_start_date: chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap(),
                fiscal_period_end_date: chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
                net_operating_income: Some(60000000),
                appraisal_price: Some(1350000000),
                cap_rate: Some(4.4),
                terminal_cap_rate: Some(4.9),
                discount_rate: Some(5.9),
                occupancy_rate: Some(0.95),
                number_of_tenants: Some(10),
            }),
            second_term_before_financial: Some(SecondTermBeforeFinancial {
                second_term_before_financial_net_operating_income: Some(6000000),
            }),
        }
    );

    let second_building = &buildings[1];

    assert_eq!(
        second_building,
        &JReitBuildingForCsv {
            j_reit_building: j_reit_buildings::Model {
                id: "test_id_2".to_string(),
                name: "テストビル2".to_string(),
                is_office: 0,
                is_residential: 1,
                is_hotel: 0,
                is_logistic: 0,
                is_retail: 0,
                is_health_care: 0,
                is_other: 0,
                office_building_id: None,
                residential_building_id: None,
                address: None,
                city_id: TEST_CITY_ID_MARUNOUCHI,
                latitude: 35.1,
                longitude: 139.1,
                nearest_station: None,
                completed_year: Some(2010),
                completed_month: None,
                gross_floor_area: Some(5000.0),
                basement: None,
                groundfloor: None,
                structure: None,
                floor_plan: None,
                land: Some(500.0),
                building_coverage_ratio: None,
                floor_area_ratio: None,
                snowflake_deleted: 0,
            },
            corporation_id: "corp2".to_string(),
            corporation_name: "テスト法人2".to_string(),
            is_delisted: 0,
            initial_acquisition: Some(InitialAcquisition {
                initial_acquisition_transaction_date: Some(
                    chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()
                ),
                initial_acquisition_transaction_price: Some(500000000),
                initial_acquisition_transaction_partner: Some("テスト売主2".to_string()),
                initial_acquisition_trustee: Some("テスト信託銀行2".to_string()),
                initial_acquisition_property_manager: Some("テストPM会社2".to_string()),
                initial_acquisition_pml_assessment_company: Some("テストPM調査会社2".to_string()),
                initial_acquisition_leasable_area: Some(500.0),
            }),
            j_reit_appraisal: Some(JReitAppraisal {
                j_reit_appraisal_appraisal_price: None,
                j_reit_appraisal_cap_rate: None,
                j_reit_appraisal_terminal_cap_rate: None,
                j_reit_appraisal_discount_rate: None,
                j_reit_appraisal_appraisal_company: None,
                j_reit_appraisal_net_operating_income: None,
            }),
            latest_transaction: Some(LatestTransaction {
                transaction_date: Some(chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()),
                transaction_price: Some(500000000),
                total_leasable_area: None,
                transaction_category: Some(TransactionCategory::InitialAcquisition as i8),
                transaction_partner: Some("テスト売主2".to_string()),
                leasable_units: None,
                building_ownership_ratio: None,
            }),
            latest_financial: Some(LatestFinancial {
                fiscal_period_start_date: chrono::NaiveDate::from_ymd_opt(2022, 4, 1).unwrap(),
                fiscal_period_end_date: chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
                net_operating_income: Some(55000000),
                appraisal_price: Some(1250000000),
                cap_rate: Some(4.4),
                terminal_cap_rate: Some(4.9),
                discount_rate: Some(5.9),
                occupancy_rate: Some(0.90),
                number_of_tenants: Some(8),
            }),
            second_term_before_financial: Some(SecondTermBeforeFinancial {
                second_term_before_financial_net_operating_income: None,
            }),
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_fetch_buildings_for_csv_duplicate_corporation_count_only() -> anyhow::Result<()> {
    let TestContext { db } = TestContext::new().await?;

    prefectures::ActiveModel {
        id: Set(1),
        name: Set("東京都".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    wards::ActiveModel {
        id: Set(1),
        prefecture_id: Set(1),
        name: Set("千代田区".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    cities::ActiveModel {
        id: Set(1),
        ward_id: Set(1),
        name: Set(Some("丸ノ内".to_string())),
        latitude: Set(0.0),
        longitude: Set(0.0),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_corporations::ActiveModel {
        id: Set("corp1".to_string()),
        name: Set("テスト法人".to_string()),
        snowflake_deleted: Set(0),
        is_delisted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_corporations::ActiveModel {
        id: Set("corp2".to_string()),
        name: Set("テスト法人".to_string()),
        snowflake_deleted: Set(0),
        is_delisted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_buildings::ActiveModel {
        id: Set("building".to_string()),
        name: Set("ビル名".to_string()),
        is_office: Set(0),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        snowflake_deleted: Set(0),
        city_id: Set(1),
        latitude: Set(0.0),
        longitude: Set(0.0),
        completed_year: Set(None),
        land: Set(None),
        gross_floor_area: Set(None),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("transaction1".to_string()),
        combined_transaction_id: Set("building-corp1".to_string()),
        j_reit_building_id: Set("building".to_string()),
        j_reit_corporation_id: Set("corp1".to_string()),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        transaction_price: Set(Some(1000000000)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("transaction2".to_string()),
        combined_transaction_id: Set("building-corp2".to_string()),
        j_reit_building_id: Set("building".to_string()),
        j_reit_corporation_id: Set("corp2".to_string()),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        transaction_price: Set(Some(1000000000)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let ids = vec![
        JReitBuildingIdAndCorporationId {
            building_id: "building".to_string(),
            corporation_id: "corp1".to_string(),
        },
        JReitBuildingIdAndCorporationId {
            building_id: "building".to_string(),
            corporation_id: "corp2".to_string(),
        },
    ];

    let buildings = fetch_buildings_for_csv(&db, &ids).await?;
    assert_eq!(buildings.len(), 2);

    Ok(())
}
