mod common;
use common::*;
mod utils;
use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use sql_entities::j_reit_mizuho_financials;
use utils::*;

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_j_reit_financials_with_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let [j_reit_building_id_1, j_reit_building_id_2] = create_j_reit_buildings(&db).await;
    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人1".into(), "test_corporation_id_1".into(), 0)
            .await;
    let j_reit_mizuho_building_id_1 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    let j_reit_mizuho_building_id_2 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    create_j_reit_financials_1_2(
        &db,
        &j_reit_mizuho_building_id_1,
        &j_reit_mizuho_building_id_2,
    )
    .await?;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!]) {
            jReitBuildings(ids: $ids) {
                id
                financials {
                    id
                }
            }
        }
    "#
    .into();
    let variables = json!({
        "ids": [j_reit_building_id_1, j_reit_building_id_2]
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [
                {
                    "id": "test_id_1",
                    "financials": [{
                        "id": "test_id_1_1",
                    }]
                },
                {
                    "id": "test_id_2",
                    "financials": [
                        {
                            "id": "test_id_2_2",
                        },
                        {
                            "id": "test_id_2_1",
                        }
                    ]
                },
            ]
        }),
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
// j-reitビル1の決算情報に関して、各項目が正しく取得できることを確認
async fn test_j_reit_financials_attributes() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let [j_reit_building_id_1, j_reit_building_id_2] = create_j_reit_buildings(&db).await;
    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人1".into(), "test_corporation_id_1".into(), 0)
            .await;
    let j_reit_mizuho_building_id_1 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    let j_reit_mizuho_building_id_2 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    create_j_reit_financials_1_2(
        &db,
        &j_reit_mizuho_building_id_1,
        &j_reit_mizuho_building_id_2,
    )
    .await?;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!]) {
            jReitBuildings(ids: $ids) {
                id
                financials {
                    id
                    fiscalPeriod {
                      name
                      startDate
                      endDate
                      operatingDays
                    }
                    income {
                      rent
                      parking
                      camFee
                      otherRentalIncome
                      other
                    }
                    expense {
                      propertyManagement
                      maintenance
                      utility
                      security
                      repair
                      cleaning
                      insurance
                      realEstateTax
                      camFee
                      other
                      capitalExpenditure
                    }
                    balance {
                      netOperatingIncome
                      depreciation
                      netIncome
                      freeCashFlow
                    }
                    leasing {
                      occupancyRate
                      numberOfTenants
                      netLeasableAreaTotal
                    }
                    appraisal {
                      appraisalPrice
                      appraisalCapRate
                      appraisalDiscountRate
                      directCapitalizationPrice
                      capRate
                      discountCashFlowPrice
                      discountRate
                      terminalCapRate
                    }
                    indicators {
                      rentalIncomePerTsubo
                      yearToDateNetOperatingIncome
                      netOperatingIncomeYield
                      netCashFlowCapRate
                    }
                    acquisitionPrice
                    bookValue
                    securityDepositBalance
                    scheduledPropertyTax
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [j_reit_building_id_1]
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [
                {
                    "id": j_reit_building_id_1,
                    "financials": [{
                        "id": "test_id_1_1",
                        "fiscalPeriod": {
                            "name": "第5期",
                            "startDate": "2023-01-01",
                            "endDate": "2023-06-30",
                            "operatingDays": 180
                        },
                        "income": {
                            "rent": 1_000_000_000,
                            "parking": 0,
                            "camFee": 500_000,
                            "otherRentalIncome": null,
                            "other": 100_000
                        },
                        "expense": {
                            "propertyManagement": 1_234_000,
                            "maintenance": 2_345_000,
                            "utility": 3_456_000,
                            "security": 4_567_000,
                            "repair": 5_678_000,
                            "cleaning": 6_789_000,
                            "insurance": 7_890_000,
                            "realEstateTax": 8_901_000,
                            "camFee": 9_012_000,
                            "other": 10_123_000,
                            "capitalExpenditure": 11_234_000
                        },
                        "balance": {
                            "netOperatingIncome": 111_000_000,
                            "depreciation": 11_000_000,
                            "netIncome": 100_000_000,
                            "freeCashFlow": 90_000_000
                        },
                        "leasing": {
                            "occupancyRate": "97.50",
                            "numberOfTenants": 10,
                            "netLeasableAreaTotal": "1234.50"
                        },
                        "appraisal": {
                            "appraisalPrice": 123_000_000_000i64,
                            "appraisalCapRate": "4.50",
                            "appraisalDiscountRate": "3.50",
                            "directCapitalizationPrice": 1_620_000_000i64,
                            "capRate": "4.60",
                            "discountCashFlowPrice": 1_580_000_000i64,
                            "discountRate": "4.40",
                            "terminalCapRate": "4.80"
                        },
                        "indicators": {
                            "rentalIncomePerTsubo": 30_000,
                            "yearToDateNetOperatingIncome": 24_084_000,
                            "netOperatingIncomeYield": "4.80",
                            "netCashFlowCapRate": "3.80"
                        },
                        "acquisitionPrice": 100_000_000_000i64,
                        "bookValue": 1_259_000_000i64,
                        "securityDepositBalance": 1_000_000,
                        "scheduledPropertyTax": 12_000_000
                    }]
                }
            ]
        }),
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
// firstが指定されたとき、先頭からその分取得する
async fn test_j_reit_financials_apply_first() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let [j_reit_building_id_1, j_reit_building_id_2] = create_j_reit_buildings(&db).await;
    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人1".into(), "test_corporation_id_1".into(), 0)
            .await;
    let j_reit_mizuho_building_id_1 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    let j_reit_mizuho_building_id_2 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    create_j_reit_financials_1_2(
        &db,
        &j_reit_mizuho_building_id_1,
        &j_reit_mizuho_building_id_2,
    )
    .await?;
    create_j_reit_financials_3_4_5(&db, &j_reit_mizuho_building_id_2).await?;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!], $first: Int) {
            jReitBuildings(ids: $ids) {
                id
                financials(first: $first) {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [j_reit_building_id_2],
        "first": 2
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);

    // 順不同であるため、各ビルから決算情報のIDを抽出して検証する
    let response_date = response.data.into_json().unwrap();
    let j_reit_building_2 = response_date["jReitBuildings"]
        .as_array()
        .unwrap()
        .first()
        .unwrap();

    let financial_ids_2 = j_reit_building_2["financials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(financial_ids_2.len(), 2);
    // end_date順に並んでいる
    assert_eq!(financial_ids_2, vec!["test_id_2_2", "test_id_2_1"]);

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
// lastが指定されたとき、末尾からその分取得する
async fn test_j_reit_financials_apply_last() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let [j_reit_building_id_1, j_reit_building_id_2] = create_j_reit_buildings(&db).await;
    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人1".into(), "test_corporation_id_1".into(), 0)
            .await;
    let j_reit_mizuho_building_id_1 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    let j_reit_mizuho_building_id_2 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    create_j_reit_financials_1_2(
        &db,
        &j_reit_mizuho_building_id_1,
        &j_reit_mizuho_building_id_2,
    )
    .await?;
    create_j_reit_financials_3_4_5(&db, &j_reit_mizuho_building_id_2).await?;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!], $last: Int) {
            jReitBuildings(ids: $ids) {
                id
                financials(last: $last) {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [j_reit_building_id_2],
        "last": 3
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);

    // 順不同であるため、各ビルから決算情報のIDを抽出して検証する
    let response_date = response.data.into_json().unwrap();
    let j_reit_building_2 = response_date["jReitBuildings"]
        .as_array()
        .unwrap()
        .first()
        .unwrap();

    let financial_ids_2 = j_reit_building_2["financials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(financial_ids_2.len(), 3);
    // end_date順に並んでいる
    assert_eq!(
        financial_ids_2,
        vec!["test_id_2_4", "test_id_2_5", "test_id_2_3"]
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
// firstとlastが指定されたとき、firstが優先される
async fn test_j_reit_financials_apply_first_and_last() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let [j_reit_building_id_1, j_reit_building_id_2] = create_j_reit_buildings(&db).await;
    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人1".into(), "test_corporation_id_1".into(), 0)
            .await;
    let j_reit_mizuho_building_id_1 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    let j_reit_mizuho_building_id_2 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    create_j_reit_financials_1_2(
        &db,
        &j_reit_mizuho_building_id_1,
        &j_reit_mizuho_building_id_2,
    )
    .await?;
    create_j_reit_financials_3_4_5(&db, &j_reit_mizuho_building_id_2).await?;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_2.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!], $first: Int, $last: Int) {
            jReitBuildings(ids: $ids) {
                id
                financials(first: $first, last: $last) {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [j_reit_building_id_2],
        "first": 2,
        "last": 3
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);

    // 順不同であるため、各ビルから決算情報のIDを抽出して検証する
    let response_date = response.data.into_json().unwrap();
    let j_reit_building_2 = response_date["jReitBuildings"]
        .as_array()
        .unwrap()
        .first()
        .unwrap();

    let financial_ids_2 = j_reit_building_2["financials"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(financial_ids_2.len(), 2);
    assert_eq!(financial_ids_2, vec!["test_id_2_2", "test_id_2_1"]);

    Ok(())
}

async fn create_j_reit_buildings(db: &sea_orm::DatabaseConnection) -> [String; 2] {
    let j_reit_building_id_1 = insert_test_j_reit_building(db, "test_id_1".into()).await;
    let j_reit_building_id_2 = insert_test_j_reit_building(db, "test_id_2".into()).await;

    [j_reit_building_id_1, j_reit_building_id_2]
}

async fn create_j_reit_financials_1_2(
    db: &sea_orm::DatabaseConnection,
    j_reit_mizuho_building_id_1: &str,
    j_reit_mizuho_building_id_2: &str,
) -> Result<()> {
    // j-reitビル1の決算情報
    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_id_1_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.to_string()),
        fiscal_period: Set(Some("第5期".to_string())),
        fiscal_period_start_date: Set(datetime_utc(2023, 1, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2023, 6, 30, 0).date_naive()),
        fiscal_period_operating_day: Set(180),
        rental_income: Set(Some(1_000_000_000i64)),
        parking_income: Set(Some(0i64)),
        common_area_charge: Set(Some(500_000i64)),
        other_rental_income: Set(None),
        other_income: Set(Some(100_000i64)),
        property_management_fee: Set(Some(1_234_000i64)),
        maintenance_fee: Set(Some(2_345_000i64)),
        utility_cost: Set(Some(3_456_000i64)),
        security_fee: Set(Some(4_567_000i64)),
        repair_cost: Set(Some(5_678_000i64)),
        cleaning_fee: Set(Some(6_789_000i64)),
        insurance_cost: Set(Some(7_890_000i64)),
        real_estate_tax: Set(Some(8_901_000i64)),
        common_area_expense: Set(Some(9_012_000i64)),
        other_operating_expense: Set(Some(10_123_000i64)),
        capital_expenditure: Set(Some(11_234_000i64)),
        net_operating_income: Set(Some(111_000_000i64)),
        depriciation: Set(Some(11_000_000i64)),
        net_income: Set(Some(100_000_000i64)),
        free_cash_flow: Set(Some(90_000_000i64)),
        occupancy_rate: Set(Some(97.5)),
        number_of_tenants: Set(Some(10)),
        security_deposit_balance: Set(Some(1_000_000i64)),
        appraisal_price: Set(Some(123_000_000_000i64)),
        appraisal_cap_rate: Set(Some(4.5)),
        appraisal_discount_rate: Set(Some(3.5)),
        direct_capitalization_price: Set(Some(1_620_000_000i64)),
        cap_rate: Set(Some(4.6)),
        discount_cash_flow_price: Set(Some(1_580_000_000i64)),
        discount_rate: Set(Some(4.4)),
        terminal_cap_rate: Set(Some(4.8)),
        acquisition_price: Set(Some(100_000_000_000i64)),
        book_value: Set(Some(1_259_000_000i64)),
        scheduled_property_tax: Set(Some(12_000_000i64)),
        net_leasable_area_total: Set(Some(1_234.5)),
        year_to_date_net_operating_income: Set(Some(24_084_000i64)),
        rental_income_per_tsubo: Set(Some(30_000i64)),
        net_operating_income_yield: Set(Some(4.8)),
        net_cash_flow_cap_rate: Set(Some(3.8)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    // // j-reitビル2の決算情報1
    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_id_2_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.to_string()),
        // 以下はnot nullなので設定（確認はしない）
        fiscal_period_start_date: Set(datetime_utc(2023, 1, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2023, 7, 30, 0).date_naive()),
        fiscal_period_operating_day: Set(180),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // // j-reitビル2の決算情報2
    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_id_2_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.to_string()),
        // 以下はnot nullなので設定（確認はしない）
        fiscal_period_start_date: Set(datetime_utc(2023, 7, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2023, 1, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(180),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn create_j_reit_financials_3_4_5(
    db: &sea_orm::DatabaseConnection,
    j_reit_mizuho_building_id_2: &str,
) -> Result<()> {
    // j-reitビル2の決算情報3
    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_id_2_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.to_string()),
        // 以下はnot nullなので設定（確認はしない）
        fiscal_period_start_date: Set(datetime_utc(2025, 1, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2025, 6, 30, 0).date_naive()),
        fiscal_period_operating_day: Set(180),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // // j-reitビル2の決算情報4
    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_id_2_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.to_string()),
        // 以下はnot nullなので設定（確認はしない）
        fiscal_period_start_date: Set(datetime_utc(2024, 1, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2024, 6, 30, 0).date_naive()),
        fiscal_period_operating_day: Set(180),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // // j-reitビル2の決算情報5
    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_id_2_5".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.to_string()),
        // 以下はnot nullなので設定（確認はしない）
        fiscal_period_start_date: Set(datetime_utc(2024, 6, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2024, 12, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(180),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}
