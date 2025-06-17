use async_graphql::{Enum, SimpleObject};
use chrono::Utc;
use proto::Roles;
use sea_orm::prelude::DateTimeUtc;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sql_entities::free_trials;
use std::collections::HashMap;
use std::fmt;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum GraphQLFreeTrialStatus {
    Ready,
    Active,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TrialIssueResult {
    /// 新規にトライアルを発行した
    Issued(GraphQLFreeTrial),
    /// 既存のトライアルが存在した
    Exists(GraphQLFreeTrial),
    /// プレミアムユーザーのため発行不要
    PremiumUser,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub(crate) enum GraphQLFreeTrialFeature {
    /// 分析機能
    Analysis,
    /// マップピンカスタム機能
    MapPinCustom,
}

impl GraphQLFreeTrialFeature {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Analysis => "analysis",
            Self::MapPinCustom => "map_pin_custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "analysis" => Some(Self::Analysis),
            "map_pin_custom" => Some(Self::MapPinCustom),
            _ => None,
        }
    }
}

#[derive(SimpleObject, Clone, Debug, PartialEq, Eq)]
pub struct GraphQLFreeTrial {
    /// トライアルステータス
    status: GraphQLFreeTrialStatus,
    /// トライアル対象機能
    feature: GraphQLFreeTrialFeature,
    /// 無料トライアル開始日時
    start_time: Option<DateTimeUtc>,
    /// 無料トライアル終了日時
    end_time: Option<DateTimeUtc>,
}

impl GraphQLFreeTrial {
    pub fn ready(feature: GraphQLFreeTrialFeature) -> Self {
        Self {
            status: GraphQLFreeTrialStatus::Ready,
            feature,
            start_time: None,
            end_time: None,
        }
    }
}

#[derive(Debug)]
pub enum FreeTrialConversionError {
    InvalidFeature(String),
}

impl fmt::Display for FreeTrialConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFeature(feature) => write!(f, "無効な機能名です: {}", feature),
        }
    }
}

impl TryFrom<free_trials::Model> for GraphQLFreeTrial {
    type Error = FreeTrialConversionError;
    fn try_from(value: free_trials::Model) -> Result<Self, Self::Error> {
        let free_trials::Model {
            id: _,
            auth0_id: _,
            feature,
            start_time,
            end_time,
            created_at: _,
            updated_at: _,
        } = value;
        let feature = GraphQLFreeTrialFeature::from_str(&feature)
            .ok_or_else(|| FreeTrialConversionError::InvalidFeature(feature.clone()))?;
        Ok(Self {
            status: get_free_trial_status(end_time),
            feature,
            start_time: Some(start_time),
            end_time: Some(end_time),
        })
    }
}

fn get_free_trial_status(end_time: DateTimeUtc) -> GraphQLFreeTrialStatus {
    let now = chrono::Utc::now();
    if now <= end_time {
        GraphQLFreeTrialStatus::Active
    } else {
        GraphQLFreeTrialStatus::Expired
    }
}

/// トライアル対象の機能を定義
pub(crate) mod features {
    use super::GraphQLFreeTrialFeature;

    /// 分析機能
    pub const ANALYSIS: GraphQLFreeTrialFeature = GraphQLFreeTrialFeature::Analysis;
    /// マップピンカスタム機能
    pub const MAP_PIN_CUSTOM: GraphQLFreeTrialFeature = GraphQLFreeTrialFeature::MapPinCustom;
}

/// トライアル期間を定義（日数）
pub(crate) mod durations {
    /// 分析機能のトライアル期間
    pub const ANALYSIS_DAYS: i64 = 10;
    /// マップピンカスタム機能のトライアル期間
    pub const MAP_PIN_CUSTOM_DAYS: i64 = 30;
}

/// トライアルの発行判断と発行を管理する構造体
pub struct TrialManager {
    pub feature_duration: HashMap<GraphQLFreeTrialFeature, i64>,
}

impl TrialManager {
    /// デフォルト設定
    pub fn default() -> Self {
        let mut feature_duration = HashMap::new();
        feature_duration.insert(features::ANALYSIS, durations::ANALYSIS_DAYS);
        feature_duration.insert(features::MAP_PIN_CUSTOM, durations::MAP_PIN_CUSTOM_DAYS);
        Self { feature_duration }
    }

    /// featureごとの日数を取得（なければNone）
    pub fn get_duration(&self, feature: &GraphQLFreeTrialFeature) -> Option<i64> {
        self.feature_duration.get(feature).copied()
    }

    /// 指定featureのトライアルを発行 or 既存を返す
    pub async fn issue_or_get_trial(
        &self,
        db: &DatabaseConnection,
        auth0_id: &str,
        feature: &GraphQLFreeTrialFeature,
        roles: &Roles,
    ) -> Result<TrialIssueResult, sea_orm::DbErr> {
        if roles.data.j_reit_premium {
            return Ok(TrialIssueResult::PremiumUser);
        }

        let now = Utc::now();
        let trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq(auth0_id)),
            )
            .one(db)
            .await?;

        if let Some(trial_model) = trial {
            let graphql_trial = GraphQLFreeTrial::try_from(trial_model)
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
            return Ok(TrialIssueResult::Exists(graphql_trial));
        }

        if let Some(duration_days) = self.get_duration(feature) {
            let start_time = now;
            let end_time = now + chrono::Duration::days(duration_days);
            let trial_model = free_trials::ActiveModel {
                feature: Set(feature.as_str().to_string()),
                auth0_id: Set(auth0_id.to_string()),
                start_time: Set(start_time),
                end_time: Set(end_time),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db)
            .await?;

            let graphql_trial = GraphQLFreeTrial::try_from(trial_model)
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
            return Ok(TrialIssueResult::Issued(graphql_trial));
        }

        Err(sea_orm::DbErr::Custom(format!(
            "機能 '{}' は設定されていません",
            feature.as_str()
        )))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::durations::*;
    use super::features::*;
    use super::*;
    use chrono::{Duration, Utc};
    use proto::DataRoles;
    use sea_orm::Database;
    use sea_orm::{ConnectionTrait, DatabaseBackend, DbConn, Schema};

    fn create_test_roles(is_premium: bool) -> Roles {
        let mut data_roles = DataRoles::default();
        data_roles.j_reit_premium = is_premium;
        let mut roles = Roles::default();
        roles.data = proto::MessageField::from_option(Some(data_roles));
        roles
    }

    async fn setup_memory_db() -> DbConn {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_default_config_duration() {
        let manager = TrialManager::default();
        assert_eq!(
            manager.get_duration(&MAP_PIN_CUSTOM),
            Some(MAP_PIN_CUSTOM_DAYS)
        );
        assert_eq!(manager.get_duration(&ANALYSIS), Some(ANALYSIS_DAYS));
    }

    #[tokio::test]
    async fn test_get_free_trial_creates_new_for_configured_feature() {
        let db = setup_memory_db().await;
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(sql_entities::free_trials::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();

        let manager = TrialManager::default();
        let auth0_id = "user1";
        let feature = &MAP_PIN_CUSTOM;
        let roles = create_test_roles(false);
        let result = manager
            .issue_or_get_trial(&db, auth0_id, feature, &roles)
            .await
            .unwrap();
        // トライアルが実際に作成されたことを確認
        let trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq(auth0_id)),
            )
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let graphql_trial = GraphQLFreeTrial::try_from(trial).unwrap();
        assert_eq!(result, TrialIssueResult::Issued(graphql_trial));
    }

    #[tokio::test]
    async fn test_get_free_trial_premium_user_does_not_create() {
        let db = setup_memory_db().await;
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(sql_entities::free_trials::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();

        let manager = TrialManager::default();
        let auth0_id = "user3";
        let feature = &MAP_PIN_CUSTOM;
        let roles = create_test_roles(true);
        let result = manager
            .issue_or_get_trial(&db, auth0_id, feature, &roles)
            .await
            .unwrap();
        assert_eq!(result, TrialIssueResult::PremiumUser);

        // トライアルが作成されていないことを確認
        let trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq(auth0_id)),
            )
            .one(&db)
            .await
            .unwrap();
        assert!(trial.is_none());
    }

    #[tokio::test]
    async fn test_existing_trial_no_new_creation_even_if_expired() {
        let db = setup_memory_db().await;
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(sql_entities::free_trials::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();

        let manager = TrialManager::default();
        let auth0_id = "user4";
        let feature = &MAP_PIN_CUSTOM;
        let now = Utc::now();
        // 期限切れのtrialを先に作成
        free_trials::ActiveModel {
            feature: Set(feature.as_str().to_string()),
            auth0_id: Set(auth0_id.to_string()),
            start_time: Set(now - Duration::days(40)),
            end_time: Set(now - Duration::days(10)),
            created_at: Set(now - Duration::days(40)),
            updated_at: Set(now - Duration::days(10)),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let roles = create_test_roles(false);
        // 新規作成されないことを確認
        let result = manager
            .issue_or_get_trial(&db, auth0_id, feature, &roles)
            .await
            .unwrap();
        // 既存のトライアルが変更されていないことを確認
        let trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq(auth0_id)),
            )
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let graphql_trial = GraphQLFreeTrial::try_from(trial).unwrap();
        assert_eq!(result, TrialIssueResult::Exists(graphql_trial));
    }

    #[tokio::test]
    async fn test_invalid_feature_in_database() {
        let db = setup_memory_db().await;
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(sql_entities::free_trials::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();

        // 無効な機能名でトライアルを作成
        let invalid_trial = free_trials::ActiveModel {
            feature: Set("invalid_feature".to_string()),
            auth0_id: Set("user5".to_string()),
            start_time: Set(Utc::now()),
            end_time: Set(Utc::now() + Duration::days(10)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // 無効な機能名のトライアルを変換しようとするとエラーになることを確認
        let result = GraphQLFreeTrial::try_from(invalid_trial);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "無効な機能名です: invalid_feature");
    }

    #[tokio::test]
    async fn test_free_trial_query() {
        let db = setup_memory_db().await;
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(sql_entities::free_trials::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .unwrap();

        // テスト用のトライアルを作成
        let auth0_id = "test_user";
        let feature = &MAP_PIN_CUSTOM;
        let now = Utc::now();
        free_trials::ActiveModel {
            feature: Set(feature.as_str().to_string()),
            auth0_id: Set(auth0_id.to_string()),
            start_time: Set(now),
            end_time: Set(now + Duration::days(30)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // トライアルが存在する場合のテスト
        let found_trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq(auth0_id)),
            )
            .one(&db)
            .await
            .unwrap();

        let graphql_trial = found_trial.map(|t| GraphQLFreeTrial::try_from(t).unwrap());
        assert!(graphql_trial.is_some());
        let trial = graphql_trial.unwrap();
        assert_eq!(trial.feature, *feature);
        assert_eq!(trial.status, GraphQLFreeTrialStatus::Active);

        // トライアルが存在しない場合のテスト
        let not_found_trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq("non_existent_user")),
            )
            .one(&db)
            .await
            .unwrap();

        let graphql_trial = not_found_trial.map(|t| GraphQLFreeTrial::try_from(t).unwrap());
        assert!(graphql_trial.is_none());
    }
}
