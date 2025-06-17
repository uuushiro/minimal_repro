use crate::dataloader::GraphQLLoader;
use crate::types::ids::JReitAppraisalHistoriesByJReitMizuhoBuildingId;
use crate::types::j_reit_buildings::j_reit_appraisal_histories::GraphQLJReitAppraisalHistory;
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use sql_entities::j_reit_mizuho_appraisal_histories;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitAppraisalHistoriesByJReitMizuhoBuildingId> for GraphQLLoader {
    type Value = Vec<GraphQLJReitAppraisalHistory>;
    type Error = async_graphql::Error;
    async fn load(
        &self,
        keys: &[JReitAppraisalHistoriesByJReitMizuhoBuildingId],
    ) -> Result<HashMap<JReitAppraisalHistoriesByJReitMizuhoBuildingId, Self::Value>> {
        let j_reit_mizuho_building_ids = keys.iter().map(|key| key.0 .0.clone());
        let j_reit_appraisal_histories = j_reit_mizuho_appraisal_histories::Entity::find()
            .filter(
                j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId
                    .is_in(j_reit_mizuho_building_ids),
            )
            .order_by_asc(j_reit_mizuho_appraisal_histories::Column::AppraisalDate)
            .all(&self.db)
            .await?;

        let mut j_reit_appraisal_histories_by_j_reit_mizuho_building_id: HashMap<
            JReitAppraisalHistoriesByJReitMizuhoBuildingId,
            Vec<GraphQLJReitAppraisalHistory>,
        > = HashMap::new();
        for j_reit_appraisal_history in j_reit_appraisal_histories {
            let j_reit_appraisal_history =
                GraphQLJReitAppraisalHistory::from(j_reit_appraisal_history);
            let j_reit_mizuho_building_id =
                j_reit_appraisal_history.j_reit_mizuho_building_id.clone();
            j_reit_appraisal_histories_by_j_reit_mizuho_building_id
                .entry(JReitAppraisalHistoriesByJReitMizuhoBuildingId(
                    j_reit_mizuho_building_id,
                ))
                .or_default()
                .push(j_reit_appraisal_history);
        }

        Ok(j_reit_appraisal_histories_by_j_reit_mizuho_building_id)
    }
}
