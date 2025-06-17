mod common;
use common::*;
mod utils;
use utils::*;

use async_graphql::{Request, Result};
use chrono::{Duration, TimeZone, Utc};
use graphql::metadata::Auth0Id;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::json;
use uuid::uuid;

const QUERY: &str = r#"
query {
    savedTransactionSearchParams {
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
async fn test_empty_result() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;
    let response = schema
        .execute(
            Request::new(QUERY)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(true)),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["savedTransactionSearchParams"], json!([]));
    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_saved_transaction_search_params_basic() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc::now();
    // user1: 2件, user2: 1件, user1 deleted: 1件
    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ab")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("test1".into()),
        params: Set(json!({"foo": 1})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;
    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ac")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("test3".into()),
        params: Set(json!({"baz": 3})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;
    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ad")
            .into_bytes()
            .into()),
        auth0_id: Set("user2".into()),
        name: Set("test2".into()),
        params: Set(json!({"bar": 2})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;
    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ae")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("deleted".into()),
        params: Set(json!({"baz": 3})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(1),
    }
    .insert(&db)
    .await?;

    // user1で取得（プレミアム: true → 2件）
    let response = schema
        .execute(
            Request::new(QUERY)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(true)),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(
        data["savedTransactionSearchParams"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // user1で取得（プレミアム: false → 1件）
    let response = schema
        .execute(
            Request::new(QUERY)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(false)),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(
        data["savedTransactionSearchParams"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_saved_transaction_search_params_order() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc::now();
    let auth0_id = "order_test_user";

    // 3件のデータを異なるcreated_atで作成
    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890af")
            .into_bytes()
            .into()),
        auth0_id: Set(auth0_id.into()),
        name: Set("oldest".into()),
        params: Set(json!({"order": 1})),
        created_at: Set(now - Duration::days(2)),
        updated_at: Set(now - Duration::days(2)),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;

    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890b0")
            .into_bytes()
            .into()),
        auth0_id: Set(auth0_id.into()),
        name: Set("middle".into()),
        params: Set(json!({"order": 2})),
        created_at: Set(now - Duration::days(1)),
        updated_at: Set(now - Duration::days(1)),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;

    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890b1")
            .into_bytes()
            .into()),
        auth0_id: Set(auth0_id.into()),
        name: Set("newest".into()),
        params: Set(json!({"order": 3})),
        created_at: Set(now),
        updated_at: Set(now),
        deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // プレミアムユーザーとして取得（3件全て）
    let response = schema
        .execute(
            Request::new(QUERY)
                .data(Auth0Id(auth0_id.into()))
                .data(make_roles_j_reit_premium(true)),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let params = data["savedTransactionSearchParams"].as_array().unwrap();
    assert_eq!(params.len(), 3);

    // created_atの降順で並んでいることを確認
    assert_eq!(params[0]["name"], "newest");
    assert_eq!(params[1]["name"], "middle");
    assert_eq!(params[2]["name"], "oldest");

    // フリープランユーザーとして取得（最新の1件のみ）
    let response = schema
        .execute(
            Request::new(QUERY)
                .data(Auth0Id(auth0_id.into()))
                .data(make_roles_j_reit_premium(false)),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let params = data["savedTransactionSearchParams"].as_array().unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0]["name"], "newest");

    Ok(())
}

#[allow(clippy::unwrap_used)]
#[tokio::test]
async fn test_saved_transaction_search_params_attributes() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc.with_ymd_and_hms(2024, 3, 15, 10, 30, 0).unwrap();

    let _ = sql_entities::saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ab")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("test_saved_search".into()),
        params: Set(json!({
            "transaction_price": 1000000000,
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
            Request::new(QUERY)
                .data(Auth0Id("user1".into()))
                .data(make_roles_j_reit_premium(true)),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let saved_params = &data["savedTransactionSearchParams"][0];

    assert_eq!(
        saved_params,
        &json!({
            "id": "018e1234-5678-90ab-cdef-1234567890ab",
            "name": "test_saved_search",
            "params": {
                "transaction_price": 1000000000,
                "max_price": 2000000000,
                "location": "東京都"
            },
            "createdAt": "2024-03-15T10:30:00+00:00",
            "updatedAt": "2024-03-15T10:30:00+00:00"
        })
    );
    Ok(())
}
