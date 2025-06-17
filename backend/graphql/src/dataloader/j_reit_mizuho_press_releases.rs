use crate::dataloader::GraphQLLoader;
use crate::types::ids::JReitPressReleasesByJReitMizuhoBuildingId;
use crate::types::j_reit_buildings::j_reit_press_releases::GraphQLJReitPressRelease;
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use sql_entities::j_reit_mizuho_press_releases;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitPressReleasesByJReitMizuhoBuildingId> for GraphQLLoader {
    type Value = Vec<GraphQLJReitPressRelease>;
    type Error = async_graphql::Error;
    async fn load(
        &self,
        keys: &[JReitPressReleasesByJReitMizuhoBuildingId],
    ) -> Result<HashMap<JReitPressReleasesByJReitMizuhoBuildingId, Self::Value>> {
        let j_reit_mizuho_building_ids = keys.iter().map(|key| key.0 .0.clone());
        let j_reit_mizuho_press_releases = j_reit_mizuho_press_releases::Entity::find()
            .filter(
                j_reit_mizuho_press_releases::Column::JReitMizuhoBuildingId
                    .is_in(j_reit_mizuho_building_ids),
            )
            .order_by_asc(j_reit_mizuho_press_releases::Column::ReleaseDate)
            .all(&self.db)
            .await?;

        let mut j_reit_mizuho_press_releases_by_j_reit_mizuho_building_id: HashMap<
            JReitPressReleasesByJReitMizuhoBuildingId,
            Vec<GraphQLJReitPressRelease>,
        > = HashMap::new();
        for j_reit_mizuho_press_release in j_reit_mizuho_press_releases {
            let j_reit_mizuho_press_release =
                GraphQLJReitPressRelease::from(j_reit_mizuho_press_release);
            let j_reit_mizuho_building_id = j_reit_mizuho_press_release
                .j_reit_mizuho_building_id
                .clone();
            j_reit_mizuho_press_releases_by_j_reit_mizuho_building_id
                .entry(JReitPressReleasesByJReitMizuhoBuildingId(
                    j_reit_mizuho_building_id,
                ))
                .or_default()
                .push(j_reit_mizuho_press_release);
        }

        Ok(j_reit_mizuho_press_releases_by_j_reit_mizuho_building_id)
    }
}
