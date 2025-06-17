use crate::types::ids::JReitCapRateHistoriesByJReitMizuhoBuildingId;
use crate::{
    dataloader::GraphQLLoader,
    types::j_reit_buildings::j_reit_cap_rate_histories::GraphQLJReitCapRateHistory,
};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use sql_entities::j_reit_mizuho_cap_rate_histories;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitCapRateHistoriesByJReitMizuhoBuildingId> for GraphQLLoader {
    type Value = Vec<GraphQLJReitCapRateHistory>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitCapRateHistoriesByJReitMizuhoBuildingId],
    ) -> Result<HashMap<JReitCapRateHistoriesByJReitMizuhoBuildingId, Self::Value>> {
        let j_reit_mizuho_building_ids = keys.iter().map(|key| key.0 .0.clone());
        let j_reit_cap_rate_histories = j_reit_mizuho_cap_rate_histories::Entity::find()
            .filter(
                j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId
                    .is_in(j_reit_mizuho_building_ids),
            )
            .order_by_asc(j_reit_mizuho_cap_rate_histories::Column::ClosingDate)
            .all(&self.db)
            .await?;

        let mut j_reit_cap_rate_histories_by_j_reit_building_id: HashMap<
            JReitCapRateHistoriesByJReitMizuhoBuildingId,
            Vec<GraphQLJReitCapRateHistory>,
        > = HashMap::new();
        for j_reit_cap_rate_history in j_reit_cap_rate_histories {
            let j_reit_cap_rate_history = GraphQLJReitCapRateHistory::from(j_reit_cap_rate_history);
            let j_reit_mizuho_building_id =
                j_reit_cap_rate_history.j_reit_mizuho_building_id.clone();
            j_reit_cap_rate_histories_by_j_reit_building_id
                .entry(JReitCapRateHistoriesByJReitMizuhoBuildingId(
                    j_reit_mizuho_building_id,
                ))
                .or_default()
                .push(j_reit_cap_rate_history);
        }

        Ok(j_reit_cap_rate_histories_by_j_reit_building_id)
    }
}
