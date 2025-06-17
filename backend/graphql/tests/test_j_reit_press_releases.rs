mod common;
use common::*;
mod utils;
use serde_json::json;
use utils::*;

use async_graphql::{Request, Result, Variables};
use sea_orm::{ActiveModelTrait, Set};
use sql_entities::j_reit_mizuho_press_releases;

#[tokio::test]
async fn test_j_reit_press_releases() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    let j_reit_building_id_1 = insert_test_j_reit_building(&db, "test_building_id_1".into()).await;
    let j_reit_building_id_2 = insert_test_j_reit_building(&db, "test_building_id_2".into()).await;
    let j_reit_building_id_3 = insert_test_j_reit_building(&db, "test_building_id_3".into()).await;
    let j_reit_corporation_id_1 = insert_test_j_reit_corporation(
        &db,
        "test_corporation_1".into(),
        "test_corporation_id_1".into(),
        0,
    )
    .await;
    let j_reit_corporation_id_2 = insert_test_j_reit_corporation(
        &db,
        "test_corporation_2".into(),
        "test_corporation_id_2".into(),
        0,
    )
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
        j_reit_corporation_id_2.clone(),
    )
    .await;
    insert_test_j_reit_transaction(
        &db,
        j_reit_building_id_3.clone(),
        j_reit_corporation_id_1.clone(),
    )
    .await;

    // j-reitビル1のプレスリリース
    j_reit_mizuho_press_releases::ActiveModel {
        id: Set("test_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        release_date: Set(naive_date(2021, 1, 1)),
        title: Set("資産取得に関するお知らせ".into()),
        url: Set("https://estie.jp/press_release.pdf".into()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル2のプレスリリース1
    j_reit_mizuho_press_releases::ActiveModel {
        id: Set("test_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        release_date: Set(naive_date(2023, 1, 1)),
        title: Set("資産譲渡に関するお知らせ".into()),
        url: Set("https://estie.jp/press_release2.pdf".into()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reitビル2のプレスリリース2
    j_reit_mizuho_press_releases::ActiveModel {
        id: Set("test_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        release_date: Set(naive_date(2020, 1, 1)),
        title: Set("資産取得に関するお知らせ".into()),
        url: Set("https://estie.jp/press_release1.pdf".into()),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // 複数ビルのプレスリリースに関して、各項目が正しく取得できることを確認
    // プレスリリースが複数ある場合はリリース日順に並び替えられる
    {
        let request: Request = r#"
        query JReitBuilding($ids: [ID!]!) {
            jReitBuildings(ids: $ids) {
                id
                pressReleases {
                    id
                    title
                    url
                    releaseDate
                }
            }
        }
        "#
        .into();
        let variables = json!({
            "ids": [j_reit_building_id_1, j_reit_building_id_2, j_reit_building_id_3],
        });
        let request = request
            .variables(Variables::from_json(variables))
            .data(test_roles_market_research_login(true));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);

        let response_data = response.data.into_json().expect("response is not json");
        let body = response_data["jReitBuildings"]
            .as_array()
            .expect("jReitBuildings is not an array");

        let j_reit_building_1 = body
            .iter()
            .find(|x| x["id"].as_str().expect("id is not a string") == j_reit_building_id_1)
            .expect("j_reit_building_1 not found");
        let j_reit_building_2 = body
            .iter()
            .find(|x| x["id"].as_str().expect("id is not a string") == j_reit_building_id_2)
            .expect("j_reit_building_2 not found");
        let j_reit_building_3 = body
            .iter()
            .find(|x| x["id"].as_str().expect("id is not a string") == j_reit_building_id_3)
            .expect("j_reit_building_3 not found");

        assert_eq!(
            j_reit_building_1.clone(),
            json!({
                    "id": j_reit_building_id_1,
                    "pressReleases": [{
                        "id": "test_id_1",
                        "title": "資産取得に関するお知らせ",
                        "url": "https://estie.jp/press_release.pdf",
                        "releaseDate": "2021-01-01",
                    }]
            })
        );
        assert_eq!(
            j_reit_building_2.clone(),
            json!({
                    "id": j_reit_building_id_2,
                    "pressReleases": [
                        {
                            "id": "test_id_3",
                            "title": "資産取得に関するお知らせ",
                            "url": "https://estie.jp/press_release1.pdf",
                            "releaseDate": "2020-01-01",
                        },
                        {
                            "id": "test_id_2",
                            "title": "資産譲渡に関するお知らせ",
                            "url": "https://estie.jp/press_release2.pdf",
                            "releaseDate": "2023-01-01",
                        }
                    ]
            })
        );
        assert_eq!(
            j_reit_building_3.clone(),
            json!({
                    "id": j_reit_building_id_3,
                    "pressReleases": []
            })
        );
    }

    Ok(())
}
