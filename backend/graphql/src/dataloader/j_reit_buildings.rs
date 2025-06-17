use super::GraphQLLoader;
use crate::types::{ids::JReitBuildingId, j_reit_buildings::GraphQLJReitBuilding};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sql_entities::j_reit_buildings;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitBuildingId> for GraphQLLoader {
    type Value = GraphQLJReitBuilding;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitBuildingId],
    ) -> Result<HashMap<JReitBuildingId, Self::Value>> {
        let j_reit_ids = keys.iter().map(|key| key.0.clone());
        let j_reit_buildings = j_reit_buildings::Entity::find()
            .filter(j_reit_buildings::Column::Id.is_in(j_reit_ids))
            .all(&self.db)
            .await?;
        Ok(j_reit_buildings
            .into_iter()
            .map(|building| {
                let j_reit_building = GraphQLJReitBuilding::from(building);
                let j_reit_building_id = JReitBuildingId::from(j_reit_building.id.clone());
                (j_reit_building_id, j_reit_building)
            })
            .collect())
    }
}
