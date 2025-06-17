use crate::dataloader::GraphQLLoader;
use crate::types::ids::{JReitMizuhoBuildingId, JReitMizuhoBuildingIdByBuildingIdAndCorporationId};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use sql_entities::j_reit_mizuho_id_mappings;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitMizuhoBuildingIdByBuildingIdAndCorporationId> for GraphQLLoader {
    type Value = Option<JReitMizuhoBuildingId>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitMizuhoBuildingIdByBuildingIdAndCorporationId],
    ) -> Result<HashMap<JReitMizuhoBuildingIdByBuildingIdAndCorporationId, Self::Value>> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conditions = Condition::any();

        for key in keys {
            let building_id = key.j_reit_building_id.0.clone();
            let corporation_id = key.j_reit_corporation_id.0.clone();

            let condition = Condition::all()
                .add(j_reit_mizuho_id_mappings::Column::JReitBuildingId.eq(building_id))
                .add(j_reit_mizuho_id_mappings::Column::JReitCorporationId.eq(corporation_id));

            conditions = conditions.add(condition);
        }

        let id_mappings = j_reit_mizuho_id_mappings::Entity::find()
            .filter(conditions)
            .all(&self.db)
            .await?;

        let mut id_mapping_map = HashMap::new();
        for mapping in &id_mappings {
            id_mapping_map.insert(
                (
                    mapping.j_reit_building_id.clone(),
                    mapping.j_reit_corporation_id.clone(),
                ),
                mapping.j_reit_mizuho_building_id.clone(),
            );
        }

        let mut result = HashMap::new();
        for key in keys {
            let mizuho_id = id_mapping_map.get(&(
                key.j_reit_building_id.0.clone(),
                key.j_reit_corporation_id.0.clone(),
            ));

            let value = mizuho_id.map(|id| JReitMizuhoBuildingId(id.clone()));
            result.insert(key.clone(), value);
        }

        Ok(result)
    }
}
