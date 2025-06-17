mod common;
use common::*;

use async_graphql::{value, Request, Result};
use graphql::metadata::{Auth0Id, OrganizationId};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};
use sql_entities::feedbacks;

#[tokio::test]
async fn test_create_feedback() -> Result<()> {
    // Arrange
    let TestContext { db, schema } = TestContext::new().await?;

    // Act
    let request: Request = r#"
    mutation {
        createFeedback(
            input: {
                feedbackType: "質問",
                content: "テスト問い合わせ",
                url: "https://example.com",
                userAgent: "Mozilla/5.0",
                auth0Id: "auth0id",
            }
        ) {
            id
            feedbackType
            content
            url
            userAgent
            createdAt
            updatedAt
        }
    }
    "#
    .into();
    let response = schema.execute(request.data(OrganizationId(9))).await;
    let feedbacks = feedbacks::Entity::find().all(&db).await?;
    let feedback = feedbacks.last().expect("feedback not found");

    // Assert
    assert_eq!(feedbacks.len(), 1);
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "createFeedback": {
                "id": feedback.id.to_string(),
                "feedbackType": "質問",
                "content": "テスト問い合わせ",
                "url": "https://example.com",
                "userAgent": "Mozilla/5.0",
                "createdAt": feedback.created_at.to_rfc3339(),
                "updatedAt": feedback.updated_at.to_rfc3339(),
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_start_free_trial() -> Result<()> {
    // Arrange
    let TestContext { db, schema } = TestContext::new().await?;

    // Act
    let request: Request = r#"
    mutation {
        startFreeTrial(
            input: {
                feature: MAP_PIN_CUSTOM,
            }
        ) {
            status
            startTime
            endTime
        }
    }
    "#
    .into();

    sql_entities::free_trials::ActiveModel {
        feature: Set("map_pin_custom".into()),
        auth0_id: Set("def456".into()),
        start_time: Set(chrono::Utc::now()),
        end_time: Set(chrono::Utc::now() + chrono::Duration::days(10)),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let roles = proto::Roles::default();
    let response = schema
        .execute(request.data(Auth0Id("abc123".into())).data(roles))
        .await;
    let free_trials = sql_entities::free_trials::Entity::find().all(&db).await?;
    let free_trial = free_trials.last().expect("record not found");

    // Assert
    assert_eq!(free_trials.len(), 2);
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "startFreeTrial": {
                "status": "ACTIVE",
                "startTime": free_trial.start_time.to_rfc3339(),
                "endTime": free_trial.end_time.to_rfc3339(),
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_start_free_trial_already_started() -> Result<()> {
    // Arrange
    let TestContext { db, schema } = TestContext::new().await?;

    // Act
    let request: Request = r#"
    mutation {
        startFreeTrial(
            input: {
                feature: MAP_PIN_CUSTOM,
            }
        ) {
            status
            startTime
            endTime
        }
    }
    "#
    .into();

    let existing_trial = sql_entities::free_trials::ActiveModel {
        feature: Set("map_pin_custom".into()),
        auth0_id: Set("abc123".into()),
        start_time: Set(chrono::Utc::now()),
        end_time: Set(chrono::Utc::now() + chrono::Duration::days(10)),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let roles = proto::Roles::default();
    let response = schema
        .execute(request.data(Auth0Id("abc123".into())).data(roles))
        .await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "startFreeTrial": {
                "status": "ACTIVE",
                "startTime": existing_trial.start_time.to_rfc3339(),
                "endTime": existing_trial.end_time.to_rfc3339(),
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_start_free_trial_premium_user() -> Result<()> {
    // Arrange
    let TestContext { schema, .. } = TestContext::new().await?;
    let auth0_id = "premium_test_user";

    // プレミアムユーザーのRolesを設定
    let mut roles = proto::Roles::default();
    let mut data_roles = proto::DataRoles::default();
    data_roles.j_reit_premium = true;
    roles.data = proto::MessageField::from_option(Some(data_roles));

    // Act
    let request: Request = r#"
    mutation {
        startFreeTrial(
            input: {
                feature: MAP_PIN_CUSTOM,
            }
        ) {
            status
            startTime
            endTime
        }
    }
    "#
    .into();

    let response = schema
        .execute(request.data(Auth0Id(auth0_id.into())).data(roles))
        .await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "startFreeTrial": null
        })
    );

    Ok(())
}
