use crate::types::j_reit_buildings::j_reit_transactions::{
    GraphQLJReitTransaction, JReitTransactionsByJReitBuildingIdWithCorporationId,
};
use crate::{dataloader::GraphQLLoader, types::ids::JReitTransactionsByJReitBuildingId};
use async_graphql::{dataloader::Loader, Result, ID};
use common::types::JReitBuildingIdAndCorporationId;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use sql_entities::j_reit_transactions;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitTransactionsByJReitBuildingId> for GraphQLLoader {
    type Value = Vec<GraphQLJReitTransaction>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitTransactionsByJReitBuildingId],
    ) -> Result<HashMap<JReitTransactionsByJReitBuildingId, Self::Value>> {
        let j_reit_building_ids = keys.iter().map(|key| key.0 .0.clone());
        let j_reit_transactions = j_reit_transactions::Entity::find()
            .filter(j_reit_transactions::Column::JReitBuildingId.is_in(j_reit_building_ids))
            .order_by_asc(j_reit_transactions::Column::TransactionDate)
            .all(&self.db)
            .await?;

        let mut j_reit_transactions_by_j_reit_building_id: HashMap<
            JReitTransactionsByJReitBuildingId,
            Vec<GraphQLJReitTransaction>,
        > = HashMap::new();
        for j_reit_transaction in j_reit_transactions {
            let j_reit_transaction = GraphQLJReitTransaction::from(j_reit_transaction);
            let j_reit_building_id = j_reit_transaction.j_reit_building_id.clone();
            j_reit_transactions_by_j_reit_building_id
                .entry(JReitTransactionsByJReitBuildingId(j_reit_building_id))
                .or_default()
                .push(j_reit_transaction);
        }

        Ok(j_reit_transactions_by_j_reit_building_id)
    }
}

#[async_trait::async_trait]
impl Loader<JReitTransactionsByJReitBuildingIdWithCorporationId> for GraphQLLoader {
    type Value = Vec<GraphQLJReitTransaction>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitTransactionsByJReitBuildingIdWithCorporationId],
    ) -> Result<HashMap<JReitTransactionsByJReitBuildingIdWithCorporationId, Self::Value>> {
        let combined_transaction_ids = keys
            .iter()
            .map(|key| {
                JReitBuildingIdAndCorporationId {
                    building_id: key.j_reit_building_id.0.clone(),
                    corporation_id: key.j_reit_corporation_id.0.clone(),
                }
                .combined_transaction_id()
            })
            .collect::<Vec<_>>();

        let j_reit_transactions = j_reit_transactions::Entity::find()
            .filter(
                j_reit_transactions::Column::CombinedTransactionId.is_in(combined_transaction_ids),
            )
            .order_by_asc(j_reit_transactions::Column::TransactionDate)
            // 同日の中では初回取得を最初に取得する
            .order_by_asc(j_reit_transactions::Column::TransactionCategory)
            .all(&self.db)
            .await?;

        let mut j_reit_transactions_by_id_pair: HashMap<
            JReitTransactionsByJReitBuildingIdWithCorporationId,
            Vec<GraphQLJReitTransaction>,
        > = HashMap::new();
        for j_reit_transaction in j_reit_transactions {
            let j_reit_transaction = GraphQLJReitTransaction::from(j_reit_transaction);
            let key = JReitTransactionsByJReitBuildingIdWithCorporationId {
                j_reit_building_id: ID::from(j_reit_transaction.j_reit_building_id.clone()),
                j_reit_corporation_id: ID::from(j_reit_transaction.j_reit_corporation_id.clone()),
            };
            j_reit_transactions_by_id_pair
                .entry(key)
                .or_default()
                .push(j_reit_transaction);
        }

        Ok(j_reit_transactions_by_id_pair)
    }
}
