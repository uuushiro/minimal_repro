mod common;
use crate::common::TestContext;
use ::common::types::{JReitBuildingIdAndCorporationId, TransactionCategory};
use pretty_assertions::assert_eq;
use rest::api::buildings::export_excel_v2::fetch_buildings_for_excel;
use rest::api::buildings::export_excel_v2::{
    fetch_financials_grouped_by_mizuho_id, InitialAcquisition, JReitAppraisal,
    JReitBuildingsForExcel, LatestFinancial, LatestTransaction,
};
use sea_orm::ActiveModelTrait;
use sea_orm::Set;
use sql_entities::j_reit_buildings;
use sql_entities::j_reit_corporations;
use sql_entities::j_reit_transactions;
use sql_entities::prefectures;
use sql_entities::wards;
use sql_entities::{cities, j_reit_mizuho_financials};

const TEST_CITY_ID_MARUNOUCHI: i64 = 1;
const TEST_PREFECTURE_ID_TOKYO: i64 = 1;
const TEST_WARD_ID_CHIYODA: i64 = 1;

#[allow(clippy::unwrap_used)]
async fn setup_database(db: sea_orm::DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    let j_reit_building_id_1 = "test_id_1".to_string();
    let j_reit_building_id_2 = "test_id_2".to_string();

    // 都道府県・区・市の準備
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

    // 法人の準備
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

    // ビルの準備
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

    // マッピングデータの準備
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
        fiscal_period: Set(Some("2023年度".to_string())),
        fiscal_period_start_date: Set(chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()),
        fiscal_period_end_date: Set(chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()),
        fiscal_period_operating_day: Set(365),
        rental_income: Set(Some(80000000)),
        parking_income: Set(Some(5000000)),
        common_area_charge: Set(Some(2000000)),
        other_rental_income: Set(Some(1000000)),
        other_income: Set(Some(500000)),
        property_management_fee: Set(Some(4000000)),
        maintenance_fee: Set(Some(3000000)),
        utility_cost: Set(Some(2000000)),
        security_fee: Set(Some(1500000)),
        repair_cost: Set(Some(1000000)),
        cleaning_fee: Set(Some(800000)),
        insurance_cost: Set(Some(500000)),
        real_estate_tax: Set(Some(15000000)),
        common_area_expense: Set(Some(1000000)),
        other_operating_expense: Set(Some(2000000)),
        capital_expenditure: Set(Some(5000000)),
        net_operating_income: Set(Some(60000000)),
        depriciation: Set(Some(20000000)),
        net_income: Set(Some(40000000)),
        free_cash_flow: Set(Some(35000000)),
        occupancy_rate: Set(Some(0.95)),
        number_of_tenants: Set(Some(10)),
        security_deposit_balance: Set(Some(100000000)),
        appraisal_price: Set(Some(1350000000)),
        appraisal_cap_rate: Set(Some(4.4)),
        appraisal_discount_rate: Set(Some(5.9)),
        direct_capitalization_price: Set(Some(1300000000)),
        cap_rate: Set(Some(4.4)),
        discount_cash_flow_price: Set(Some(1350000000)),
        discount_rate: Set(Some(5.9)),
        terminal_cap_rate: Set(Some(4.9)),
        acquisition_price: Set(Some(1000000000)),
        book_value: Set(Some(950000000)),
        scheduled_property_tax: Set(Some(15000000)),
        net_leasable_area_total: Set(Some(8000.0)),
        year_to_date_net_operating_income: Set(Some(30000000)),
        rental_income_per_tsubo: Set(Some(100000)),
        net_operating_income_yield: Set(Some(4.4)),
        net_cash_flow_cap_rate: Set(Some(4.2)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    sql_entities::j_reit_mizuho_financials::ActiveModel {
        id: Set("financial2".to_string()),
        j_reit_mizuho_building_id: Set("test_mizuho_building_id_2".to_string()),
        fiscal_period: Set(Some("2022年度".to_string())),
        fiscal_period_start_date: Set(chrono::NaiveDate::from_ymd_opt(2022, 4, 1).unwrap()),
        fiscal_period_end_date: Set(chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap()),
        fiscal_period_operating_day: Set(365),
        rental_income: Set(Some(60000000)),
        parking_income: Set(Some(3000000)),
        common_area_charge: Set(Some(1500000)),
        other_rental_income: Set(Some(800000)),
        other_income: Set(Some(300000)),
        property_management_fee: Set(Some(3000000)),
        maintenance_fee: Set(Some(2000000)),
        utility_cost: Set(Some(1500000)),
        security_fee: Set(Some(1000000)),
        repair_cost: Set(Some(800000)),
        cleaning_fee: Set(Some(500000)),
        insurance_cost: Set(Some(300000)),
        real_estate_tax: Set(Some(12000000)),
        common_area_expense: Set(Some(800000)),
        other_operating_expense: Set(Some(1500000)),
        capital_expenditure: Set(Some(4000000)),
        net_operating_income: Set(Some(55000000)),
        depriciation: Set(Some(15000000)),
        net_income: Set(Some(40000000)),
        free_cash_flow: Set(Some(35000000)),
        occupancy_rate: Set(Some(0.90)),
        number_of_tenants: Set(Some(8)),
        security_deposit_balance: Set(Some(80000000)),
        appraisal_price: Set(Some(1250000000)),
        appraisal_cap_rate: Set(Some(4.4)),
        appraisal_discount_rate: Set(Some(5.9)),
        direct_capitalization_price: Set(Some(1200000000)),
        cap_rate: Set(Some(4.4)),
        discount_cash_flow_price: Set(Some(1250000000)),
        discount_rate: Set(Some(5.9)),
        terminal_cap_rate: Set(Some(4.9)),
        acquisition_price: Set(Some(500000000)),
        book_value: Set(Some(450000000)),
        scheduled_property_tax: Set(Some(12000000)),
        net_leasable_area_total: Set(Some(4000.0)),
        year_to_date_net_operating_income: Set(Some(27500000)),
        rental_income_per_tsubo: Set(Some(150000)),
        net_operating_income_yield: Set(Some(4.4)),
        net_cash_flow_cap_rate: Set(Some(4.2)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_fetch_buildings_for_excel() -> Result<(), Box<dyn std::error::Error>> {
    let TestContext { db } = TestContext::new().await?;

    setup_database(db.clone()).await?;

    let ids = vec![
        JReitBuildingIdAndCorporationId {
            building_id: "test_id_1".to_string(),
            corporation_id: "corp1".to_string(),
        },
        JReitBuildingIdAndCorporationId {
            building_id: "test_id_2".to_string(),
            corporation_id: "corp2".to_string(),
        },
    ];

    let buildings = fetch_buildings_for_excel(&db, &ids)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    assert_eq!(buildings.len(), 2);

    let first_building = &buildings[0];

    assert_eq!(
        first_building,
        &JReitBuildingsForExcel {
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
                initial_acquisition_leasable_area: Some(1000.0),
            }),
            j_reit_appraisal: Some(JReitAppraisal {
                appraisal_cap_rate: Some(4.5),
            }),
            latest_transaction: Some(LatestTransaction {
                transaction_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                total_leasable_area: Some(1000.0),
                transaction_category: Some(TransactionCategory::PartialTransfer as i8),
                building_ownership_ratio: Some(50.0),
                building_ownership_type: None,
                land_ownership_type: None,
                land_ownership_ratio: None,
            }),
            latest_financial: Some(LatestFinancial {
                latest_financial_cap_rate: Some(4.4),
                latest_financial_fiscal_period_end_date: chrono::NaiveDate::from_ymd_opt(
                    2024, 3, 31
                )
                .unwrap(),
                latest_financial_appraisal_price: Some(1350000000),
            }),
            j_reit_mizuho_building_id: Some("test_mizuho_building_id_1".to_string()),
        }
    );

    let second_building = &buildings[1];

    assert_eq!(
        second_building,
        &JReitBuildingsForExcel {
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
                initial_acquisition_leasable_area: Some(500.0),
            }),
            j_reit_appraisal: Some(JReitAppraisal {
                appraisal_cap_rate: None
            }),
            latest_transaction: Some(LatestTransaction {
                transaction_date: Some(chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()),
                total_leasable_area: None,
                transaction_category: Some(TransactionCategory::InitialAcquisition as i8),
                building_ownership_ratio: None,
                building_ownership_type: None,
                land_ownership_type: None,
                land_ownership_ratio: None,
            }),
            latest_financial: Some(LatestFinancial {
                latest_financial_cap_rate: Some(4.4),
                latest_financial_fiscal_period_end_date: chrono::NaiveDate::from_ymd_opt(
                    2023, 3, 31
                )
                .unwrap(),
                latest_financial_appraisal_price: Some(1250000000),
            }),
            j_reit_mizuho_building_id: Some("test_mizuho_building_id_2".to_string()),
        }
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_fetch_financials_grouped_by_mizuho_id() -> Result<(), Box<dyn std::error::Error>> {
    let TestContext { db } = TestContext::new().await?;

    setup_database(db.clone()).await?;

    let buildings = fetch_buildings_for_excel(
        &db,
        &[
            JReitBuildingIdAndCorporationId {
                building_id: "test_id_1".to_string(),
                corporation_id: "corp1".to_string(),
            },
            JReitBuildingIdAndCorporationId {
                building_id: "test_id_2".to_string(),
                corporation_id: "corp2".to_string(),
            },
        ],
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let financials_grouped_by_mizuho_id = fetch_financials_grouped_by_mizuho_id(&db, &buildings)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    assert_eq!(financials_grouped_by_mizuho_id.len(), 2);

    let financials1 = financials_grouped_by_mizuho_id
        .get("test_mizuho_building_id_1")
        .unwrap();
    assert_eq!(financials1.len(), 1);
    assert_eq!(
        &financials1[0],
        &j_reit_mizuho_financials::Model {
            id: "financial1".to_string(),
            j_reit_mizuho_building_id: "test_mizuho_building_id_1".to_string(),
            fiscal_period: Some("2023年度".to_string()),
            fiscal_period_start_date: chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap(),
            fiscal_period_end_date: chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            fiscal_period_operating_day: 365,
            rental_income: Some(80000000),
            parking_income: Some(5000000),
            common_area_charge: Some(2000000),
            other_rental_income: Some(1000000),
            other_income: Some(500000),
            property_management_fee: Some(4000000),
            maintenance_fee: Some(3000000),
            utility_cost: Some(2000000),
            security_fee: Some(1500000),
            repair_cost: Some(1000000),
            cleaning_fee: Some(800000),
            insurance_cost: Some(500000),
            real_estate_tax: Some(15000000),
            common_area_expense: Some(1000000),
            other_operating_expense: Some(2000000),
            capital_expenditure: Some(5000000),
            net_operating_income: Some(60000000),
            depriciation: Some(20000000),
            net_income: Some(40000000),
            free_cash_flow: Some(35000000),
            occupancy_rate: Some(0.95),
            number_of_tenants: Some(10),
            security_deposit_balance: Some(100000000),
            appraisal_price: Some(1350000000),
            appraisal_cap_rate: Some(4.4),
            appraisal_discount_rate: Some(5.9),
            direct_capitalization_price: Some(1300000000),
            cap_rate: Some(4.4),
            discount_cash_flow_price: Some(1350000000),
            discount_rate: Some(5.9),
            terminal_cap_rate: Some(4.9),
            acquisition_price: Some(1000000000),
            book_value: Some(950000000),
            scheduled_property_tax: Some(15000000),
            net_leasable_area_total: Some(8000.0),
            year_to_date_net_operating_income: Some(30000000),
            rental_income_per_tsubo: Some(100000),
            net_operating_income_yield: Some(4.4),
            net_cash_flow_cap_rate: Some(4.2),
            snowflake_deleted: 0,
        }
    );

    let financials2 = financials_grouped_by_mizuho_id
        .get("test_mizuho_building_id_2")
        .unwrap();
    assert_eq!(financials2.len(), 1);
    assert_eq!(
        &financials2[0],
        &j_reit_mizuho_financials::Model {
            id: "financial2".to_string(),
            j_reit_mizuho_building_id: "test_mizuho_building_id_2".to_string(),
            fiscal_period: Some("2022年度".to_string()),
            fiscal_period_start_date: chrono::NaiveDate::from_ymd_opt(2022, 4, 1).unwrap(),
            fiscal_period_end_date: chrono::NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
            fiscal_period_operating_day: 365,
            rental_income: Some(60000000),
            parking_income: Some(3000000),
            common_area_charge: Some(1500000),
            other_rental_income: Some(800000),
            other_income: Some(300000),
            property_management_fee: Some(3000000),
            maintenance_fee: Some(2000000),
            utility_cost: Some(1500000),
            security_fee: Some(1000000),
            repair_cost: Some(800000),
            cleaning_fee: Some(500000),
            insurance_cost: Some(300000),
            real_estate_tax: Some(12000000),
            common_area_expense: Some(800000),
            other_operating_expense: Some(1500000),
            capital_expenditure: Some(4000000),
            net_operating_income: Some(55000000),
            depriciation: Some(15000000),
            net_income: Some(40000000),
            free_cash_flow: Some(35000000),
            occupancy_rate: Some(0.90),
            number_of_tenants: Some(8),
            security_deposit_balance: Some(80000000),
            appraisal_price: Some(1250000000),
            appraisal_cap_rate: Some(4.4),
            appraisal_discount_rate: Some(5.9),
            direct_capitalization_price: Some(1200000000),
            cap_rate: Some(4.4),
            discount_cash_flow_price: Some(1250000000),
            discount_rate: Some(5.9),
            terminal_cap_rate: Some(4.9),
            acquisition_price: Some(500000000),
            book_value: Some(450000000),
            scheduled_property_tax: Some(12000000),
            net_leasable_area_total: Some(4000.0),
            year_to_date_net_operating_income: Some(27500000),
            rental_income_per_tsubo: Some(150000),
            net_operating_income_yield: Some(4.4),
            net_cash_flow_cap_rate: Some(4.2),
            snowflake_deleted: 0,
        }
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_fetch_buildings_for_excel_single_building_two_corporations_minimal_setup(
) -> Result<(), Box<dyn std::error::Error>> {
    let TestContext { db } = TestContext::new().await?;

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
        id: Set("building".to_string()),
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

    j_reit_transactions::ActiveModel {
        id: Set("transaction1".to_string()),
        combined_transaction_id: Set("building-corp1".to_string()),
        j_reit_building_id: Set("building".to_string()),
        j_reit_corporation_id: Set("corp1".to_string()),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("transaction2".to_string()),
        combined_transaction_id: Set("building-corp2".to_string()),
        j_reit_building_id: Set("building".to_string()),
        j_reit_corporation_id: Set("corp2".to_string()),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        transaction_date: Set(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
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

    let buildings = fetch_buildings_for_excel(&db, &ids).await?;
    assert_eq!(buildings.len(), 2);
    Ok(())
}
