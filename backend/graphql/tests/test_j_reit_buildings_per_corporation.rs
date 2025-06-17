mod common;
use ::common::types::TransactionCategory;
use common::*;
mod utils;
use sea_orm::{ActiveModelTrait, Set};
use sql_entities::{
    j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_mizuho_appraisal_histories,
    j_reit_mizuho_cap_rate_histories, j_reit_transactions,
};
use utils::*;

use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use serde_json::json;

#[tokio::test]
async fn test_j_reit_buildings_per_corporation() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    create_j_reit_building3(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "test_company_id_2".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;
    // 同一の組み合わせに複数の transactions が紐づいた場合のデータ
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_2",
            },
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_1",
            },
            {
                "buildingId": "test_id_2",
                "corporationId": "test_company_id_1",
            },
        ]
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
            "jReitBuildingsPerCorporation": [
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
                },
            ]
        }),
    );

    Ok(())
}

#[tokio::test]
async fn test_j_reit_buildings_per_corporation_if_corporation_is_not_exist() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_xxx",
            },
        ]
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
            "jReitBuildingsPerCorporation": []
        }),
    );

    Ok(())
}

#[tokio::test]
async fn test_j_reit_buildings_per_corporation_with_cap_rate_histories() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    create_j_reit_building3(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "test_company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "test_company_id_3".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_3".into(), "test_company_id_3".into()).await;

    let j_reit_mizuho_building_id_1 =
        insert_test_mizuho_id_mapping(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    let j_reit_mizuho_building_id_2 =
        insert_test_mizuho_id_mapping(&db, "test_id_2".into(), "test_company_id_1".into()).await;
    let j_reit_mizuho_building_id_3 =
        insert_test_mizuho_id_mapping(&db, "test_id_1".into(), "test_company_id_2".into()).await;

    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(4.2),
        closing_date: Set(naive_date(2021, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        cap_rate: Set(5.1),
        closing_date: Set(naive_date(2023, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        cap_rate: Set(4.9),
        closing_date: Set(naive_date(2020, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        cap_rate: Set(3.0),
        closing_date: Set(naive_date(2025, 2, 14)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                jReitCorporation {
                    id
                }
                capRateHistories {
                    id
                }
            }
        }
        "#
    .into();

    let variables = json!({
        "ids": [
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_2",
            },
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_1",
            },
            {
                "buildingId": "test_id_2",
                "corporationId": "test_company_id_1",
            },
            {
                "buildingId": "test_id_3",
                "corporationId": "test_company_id_3",
            },
        ]
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "test_id_1",
                    "jReitCorporation": {
                        "id": "test_company_id_1",
                    },
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_1",
                        }
                    ]
                },
                {
                    "id": "test_id_1",
                    "jReitCorporation": {
                        "id": "test_company_id_2",
                    },
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_4",
                        },
                    ]
                },
                {
                    "id": "test_id_2",
                    "jReitCorporation": {
                        "id": "test_company_id_1",
                    },
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_3",
                        },
                        {
                            "id": "test_cap_rate_history_id_2",
                        }
                    ]
                },
                {
                    "id": "test_id_3",
                    "jReitCorporation": {
                        "id": "test_company_id_3",
                    },
                    "capRateHistories": []
                },
            ]
        }),
    );

    Ok(())
}

// 企業に紐づく取引データが取得できる
#[tokio::test]
async fn test_search_j_reit_buildings_transactions() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;

    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    let j_reit_corporation_id_2 =
        insert_test_j_reit_corporation(&db, "法人B".into(), "test_company_id_2".into(), 0).await;

    j_reit_transactions::ActiveModel {
        id: Set("transaction01_1".into()),
        j_reit_building_id: Set("test_id_1".into()),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        j_reit_appraisal_id: Set(None),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(naive_date(2021, 1, 31)),
        transaction_price: Set(Some(1_000_000_000)),
        leasable_area: Set(Some(1000.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id_1,
        )),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("transaction01_2".into()),
        j_reit_building_id: Set("test_id_1".into()),
        j_reit_corporation_id: Set(j_reit_corporation_id_2.clone()),
        j_reit_appraisal_id: Set(None),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(naive_date(2020, 1, 31)),
        transaction_price: Set(Some(2_000_000_000)),
        leasable_area: Set(Some(2000.0)),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id_2,
        )),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                jReitCorporation {
                    id
                }
                transactions {
                    id
                    transactionPrice
                    leasableArea
                }
            }
        }
    "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "test_id_1",
                "corporationId": j_reit_corporation_id_1,
            },
            {
                "buildingId": "test_id_1",
                "corporationId": j_reit_corporation_id_2,
            },
        ]
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildingsPerCorporation": [
                {
                    "id": "test_id_1",
                    "jReitCorporation": {
                        "id": j_reit_corporation_id_1
                    },
                    "transactions": [
                        {
                            "id": "transaction01_1",
                            "transactionPrice": 1_000_000_000,
                            "leasableArea": 1000.0
                        }
                    ]
                },
                {
                    "id": "test_id_1",
                    "jReitCorporation": {
                        "id": j_reit_corporation_id_2
                    },
                    "transactions": [
                        {
                            "id": "transaction01_2",
                            "transactionPrice": 2_000_000_000,
                            "leasableArea": 2000.0
                        }
                    ]
                }
            ]
        })
    );
    Ok(())
}

// 投資法人名でソート
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_corporation_name() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    new_j_reit_building("building_id_1", "大阪ビル")
        .insert(&db)
        .await?;
    new_j_reit_building("building_id_2", "京都ビル")
        .insert(&db)
        .await?;
    new_j_reit_corporation("company_id_1", "Builder法人")
        .insert(&db)
        .await?;
    new_j_reit_corporation("company_id_2", "Associated法人")
        .insert(&db)
        .await?;
    new_j_reit_corporation("company_id_3", "Corporate法人")
        .insert(&db)
        .await?;
    new_j_reit_transaction("transaction_id_1", "building_id_1", "company_id_1")
        .insert(&db)
        .await?;
    new_j_reit_transaction("transaction_id_2", "building_id_1", "company_id_2")
        .insert(&db)
        .await?;
    new_j_reit_transaction("transaction_id_3", "building_id_2", "company_id_3")
        .insert(&db)
        .await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                    name
                }
                buildingSpec {
                    name
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "J_REIT_CORPORATION_NAME",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_2",
                    "buildingSpec": {
                        "name": "京都ビル"
                    },
                    "jReitCorporation": {
                        "id": "company_id_3",
                        "name": "Corporate法人"
                    }
                },
                {
                    "id": "building_id_1",
                    "buildingSpec": {
                        "name": "大阪ビル"
                    },
                    "jReitCorporation": {
                        "id": "company_id_1",
                        "name": "Builder法人"
                    }
                },
                {
                    "id": "building_id_1",
                    "buildingSpec": {
                        "name": "大阪ビル"
                    },
                    "jReitCorporation": {
                        "id": "company_id_2",
                        "name": "Associated法人"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 竣工年でソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_completed_year() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let mut building1 = new_j_reit_building("building_id_1", "わからないビル");
    building1.completed_year = Set(None);
    building1.insert(&db).await?;

    let mut building2 = new_j_reit_building("building_id_2", "京都ビル");
    building2.completed_year = Set(Some(2020));
    building2.insert(&db).await?;

    let mut building3 = new_j_reit_building("building_id_3", "東京ビル");
    building3.completed_year = Set(Some(2010));
    building3.insert(&db).await?;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    insert_test_j_reit_transaction(&db, "building_id_1".into(), "company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_2".into(), "company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_3".into(), "company_id_3".into()).await;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "COMPLETED_YEAR",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 敷地面積でソート（降順）のテスト
// 敷地面積が不明な場合は最後に表示されることを確認
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_land_area_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let mut building1 = new_j_reit_building("building_id_1", "わからないビル");
    building1.land = Set(None);
    building1.insert(&db).await?;

    let mut building2 = new_j_reit_building("building_id_2", "京都ビル");
    building2.land = Set(Some(1000.0));
    building2.insert(&db).await?;

    let mut building3 = new_j_reit_building("building_id_3", "東京ビル");
    building3.land = Set(Some(2000.0));
    building3.insert(&db).await?;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    insert_test_j_reit_transaction(&db, "building_id_1".into(), "company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_2".into(), "company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_3".into(), "company_id_3".into()).await;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "LAND_AREA",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 延床面積でソート（降順）のテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_gross_floor_area() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let mut building1 = new_j_reit_building("building_id_1", "わからないビル");
    building1.gross_floor_area = Set(None);
    building1.insert(&db).await?;

    let mut building2 = new_j_reit_building("building_id_2", "小さいビル");
    building2.gross_floor_area = Set(Some(1000.0));
    building2.insert(&db).await?;

    let mut building3 = new_j_reit_building("building_id_3", "大きいビル");
    building3.gross_floor_area = Set(Some(2000.0));
    building3.insert(&db).await?;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    insert_test_j_reit_transaction(&db, "building_id_1".into(), "company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_2".into(), "company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_3".into(), "company_id_3".into()).await;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "GROSS_FLOOR_AREA",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// キャップレートでソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_cap_rate() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    insert_test_j_reit_transaction(&db, "building_id_1".into(), "company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_2".into(), "company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_3".into(), "company_id_3".into()).await;

    let mizuho_id_2 =
        insert_test_mizuho_id_mapping(&db, "building_id_2".into(), "company_id_2".into()).await;
    let mizuho_id_3 =
        insert_test_mizuho_id_mapping(&db, "building_id_3".into(), "company_id_3".into()).await;

    let cap_rate_1 = j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("cap_rate_id_1".into()),
        j_reit_mizuho_building_id: Set(mizuho_id_2.clone()),
        cap_rate: Set(6.0),
        closing_date: Set(naive_date(2022, 1, 31)),
        snowflake_deleted: Set(0),
    };
    cap_rate_1.insert(&db).await?;

    let cap_rate_2 = j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("cap_rate_id_2".into()),
        j_reit_mizuho_building_id: Set(mizuho_id_2),
        cap_rate: Set(2.0),
        closing_date: Set(naive_date(2023, 1, 31)),
        snowflake_deleted: Set(0),
    };
    cap_rate_2.insert(&db).await?;

    let cap_rate_3 = j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("cap_rate_id_3".into()),
        j_reit_mizuho_building_id: Set(mizuho_id_3),
        cap_rate: Set(4.0),
        closing_date: Set(naive_date(2023, 1, 31)),
        snowflake_deleted: Set(0),
    };
    cap_rate_3.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "CAP_RATE",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 取得時キャップレートでソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_initial_cap_rate() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    let transaction1_1 =
        new_j_reit_transaction("transaction_id_1_1", "building_id_1", "company_id_1");
    transaction1_1.insert(&db).await?;

    let mut appraisal_2_1 = new_j_reit_appraisal("appraisal_id_2_1");
    appraisal_2_1.cap_rate = Set(Some(3.0));
    appraisal_2_1.insert(&db).await?;

    let mut transaction2_1 =
        new_j_reit_transaction("transaction_id_2_1", "building_id_2", "company_id_2");
    transaction2_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction2_1.j_reit_appraisal_id = Set(Some("appraisal_id_2_1".into()));
    transaction2_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction2_1.insert(&db).await?;

    let mut appraisal_2_2 = new_j_reit_appraisal("appraisal_id_2_2");
    appraisal_2_2.cap_rate = Set(Some(7.0));
    appraisal_2_2.insert(&db).await?;

    let mut transaction2_2 =
        new_j_reit_transaction("transaction_id_2_2", "building_id_2", "company_id_2");
    transaction2_2.transaction_date = Set(naive_date(2024, 1, 1));
    transaction2_2.j_reit_appraisal_id = Set(Some("appraisal_id_2_2".into()));
    transaction2_2.transaction_category = Set(TransactionCategory::AdditionalAcquisition as i8);
    transaction2_2.insert(&db).await?;

    let mut appraisal_3_1 = new_j_reit_appraisal("appraisal_id_3_1");
    appraisal_3_1.cap_rate = Set(Some(5.0));
    appraisal_3_1.insert(&db).await?;

    let mut transaction3_1 =
        new_j_reit_transaction("transaction_id_3_1", "building_id_3", "company_id_3");
    transaction3_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction3_1.j_reit_appraisal_id = Set(Some("appraisal_id_3_1".into()));
    transaction3_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction3_1.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();

    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "INITIAL_CAP_RATE",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 鑑定価格でソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_appraised_price_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    insert_test_j_reit_transaction(&db, "building_id_1".into(), "company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_2".into(), "company_id_2".into()).await;
    insert_test_j_reit_transaction(&db, "building_id_3".into(), "company_id_3".into()).await;

    insert_test_mizuho_id_mapping(&db, "building_id_1".into(), "company_id_1".into()).await;
    let mizuho_id_2 =
        insert_test_mizuho_id_mapping(&db, "building_id_2".into(), "company_id_2".into()).await;
    let mizuho_id_3 =
        insert_test_mizuho_id_mapping(&db, "building_id_3".into(), "company_id_3".into()).await;

    let appraisal_2_1 = j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("appraisal_id_2_1".into()),
        j_reit_mizuho_building_id: Set(mizuho_id_2.clone()),
        appraisal_price: Set(800_000_000),
        appraisal_date: Set(naive_date(2020, 1, 1)),
        snowflake_deleted: Set(0),
    };
    appraisal_2_1.insert(&db).await?;
    let appraisal_2_2 = j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("appraisal_id_2_2".into()),
        j_reit_mizuho_building_id: Set(mizuho_id_2),
        appraisal_price: Set(200_000_000),
        appraisal_date: Set(naive_date(2024, 1, 1)),
        snowflake_deleted: Set(0),
    };
    appraisal_2_2.insert(&db).await?;
    let appraisal_3_1 = j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("appraisal_id_3_1".into()),
        j_reit_mizuho_building_id: Set(mizuho_id_3),
        appraisal_price: Set(500_000_000),
        appraisal_date: Set(naive_date(2020, 1, 1)),
        snowflake_deleted: Set(0),
    };
    appraisal_3_1.insert(&db).await?;

    let request = Request::new(
        r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#,
    );

    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "APPRAISED_PRICE",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 取得時鑑定価格でソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_initial_appraised_price() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    let transaction1_1 =
        new_j_reit_transaction("transaction_id_1_1", "building_id_1", "company_id_1");
    transaction1_1.insert(&db).await?;

    let mut appraisal_2_1 = new_j_reit_appraisal("appraisal_id_2_1");
    appraisal_2_1.appraisal_price = Set(Some(200000000));
    appraisal_2_1.insert(&db).await?;

    let mut transaction2_1 =
        new_j_reit_transaction("transaction_id_2_1", "building_id_2", "company_id_2");
    transaction2_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction2_1.j_reit_appraisal_id = Set(Some("appraisal_id_2_1".into()));
    transaction2_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction2_1.insert(&db).await?;

    let mut appraisal_3_1 = new_j_reit_appraisal("appraisal_id_3_1");
    appraisal_3_1.appraisal_price = Set(Some(300000000));
    appraisal_3_1.insert(&db).await?;

    let mut transaction3_1 =
        new_j_reit_transaction("transaction_id_3_1", "building_id_3", "company_id_3");
    transaction3_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction3_1.j_reit_appraisal_id = Set(Some("appraisal_id_3_1".into()));
    transaction3_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction3_1.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
    "#.into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "INITIAL_APPRAISED_PRICE",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 取得価格でソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_acquisition_price() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    let mut transaction1_1 =
        new_j_reit_transaction("transaction_id_1_1", "building_id_1", "company_id_1");
    transaction1_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction1_1.transaction_price = Set(None);
    transaction1_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction1_1.insert(&db).await?;

    let mut transaction2_1 =
        new_j_reit_transaction("transaction_id_2_1", "building_id_2", "company_id_2");
    transaction2_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction2_1.transaction_price = Set(Some(2000));
    transaction2_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction2_1.insert(&db).await?;

    let mut transaction2_2 =
        new_j_reit_transaction("transaction_id_2_2", "building_id_2", "company_id_2");
    transaction2_2.transaction_date = Set(naive_date(2024, 1, 1));
    transaction2_2.transaction_price = Set(Some(4000));
    transaction2_2.transaction_category = Set(TransactionCategory::AdditionalAcquisition as i8);
    transaction2_2.insert(&db).await?;

    let mut transaction3_1 =
        new_j_reit_transaction("transaction_id_3_1", "building_id_3", "company_id_3");
    transaction3_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction3_1.transaction_price = Set(Some(3000));
    transaction3_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction3_1.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_PRICE",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 取得日でソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_acquisition_date() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    let mut transaction1_1 =
        new_j_reit_transaction("transaction_id_1_1", "building_id_1", "company_id_1");
    transaction1_1.transaction_date = Set(naive_date(2024, 1, 1));
    transaction1_1.transaction_category = Set(TransactionCategory::AdditionalAcquisition as i8);
    transaction1_1.insert(&db).await?;

    let mut transaction2_1 =
        new_j_reit_transaction("transaction_id_2_1", "building_id_2", "company_id_2");
    transaction2_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction2_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction2_1.insert(&db).await?;

    let mut transaction2_2 =
        new_j_reit_transaction("transaction_id_2_2", "building_id_2", "company_id_2");
    transaction2_2.transaction_date = Set(naive_date(2024, 1, 1));
    transaction2_2.transaction_category = Set(TransactionCategory::AdditionalAcquisition as i8);
    transaction2_2.insert(&db).await?;

    let mut transaction3_1 =
        new_j_reit_transaction("transaction_id_3_1", "building_id_3", "company_id_3");
    transaction3_1.transaction_date = Set(naive_date(2022, 1, 1));
    transaction3_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction3_1.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_DATE",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 賃貸可能面積でソートのテスト
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_sort_by_total_leasable_area() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;
    insert_test_j_reit_building(&db, "building_id_2".into()).await;
    insert_test_j_reit_building(&db, "building_id_3".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "company_id_2".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人C".into(), "company_id_3".into(), 0).await;

    let mut transaction1_1 =
        new_j_reit_transaction("transaction_id_1_1", "building_id_1", "company_id_1");
    transaction1_1.total_leasable_area = Set(None);
    transaction1_1.insert(&db).await?;

    let mut transaction2_1 =
        new_j_reit_transaction("transaction_id_2_1", "building_id_2", "company_id_2");
    transaction2_1.total_leasable_area = Set(Some(400.0));
    transaction2_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction2_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction2_1.insert(&db).await?;

    let mut transaction2_2 =
        new_j_reit_transaction("transaction_id_2_2", "building_id_2", "company_id_2");
    transaction2_2.total_leasable_area = Set(Some(200.0));
    transaction2_2.transaction_category = Set(TransactionCategory::PartialTransfer as i8);
    transaction2_2.transaction_date = Set(naive_date(2024, 1, 1));
    transaction2_2.insert(&db).await?;

    let mut transaction3_1 =
        new_j_reit_transaction("transaction_id_3_1", "building_id_3", "company_id_3");
    transaction3_1.total_leasable_area = Set(Some(300.0));
    transaction3_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction3_1.transaction_date = Set(naive_date(2024, 1, 1));
    transaction3_1.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            },
            {
                "buildingId": "building_id_2",
                "corporationId": "company_id_2",
            },
            {
                "buildingId": "building_id_3",
                "corporationId": "company_id_3",
            }
        ],
        "sortAndPagination": {
            "sort": {
                "key": "TOTAL_LEASABLE_AREA",
                "order": "DESC"
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_3",
                    "jReitCorporation": {
                        "id": "company_id_3"
                    }
                },
                {
                    "id": "building_id_2",
                    "jReitCorporation": {
                        "id": "company_id_2"
                    }
                },
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    }
                }
            ]
        }),
    );

    Ok(())
}

// 累計の賃貸可能面積が得られる
#[tokio::test]
async fn test_j_reit_buildings_per_corporation_total_leasable_area() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    insert_test_j_reit_building(&db, "building_id_1".into()).await;

    insert_test_j_reit_corporation(&db, "法人A".into(), "company_id_1".into(), 0).await;

    let mut transaction1_1 =
        new_j_reit_transaction("transaction_id_1_1", "building_id_1", "company_id_1");
    transaction1_1.leasable_area = Set(Some(2000.0));
    transaction1_1.total_leasable_area = Set(Some(2000.0));
    transaction1_1.transaction_category = Set(TransactionCategory::InitialAcquisition as i8);
    transaction1_1.transaction_date = Set(naive_date(2020, 1, 1));
    transaction1_1.insert(&db).await?;

    let mut transaction1_2 =
        new_j_reit_transaction("transaction_id_1_2", "building_id_1", "company_id_1");
    transaction1_2.leasable_area = Set(Some(1000.0));
    transaction1_2.total_leasable_area = Set(Some(3000.0));
    transaction1_2.transaction_category = Set(TransactionCategory::AdditionalAcquisition as i8);
    transaction1_2.transaction_date = Set(naive_date(2023, 1, 1));
    transaction1_2.insert(&db).await?;

    let mut transaction1_3 =
        new_j_reit_transaction("transaction_id_1_3", "building_id_1", "company_id_1");
    transaction1_3.leasable_area = Set(Some(1500.0));
    transaction1_3.total_leasable_area = Set(Some(2500.0));
    transaction1_3.transaction_category = Set(TransactionCategory::PartialTransfer as i8);
    transaction1_3.transaction_date = Set(naive_date(2024, 1, 1));
    transaction1_3.insert(&db).await?;

    let request: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
            jReitBuildingsPerCorporation(ids: $ids, sortAndPagination: $sortAndPagination) {
                id
                jReitCorporation {
                    id
                }
                transactions {
                    transactionCategory
                    leasableArea
                    totalLeasableArea
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [
            {
                "buildingId": "building_id_1",
                "corporationId": "company_id_1",
            }
        ]
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
            "jReitBuildingsPerCorporation": [
                {
                    "id": "building_id_1",
                    "jReitCorporation": {
                        "id": "company_id_1"
                    },
                    "transactions": [
                        {
                            "transactionCategory": "INITIAL_ACQUISITION",
                            "leasableArea": 2000.0,
                            "totalLeasableArea": 2000.0
                        },
                        {
                            "transactionCategory": "ADDITIONAL_ACQUISITION",
                            "leasableArea": 1000.0,
                            "totalLeasableArea": 3000.0
                        },
                        {
                            "transactionCategory": "PARTIAL_TRANSFER",
                            "leasableArea": 1500.0,
                            "totalLeasableArea": 2500.0
                        }
                    ]
                }
            ]
        }),
    );

    Ok(())
}

fn new_j_reit_building(id: &str, name: &str) -> j_reit_buildings::ActiveModel {
    j_reit_buildings::ActiveModel {
        id: Set(id.into()),
        name: Set(name.into()),
        is_office: Set(1),
        is_retail: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_residential: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
}
fn new_j_reit_corporation(id: &str, name: &str) -> j_reit_corporations::ActiveModel {
    j_reit_corporations::ActiveModel {
        id: Set(id.into()),
        name: Set(name.into()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    }
}
fn new_j_reit_transaction(
    id: &str,
    building_id: &str,
    corporation_id: &str,
) -> j_reit_transactions::ActiveModel {
    j_reit_transactions::ActiveModel {
        id: Set(id.into()),
        j_reit_building_id: Set(building_id.into()),
        j_reit_corporation_id: Set(corporation_id.into()),
        combined_transaction_id: Set(get_combined_transaction_id(building_id, corporation_id)),
        transaction_date: Set(naive_date(2024, 1, 1)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
}

fn new_j_reit_appraisal(id: &str) -> j_reit_appraisals::ActiveModel {
    j_reit_appraisals::ActiveModel {
        id: Set(id.into()),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
}
