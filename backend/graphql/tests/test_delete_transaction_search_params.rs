mod common;
use common::*;
mod utils;
use utils::*;

use async_graphql::{Request, Result, Variables};
use chrono::Utc;
use graphql::metadata::Auth0Id;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde_json::json;
use sql_entities::saved_transaction_search_params;
use uuid::uuid;

const MUTATION: &str = r#"
    mutation DeleteTransactionSearchParams($input: DeleteTransactionSearchParamsInput!) {
        deleteTransactionSearchParams(input: $input) {
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
async fn test_delete_transaction_search_params() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc::now();

    saved_transaction_search_params::ActiveModel {
        id: Set(uuid!("018e1234-5678-90ab-cdef-1234567890ab")
            .into_bytes()
            .into()),
        auth0_id: Set("user1".into()),
        name: Set("test_saved_search".into()),
        params: Set(json!({
            "building_name": "テストビル",
        })),
        deleted: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
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
                        "id": "018e1234-5678-90ab-cdef-1234567890ab"
                    }
                }))),
        )
        .await;

    let response = response.into_result().map_err(|e| e[0].clone())?;
    let data = response.data.into_json()?;
    let saved = data.get("deleteTransactionSearchParams").unwrap();

    assert_eq!(
        saved.get("id").unwrap(),
        "018e1234-5678-90ab-cdef-1234567890ab"
    );
    assert_eq!(saved.get("name").unwrap(), "test_saved_search");
    assert_eq!(
        saved.get("params").unwrap().to_string(),
        json!({
            "building_name": "テストビル",
        })
        .to_string()
    );
    assert!(saved.get("createdAt").unwrap().is_string());
    assert!(saved.get("updatedAt").unwrap().is_string());

    let count_result = saved_transaction_search_params::Entity::find()
        .filter(
            Condition::all()
                .add(saved_transaction_search_params::Column::Auth0Id.eq("user1".to_string()))
                .add(saved_transaction_search_params::Column::Deleted.eq(1)),
        )
        .count(&db)
        .await?;
    assert_eq!(count_result, 1);

    Ok(())
}
