use async_graphql::{Context, Error, Object, Result, ID};
use proto::Roles;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_query::Condition;
use sql_entities::saved_transaction_search_params;
use sql_entities::{
    free_trials, j_reit_mizuho_id_mappings, j_reit_transactions, saved_building_search_params,
};

use crate::custom_sql_query::get_j_reit_buildings::get_j_reit_buildings_per_corporation;
use crate::custom_sql_query::get_j_reit_buildings::get_paginated_j_reit_buildings;
use crate::custom_sql_query::get_j_reit_corporations::get_j_reit_corporations;
use crate::custom_sql_query::search_j_reit_buildings::search_j_reit_buildings;
use crate::custom_sql_query::search_transactions::search_transactions;
use crate::dataloader::DataLoader;
use crate::metadata::Auth0Id;
use crate::types::free_trials::{GraphQLFreeTrial, GraphQLFreeTrialFeature};
use crate::types::ids::{JReitBuildingByOfficeBuildingId, JReitBuildingId, OfficeBuildingId};
use crate::types::j_reit_buildings::j_reit_transactions::{
    search::{GraphQLSearchTransactionInput, GraphQLSearchTransactionResult},
    sort_and_pagination::GraphQLJReitTransactionSortAndPaginateCondition,
    GraphQLJReitTransaction,
};
use crate::types::j_reit_buildings::search::{
    GraphQLSearchJReitBuildingCondition, GraphQLSearchJReitBuildingResult,
    SearchJReitBuildingCondition,
};
use crate::types::j_reit_buildings::sort_and_pagination::GraphQLJReitBuildingSortAndPaginateCondition;
use crate::types::j_reit_buildings::GraphQLJReitBuilding;
use crate::types::j_reit_buildings::GraphQLJReitBuildingIdWithCorporationId;
use crate::types::j_reit_corporations::GraphQLJReitCorporation;
use crate::types::j_reit_id_mapping::GraphQLJReitIdMapping;
use crate::types::saved_building_search_params::GraphQLSavedBuildingSearchParams;
use crate::types::saved_transaction_search_params::GraphQLSavedTransactionSearchParams;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// 機能のトライアル期間を確認する
    async fn free_trial(
        &self,
        ctx: &Context<'_>,
        feature: GraphQLFreeTrialFeature,
    ) -> Result<GraphQLFreeTrial> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;

        let trial = free_trials::Entity::find()
            .filter(
                Condition::all()
                    .add(free_trials::Column::Feature.eq(feature.as_str()))
                    .add(free_trials::Column::Auth0Id.eq(auth0_id)),
            )
            .one(db)
            .await?;

        match trial {
            Some(trial) => Ok(GraphQLFreeTrial::try_from(trial)
                .map_err(|e| async_graphql::Error::new(e.to_string()))?),
            None => Ok(GraphQLFreeTrial::ready(feature)),
        }
    }

    /// Jリート投資法人の一覧
    async fn j_reit_corporations(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] include_delisted: bool,
    ) -> Result<Vec<GraphQLJReitCorporation>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let j_reit_corporations = get_j_reit_corporations(db, include_delisted).await?;
        Ok(j_reit_corporations)
    }

    /// Jリート物件のデータ
    /// IDを指定するとそのIDの物件のみ、指定しない場合は全ての物件を取得する
    async fn j_reit_buildings(
        &self,
        ctx: &Context<'_>,
        ids: Option<Vec<ID>>,
        sort_and_pagination: Option<GraphQLJReitBuildingSortAndPaginateCondition>,
    ) -> Result<Vec<GraphQLJReitBuilding>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let j_reit_buildings = get_paginated_j_reit_buildings(
            db,
            ids.map(|ids| ids.into_iter().map(JReitBuildingId::from).collect()),
            sort_and_pagination.and_then(|s| s.sort).into(),
            sort_and_pagination.and_then(|s| s.pagination),
        )
        .await?;
        Ok(j_reit_buildings)
    }

    async fn j_reit_buildings_per_corporation(
        &self,
        ctx: &Context<'_>,
        ids: Vec<GraphQLJReitBuildingIdWithCorporationId>,
        sort_and_pagination: Option<GraphQLJReitBuildingSortAndPaginateCondition>,
    ) -> Result<Vec<GraphQLJReitBuilding>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let j_reit_buildings = get_j_reit_buildings_per_corporation(
            db,
            ids,
            sort_and_pagination.and_then(|s| s.sort).into(),
            sort_and_pagination.and_then(|s| s.pagination),
        )
        .await?;

        Ok(j_reit_buildings)
    }

    /// Jリート物件のデータをオフィスビルIDで取得する（データがないビルも当然ある）
    /// MRでの仕様に合わせて上場廃止物件は除外している
    async fn j_reit_building_by_office_building_ids(
        &self,
        ctx: &Context<'_>,
        office_building_ids: Vec<ID>,
    ) -> Result<Vec<GraphQLJReitBuilding>> {
        let dataloader = ctx.data::<DataLoader>()?;

        let office_building_ids = office_building_ids
            .into_iter()
            .map(OfficeBuildingId::try_from)
            .map(|r| r.map(JReitBuildingByOfficeBuildingId))
            .collect::<Result<Vec<_>, async_graphql::Error>>()?;
        let j_reit_buildings = dataloader.load_many(office_building_ids).await?;

        Ok(j_reit_buildings.into_values().flatten().collect())
    }

    /// Jリート物件データの検索
    async fn search_j_reit_buildings_per_corporation(
        &self,
        ctx: &Context<'_>,
        search_condition: GraphQLSearchJReitBuildingCondition,
        sort_and_pagination: Option<GraphQLJReitBuildingSortAndPaginateCondition>,
    ) -> Result<GraphQLSearchJReitBuildingResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let dataloader = ctx.data::<DataLoader>()?;
        let roles = ctx.data::<Roles>()?;

        // 検索条件に合致するデータの件数とIDを取得
        let search_condition = SearchJReitBuildingCondition::from(search_condition);
        let search_result = search_j_reit_buildings(
            db,
            roles,
            search_condition.clone(),
            sort_and_pagination.and_then(|s| s.sort).into(),
            sort_and_pagination.and_then(|s| s.pagination),
        )
        .await?;

        // データを取得し、元の順序で並べる
        let j_reit_building_by_id = dataloader
            .load_many(search_result.j_reit_building_id_and_corporation_ids.clone())
            .await?;

        let j_reit_buildings = search_result
            .j_reit_building_id_and_corporation_ids
            .into_iter()
            .filter_map(|id| j_reit_building_by_id.get(&id).cloned())
            .collect();

        Ok(GraphQLSearchJReitBuildingResult::new(
            search_result.count,
            j_reit_buildings,
        ))
    }

    /// 取引データの検索
    async fn search_transactions(
        &self,
        ctx: &Context<'_>,
        input: GraphQLSearchTransactionInput,
        sort_and_pagination: Option<GraphQLJReitTransactionSortAndPaginateCondition>,
    ) -> Result<GraphQLSearchTransactionResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let search_result = search_transactions(
            db,
            input.into(),
            sort_and_pagination.and_then(|s| s.sort).into(),
            sort_and_pagination.and_then(|s| s.pagination),
        )
        .await?;

        Ok(search_result)
    }

    /// ID指定での取引データの取得
    async fn transactions(
        &self,
        ctx: &Context<'_>,
        ids: Vec<ID>,
    ) -> Result<Vec<GraphQLJReitTransaction>> {
        if ids.is_empty() {
            return Err(Error::new("ids must not be empty"));
        }

        let db = ctx.data::<DatabaseConnection>()?;
        let transaction_ids: Vec<_> = ids.iter().map(|id| id.0.clone()).collect();
        let transactions = j_reit_transactions::Entity::find()
            .filter(j_reit_transactions::Column::Id.is_in(transaction_ids))
            .all(db)
            .await?;

        // 指定されたidsの順序を維持するために、HashMapを使用して結果を並べ替える
        let transaction_map: std::collections::HashMap<_, _> = transactions
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect();

        Ok(ids
            .into_iter()
            .filter_map(|id| transaction_map.get(&id.0).cloned())
            .map(Into::into)
            .collect())
    }

    /// 保存された物件検索条件の一覧を取得
    async fn saved_building_search_params(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<GraphQLSavedBuildingSearchParams>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let roles = ctx.data::<Roles>()?;
        let is_j_reit_premium = roles.data.j_reit_premium;

        let mut query = saved_building_search_params::Entity::find()
            .filter(
                Condition::all()
                    .add(saved_building_search_params::Column::Auth0Id.eq(auth0_id))
                    .add(saved_building_search_params::Column::Deleted.eq(0)),
            )
            .order_by_desc(saved_building_search_params::Column::CreatedAt);

        if !is_j_reit_premium {
            query = query.limit(1);
        }

        let params = query.all(db).await?;

        Ok(params.into_iter().map(Into::into).collect())
    }

    /// 保存された取引検索条件の一覧を取得
    async fn saved_transaction_search_params(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<GraphQLSavedTransactionSearchParams>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let auth0_id = &ctx.data::<Auth0Id>()?.0;
        let roles = ctx.data::<Roles>()?;
        let is_j_reit_premium = roles.data.j_reit_premium;

        let mut query = saved_transaction_search_params::Entity::find()
            .filter(
                Condition::all()
                    .add(saved_transaction_search_params::Column::Auth0Id.eq(auth0_id))
                    .add(saved_transaction_search_params::Column::Deleted.eq(0)),
            )
            .order_by_desc(saved_transaction_search_params::Column::CreatedAt);

        if !is_j_reit_premium {
            query = query.limit(1);
        }

        let params = query.all(db).await?;

        Ok(params.into_iter().map(Into::into).collect())
    }

    async fn j_reit_id_mapping_by_data_hub_id(
        &self,
        ctx: &Context<'_>,
        data_hub_building_id: ID,
    ) -> Result<Option<GraphQLJReitIdMapping>> {
        let db = ctx.data::<DatabaseConnection>()?;

        let mapping = j_reit_mizuho_id_mappings::Entity::find()
            .filter(
                j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId
                    .eq(data_hub_building_id.as_str()),
            )
            .one(db)
            .await?;

        Ok(mapping.map(Into::into))
    }
}
