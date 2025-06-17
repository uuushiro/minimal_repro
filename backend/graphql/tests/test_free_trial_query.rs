mod common;
use common::*;

use async_graphql::{value, Request, Result};
use chrono::{Duration, Utc};
use graphql::metadata::Auth0Id;
use sea_orm::{ActiveModelTrait, Set};

/// マップピンカスタム機能のトライアル状態を取得するクエリ
const QUERY_MAP_PIN_CUSTOM_TRIAL: &str = r#"
query {
    freeTrial(feature: MAP_PIN_CUSTOM) {
        status
        startTime
        endTime
    }
}
"#;

/// マップピンカスタム機能のトライアルを開始するミューテーション
const MUTATION_START_MAP_PIN_CUSTOM_TRIAL: &str = r#"
mutation {
    startFreeTrial(input: { feature: MAP_PIN_CUSTOM }) {
        status
        feature
        startTime
        endTime
    }
}
"#;

#[tokio::test]
async fn test_free_trial_status_ready() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;
    let roles = proto::Roles::default();
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id("new_user".into()))
                .data(roles),
        )
        .await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "READY",
                "startTime": null,
                "endTime": null
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_free_trial_status_active() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc::now();
    let start_time = now - Duration::days(5); // 5日前に開始
    let end_time = now + Duration::days(25); // 25日後に終了

    sql_entities::free_trials::ActiveModel {
        feature: Set("map_pin_custom".into()),
        auth0_id: Set("active_user".into()),
        start_time: Set(start_time),
        end_time: Set(end_time),
        created_at: Set(start_time),
        updated_at: Set(start_time),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let roles = proto::Roles::default();
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id("active_user".into()))
                .data(roles),
        )
        .await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "ACTIVE",
                "startTime": start_time.to_rfc3339(),
                "endTime": end_time.to_rfc3339()
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_free_trial_status_expired() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let now = Utc::now();
    let start_time = now - Duration::days(35); // 35日前に開始
    let end_time = now - Duration::days(5); // 5日前に終了

    sql_entities::free_trials::ActiveModel {
        feature: Set("map_pin_custom".into()),
        auth0_id: Set("expired_user".into()),
        start_time: Set(start_time),
        end_time: Set(end_time),
        created_at: Set(start_time),
        updated_at: Set(start_time),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let roles = proto::Roles::default();
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id("expired_user".into()))
                .data(roles),
        )
        .await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "EXPIRED",
                "startTime": start_time.to_rfc3339(),
                "endTime": end_time.to_rfc3339()
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_start_free_trial_multiple_times() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;
    let auth0_id = "integration_test_user";
    let roles = proto::Roles::default();

    // 1. 最初はトライアルが存在しないことを確認
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles.clone()),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "READY",
                "startTime": null,
                "endTime": null
            }
        })
    );

    // 2. start_free_trialを実行
    let response = schema
        .execute(
            Request::new(MUTATION_START_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles.clone()),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);

    // レスポンスから時刻を取得
    let response_data = response.data.into_json().unwrap();
    let trial = response_data.get("startFreeTrial").unwrap();
    let start_time = trial
        .get("startTime")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let end_time = trial.get("endTime").unwrap().as_str().unwrap().to_string();

    // 開始時刻と終了時刻の差分が30日であることを確認
    let start = chrono::DateTime::parse_from_rfc3339(&start_time).unwrap();
    let end = chrono::DateTime::parse_from_rfc3339(&end_time).unwrap();
    assert_eq!(end - start, Duration::days(30));

    // トライアルの内容を確認
    assert_eq!(
        response_data,
        serde_json::json!({
            "startFreeTrial": {
                "status": "ACTIVE",
                "feature": "MAP_PIN_CUSTOM",
                "startTime": start_time,
                "endTime": end_time
            }
        })
    );

    // 3. トライアルが作成されたことを確認
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles.clone()),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "ACTIVE",
                "startTime": start_time,
                "endTime": end_time
            }
        })
    );

    // 4. 再度start_free_trialを実行して、同じトライアルが返されることを確認
    let response = schema
        .execute(
            Request::new(MUTATION_START_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles.clone()),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "startFreeTrial": {
                "status": "ACTIVE",
                "feature": "MAP_PIN_CUSTOM",
                "startTime": start_time,
                "endTime": end_time
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_start_free_trial_premium_user() -> Result<()> {
    let TestContext { schema, .. } = TestContext::new().await?;
    let auth0_id = "premium_test_user";

    // プレミアムユーザーのRolesを設定
    let mut roles = proto::Roles::default();
    let mut data_roles = proto::DataRoles::default();
    data_roles.j_reit_premium = true;
    roles.data = proto::MessageField::from_option(Some(data_roles));

    // 1. 最初はトライアルが存在しないことを確認
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles.clone()),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "READY",
                "startTime": null,
                "endTime": null
            }
        })
    );

    // 2. start_free_trialを実行
    let response = schema
        .execute(
            Request::new(MUTATION_START_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles.clone()),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "startFreeTrial": null
        })
    );

    // 3. トライアルが作成されていないことを確認
    let response = schema
        .execute(
            Request::new(QUERY_MAP_PIN_CUSTOM_TRIAL)
                .data(Auth0Id(auth0_id.into()))
                .data(roles),
        )
        .await;
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "freeTrial": {
                "status": "READY",
                "startTime": null,
                "endTime": null
            }
        })
    );

    Ok(())
}
