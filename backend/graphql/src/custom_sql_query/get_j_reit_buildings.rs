use async_graphql::Result;
use common::types::JReitBuildingIdAndCorporationId;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType::LeftJoin, QueryFilter,
    QueryOrder, QuerySelect, QueryTrait, RelationTrait,
};
use sea_query::{all, Alias, Expr, IntoCondition, NullOrdering};
use sql_entities::{
    j_reit_buildings, j_reit_corporations, j_reit_mizuho_id_mappings, j_reit_transactions,
};

use crate::{
    custom_sql_query::search_j_reit_buildings::get_first_acquisitions_join_subquery,
    types::{
        common::sort_and_pagination::GraphQLPaginateCondition,
        ids::{JReitBuildingId, JReitCorporationId},
        j_reit_buildings::{
            sort_and_pagination::SortJReitBuildingCondition, GraphQLJReitBuilding,
            GraphQLJReitBuildingIdWithCorporationId,
        },
    },
};

use super::search_j_reit_buildings::{
    get_latest_transactions_join_subquery, join_latest_appraisal_history,
    join_latest_cap_rate_history,
};

/// Jリート物件のレコードを指定した順序、ページネーションで取得する
pub(crate) async fn get_paginated_j_reit_buildings(
    db: &DatabaseConnection,
    ids: Option<Vec<JReitBuildingId>>,
    sort_condition: SortJReitBuildingCondition,
    pagination_condition: Option<GraphQLPaginateCondition>,
) -> Result<Vec<GraphQLJReitBuilding>> {
    let j_reit_buildings = j_reit_buildings::Entity::find()
        .distinct()
        .column(j_reit_transactions::Column::JReitCorporationId)
        .inner_join(j_reit_transactions::Entity)
        .join(
            LeftJoin,
            j_reit_buildings::Relation::JReitMizuhoIdMappings
                .def()
                .on_condition(|_left, right| {
                    Expr::col((right, j_reit_mizuho_id_mappings::Column::JReitCorporationId))
                        .equals((
                            j_reit_transactions::Entity,
                            j_reit_transactions::Column::JReitCorporationId,
                        ))
                        .into_condition()
                }),
        )
        .apply_if(ids, |query, ids| {
            query.filter(j_reit_buildings::Column::Id.is_in(ids.into_iter().map(|id| id.0)))
        })
        // ソート条件の適用
        .apply_if(sort_condition.key_in_j_reit_buildings, |query, key| {
            query.order_by_with_nulls(key, sort_condition.order.clone(), NullOrdering::Last)
        })
        .apply_if(sort_condition.key_in_j_reit_corporations, |query, key| {
            query
                .join(
                    LeftJoin,
                    j_reit_transactions::Relation::JReitCorporations.def(),
                )
                .order_by(key, sort_condition.order)
        })
        // 常に順序を一意にするためのソート
        .order_by_asc(j_reit_buildings::Column::Id)
        .order_by_asc(j_reit_transactions::Column::JReitCorporationId)
        // paginationの適用
        .offset(pagination_condition.map(|p| p.offset))
        .limit(pagination_condition.map(|p| p.limit))
        .into_model::<JReitBuildingWithRelationIds>()
        .all(db)
        .await?
        .into_iter()
        .map(|building| building.into())
        .collect();

    Ok(j_reit_buildings)
}

/// Jリート物件のレコードを建物IDと法人IDの組み合わせで取得する
pub(crate) async fn get_j_reit_buildings_per_corporation(
    db: &DatabaseConnection,
    ids: Vec<GraphQLJReitBuildingIdWithCorporationId>,
    sort_condition: SortJReitBuildingCondition,
    pagination_condition: Option<GraphQLPaginateCondition>,
) -> Result<Vec<GraphQLJReitBuilding>> {
    let building_ids: Vec<_> = ids.iter().map(|id| id.building_id.0.clone()).collect();
    let combined_transaction_ids: Vec<_> = ids
        .iter()
        .map(|id| JReitBuildingIdAndCorporationId::from(id.clone()).combined_transaction_id())
        .collect();

    // j_reit_transactionsとjoinしてbuilding_idとcorporation_idの組み合わせを取得
    let query = j_reit_buildings::Entity::find()
        .distinct()
        .column(j_reit_transactions::Column::JReitCorporationId)
        .left_join(j_reit_corporations::Entity)
        .join(
            LeftJoin,
            j_reit_buildings::Relation::JReitMizuhoIdMappings
                .def()
                .on_condition(|_left, right| {
                    all![
                        Expr::col((
                            right.clone(),
                            j_reit_mizuho_id_mappings::Column::JReitBuildingId
                        ))
                        .equals((
                            j_reit_transactions::Entity,
                            j_reit_transactions::Column::JReitBuildingId,
                        )),
                        Expr::col((right, j_reit_mizuho_id_mappings::Column::JReitCorporationId))
                            .equals((
                                j_reit_transactions::Entity,
                                j_reit_transactions::Column::JReitCorporationId,
                            ))
                    ]
                    .into_condition()
                }),
        )
        .filter(j_reit_buildings::Column::Id.is_in(building_ids))
        .filter(j_reit_transactions::Column::CombinedTransactionId.is_in(combined_transaction_ids))
        // ソート条件の適用
        .apply_if(sort_condition.key_in_j_reit_buildings, |query, key| {
            query.order_by_with_nulls(key, sort_condition.order.clone(), NullOrdering::Last)
        })
        .apply_if(sort_condition.key_in_j_reit_corporations, |query, key| {
            // Workaround: BuildingのnameとCorporationのnameがSELECTされて、Buildingのnameが上書きされてしまう
            if key.as_column_ref().1.to_string() == "name" {
                query
                    .column_as(
                        j_reit_corporations::Column::Name,
                        "j_reit_corporations_name",
                    )
                    .order_by_with_nulls(
                        Expr::col(Alias::new("j_reit_corporations_name")),
                        sort_condition.order.clone(),
                        NullOrdering::Last,
                    )
            } else {
                query.column(key).order_by_with_nulls(
                    key,
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            }
        })
        .apply_if(
            sort_condition.key_in_first_acquisitions,
            |mut query, key| {
                QueryTrait::query(&mut query)
                    .join_subquery(
                        LeftJoin,
                        get_first_acquisitions_join_subquery(),
                        Alias::new("joined_first_acquisitions"),
                        all![
                            Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id,))
                                .equals((
                                    Alias::new("joined_first_acquisitions"),
                                    j_reit_transactions::Column::JReitBuildingId,
                                )),
                            Expr::col((
                                j_reit_corporations::Entity,
                                j_reit_corporations::Column::Id,
                            ))
                            .equals((
                                Alias::new("joined_first_acquisitions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            )),
                        ],
                    )
                    .column((Alias::new("joined_first_acquisitions"), key.clone()));
                query.order_by_with_nulls(
                    Expr::col((Alias::new("joined_first_acquisitions"), key.clone())),
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            },
        )
        .apply_if(
            sort_condition.key_in_latest_transactions,
            |mut query, key| {
                QueryTrait::query(&mut query)
                    .join_subquery(
                        LeftJoin,
                        get_latest_transactions_join_subquery(),
                        Alias::new("joined_latest_transactions"),
                        all![
                            Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id,))
                                .equals((
                                    Alias::new("joined_latest_transactions"),
                                    j_reit_transactions::Column::JReitBuildingId,
                                )),
                            Expr::col((
                                j_reit_corporations::Entity,
                                j_reit_corporations::Column::Id,
                            ))
                            .equals((
                                Alias::new("joined_latest_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            )),
                        ],
                    )
                    .column((Alias::new("joined_latest_transactions"), key));
                query.order_by_with_nulls(
                    Expr::col((Alias::new("joined_latest_transactions"), key)),
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            },
        )
        .apply_if(
            sort_condition.key_in_latest_cap_rate_history,
            |mut query, key| {
                join_latest_cap_rate_history(&mut query);
                QueryTrait::query(&mut query)
                    .column((Alias::new("joined_latest_cap_rate_history"), key));
                query.order_by_with_nulls(
                    Expr::col((Alias::new("joined_latest_cap_rate_history"), key)),
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            },
        )
        .apply_if(
            sort_condition.key_in_latest_appraisal_history,
            |mut query, key| {
                join_latest_appraisal_history(&mut query);
                QueryTrait::query(&mut query)
                    .column((Alias::new("joined_latest_appraisal_history"), key));
                query.order_by_with_nulls(
                    Expr::col((Alias::new("joined_latest_appraisal_history"), key)),
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            },
        )
        // 常に順序を一意にするためのソート
        .order_by_asc(j_reit_buildings::Column::Id)
        .order_by_asc(j_reit_transactions::Column::JReitCorporationId)
        // paginationの適用
        .offset(pagination_condition.map(|p| p.offset))
        .limit(pagination_condition.map(|p| p.limit));

    let j_reit_buildings = query
        .into_model::<JReitBuildingWithRelationIds>()
        .all(db)
        .await?
        .into_iter()
        .map(|building| building.into())
        .collect();

    Ok(j_reit_buildings)
}

#[derive(Debug, FromQueryResult)]
struct JReitBuildingWithRelationIds {
    pub(crate) j_reit_corporation_id: Option<String>,
    #[sea_orm(nested)]
    pub(crate) j_reit_building: j_reit_buildings::Model,
}

impl From<JReitBuildingWithRelationIds> for GraphQLJReitBuilding {
    fn from(value: JReitBuildingWithRelationIds) -> Self {
        let j_reit_building = value.j_reit_building;
        let model = j_reit_buildings::Model {
            id: j_reit_building.id,
            is_office: j_reit_building.is_office,
            is_retail: j_reit_building.is_retail,
            is_hotel: j_reit_building.is_hotel,
            is_logistic: j_reit_building.is_logistic,
            is_residential: j_reit_building.is_residential,
            is_health_care: j_reit_building.is_health_care,
            is_other: j_reit_building.is_other,
            office_building_id: j_reit_building.office_building_id,
            residential_building_id: j_reit_building.residential_building_id,
            name: j_reit_building.name,
            address: j_reit_building.address,
            city_id: j_reit_building.city_id,
            latitude: j_reit_building.latitude,
            longitude: j_reit_building.longitude,
            nearest_station: j_reit_building.nearest_station,
            completed_year: j_reit_building.completed_year,
            completed_month: j_reit_building.completed_month,
            gross_floor_area: j_reit_building.gross_floor_area,
            basement: j_reit_building.basement,
            groundfloor: j_reit_building.groundfloor,
            structure: j_reit_building.structure,
            floor_plan: j_reit_building.floor_plan,
            land: j_reit_building.land,
            building_coverage_ratio: j_reit_building.building_coverage_ratio,
            floor_area_ratio: j_reit_building.floor_area_ratio,
            snowflake_deleted: 0,
        };
        let mut building = GraphQLJReitBuilding::from(model);
        building.j_reit_corporation_id = value.j_reit_corporation_id.map(JReitCorporationId);
        building
    }
}
