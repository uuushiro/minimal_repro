use async_graphql::{SimpleObject, ID};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sql_entities::saved_building_search_params;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct GraphQLSavedBuildingSearchParams {
    pub(crate) id: ID,
    pub(crate) name: String,
    pub(crate) params: Value,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl From<saved_building_search_params::Model> for GraphQLSavedBuildingSearchParams {
    fn from(model: saved_building_search_params::Model) -> Self {
        let uuid = Uuid::from_slice(&model.id).expect("Failed to convert ID to UUID");
        Self {
            id: ID::from(uuid),
            name: model.name,
            params: model.params,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
