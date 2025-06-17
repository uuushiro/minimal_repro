use async_graphql::{SimpleObject, ID};
use sea_orm::prelude::DateTimeUtc;
use sql_entities::feedbacks;

#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLFeedback {
    pub(crate) id: ID,
    /// お問い合わせ種別
    feedback_type: String,
    /// お問い合わせ内容
    content: String,
    /// 問い合わせアクセス元
    url: String,
    /// ユーザーエージェント
    user_agent: String,
    /// 作成日時
    created_at: DateTimeUtc,
    /// 更新日時
    updated_at: DateTimeUtc,
}

impl From<feedbacks::Model> for GraphQLFeedback {
    fn from(value: feedbacks::Model) -> Self {
        let feedbacks::Model {
            id,
            feedback_type,
            content,
            url,
            user_agent,
            auth0_id: _,
            created_at,
            updated_at,
        } = value;
        Self {
            id: ID::from(id),
            feedback_type,
            content,
            url,
            user_agent,
            created_at,
            updated_at,
        }
    }
}
