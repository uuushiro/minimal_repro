mod common;
use ::common::types::TransactionCategory;
use common::*;
mod utils;
use chrono::NaiveDate;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sql_entities::{j_reit_mizuho_financials, j_reit_transactions};
use utils::*;

use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use serde_json::json;

#[tokio::test]
// j-reitビル 1 に関して、各項目が正しく取得できることを確認
async fn test_j_reit_buildings_attributes() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    create_j_reit_transactions(&db).await?;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!]) {
            jReitBuildings(ids: $ids) {
                id
                officeBuildingId
                residentialBuildingId
                assetType {
                    isOffice
                    isRetail
                    isHotel
                    isLogistic
                    isResidential
                    isHealthCare
                    isOther
                }
                buildingSpec {
                    name
                    address
                    latitude
                    longitude
                    nearestStation
                    completedYear
                    completedMonth
                    grossFloorArea
                    basement
                    groundfloor
                    structure
                    floorPlan
                }
                landSpec {
                    land
                    buildingCoverageRatio
                    floorAreaRatio
                }
                # # city{
                # #     id
                # # }
                transactions {
                    id
                    isBulk
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
                    appraisal {
                        capRate
                    }
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": ["test_id_1"]
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
                    "officeBuildingId": "1111",
                    "residentialBuildingId": "2222",
                    "assetType": {
                        "isOffice": true,
                        "isRetail": false,
                        "isHotel": false,
                        "isLogistic": false,
                        "isResidential": true,
                        "isHealthCare": false,
                        "isOther": false
                    },
                    "buildingSpec": {
                        "name": "estieビル",
                        "address": "東京都千代田区丸の内1-1",
                        "latitude": 35.0,
                        "longitude": 139.0,
                        "nearestStation": "東京駅徒歩3分",
                        "completedYear": 2000,
                        "completedMonth": 1,
                        "grossFloorArea": "1000.00",
                        "basement": 2,
                        "groundfloor": 23,
                        "structure": "SRC",
                        "floorPlan": "1K:30",
                    },
                    "landSpec": {
                        "land": "500.00",
                        "buildingCoverageRatio": "70.00",
                        "floorAreaRatio": "300.00"
                    },
                    // "city": {
                    //     "id": TEST_CITY_ID_MARUNOUCHI
                    // },
                    "transactions": [
                        {
                            "id": "test_transaction_id_1",
                            "isBulk": false,
                            "leasableUnits": 40,
                            "landOwnershipType": "所有権",
                            "landOwnershipRatio": 100.0,
                            "buildingOwnershipType": "全体所有",
                            "buildingOwnershipRatio": 100.0,
                            "transactionPartner": "売主A",
                            "transactionDate": "2021-01-01",
                            "transactionPrice": 1000000000,
                            "transactionCategory": "INITIAL_ACQUISITION",
                            "leasableArea": 100.00,
                            "propertyManager": "estie PM",
                            "pmlAssessmentCompany": "estie調査会社",
                            "trustee": null,
                            "appraisal": {
                                "capRate": 5.0,
                            }
                        },
                        {
                            "id": "test_transaction_id_2",
                            "isBulk": false,
                            "leasableUnits": 60,
                            "landOwnershipType": null,
                            "landOwnershipRatio": 50.0,
                            "buildingOwnershipType": null,
                            "buildingOwnershipRatio": 33.3,
                            "transactionPartner": null,
                            "transactionDate": "2021-01-01",
                            "transactionPrice": null,
                            "transactionCategory": "ADDITIONAL_ACQUISITION",
                            "leasableArea": 200.00,
                            "propertyManager": null,
                            "pmlAssessmentCompany": null,
                            "trustee": null,
                            "appraisal": {
                                "capRate": 5.0,
                            }
                        },
                        {
                            "id": "test_transaction_id_3",
                            "isBulk": true,
                            "leasableUnits": 0,
                            "landOwnershipType": "所有権",
                            "landOwnershipRatio": 33.3,
                            "buildingOwnershipType": null,
                            "buildingOwnershipRatio": null,
                            "transactionPartner": null,
                            "transactionDate": "2023-01-01",
                            "transactionPrice": null,
                            "transactionCategory": "INITIAL_ACQUISITION",
                            "leasableArea": 500.00,
                            "propertyManager": null,
                            "pmlAssessmentCompany": null,
                            "trustee": null,
                            "appraisal": {
                                "capRate": 5.0,
                            }
                        },
                    ]
                }
            ]
        }),
    );

    Ok(())
}

#[tokio::test]
// 複数の j_reit_building_id に対して該当の物件が正しく取得できることを確認
async fn test_j_reit_buildings_by_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!]) {
        jReitBuildings(ids: $ids) {
            id
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": ["test_id_1", "test_id_2"]
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
                {"id": "test_id_1"},
                {"id": "test_id_2"}
            ]
        })
    );

    Ok(())
}

#[tokio::test]
// ids を指定しない場合全てのデータが取得できることの確認
async fn test_j_reit_buildings_all() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
        query jReitBuildings($ids: [ID!]) {
            jReitBuildings(ids: $ids) {
                id
            }
        }
        "#
    .into();
    let request = request.data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [
               {"id": "test_id_1"},
               {"id": "test_id_2"}
            ]
        })
    );

    Ok(())
}

#[tokio::test]
// ids が空の場合空のデータが返ることの確認
async fn test_j_reit_buildings_empty_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!]) {
        jReitBuildings(ids: $ids) {
            id
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": []
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
            "jReitBuildings": []
        }),
    );

    Ok(())
}

#[tokio::test]
// ソート条件を指定した場合、正しくソートされたデータが取得できることの確認(j_reit_buildings のカラムを指定)
async fn test_j_reit_buildings_with_sort_by_j_reit_buildings_column() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!], $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        jReitBuildings(ids: $ids, sortAndPagination: $sortAndPagination) {
            id
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": null,
        "sortAndPagination": {
            "sort": {
                "key": "COMPLETED_YEAR",
                "order": "ASC"
            }
        }
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
                { "id": "test_id_2" },
                { "id": "test_id_1" }
            ]
        }),
    );

    Ok(())
}

#[tokio::test]
// ソート条件を指定した場合、正しくソートされたデータが取得できることの確認(j_reit_corporations のカラムを指定)
async fn test_j_reit_buildings_with_sort_by_j_reit_corporations_column() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    create_j_reit_building3(&db).await?;

    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "A法人".into(), "test_company_id_1".into(), 0).await;
    let j_reit_corporation_id_2 =
        insert_test_j_reit_corporation(&db, "B法人".into(), "test_company_id_2".into(), 0).await;
    let j_reit_corporation_id_3 =
        insert_test_j_reit_corporation(&db, "C法人".into(), "test_company_id_3".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), j_reit_corporation_id_2.clone()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), j_reit_corporation_id_1.clone()).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), j_reit_corporation_id_1.clone()).await;
    insert_test_j_reit_transaction(&db, "test_id_3".into(), j_reit_corporation_id_3.clone()).await;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!], $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        jReitBuildings(ids: $ids, sortAndPagination: $sortAndPagination) {
            id
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": null,
        "sortAndPagination": {
            "sort": {
                "key": "J_REIT_CORPORATION_NAME",
                "order": "ASC"
            }
        }
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
                { "id": "test_id_1" },
                { "id": "test_id_2" },
                { "id": "test_id_1" },
                { "id": "test_id_3" },
            ]
        }),
    );

    Ok(())
}

#[tokio::test]
// ページネーションを指定した場合、正しくページネーションされたデータが取得できることの確認
async fn test_j_reit_buildings_with_pagination() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!], $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        jReitBuildings(ids: $ids, sortAndPagination: $sortAndPagination) {
            id
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": null,
        "sortAndPagination": {
            "pagination": {
                "limit": 1,
                "offset": 1
            }
        }
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
                { "id": "test_id_2" }
            ]
        }),
    );

    Ok(())
}

#[tokio::test]
// 複数のtransactionsが紐づいている場合に正しく取得できることを確認
async fn test_j_reit_buildings_with_multiple_transactions() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "test_company_id_2".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_2".into()).await;

    // ２つtransactionが紐づいているデータ
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!]) {
        jReitBuildings(ids: $ids) {
            id
            jReitCorporation {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": ["test_id_1", "test_id_2"]
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
                    "jReitCorporation": {
                        "id": "test_company_id_1",
                    }
                },
                {
                    "id": "test_id_1",
                    "jReitCorporation": {
                        "id": "test_company_id_2",
                    }
                },
                {
                    "id": "test_id_2",
                    "jReitCorporation": {
                        "id": "test_company_id_1",
                    }
                }
            ]
        })
    );

    Ok(())
}

// 初回取得の日付が同じ場合、初回取得を最初に取得する
#[tokio::test]
async fn test_j_reit_buildings_with_multiple_transactions_with_same_initial_acquisition_date(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    let transaction1_1 = j_reit_transactions::ActiveModel {
        id: Set("transaction_id_1_1".into()),
        j_reit_building_id: Set("building_id_1".into()),
        j_reit_corporation_id: Set("test_company_id_1".into()),
        combined_transaction_id: Set(get_combined_transaction_id(
            "building_id_1",
            "test_company_id_1",
        )),
        transaction_date: Set(naive_date(2024, 1, 1)),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    };
    transaction1_1.clone().insert(&db).await?;
    let mut transaction1_2 = transaction1_1.clone();
    transaction1_2.id = Set("transaction_id_1_2".into());
    transaction1_2.combined_transaction_id = Set(get_combined_transaction_id(
        "building_id_1",
        "test_company_id_1",
    ));
    transaction1_2.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction1_2.insert(&db).await?;

    let request: Request = r#"
    query jReitBuildings($ids: [ID!]) {
        jReitBuildings(ids: $ids) {
            id
            transactions {
                id
                transactionDate
                transactionCategory
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "ids": ["building_id_1"]
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
                    "id": "building_id_1",
                    "transactions": [
                        { "id": "transaction_id_1_2", "transactionDate": "2024-01-01", "transactionCategory": "INITIAL_ACQUISITION" },
                        { "id": "transaction_id_1_1", "transactionDate": "2024-01-01", "transactionCategory": "ADDITIONAL_ACQUISITION" }
                    ]
                }
            ]
        })
    );
    Ok(())
}

#[tokio::test]
// 初回取得トランザクションのみを取得できることを確認
async fn test_j_reit_buildings_initial_acquisition() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // ビルディングと法人を作成
    insert_test_j_reit_building(&db, "building_1".into()).await;
    insert_test_j_reit_corporation(&db, "法人A".into(), "corp_1".into(), 0).await;

    // 初回取得トランザクションを作成
    let initial_acquisition = j_reit_transactions::ActiveModel {
        id: Set("initial_acq_1".into()),
        combined_transaction_id: Set(get_combined_transaction_id("building_1", "corp_1")),
        j_reit_building_id: Set("building_1".into()),
        j_reit_corporation_id: Set("corp_1".into()),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(NaiveDate::parse_from_str("2020-01-01", "%Y-%m-%d").unwrap()),
        transaction_price: Set(Some(1000000000)),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    initial_acquisition.insert(&db).await?;

    // 追加取得トランザクションも作成
    let additional_acquisition = j_reit_transactions::ActiveModel {
        id: Set("additional_acq_1".into()),
        combined_transaction_id: Set(get_combined_transaction_id("building_1", "corp_1")),
        j_reit_building_id: Set("building_1".into()),
        j_reit_corporation_id: Set("corp_1".into()),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        transaction_date: Set(NaiveDate::parse_from_str("2021-01-01", "%Y-%m-%d").unwrap()),
        transaction_price: Set(Some(500000000)),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    additional_acquisition.insert(&db).await?;

    let request: Request = r#"
    query {
        jReitBuildings(ids: ["building_1"]) {
            id
            initialAcquisition {
                id
                transactionCategory
                transactionDate
                transactionPrice
            }
        }
    }
    "#
    .into();

    let request = request.data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [{
                "id": "building_1",
                "initialAcquisition": {
                    "id": "initial_acq_1",
                    "transactionCategory": "INITIAL_ACQUISITION",
                    "transactionDate": "2020-01-01",
                    "transactionPrice": 1000000000
                }
            }]
        })
    );
    Ok(())
}

#[tokio::test]
// 最新のトランザクションを取得できることを確認
async fn test_j_reit_buildings_latest_transaction() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // ビルディングと法人を作成
    insert_test_j_reit_building(&db, "building_1".into()).await;
    insert_test_j_reit_corporation(&db, "法人A".into(), "corp_1".into(), 0).await;

    // 複数のトランザクションを異なる日付で作成
    let transactions = vec![
        (
            "trans_1",
            "2020-01-01",
            TransactionCategory::InitialAcquisition,
        ),
        (
            "trans_2",
            "2021-06-15",
            TransactionCategory::AdditionalAcquisition,
        ),
        (
            "trans_3",
            "2022-12-31",
            TransactionCategory::PartialTransfer,
        ),
    ];

    for (id, date, category) in transactions {
        let transaction = j_reit_transactions::ActiveModel {
            id: Set(id.into()),
            combined_transaction_id: Set(get_combined_transaction_id("building_1", "corp_1")),
            j_reit_building_id: Set("building_1".into()),
            j_reit_corporation_id: Set("corp_1".into()),
            transaction_category: Set(category as i8),
            transaction_date: Set(NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap()),
            transaction_price: Set(Some(1000000000)),
            is_bulk: Set(0),
            snowflake_deleted: Set(0),
            ..Default::default()
        };
        transaction.insert(&db).await?;
    }

    let request: Request = r#"
    query {
        jReitBuildings(ids: ["building_1"]) {
            id
            latestTransaction {
                id
                transactionDate
                transactionCategory
            }
        }
    }
    "#
    .into();

    let request = request.data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [{
                "id": "building_1",
                "latestTransaction": {
                    "id": "trans_3",
                    "transactionDate": "2022-12-31",
                    "transactionCategory": "PARTIAL_TRANSFER"
                }
            }]
        })
    );
    Ok(())
}

#[tokio::test]
// 譲渡トランザクション（全部譲渡・一部譲渡）を取得できることを確認
async fn test_j_reit_buildings_transfer_transaction() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // ビルディングと法人を作成
    insert_test_j_reit_building(&db, "building_1".into()).await;
    insert_test_j_reit_corporation(&db, "法人A".into(), "corp_1".into(), 0).await;

    // 初回取得と全部譲渡のトランザクションを作成
    let initial = j_reit_transactions::ActiveModel {
        id: Set("initial_1".into()),
        combined_transaction_id: Set(get_combined_transaction_id("building_1", "corp_1")),
        j_reit_building_id: Set("building_1".into()),
        j_reit_corporation_id: Set("corp_1".into()),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(NaiveDate::parse_from_str("2020-01-01", "%Y-%m-%d").unwrap()),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    initial.insert(&db).await?;

    let transfer = j_reit_transactions::ActiveModel {
        id: Set("transfer_1".into()),
        combined_transaction_id: Set(get_combined_transaction_id("building_1", "corp_1")),
        j_reit_building_id: Set("building_1".into()),
        j_reit_corporation_id: Set("corp_1".into()),
        transaction_category: Set(TransactionCategory::FullTransfer as i8),
        transaction_date: Set(NaiveDate::parse_from_str("2023-12-31", "%Y-%m-%d").unwrap()),
        transaction_price: Set(Some(1500000000)),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    transfer.insert(&db).await?;

    let request: Request = r#"
    query {
        jReitBuildings(ids: ["building_1"]) {
            id
            transferTransaction {
                id
                transactionCategory
                transactionDate
                transactionPrice
            }
        }
    }
    "#
    .into();

    let request = request.data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [{
                "id": "building_1",
                "transferTransaction": {
                    "id": "transfer_1",
                    "transactionCategory": "FULL_TRANSFER",
                    "transactionDate": "2023-12-31",
                    "transactionPrice": 1500000000
                }
            }]
        })
    );
    Ok(())
}

#[tokio::test]
// 最新の決算データを取得できることを確認
async fn test_j_reit_buildings_latest_financial() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // ビルディングと法人を作成
    let building_id = "building_1";
    let corporation_id = "corp_1";
    insert_test_j_reit_building(&db, building_id.into()).await;
    insert_test_j_reit_corporation(&db, "法人A".into(), corporation_id.into(), 0).await;

    // トランザクションを作成（必須）
    insert_test_j_reit_transaction(&db, building_id.into(), corporation_id.into()).await;

    // Mizuho IDマッピングを作成
    let mizuho_building_id =
        insert_test_mizuho_id_mapping(&db, building_id.into(), corporation_id.into()).await;

    // 複数の決算データを作成（異なる期末日で）
    j_reit_mizuho_financials::ActiveModel {
        id: Set("financial_1".into()),
        j_reit_mizuho_building_id: Set(mizuho_building_id.clone()),
        fiscal_period: Set(Some("第1期".into())),
        fiscal_period_start_date: Set(NaiveDate::parse_from_str("2022-01-01", "%Y-%m-%d").unwrap()),
        fiscal_period_end_date: Set(NaiveDate::parse_from_str("2022-06-30", "%Y-%m-%d").unwrap()),
        fiscal_period_operating_day: Set(180),
        appraisal_price: Set(Some(1000000000)),
        appraisal_cap_rate: Set(Some(4.5)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("financial_2".into()),
        j_reit_mizuho_building_id: Set(mizuho_building_id.clone()),
        fiscal_period: Set(Some("第2期".into())),
        fiscal_period_start_date: Set(NaiveDate::parse_from_str("2022-07-01", "%Y-%m-%d").unwrap()),
        fiscal_period_end_date: Set(NaiveDate::parse_from_str("2022-12-31", "%Y-%m-%d").unwrap()),
        fiscal_period_operating_day: Set(180),
        appraisal_price: Set(Some(1100000000)),
        appraisal_cap_rate: Set(Some(4.3)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("financial_3".into()),
        j_reit_mizuho_building_id: Set(mizuho_building_id),
        fiscal_period: Set(Some("第3期".into())),
        fiscal_period_start_date: Set(NaiveDate::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap()),
        fiscal_period_end_date: Set(NaiveDate::parse_from_str("2023-06-30", "%Y-%m-%d").unwrap()),
        fiscal_period_operating_day: Set(180),
        appraisal_price: Set(Some(1200000000)),
        appraisal_cap_rate: Set(Some(4.2)),
        cap_rate: Set(Some(4.2)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let request: Request = r#"
    query {
        jReitBuildings(ids: ["building_1"]) {
            id
            latestFinancial {
                fiscalPeriod {
                    endDate
                }
                appraisal {
                    appraisalPrice
                    capRate
                }
            }
        }
    }
    "#
    .into();

    let request = request.data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildings": [{
                "id": "building_1",
                "latestFinancial": {
                    "fiscalPeriod": {
                        "endDate": "2023-06-30"
                    },
                    "appraisal": {
                        "appraisalPrice": 1200000000,
                        "capRate": "4.20"
                    }
                }
            }]
        })
    );
    Ok(())
}
