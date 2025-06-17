use super::GraphQLLoader;
use crate::types::{
    ids::JReitFinancialsByJReitMizuhoBuildingId, j_reit_financials::GraphQLJReitFinancial,
};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use sql_entities::j_reit_mizuho_financials;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitFinancialsByJReitMizuhoBuildingId> for GraphQLLoader {
    type Value = Vec<GraphQLJReitFinancial>;
    type Error = async_graphql::Error;
    async fn load(
        &self,
        keys: &[JReitFinancialsByJReitMizuhoBuildingId],
    ) -> Result<HashMap<JReitFinancialsByJReitMizuhoBuildingId, Self::Value>> {
        let j_reit_mizuho_building_ids = keys.iter().map(|key| key.0 .0.clone());
        let j_reit_financials = j_reit_mizuho_financials::Entity::find()
            .filter(
                j_reit_mizuho_financials::Column::JReitMizuhoBuildingId
                    .is_in(j_reit_mizuho_building_ids),
            )
            .order_by_asc(j_reit_mizuho_financials::Column::FiscalPeriodStartDate)
            .all(&self.db)
            .await?;

        let mut j_reit_financials_by_j_reit_mizuho_building_id: HashMap<
            JReitFinancialsByJReitMizuhoBuildingId,
            Vec<GraphQLJReitFinancial>,
        > = HashMap::new();
        for j_reit_financial in j_reit_financials {
            let j_reit_financial = GraphQLJReitFinancial::from(j_reit_financial);
            let j_reit_mizuho_building_id = j_reit_financial.j_reit_mizuho_building_id.clone();
            j_reit_financials_by_j_reit_mizuho_building_id
                .entry(JReitFinancialsByJReitMizuhoBuildingId(
                    j_reit_mizuho_building_id,
                ))
                .or_default()
                .push(j_reit_financial);
        }

        Ok(j_reit_financials_by_j_reit_mizuho_building_id)
    }
}
