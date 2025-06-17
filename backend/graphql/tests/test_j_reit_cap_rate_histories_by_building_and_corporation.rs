mod common;
use common::*;
mod utils;
use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use sql_entities::j_reit_mizuho_cap_rate_histories;
use utils::*;

#[tokio::test]
async fn test_j_reit_cap_rate_histories_by_building_and_corporation() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "test_company_id_2".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_2".into()).await;

    let j_reit_mizuho_building_id_1 =
        insert_test_mizuho_id_mapping(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    let j_reit_mizuho_building_id_2 =
        insert_test_mizuho_id_mapping(&db, "test_id_2".into(), "test_company_id_2".into()).await;

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
        closing_date: Set(naive_date(2022, 1, 31)),
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
                    capRate
                    closingDate
                }
            }
        }
        "#
    .into();

    let variables = json!({
        "ids": [
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_1",
            },
            {
                "buildingId": "test_id_2",
                "corporationId": "test_company_id_2",
            },
        ]
    });

    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

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
                            "capRate": "4.20",
                            "closingDate": "2021-01-31"
                        }
                    ]
                },
                {
                    "id": "test_id_2",
                    "jReitCorporation": {
                        "id": "test_company_id_2",
                    },
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_2",
                            "capRate": "5.10",
                            "closingDate": "2022-01-31"
                        }
                    ]
                },
            ]
        }),
    );

    Ok(())
}

#[tokio::test]
async fn test_search_j_reit_buildings_per_corporation_with_cap_rate_histories() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_corporation(&db, "法人B".into(), "test_company_id_2".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    insert_test_j_reit_transaction(&db, "test_id_2".into(), "test_company_id_2".into()).await;

    let j_reit_mizuho_building_id_1 =
        insert_test_mizuho_id_mapping(&db, "test_id_1".into(), "test_company_id_1".into()).await;
    let j_reit_mizuho_building_id_2 =
        insert_test_mizuho_id_mapping(&db, "test_id_2".into(), "test_company_id_2".into()).await;

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
        closing_date: Set(naive_date(2022, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                    jReitCorporation {
                        id
                    }
                    capRateHistories {
                        id
                        capRate
                        closingDate
                    }
                }
            }
        }
        "#
    .into();

    let variables = json!({
        "searchCondition": {
            "jReitCorporationIds": ["test_company_id_1", "test_company_id_2"]
        }
    });

    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    {
                        "id": "test_id_1",
                        "jReitCorporation": {
                            "id": "test_company_id_1",
                        },
                        "capRateHistories": [
                            {
                                "id": "test_cap_rate_history_id_1",
                                "capRate": "4.20",
                                "closingDate": "2021-01-31"
                            }
                        ]
                    },
                    {
                        "id": "test_id_2",
                        "jReitCorporation": {
                            "id": "test_company_id_2",
                        },
                        "capRateHistories": [
                            {
                                "id": "test_cap_rate_history_id_2",
                                "capRate": "5.10",
                                "closingDate": "2022-01-31"
                            }
                        ]
                    }
                ]
            }
        }),
    );

    Ok(())
}

#[tokio::test]
async fn test_cap_rate_histories_with_first_and_last_parameters() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    insert_test_j_reit_corporation(&db, "法人A".into(), "test_company_id_1".into(), 0).await;
    insert_test_j_reit_transaction(&db, "test_id_1".into(), "test_company_id_1".into()).await;

    let j_reit_mizuho_building_id_1 =
        insert_test_mizuho_id_mapping(&db, "test_id_1".into(), "test_company_id_1".into()).await;

    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(4.2),
        closing_date: Set(naive_date(2020, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(4.5),
        closing_date: Set(naive_date(2021, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(4.8),
        closing_date: Set(naive_date(2022, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(5.0),
        closing_date: Set(naive_date(2023, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    let request_all: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                capRateHistories {
                    id
                    capRate
                    closingDate
                }
            }
        }
        "#
    .into();

    let variables = json!({
        "ids": [
            {
                "buildingId": "test_id_1",
                "corporationId": "test_company_id_1",
            }
        ]
    });

    let request = request_all
        .variables(Variables::from_json(variables.clone()))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildingsPerCorporation": [
                {
                    "id": "test_id_1",
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_1",
                            "capRate": "4.20",
                            "closingDate": "2020-01-31"
                        },
                        {
                            "id": "test_cap_rate_history_id_2",
                            "capRate": "4.50",
                            "closingDate": "2021-01-31"
                        },
                        {
                            "id": "test_cap_rate_history_id_3",
                            "capRate": "4.80",
                            "closingDate": "2022-01-31"
                        },
                        {
                            "id": "test_cap_rate_history_id_4",
                            "capRate": "5.00",
                            "closingDate": "2023-01-31"
                        }
                    ]
                }
            ]
        }),
    );

    let request_first: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                capRateHistories(first: 2) {
                    id
                    capRate
                    closingDate
                }
            }
        }
        "#
    .into();

    let request = request_first
        .variables(Variables::from_json(variables.clone()))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildingsPerCorporation": [
                {
                    "id": "test_id_1",
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_1",
                            "capRate": "4.20",
                            "closingDate": "2020-01-31"
                        },
                        {
                            "id": "test_cap_rate_history_id_2",
                            "capRate": "4.50",
                            "closingDate": "2021-01-31"
                        }
                    ]
                }
            ]
        }),
    );

    let request_last: Request = r#"
        query jReitBuildingsPerCorporation($ids: [GraphQLJReitBuildingIdWithCorporationId!]!) {
            jReitBuildingsPerCorporation(ids: $ids) {
                id
                capRateHistories(last: 2) {
                    id
                    capRate
                    closingDate
                }
            }
        }
        "#
    .into();

    let request = request_last
        .variables(Variables::from_json(variables.clone()))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "jReitBuildingsPerCorporation": [
                {
                    "id": "test_id_1",
                    "capRateHistories": [
                        {
                            "id": "test_cap_rate_history_id_3",
                            "capRate": "4.80",
                            "closingDate": "2022-01-31"
                        },
                        {
                            "id": "test_cap_rate_history_id_4",
                            "capRate": "5.00",
                            "closingDate": "2023-01-31"
                        }
                    ]
                }
            ]
        }),
    );

    let request_search: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                    capRateHistories(first: 1) {
                        id
                        capRate
                        closingDate
                    }
                }
            }
        }
        "#
    .into();

    let search_variables = json!({
        "searchCondition": {
            "jReitCorporationIds": ["test_company_id_1"]
        }
    });

    let request = request_search
        .variables(Variables::from_json(search_variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    {
                        "id": "test_id_1",
                        "capRateHistories": [
                            {
                                "id": "test_cap_rate_history_id_1",
                                "capRate": "4.20",
                                "closingDate": "2020-01-31"
                            }
                        ]
                    }
                ]
            }
        }),
    );

    Ok(())
}
