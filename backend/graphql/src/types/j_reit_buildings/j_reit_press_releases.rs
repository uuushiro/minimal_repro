use crate::types::ids::JReitMizuhoBuildingId;
use async_graphql::{SimpleObject, ID};
use chrono::NaiveDate;
use sql_entities::j_reit_mizuho_press_releases;

#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitPressRelease {
    id: ID,
    #[graphql(skip)]
    pub(crate) j_reit_mizuho_building_id: JReitMizuhoBuildingId,
    /// プレスリリースタイトル
    title: String,
    /// プレスリリースURL
    url: String,
    /// プレスリリース公開日
    release_date: NaiveDate,
}

impl From<j_reit_mizuho_press_releases::Model> for GraphQLJReitPressRelease {
    fn from(model: j_reit_mizuho_press_releases::Model) -> Self {
        Self {
            id: ID(model.id),
            j_reit_mizuho_building_id: JReitMizuhoBuildingId(model.j_reit_mizuho_building_id),
            title: model.title,
            url: model.url,
            release_date: model.release_date,
        }
    }
}
