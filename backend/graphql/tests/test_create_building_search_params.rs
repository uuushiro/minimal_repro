mod common;
use common::*;
mod utils;
use utils::*;

use async_graphql::{Request, Result, Variables};
use graphql::metadata::Auth0Id;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::json;
use sql_entities::saved_building_search_params;

const MUTATION: &str = r#"
    mutation CreateBuildingSearchParams($input: CreateBuildingSearchParamsInput!) {
        createBuildingSearchParams(input: $input) {
            id
            name
            params
            createdAt
            updatedAt
        }
    }
"#;

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_create_building_search_params_basic() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;

    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(true))
                .variables(Variables::from_json(json!({
                    "input": {
                        "name": "test_saved_search",
                        "params": {
                            "building_name": "テストビル",
                            "min_price": 1000000000,
                            "max_price": 2000000000,
                            "location": "東京都"
                        }
                    }
                }))),
        )
        .await;

    let response = response.into_result().map_err(|e| e[0].clone())?;
    let data = response.data.into_json()?;
    let saved = data.get("createBuildingSearchParams").unwrap();

    assert_eq!(saved.get("name").unwrap(), "test_saved_search");
    assert_eq!(
        saved.get("params").unwrap().to_string(),
        json!({
            "building_name": "テストビル",
            "min_price": 1000000000,
            "max_price": 2000000000,
            "location": "東京都"
        })
        .to_string()
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_create_building_search_params_free_plan_insert_limit() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;

    // 1件目の保存（成功）
    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(false))
                .variables(Variables::from_json(json!({
                    "input": {
                        "name": "first_saved_search",
                        "params": {
                            "building_name": "テストビル1",
                            "location": "東京都"
                        }
                    }
                }))),
        )
        .await;

    let response = response.into_result().map_err(|e| e[0].clone())?;
    let data = response.data.into_json()?;
    let saved_first = data.get("createBuildingSearchParams").unwrap();
    assert_eq!(saved_first.get("name").unwrap(), "first_saved_search");

    // 2件目の保存（失敗）
    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(false))
                .variables(Variables::from_json(json!({
                    "input": {
                        "name": "second_saved_search",
                        "params": {
                            "building_name": "テストビル2",
                            "location": "東京都"
                        }
                    }
                }))),
        )
        .await;

    let response = response.into_result();
    assert!(response.is_err());
    let error = response.unwrap_err();
    assert_eq!(
        error[0].message,
        "Free plan users can only save one search condition"
    );

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_create_building_search_params_premium_inserts() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;

    // 1件目の保存
    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(true))
                .variables(Variables::from_json(json!({
                    "input": {
                        "name": "first_saved_search",
                        "params": {
                            "building_name": "テストビル1",
                            "location": "東京都"
                        }
                    }
                }))),
        )
        .await;

    let response = response.into_result().map_err(|e| e[0].clone())?;
    let data = response.data.into_json()?;
    let saved = data.get("createBuildingSearchParams").unwrap();
    assert_eq!(saved.get("name").unwrap(), "first_saved_search");

    // 2件目の保存（有料プランユーザーは制限なし）
    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(true))
                .variables(Variables::from_json(json!({
                    "input": {
                        "name": "second_saved_search",
                        "params": {
                            "building_name": "テストビル2",
                            "location": "東京都"
                        }
                    }
                }))),
        )
        .await;

    let response = response.into_result().map_err(|e| e[0].clone())?;
    let data = response.data.into_json()?;
    let saved_second = data.get("createBuildingSearchParams").unwrap();
    assert_eq!(saved_second.get("name").unwrap(), "second_saved_search");

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_create_building_search_params_concurrent_inserts() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;

    // 同時に2つの保存リクエストを実行
    let schema1 = schema.clone();
    let schema2 = schema.clone();
    let handle1 = tokio::spawn(async move {
        schema1
            .execute(
                Request::new(MUTATION)
                    .data(Auth0Id("user1".into()))
                    .data(make_roles_j_reit_premium(false))
                    .variables(Variables::from_json(json!({
                        "input": {
                            "name": "concurrent_saved_search_1",
                            "params": {
                                "building_name": "テストビル1",
                                "location": "東京都"
                            }
                        }
                    }))),
            )
            .await
    });
    let handle2 = tokio::spawn(async move {
        schema2
            .execute(
                Request::new(MUTATION)
                    .data(Auth0Id("user1".into()))
                    .data(make_roles_j_reit_premium(false))
                    .variables(Variables::from_json(json!({
                        "input": {
                            "name": "concurrent_saved_search_2",
                            "params": {
                                "building_name": "テストビル2",
                                "location": "大阪府"
                            }
                        }
                    }))),
            )
            .await
    });

    // 両方のリクエストの結果を待つ
    let _ = tokio::join!(handle1, handle2);

    // １件しかないことを確認する
    let saved_count = saved_building_search_params::Entity::find()
        .filter(
            Condition::all()
                .add(saved_building_search_params::Column::Auth0Id.eq("user1"))
                .add(saved_building_search_params::Column::Deleted.eq(0)),
        )
        .count(&db)
        .await?;
    assert_eq!(saved_count, 1);

    Ok(())
}
