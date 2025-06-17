mod common;
use common::*;
mod utils;
use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use sql_entities::j_reit_mizuho_appraisal_histories;
use utils::*;

#[tokio::test]
async fn test_j_reit_appraisal_histories() -> Result<()> {
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
    insert_test_mizuho_id_mapping(
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

    // j-reitビル1の鑑定履歴
    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        appraisal_date: Set(naive_date(2021, 1, 1)),
        appraisal_price: Set(1000000),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル2の鑑定履歴1
    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        appraisal_date: Set(naive_date(2023, 1, 1)),
        appraisal_price: Set(2000000),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル2の鑑定履歴2
    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        appraisal_date: Set(naive_date(2020, 1, 1)),
        appraisal_price: Set(3000000),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // 複数ビルの鑑定履歴に関して、各項目が正しく取得できることを確認
    // 鑑定履歴が複数ある場合は鑑定日順に並び替えられる
    {
        let request: Request = r#"
        query JReitBuilding($ids: [ID!]!) {
            jReitBuildings(ids: $ids) {
                id
                appraisalHistories {
                    id
                    appraisalPrice
                    appraisalDate
                }
            }
        }
        "#
        .into();
        let variables = json!({
            "ids": [j_reit_building_id_1, j_reit_building_id_2, j_reit_building_id_3]
        });
        let request = request
            .variables(Variables::from_json(variables))
            .data(test_roles_market_research_login(true));
        let response = schema.execute(request).await;

        assert!(response.is_ok(), "{:?}", response.errors);

        assert_eq!(
            response.data,
            value!({
                "jReitBuildings": [
                    {
                        "id": "test_id_1",
                        "appraisalHistories": [
                            {
                                "id": "test_appraisal_history_id_1",
                                "appraisalPrice": 1000000,
                                "appraisalDate": "2021-01-01"
                            }
                        ]
                    },
                    {
                        "id": "test_id_1",
                        "appraisalHistories": []
                    },
                    {
                        "id": "test_id_2",
                        "appraisalHistories": [
                            {
                                "id": "test_appraisal_history_id_3",
                                "appraisalPrice": 3000000,
                                "appraisalDate": "2020-01-01"
                            },
                            {
                                "id": "test_appraisal_history_id_2",
                                "appraisalPrice": 2000000,
                                "appraisalDate": "2023-01-01"
                            }
                        ]
                    },
                    {
                        "id": "test_id_3",
                        "appraisalHistories": []
                    }
                ]
            })
        );
    }

    Ok(())
}
