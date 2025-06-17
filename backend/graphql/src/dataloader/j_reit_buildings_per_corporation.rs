use super::GraphQLLoader;
use crate::types::{
    ids::JReitCorporationId,
    j_reit_buildings::{GraphQLJReitBuilding, GraphQLJReitBuildingIdWithCorporationId},
};
use async_graphql::{dataloader::Loader, Result, ID};
use common::types::JReitBuildingIdAndCorporationId;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QuerySelect};
use sql_entities::{j_reit_buildings, j_reit_transactions};
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<GraphQLJReitBuildingIdWithCorporationId> for GraphQLLoader {
    type Value = GraphQLJReitBuilding;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[GraphQLJReitBuildingIdWithCorporationId],
    ) -> Result<HashMap<GraphQLJReitBuildingIdWithCorporationId, Self::Value>> {
        let combined_ids = keys
            .iter()
            .map(|key| JReitBuildingIdAndCorporationId::from(key.clone()).combined_transaction_id())
            .collect::<Vec<_>>();
        let j_reit_buildings = j_reit_buildings::Entity::find()
            .distinct()
            .column(j_reit_transactions::Column::JReitCorporationId)
            .inner_join(j_reit_transactions::Entity)
            .filter(j_reit_transactions::Column::CombinedTransactionId.is_in(combined_ids))
            .into_model::<JReitBuildingWithCorporationId>()
            .all(&self.db)
            .await?;
        Ok(j_reit_buildings
            .into_iter()
            .map(|building| {
                let mut j_reit_building = GraphQLJReitBuilding::from(building.building);
                j_reit_building.j_reit_corporation_id =
                    Some(JReitCorporationId(building.j_reit_corporation_id.clone()));
                (
                    GraphQLJReitBuildingIdWithCorporationId {
                        building_id: j_reit_building.id.clone(),
                        corporation_id: ID::from(JReitCorporationId(
                            building.j_reit_corporation_id.clone(),
                        )),
                    },
                    j_reit_building,
                )
            })
            .collect())
    }
}

#[derive(FromQueryResult)]
struct JReitBuildingWithCorporationId {
    #[sea_orm(nested)]
    building: j_reit_buildings::Model,
    j_reit_corporation_id: String,
}
