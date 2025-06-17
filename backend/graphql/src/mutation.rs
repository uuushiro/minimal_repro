use crate::metadata::Auth0Id;
use crate::types::free_trials::{
    GraphQLFreeTrial, GraphQLFreeTrialFeature, TrialIssueResult, TrialManager,
};
use crate::types::saved_building_search_params::GraphQLSavedBuildingSearchParams;
use crate::types::saved_transaction_search_params::GraphQLSavedTransactionSearchParams;
use crate::utils::uuid::{generate_uuid_v7_binary, UuidVec};
use crate::{metadata::OrganizationId, types::feedbacks::GraphQLFeedback};
use async_graphql::{Context, Error, InputObject, Object, Result, ID};
use chrono::Utc;
use datadog_api_client::{
    datadog::Configuration,
    datadogV2::{
        api_metrics::{MetricsAPI, SubmitMetricsOptionalParams},
        model::{MetricIntakeType, MetricPayload, MetricPoint, MetricSeries},
    },
};
use proto::Roles;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set, TransactionTrait,
};
use sea_query::Condition;
use serde_json::Value;
use sql_entities::{feedbacks, saved_building_search_params, saved_transaction_search_params};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_feedback(
        &self,
        ctx: &Context<'_>,
        input: CreateFeedbackInput,
    ) -> Result<GraphQLFeedback> {
        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let feedback = feedbacks::ActiveModel {
            feedback_type: Set(input.feedback_type),
            content: Set(input.content),
            url: Set(input.url),
            user_agent: Set(input.user_agent),
            auth0_id: Set(input.auth0_id),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        let feedback = feedback.insert(db).await?;
        let env_name = std::env::var("ENV_NAME").unwrap_or_else(|_| "dev".to_string());

        let organization_id = ctx.data::<OrganizationId>()?.0;
        let configuration = Configuration::new();
        let body = MetricPayload::new(vec![MetricSeries::new(
            "j_reit.feedbacks.create".to_string(),
            vec![MetricPoint::new()
                .timestamp(chrono::Utc::now().timestamp())
                .value(1.0)],
        )
        .tags(vec![
            format!("organization:{}", organization_id),
            format!("env:{}", env_name),
        ])
        .type_(MetricIntakeType::UNSPECIFIED)]);
        let api = MetricsAPI::with_config(configuration);
        let _ = api
            .submit_metrics(body, SubmitMetricsOptionalParams::default())
            .await;

        Ok(GraphQLFeedback::from(feedback))
    }

    /// 指定された機能のトライアルを開始する。
    async fn start_free_trial(
        &self,
        ctx: &Context<'_>,
        input: StartFreeTrialInput,
    ) -> Result<Option<GraphQLFreeTrial>> {
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let roles = ctx.data::<Roles>()?;

        let manager = TrialManager::default();
        let result = manager
            .issue_or_get_trial(db, auth0_id, &input.feature, roles)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        match result {
            TrialIssueResult::Issued(trial) => Ok(Some(trial)),
            TrialIssueResult::Exists(trial) => Ok(Some(trial)),
            TrialIssueResult::PremiumUser => Ok(None),
        }
    }

    /// 物件検索条件を新規作成
    async fn create_building_search_params(
        &self,
        ctx: &Context<'_>,
        input: CreateBuildingSearchParamsInput,
    ) -> Result<GraphQLSavedBuildingSearchParams> {
        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let roles = ctx.data::<Roles>()?;

        let txn = db.begin().await?;

        // 有料プランユーザーでない場合、保存済みの検索条件が1件以上ある場合はエラー
        if !roles.data.j_reit_premium {
            let count = saved_building_search_params::Entity::find()
                .filter(
                    Condition::all()
                        .add(saved_building_search_params::Column::Auth0Id.eq(auth0_id))
                        .add(saved_building_search_params::Column::Deleted.eq(0)),
                )
                .count(&txn)
                .await?;

            if count > 0 {
                return Err(Error::new(
                    "Free plan users can only save one search condition",
                ));
            }
        }

        let now = Utc::now();
        let id = generate_uuid_v7_binary();
        let model = saved_building_search_params::ActiveModel {
            id: Set(id),
            auth0_id: Set(auth0_id.into()),
            name: Set(input.name),
            params: Set(input.params),
            created_at: Set(now),
            updated_at: Set(now),
            deleted: Set(0),
        };
        let result = model.insert(&txn).await?;

        txn.commit().await?;

        Ok(result.into())
    }

    /// 物件検索条件を更新
    async fn update_building_search_params(
        &self,
        ctx: &Context<'_>,
        input: UpdateBuildingSearchParamsInput,
    ) -> Result<GraphQLSavedBuildingSearchParams> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let id = UuidVec::try_from(input.id)
            .map_err(|e| Error::new(format!("Invalid ID format - {}", e)))?
            .0;

        let txn = db.begin().await?;

        let model = saved_building_search_params::Entity::find()
            .filter(
                Condition::all()
                    .add(saved_building_search_params::Column::Id.eq(id))
                    .add(saved_building_search_params::Column::Auth0Id.eq(auth0_id))
                    .add(saved_building_search_params::Column::Deleted.eq(0)),
            )
            .one(&txn)
            .await?
            .ok_or_else(|| Error::new("Not found"))?;

        let mut model: saved_building_search_params::ActiveModel = model.into();
        model.name = Set(input.name);
        model.params = Set(input.params);
        model.updated_at = Set(Utc::now());
        let result = model.update(&txn).await?;

        txn.commit().await?;

        Ok(result.into())
    }

    /// 取引検索条件を新規作成
    async fn create_transaction_search_params(
        &self,
        ctx: &Context<'_>,
        input: CreateTransactionSearchParamsInput,
    ) -> Result<GraphQLSavedTransactionSearchParams> {
        let db = ctx.data::<sea_orm::DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let roles = ctx.data::<Roles>()?;

        let txn = db.begin().await?;

        // 有料プランユーザーでない場合、保存済みの検索条件が1件以上ある場合はエラー
        if !roles.data.j_reit_premium {
            let count = saved_transaction_search_params::Entity::find()
                .filter(
                    Condition::all()
                        .add(saved_transaction_search_params::Column::Auth0Id.eq(auth0_id))
                        .add(saved_transaction_search_params::Column::Deleted.eq(0)),
                )
                .count(&txn)
                .await?;

            if count > 0 {
                return Err(Error::new(
                    "Free plan users can only save one search condition",
                ));
            }
        }

        let now = Utc::now();
        let id = generate_uuid_v7_binary();
        let model = saved_transaction_search_params::ActiveModel {
            id: Set(id),
            auth0_id: Set(auth0_id.into()),
            name: Set(input.name),
            params: Set(input.params),
            created_at: Set(now),
            updated_at: Set(now),
            deleted: Set(0),
        };
        let result = model.insert(&txn).await?;

        txn.commit().await?;

        Ok(result.into())
    }

    /// 取引検索条件を更新
    async fn update_transaction_search_params(
        &self,
        ctx: &Context<'_>,
        input: UpdateTransactionSearchParamsInput,
    ) -> Result<GraphQLSavedTransactionSearchParams> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let id = UuidVec::try_from(input.id)
            .map_err(|e| Error::new(format!("Invalid ID format - {}", e)))?
            .0;

        let txn = db.begin().await?;

        let model = saved_transaction_search_params::Entity::find()
            .filter(
                Condition::all()
                    .add(saved_transaction_search_params::Column::Id.eq(id))
                    .add(saved_transaction_search_params::Column::Auth0Id.eq(auth0_id))
                    .add(saved_transaction_search_params::Column::Deleted.eq(0)),
            )
            .one(&txn)
            .await?
            .ok_or_else(|| Error::new("Not found"))?;

        let mut model: saved_transaction_search_params::ActiveModel = model.into();
        model.name = Set(input.name);
        model.params = Set(input.params);
        model.updated_at = Set(Utc::now());
        let result = model.update(&txn).await?;

        txn.commit().await?;

        Ok(result.into())
    }

    /// 物件検索条件を削除
    async fn delete_building_search_params(
        &self,
        ctx: &Context<'_>,
        input: DeleteBuildingSearchParamsInput,
    ) -> Result<GraphQLSavedBuildingSearchParams> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let id = UuidVec::try_from(input.id)
            .map_err(|e| Error::new(format!("Invalid ID format - {}", e)))?
            .0;

        let txn = db.begin().await?;

        let model = saved_building_search_params::Entity::find()
            .filter(
                Condition::all()
                    .add(saved_building_search_params::Column::Id.eq(id))
                    .add(saved_building_search_params::Column::Auth0Id.eq(auth0_id))
                    .add(saved_building_search_params::Column::Deleted.eq(0)),
            )
            .one(&txn)
            .await?
            .ok_or_else(|| Error::new("Not found"))?;

        let mut model: saved_building_search_params::ActiveModel = model.into();
        model.deleted = Set(1);
        model.updated_at = Set(Utc::now());
        let result = model.update(&txn).await?;

        txn.commit().await?;

        Ok(result.into())
    }

    /// 取引検索条件を削除
    async fn delete_transaction_search_params(
        &self,
        ctx: &Context<'_>,
        input: DeleteTransactionSearchParamsInput,
    ) -> Result<GraphQLSavedTransactionSearchParams> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let id = UuidVec::try_from(input.id)
            .map_err(|e| Error::new(format!("Invalid ID format - {}", e)))?
            .0;

        let txn = db.begin().await?;

        let model = saved_transaction_search_params::Entity::find()
            .filter(
                Condition::all()
                    .add(saved_transaction_search_params::Column::Id.eq(id))
                    .add(saved_transaction_search_params::Column::Auth0Id.eq(auth0_id))
                    .add(saved_transaction_search_params::Column::Deleted.eq(0)),
            )
            .one(&txn)
            .await?
            .ok_or_else(|| Error::new("Not found"))?;

        let mut model: saved_transaction_search_params::ActiveModel = model.into();
        model.deleted = Set(1);
        model.updated_at = Set(Utc::now());
        let result = model.update(&txn).await?;

        txn.commit().await?;

        Ok(result.into())
    }
}

#[derive(InputObject)]
struct CreateFeedbackInput {
    feedback_type: String,
    content: String,
    url: String,
    user_agent: String,
    // FIXME: auth0_idは認証情報から取得する
    auth0_id: String,
}

#[derive(InputObject)]
struct StartFreeTrialInput {
    /// トライアルを開始する機能
    feature: GraphQLFreeTrialFeature,
}

#[derive(InputObject)]
pub struct CreateBuildingSearchParamsInput {
    pub(crate) name: String,
    pub(crate) params: Value,
}

#[derive(InputObject)]
pub struct UpdateBuildingSearchParamsInput {
    pub(crate) id: ID,
    pub(crate) name: String,
    pub(crate) params: Value,
}

#[derive(InputObject)]
pub struct CreateTransactionSearchParamsInput {
    pub(crate) name: String,
    pub(crate) params: Value,
}

#[derive(InputObject)]
pub struct UpdateTransactionSearchParamsInput {
    pub(crate) id: ID,
    pub(crate) name: String,
    pub(crate) params: Value,
}

#[derive(InputObject)]
pub struct DeleteBuildingSearchParamsInput {
    pub(crate) id: ID,
}

#[derive(InputObject)]
pub struct DeleteTransactionSearchParamsInput {
    pub(crate) id: ID,
}
