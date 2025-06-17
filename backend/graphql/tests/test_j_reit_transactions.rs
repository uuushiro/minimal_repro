mod common;
use ::common::types::TransactionCategory;
use common::*;
mod utils;
use utils::*;

use async_graphql::{value, Request, Result, Variables};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde_json::json;
use sql_entities::{j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_transactions};

#[tokio::test]
async fn test_search_transactions_all_fields() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let building = j_reit_buildings::ActiveModel {
        id: Set("building1".into()),
        is_office: Set(1),
        is_retail: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_residential: Set(1),
        is_health_care: Set(0),
        is_other: Set(0),
        office_building_id: Set(Some(1111)),
        residential_building_id: Set(Some(2222)),
        name: Set("estieビル".into()),
        address: Set(Some("東京都千代田区丸の内1-1".into())),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        nearest_station: Set(Some("東京駅徒歩3分".into())),
        completed_year: Set(Some(2000)),
        completed_month: Set(Some(1)),
        gross_floor_area: Set(Some(1000.0)),
        basement: Set(Some(2)),
        groundfloor: Set(Some(23)),
        structure: Set(Some("SRC".into())),
        floor_plan: Set(Some("1K:30".into())),
        land: Set(Some(500.0)),
        building_coverage_ratio: Set(Some(70.0)),
        floor_area_ratio: Set(Some(300.0)),
        snowflake_deleted: Set(0),
    };
    building.insert(&db).await?;

    let corporation = j_reit_corporations::ActiveModel {
        id: Set("corporation1".into()),
        name: Set("テストJ-REIT".into()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    };
    corporation.insert(&db).await?;

    let appraisal = j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal".into()),
        appraisal_date: Set(Some(datetime_utc(2025, 1, 1, 0).date_naive())),
        appraisal_price: Set(Some(3200000000)),
        appraisal_company: Set(Some("テスト鑑定会社".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    appraisal.insert(&db).await?;

    let transaction = j_reit_transactions::ActiveModel {
        id: Set("test_transaction".into()),
        j_reit_building_id: Set("building1".into()),
        j_reit_corporation_id: Set("corporation1".into()),
        combined_transaction_id: Set("building1-corporation1".into()),
        transaction_date: Set(datetime_utc(2025, 1, 1, 0).date_naive()),
        transaction_price: Set(Some(3000000000)),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        leasable_area: Set(Some(1000.0)),
        leasable_units: Set(Some(50)),
        land_ownership_type: Set(Some("所有権".into())),
        land_ownership_ratio: Set(Some(100.0)),
        building_ownership_type: Set(Some("区分所有権".into())),
        building_ownership_ratio: Set(Some(80.0)),
        transaction_partner: Set(Some("テスト不動産株式会社".into())),
        property_manager: Set(Some("PMテスト株式会社".into())),
        pml_assessment_company: Set(Some("PML評価会社".into())),
        trustee: Set(Some("信託銀行テスト".into())),
        press_release_date: Set(Some(datetime_utc(2025, 1, 2, 0).date_naive())),
        j_reit_appraisal_id: Set(Some("test_appraisal".into())),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    };
    transaction.insert(&db).await?;

    let request: Request = r#"
        query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
            searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                nodes {
                    id
                    leasableUnits
                    landOwnershipType
                    landOwnershipRatio
                    buildingOwnershipType
                    buildingOwnershipRatio
                    transactionPartner
                    transactionDate
                    transactionPrice
                    transactionCategory
                    leasableArea
                    propertyManager
                    pmlAssessmentCompany
                    trustee
                    pressReleaseDate
                    building {
                        id
                        buildingSpec {
                            name
                        }
                    }
                    appraisal {
                        id
                        appraisalDate
                        appraisalPrice
                        appraisalCompany
                    }
                    corporation {
                        id
                        name
                    }
                }
                pageInfo {
                    totalCount
                    page
                    totalPages
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "input": {
            "transactionDate": {
                "min": "2025-01-01",
                "max": "2025-12-31"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "test_transaction",
                        "leasableUnits": 50,
                        "landOwnershipType": "所有権",
                        "landOwnershipRatio": 100.0,
                        "buildingOwnershipType": "区分所有権",
                        "buildingOwnershipRatio": 80.0,
                        "transactionPartner": "テスト不動産株式会社",
                        "transactionDate": "2025-01-01",
                        "transactionPrice": 3000000000u32,
                        "transactionCategory": "ADDITIONAL_ACQUISITION",
                        "leasableArea": 1000.0,
                        "propertyManager": "PMテスト株式会社",
                        "pmlAssessmentCompany": "PML評価会社",
                        "trustee": "信託銀行テスト",
                        "pressReleaseDate": "2025-01-02",
                        "building": {
                            "id": "building1",
                            "buildingSpec": {
                                "name": "estieビル"
                            }
                        },
                        "appraisal": {
                            "id": "test_appraisal",
                            "appraisalDate": "2025-01-01",
                            "appraisalPrice": 3200000000u32,
                            "appraisalCompany": "テスト鑑定会社"
                        },
                        "corporation": {
                            "id": "corporation1",
                            "name": "テストJ-REIT"
                        }
                    }
                ],
                "pageInfo": {
                    "totalCount": 1,
                    "page": 0,
                    "totalPages": 1
                }
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_transactions_by_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // テストデータの準備
    insert_test_j_reit_building(&db, "test_building_1".into()).await;
    insert_test_j_reit_building(&db, "test_building_2".into()).await;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_corporation_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "test_corporation_2".into(), 0).await;

    let transaction1 = j_reit_transactions::ActiveModel {
        id: Set("test_transaction_1".into()),
        j_reit_building_id: Set("test_building_1".into()),
        j_reit_corporation_id: Set("test_corporation_1".into()),
        transaction_date: Set(naive_date(2024, 1, 1)),
        transaction_price: Set(Some(1000000000)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        leasable_area: Set(Some(1000.0)),
        total_leasable_area: Set(Some(1000.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_building_1",
            "test_corporation_1",
        )),
        ..Default::default()
    };
    transaction1.insert(&db).await?;

    let transaction2 = j_reit_transactions::ActiveModel {
        id: Set("test_transaction_2".into()),
        j_reit_building_id: Set("test_building_2".into()),
        j_reit_corporation_id: Set("test_corporation_2".into()),
        transaction_date: Set(naive_date(2024, 2, 1)),
        transaction_price: Set(Some(2000000000)),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        leasable_area: Set(Some(2000.0)),
        total_leasable_area: Set(Some(2000.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_building_2",
            "test_corporation_2",
        )),
        ..Default::default()
    };
    transaction2.insert(&db).await?;

    // テストクエリの実行
    let request: Request = r#"
        query transactions($ids: [ID!]!) {
            transactions(ids: $ids) {
                id
                transactionDate
                transactionPrice
                transactionCategory
                leasableArea
                totalLeasableArea
            }
        }
    "#
    .into();
    let variables = json!({
        "ids": ["test_transaction_2", "test_transaction_1"]
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // アサーション
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "transactions": [
                {
                    "id": "test_transaction_2",
                    "transactionDate": "2024-02-01",
                    "transactionPrice": 2000000000,
                    "transactionCategory": "ADDITIONAL_ACQUISITION",
                    "leasableArea": 2000.0,
                    "totalLeasableArea": 2000.0
                },
                {
                    "id": "test_transaction_1",
                    "transactionDate": "2024-01-01",
                    "transactionPrice": 1000000000,
                    "transactionCategory": "INITIAL_ACQUISITION",
                    "leasableArea": 1000.0,
                    "totalLeasableArea": 1000.0
                },
            ]
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_transactions_empty_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // テストクエリの実行
    let request: Request = r#"
        query transactions($ids: [ID!]!) {
            transactions(ids: $ids) {
                id
            }
        }
    "#
    .into();
    let variables = json!({
        "ids": []
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // アサーション
    assert!(response.is_err());
    assert!(response
        .errors
        .iter()
        .any(|e| e.message == "ids must not be empty"));

    Ok(())
}
