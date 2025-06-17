mod common;
use common::*;
mod utils;
use serde_json::json;
use utils::*;

use async_graphql::{Request, Result, Variables};
use sea_orm::{ActiveModelTrait, Set};
use sql_entities::j_reit_corporations;

#[tokio::test]
async fn test_j_reit_corporations() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;

    // j-reit-corporation 1
    j_reit_corporations::ActiveModel {
        id: Set("test_id_1".into()),
        name: Set("株式会社estie".into()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reit-corporation 2
    j_reit_corporations::ActiveModel {
        id: Set("test_id_2".into()),
        name: Set("日比谷投資法人".into()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // j-reit-corporation （上場廃止）
    j_reit_corporations::ActiveModel {
        id: Set("test_id_3".into()),
        name: Set("上場廃止法人".into()),
        is_delisted: Set(1),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    // デフォルトの条件の場合、上場廃止している法人も含めて情報が全て正しく取得できることの確認
    {
        let request: Request = r#"
            query jReitCorporations {
                jReitCorporations {
                    id
                    name
                    isDelisted
                }
            }
            "#
        .into();
        let request = request.data(test_roles_market_research_login(true));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);

        let response_data = response.data.into_json()?;
        let expect = json!({
            "jReitCorporations": [
                {
                    "id": "test_id_1",
                    "name": "株式会社estie",
                    "isDelisted": false
                },
                {
                    "id": "test_id_2",
                    "name": "日比谷投資法人",
                    "isDelisted": false
                },
                {
                    "id": "test_id_3",
                    "name": "上場廃止法人",
                    "isDelisted": true
                }
            ]
        });
        assert!(is_equal_json_set(
            response_data,
            expect,
            "jReitCorporations"
        ));
    }

    // include_delistedをtrueにした場合、上場廃止している法人も含めて全て取得できることの確認
    {
        let request: Request = r#"
            query jReitCorporations ($includeDelisted: Boolean) {
                jReitCorporations (includeDelisted: $includeDelisted) {
                    id
                    isDelisted
                }
            }
            "#
        .into();
        let variables = json!({
            "includeDelisted": true
        });
        let request = request
            .variables(Variables::from_json(variables))
            .data(test_roles_market_research_login(true));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);

        let response_data = response.data.into_json()?;
        let expect = json!({
            "jReitCorporations": [
                {
                    "id": "test_id_1",
                    "isDelisted": false
                },
                {
                    "id": "test_id_2",
                    "isDelisted": false
                },
                {
                    "id": "test_id_3",
                    "isDelisted": true
                }
            ]
        });
        assert!(is_equal_json_set(
            response_data,
            expect,
            "jReitCorporations"
        ));
    }

    // include_delistedをfalseにした場合、上場廃止していない法人のみが全て取得できることの確認
    {
        let request: Request = r#"
            query jReitCorporations ($includeDelisted: Boolean) {
                jReitCorporations (includeDelisted: $includeDelisted) {
                    id
                    isDelisted
                }
            }
            "#
        .into();
        let variables = json!({
            "includeDelisted": false
        });
        let request = request
            .variables(Variables::from_json(variables))
            .data(test_roles_market_research_login(true));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);

        let response_data = response.data.into_json()?;
        let expect = json!({
            "jReitCorporations": [
                {
                    "id": "test_id_1",
                    "isDelisted": false
                },
                {
                    "id": "test_id_2",
                    "isDelisted": false
                }
            ]
        });
        assert!(is_equal_json_set(
            response_data,
            expect,
            "jReitCorporations"
        ));
    }

    Ok(())
}
