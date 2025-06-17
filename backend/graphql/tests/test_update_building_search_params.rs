mod common;
use common::*;
mod utils;
use utils::*;

use async_graphql::{Request, Result, Variables};
use chrono::{TimeZone, Utc};
use graphql::metadata::Auth0Id;
use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use sql_entities::saved_building_search_params;
use uuid::uuid;

const MUTATION: &str = r#"
    mutation UpdateBuildingSearchParams($input: UpdateBuildingSearchParamsInput!) {
        updateBuildingSearchParams(input: $input) {
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
async fn test_update_building_search_params_basic() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc.with_ymd_and_hms(2024, 3, 15, 10, 30, 0).unwrap();

    saved_building_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ab")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("test_saved_search".into()),
        params: Set(json!({
            "building_name": "テストビル",
            "min_price": 1000000000,
            "max_price": 2000000000,
            "location": "東京都"
        })),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;

    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(false))
                .variables(Variables::from_json(json!({
                    "input": {
                        "id": "018e1234-5678-90ab-cdef-1234567890ab",
                        "name": "edited_saved_search",
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
    let saved = data.get("updateBuildingSearchParams").unwrap();

    assert_eq!(
        saved.get("id").unwrap(),
        "018e1234-5678-90ab-cdef-1234567890ab"
    );
    assert_eq!(saved.get("name").unwrap(), "edited_saved_search");
    assert_eq!(
        saved.get("params").unwrap(),
        &json!({
            "building_name": "テストビル",
            "min_price": 1000000000,
            "max_price": 2000000000,
            "location": "東京都"
        })
    );
    assert_eq!(
        saved.get("createdAt").unwrap().as_str().unwrap(),
        now.to_rfc3339()
    );
    assert!(saved.get("updatedAt").unwrap().is_string());

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_update_building_search_params_edit_other_user() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc.with_ymd_and_hms(2024, 3, 15, 10, 30, 0).unwrap();

    // user1が検索条件を保存
    saved_building_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ab")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("test_saved_search".into()),
        params: Set(json!({
            "building_name": "テストビル",
            "min_price": 1000000000,
            "max_price": 2000000000,
            "location": "東京都"
        })),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // user2がuser1の検索条件を編集しようとする（失敗）
    let response = schema
        .execute(
            Request::new(MUTATION)
                .data(Auth0Id("user2".into()))
                .data(make_roles_j_reit_premium(true))
                .variables(Variables::from_json(json!({
                    "input": {
                        "id": "018e1234-5678-90ab-cdef-1234567890ab",
                        "name": "user2_trying_to_edit",
                        "params": {
                            "building_name": "編集されたビル",
                            "location": "大阪府"
                        }
                    }
                }))),
        )
        .await;

    let response = response.into_result();
    assert!(response.is_err());
    let error = response.unwrap_err();
    assert_eq!(error[0].message, "Not found");

    Ok(())
}
