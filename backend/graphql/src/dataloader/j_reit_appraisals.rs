use super::GraphQLLoader;
use crate::types::{
    ids::JReitAppraisalId, j_reit_buildings::j_reit_appraisals::GraphQLJReitAppraisal,
};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sql_entities::j_reit_appraisals;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitAppraisalId> for GraphQLLoader {
    type Value = GraphQLJReitAppraisal;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitAppraisalId],
    ) -> Result<HashMap<JReitAppraisalId, Self::Value>> {
        let j_reit_appraisal_ids = keys.iter().map(|key| key.0.clone());
        let j_reit_appraisals = j_reit_appraisals::Entity::find()
            .filter(j_reit_appraisals::Column::Id.is_in(j_reit_appraisal_ids))
            .all(&self.db)
            .await?;
        Ok(j_reit_appraisals
            .into_iter()
            .map(|appraisal| {
                let j_reit_appraisal_id = JReitAppraisalId(appraisal.id.clone());
                let j_reit_appraisal = GraphQLJReitAppraisal::from(appraisal);
                (j_reit_appraisal_id, j_reit_appraisal)
            })
            .collect())
    }
}
