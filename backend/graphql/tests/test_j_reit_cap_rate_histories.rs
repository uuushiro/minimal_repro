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
async fn test_j_reit_cap_rate_histories() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let j_reit_building_id_1 = insert_test_j_reit_building(&db, "test_id_1".into()).await;
    let j_reit_building_id_2 = insert_test_j_reit_building(&db, "test_id_2".into()).await;
    let j_reit_building_id_3 = insert_test_j_reit_building(&db, "test_id_3".into()).await;
    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(&db, "法人1".into(), "test_corporation_id_1".into(), 0)
            .await;
    let j_reit_corporation_id_2 =
        insert_test_j_reit_corporation(&db, "法人2".into(), "test_corporation_id_2".into(), 0)
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
    // building 1 に紐づくが、投資法人が別パターン
    let j_reit_mizuho_building_id_3 = insert_test_mizuho_id_mapping(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_2.clone(),
    )
    .await;
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
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_1.clone(),
        j_reit_corporation_id_2.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_3.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    // j-reitビル1のキャップレート履歴1
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(4.2),
        closing_date: Set(naive_date(2021, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル2のキャップレート履歴1
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        cap_rate: Set(5.1),
        closing_date: Set(naive_date(2023, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル2のキャップレート履歴2
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        cap_rate: Set(4.9),
        closing_date: Set(naive_date(2020, 1, 31)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル1のキャップレート履歴2(投資法人が異なる)
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        cap_rate: Set(3.0),
        closing_date: Set(naive_date(2025, 2, 14)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル1のキャップレート履歴3(投資法人が異なる)
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_5".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        cap_rate: Set(4.0),
        closing_date: Set(naive_date(2025, 3, 14)),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // 複数ビルのキャップレート履歴に関して、各項目が正しく取得できることを確認
    // キャップレート履歴が複数ある場合は決算期の締め日順に並び替えられる
    {
        let request: Request = r#"
        query JReitBuilding($ids: [ID!]!) {
            jReitBuildings(ids: $ids) {
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
            "ids": [j_reit_building_id_1.clone(), j_reit_building_id_2.clone(), j_reit_building_id_3.clone()]
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
                        "capRateHistories": [
                            {
                                "id": "test_cap_rate_history_id_1",
                                "capRate": "4.20",
                                "closingDate": "2021-01-31"
                            }
                        ]
                    },
                    {
                        "id": "test_id_1",
                        "capRateHistories": [
                            {
                                "id": "test_cap_rate_history_id_4",
                                "capRate": "3.00",
                                "closingDate": "2025-02-14"
                            },
                            {
                                "id": "test_cap_rate_history_id_5",
                                "capRate": "4.00",
                                "closingDate": "2025-03-14"
                            }
                        ]
                    },
                    {
                        "id": "test_id_2",
                        "capRateHistories": [
                            {
                                "id": "test_cap_rate_history_id_3",
                                "capRate": "4.90",
                                "closingDate": "2020-01-31"
                            },
                            {
                                "id": "test_cap_rate_history_id_2",
                                "capRate": "5.10",
                                "closingDate": "2023-01-31"
                            }
                        ]
                    },
                    {
                        "id": "test_id_3",
                        "capRateHistories": []
                    }
                ]
            })
        );
    }

    Ok(())
}
