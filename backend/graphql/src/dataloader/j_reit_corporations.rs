use super::GraphQLLoader;
use crate::types::{ids::JReitCorporationId, j_reit_corporations::GraphQLJReitCorporation};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sql_entities::j_reit_corporations;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitCorporationId> for GraphQLLoader {
    type Value = GraphQLJReitCorporation;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitCorporationId],
    ) -> Result<HashMap<JReitCorporationId, Self::Value>> {
        let j_reit_corporation_ids = keys.iter().map(|key| key.0.clone());
        let j_reit_corporations = j_reit_corporations::Entity::find()
            .filter(j_reit_corporations::Column::Id.is_in(j_reit_corporation_ids))
            .all(&self.db)
            .await?;
        Ok(j_reit_corporations
            .into_iter()
            .map(|corporation| {
                let id = JReitCorporationId(corporation.id.clone());
                let j_reit_corporation = GraphQLJReitCorporation::from(corporation);
                (id, j_reit_corporation)
            })
            .collect())
    }
}
