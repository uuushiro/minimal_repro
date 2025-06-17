use async_graphql::{Result, ID};
use common::{constants::METERS_PER_MINUTE_ON_FOOT, types::TransactionCategory};
use proto::Roles;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult,
    JoinType::{InnerJoin, LeftJoin},
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait, Statement,
    Value,
};
use sea_query::{all, Alias, Expr, NullOrdering, Query, SelectStatement, SimpleExpr};
use sql_entities::{
    cities, j_reit_appraisals, j_reit_buildings, j_reit_corporations,
    j_reit_mizuho_appraisal_histories, j_reit_mizuho_cap_rate_histories, j_reit_mizuho_id_mappings,
    j_reit_transactions, wards,
};

use super::common::StringId;
use crate::custom_sql_query::utils::escape_like_pattern;
use crate::types::{
    common::sort_and_pagination::GraphQLPaginateCondition,
    ids::{JReitBuildingId, JReitCorporationId},
    j_reit_buildings::{
        search::SearchJReitBuildingCondition, sort_and_pagination::SortJReitBuildingCondition,
        GraphQLJReitBuildingIdWithCorporationId,
    },
};

pub(crate) struct JReitBuildingSearchResult {
    pub(crate) count: i64,
    pub(crate) j_reit_building_id_and_corporation_ids: Vec<GraphQLJReitBuildingIdWithCorporationId>,
}

#[derive(FromQueryResult)]
struct BuildingWithCorporation {
    id: String,
    j_reit_corporation_id: String,
}

/// Jリート物件の検索条件に合致するデータの件数とページネーションを適用したIDの列を取得する
#[allow(clippy::print_stdout)]
pub(crate) async fn search_j_reit_buildings(
    db: &DatabaseConnection,
    roles: &Roles,
    search_condition: SearchJReitBuildingCondition,
    sort_condition: SortJReitBuildingCondition,
    pagination_condition: Option<GraphQLPaginateCondition>,
) -> Result<JReitBuildingSearchResult> {
    // 閲覧権限がある物件のみ検索でヒットさせる
    // J-REITの場合は現状「全てOK」か「全てNG」なので単に後者の場合空の配列を返す
    if !roles.market_research.login && !roles.deal_management__jreit {
        return Ok(JReitBuildingSearchResult {
            count: 0,
            j_reit_building_id_and_corporation_ids: Vec::new(),
        });
    }

    let filtered_j_reit_building_ids = match search_condition.station {
        Some(station_condition) => {
            // FIXME: このSQLは動かないはずだが、テストでカバーされていないのでテスト追加して確認する
            let sql_statement = format!(
                "SELECT DISTINCT j_reit_buildings.id \
                FROM j_reit_buildings \
                CROSS JOIN stations \
                WHERE stations.id IN ({}) AND \
                ST_Distance_Sphere(\
                    Point(j_reit_buildings.longitude, j_reit_buildings.latitude),\
                    Point(stations.longitude, stations.latitude)\
                ) BETWEEN {} AND {}",
                station_condition
                    .station_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                station_condition.min_time * METERS_PER_MINUTE_ON_FOOT,
                station_condition.max_time * METERS_PER_MINUTE_ON_FOOT
            );
            Some(
                j_reit_buildings::Entity::find()
                    .from_raw_sql(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        sql_statement,
                        vec![],
                    ))
                    .into_model::<StringId>()
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|building| JReitBuildingId(building.id))
                    .collect::<Vec<_>>(),
            )
        }
        None => None,
    };

    let is_search_condition_include_is_delisted_option: Option<bool> =
        (search_condition.include_is_delisted_true || search_condition.include_is_delisted_false)
            .then_some(true);
    let mut is_joined_first_acquisitions = false;
    let mut is_joined_mizuho_id_mappings = false;
    let mut is_joined_latest_cap_rate_history = false;
    let mut is_joined_latest_appraisal_history = false;

    let mut query = j_reit_buildings::Entity::find()
        .select_only()
        .expr_as(
            Expr::cust_with_expr(
                "DISTINCT ?",
                Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)),
            ),
            "id",
        )
        .column_as(j_reit_corporations::Column::Id, "j_reit_corporation_id")
        .inner_join(j_reit_corporations::Entity)
        .apply_if(filtered_j_reit_building_ids, |query, ids| {
            query.filter(j_reit_buildings::Column::Id.is_in(ids.iter().map(|id| id.0.clone())))
        })
        .apply_if(search_condition.name, |query, name| {
            let escaped_name = escape_like_pattern(&name);
            query.filter(j_reit_buildings::Column::Name.contains(escaped_name))
        })
        .apply_if(search_condition.j_reit_corporation_ids, |query, ids| {
            query.filter(
                Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id))
                    .is_in(ids),
            )
        })
        .apply_if(search_condition.location, |query, location| {
            // 所在地の条件
            // prefecture_id, ward_id, city_idのいずれかに合致する
            query
                .join(LeftJoin, j_reit_buildings::Relation::Cities.def())
                .join(LeftJoin, cities::Relation::Wards.def())
                .filter(
                    Condition::any()
                        .add(j_reit_buildings::Column::CityId.is_in(location.city_ids))
                        .add(cities::Column::WardId.is_in(location.ward_ids))
                        .add(wards::Column::PrefectureId.is_in(location.prefecture_ids)),
                )
        })
        .apply_if(search_condition.latitude_and_longitude, |query, lat_lng| {
            query
                .filter(j_reit_buildings::Column::Latitude.gte(lat_lng.south))
                .filter(j_reit_buildings::Column::Latitude.lte(lat_lng.north))
                .filter(j_reit_buildings::Column::Longitude.gte(lat_lng.west))
                .filter(j_reit_buildings::Column::Longitude.lte(lat_lng.east))
        })
        .apply_if(search_condition.completed_year_min, |query, min| {
            query.filter(j_reit_buildings::Column::CompletedYear.gte(min))
        })
        .apply_if(search_condition.completed_year_max, |query, max| {
            query.filter(j_reit_buildings::Column::CompletedYear.lte(max))
        })
        .apply_if(search_condition.land_area_min, |query, min| {
            query.filter(j_reit_buildings::Column::Land.gte(min))
        })
        .apply_if(search_condition.land_area_max, |query, max| {
            query.filter(j_reit_buildings::Column::Land.lte(max))
        })
        .apply_if(search_condition.gross_floor_area_min, |query, min| {
            query.filter(j_reit_buildings::Column::GrossFloorArea.gte(min))
        })
        .apply_if(search_condition.gross_floor_area_max, |query, max| {
            query.filter(j_reit_buildings::Column::GrossFloorArea.lte(max))
        })
        // .join(
        //     LeftJoin,
        //     j_reit_buildings::Relation::JReitTransactions.def(),
        // )
        // })
        .filter(
            // アセットタイプ検索（or条件）
            Condition::any()
                .add(
                    // アセットタイプ検索のinclude_is_officeがtrueである場合、IsOfficeが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_office,
                        )))
                        .add(j_reit_buildings::Column::IsOffice.eq(1)),
                )
                .add(
                    // アセットタイプ検索のinclude_is_retailがtrueである場合、IsRetailが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_retail,
                        )))
                        .add(j_reit_buildings::Column::IsRetail.eq(1)),
                )
                .add(
                    // アセットタイプ検索のinclude_is_hotelがtrueである場合、IsHotelが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_hotel,
                        )))
                        .add(j_reit_buildings::Column::IsHotel.eq(1)),
                )
                .add(
                    // アセットタイプ検索のinclude_is_logisticがtrueである場合、IsLogisticが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_logistic,
                        )))
                        .add(j_reit_buildings::Column::IsLogistic.eq(1)),
                )
                .add(
                    // アセットタイプ検索のinclude_is_residentialがtrueである場合、IsResidentialが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_residential,
                        )))
                        .add(j_reit_buildings::Column::IsResidential.eq(1)),
                )
                .add(
                    // アセットタイプ検索のinclude_is_health_careがtrueである場合、IsHealthCareが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_health_care,
                        )))
                        .add(j_reit_buildings::Column::IsHealthCare.eq(1)),
                )
                .add(
                    // アセットタイプ検索のinclude_is_otherがtrueである場合、IsOtherが1の物件は取得する
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.asset_type.include_is_other,
                        )))
                        .add(j_reit_buildings::Column::IsOther.eq(1)),
                ),
        )
        .filter(
            // 保有中/譲渡済みの条件
            Condition::any()
                // include_trueの場合は譲渡済み。完全譲渡済取引がある
                .add(
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.include_is_transferred_true,
                        )))
                        .add(
                            Expr::col((
                                Alias::new("joined_transferred_transactions"),
                                j_reit_transactions::Column::TransactionDate,
                            ))
                            .is_not_null(),
                        ),
                )
                // include_falseの場合は保有中。完全譲渡済取引がない
                .add(
                    Condition::all()
                        .add(SimpleExpr::from(Value::from(
                            search_condition.include_is_transferred_false,
                        )))
                        .add(
                            Expr::col((
                                Alias::new("joined_transferred_transactions"),
                                j_reit_transactions::Column::TransactionDate,
                            ))
                            .is_null(),
                        ),
                ),
        )
        .apply_if(
            is_search_condition_include_is_delisted_option,
            |query, _| {
                let mut condition = Condition::any();
                if search_condition.include_is_delisted_true {
                    condition = condition.add(j_reit_corporations::Column::IsDelisted.eq(1));
                }
                if search_condition.include_is_delisted_false {
                    condition = condition.add(j_reit_corporations::Column::IsDelisted.eq(0));
                }
                query.filter(condition)
            },
        );

    // sea-orm では表現できないクエリを sea-query で直接記述する
    QueryTrait::query(&mut query)
        .apply_if(
            (search_condition.cap_rate_min.is_some()
                || search_condition.cap_rate_max.is_some()
                || search_condition.appraised_price_min.is_some()
                || search_condition.appraised_price_max.is_some())
            .then_some(true),
            |query, _| {
                is_joined_mizuho_id_mappings = true;
                query.inner_join(
                    j_reit_mizuho_id_mappings::Entity,
                    all![
                        Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals(
                            (
                                j_reit_mizuho_id_mappings::Entity,
                                j_reit_mizuho_id_mappings::Column::JReitBuildingId,
                            )
                        ),
                        Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id))
                            .equals((
                                j_reit_mizuho_id_mappings::Entity,
                                j_reit_mizuho_id_mappings::Column::JReitCorporationId,
                            ))
                    ],
                );
            },
        )
        .apply_if(
            (search_condition.cap_rate_min.is_some() || search_condition.cap_rate_max.is_some())
                .then_some(true),
            |query, _| {
                is_joined_latest_cap_rate_history = true;
                query.join_subquery(
                    InnerJoin,
                    get_latest_cap_rate_history_join_subquery(),
                    Alias::new("joined_latest_cap_rate_history"),
                    all![Expr::col((
                        Alias::new("joined_latest_cap_rate_history"),
                        j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId,
                    ))
                    .eq(Expr::col((
                        j_reit_mizuho_id_mappings::Entity,
                        j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId,
                    ))),],
                );
            },
        )
        .apply_if(search_condition.cap_rate_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_latest_cap_rate_history"),
                    j_reit_mizuho_cap_rate_histories::Column::CapRate,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.cap_rate_max, |query, max| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_latest_cap_rate_history"),
                    j_reit_mizuho_cap_rate_histories::Column::CapRate,
                ))
                .lte(max),
            );
        })
        .apply_if(
            (search_condition.appraised_price_min.is_some()
                || search_condition.appraised_price_max.is_some())
            .then_some(true),
            |query, _| {
                is_joined_latest_appraisal_history = true;
                query.join_subquery(
                    InnerJoin,
                    get_latest_appraisal_history_join_subquery(),
                    Alias::new("joined_latest_appraisal_history"),
                    all![Expr::col((
                        Alias::new("joined_latest_appraisal_history"),
                        j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId,
                    ))
                    .eq(Expr::col((
                        j_reit_mizuho_id_mappings::Entity,
                        j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId,
                    ))),],
                );
            },
        )
        .apply_if(search_condition.appraised_price_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_latest_appraisal_history"),
                    j_reit_mizuho_appraisal_histories::Column::AppraisalPrice,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.appraised_price_max, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_latest_appraisal_history"),
                    j_reit_mizuho_appraisal_histories::Column::AppraisalPrice,
                ))
                .lte(min),
            );
        })
        .apply_if(
            (search_condition.acquisition_date_min.is_some()
                || search_condition.acquisition_date_max.is_some()
                || search_condition.acquisition_price_min.is_some()
                || search_condition.acquisition_price_max.is_some()
                || search_condition.initial_cap_rate_min.is_some()
                || search_condition.initial_cap_rate_max.is_some())
            .then_some(true),
            |query, _| {
                query.join_subquery(
                    InnerJoin,
                    get_first_acquisitions_join_subquery(),
                    Alias::new("joined_first_acquisitions"),
                    all![
                        Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).eq(
                            Expr::col((
                                Alias::new("joined_first_acquisitions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            )),
                        ),
                        Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id))
                            .eq(Expr::col((
                                Alias::new("joined_first_acquisitions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ))),
                    ],
                );
                is_joined_first_acquisitions = true;
            },
        )
        .apply_if(search_condition.acquisition_date_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_first_acquisitions"),
                    j_reit_transactions::Column::TransactionDate,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.acquisition_date_max, |query, max| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_first_acquisitions"),
                    j_reit_transactions::Column::TransactionDate,
                ))
                .lte(max),
            );
        })
        .apply_if(search_condition.acquisition_price_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_first_acquisitions"),
                    j_reit_transactions::Column::TransactionPrice,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.acquisition_price_max, |query, max| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_first_acquisitions"),
                    j_reit_transactions::Column::TransactionPrice,
                ))
                .lte(max),
            );
        })
        .apply_if(search_condition.initial_cap_rate_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_first_acquisitions"),
                    j_reit_appraisals::Column::CapRate,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.initial_cap_rate_max, |query, max| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_first_acquisitions"),
                    j_reit_appraisals::Column::CapRate,
                ))
                .lte(max),
            );
        })
        .apply_if(
            (search_condition.total_leasable_area_min.is_some()
                || search_condition.total_leasable_area_max.is_some())
            .then_some(true),
            |query, _| {
                query.join_subquery(
                    LeftJoin,
                    get_latest_transactions_join_subquery(),
                    Alias::new("joined_latest_transactions"),
                    all![
                        Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).eq(
                            Expr::col((
                                Alias::new("joined_latest_transactions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            ))
                        ),
                        Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id))
                            .eq(Expr::col((
                                Alias::new("joined_latest_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ))),
                    ],
                );
            },
        )
        .apply_if(search_condition.total_leasable_area_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_latest_transactions"),
                    j_reit_transactions::Column::TotalLeasableArea,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.total_leasable_area_max, |query, max| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_latest_transactions"),
                    j_reit_transactions::Column::TotalLeasableArea,
                ))
                .lte(max),
            );
        })
        .apply_if(
            (search_condition.transfer_date_min.is_some()
                || search_condition.transfer_date_max.is_some()
                || search_condition.include_is_transferred_true
                || search_condition.include_is_transferred_false)
                .then_some(true),
            |query, _| {
                query.join_subquery(
                    LeftJoin,
                    get_transferred_transactions_join_subquery(),
                    Alias::new("joined_transferred_transactions"),
                    all![
                        Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).eq(
                            Expr::col((
                                Alias::new("joined_transferred_transactions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            )),
                        ),
                        Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id))
                            .eq(Expr::col((
                                Alias::new("joined_transferred_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ))),
                    ],
                );
            },
        )
        .apply_if(search_condition.transfer_date_min, |query, min| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_transferred_transactions"),
                    j_reit_transactions::Column::TransactionDate,
                ))
                .gte(min),
            );
        })
        .apply_if(search_condition.transfer_date_max, |query, max| {
            query.and_where(
                Expr::col((
                    Alias::new("joined_transferred_transactions"),
                    j_reit_transactions::Column::TransactionDate,
                ))
                .lte(max),
            );
        });

    let count = query.clone().count(db).await?;
    let j_reit_building_id_and_corporation_ids = query
        .clone()
        // ソート条件の適用
        .apply_if(sort_condition.key_in_j_reit_buildings, |query, key| {
            query.order_by_with_nulls(key, sort_condition.order.clone(), NullOrdering::Last)
        })
        .apply_if(sort_condition.key_in_j_reit_corporations, |query, key| {
            query.order_by(key, sort_condition.order.clone())
        })
        .apply_if(
            sort_condition.key_in_first_acquisitions,
            |mut query, key| {
                if !is_joined_first_acquisitions {
                    QueryTrait::query(&mut query).join_subquery(
                        LeftJoin, // ソート条件のみ指定された場合なので LeftJoin にする
                        get_first_acquisitions_join_subquery(),
                        Alias::new("joined_first_acquisitions"),
                        Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).eq(
                            Expr::col((
                                Alias::new("joined_first_acquisitions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            )),
                        ),
                    );
                }
                query.order_by_with_nulls(
                    Expr::col((Alias::new("joined_first_acquisitions"), key)),
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            },
        )
        .apply_if(
            sort_condition.key_in_latest_cap_rate_history,
            |mut query, key| {
                if !is_joined_mizuho_id_mappings {
                    join_mizuho_id_mapping(&mut query);
                }
                if !is_joined_latest_cap_rate_history {
                    join_latest_cap_rate_history(&mut query);
                }
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
                if !is_joined_mizuho_id_mappings {
                    join_mizuho_id_mapping(&mut query);
                };
                if !is_joined_latest_appraisal_history {
                    join_latest_appraisal_history(&mut query);
                }
                query.order_by_with_nulls(
                    Expr::col((Alias::new("joined_latest_appraisal_history"), key)),
                    sort_condition.order.clone(),
                    NullOrdering::Last,
                )
            },
        )
        // 第二のsort indexとしてidを用い、常に順序を一意にする
        .order_by_asc(j_reit_buildings::Column::Id)
        // paginationの適用
        .offset(pagination_condition.map(|p| p.offset))
        .limit(pagination_condition.map(|p| p.limit))
        .into_model::<BuildingWithCorporation>()
        .all(db)
        .await?
        .into_iter()
        .map(|building| GraphQLJReitBuildingIdWithCorporationId {
            building_id: ID::from(building.id),
            corporation_id: ID::from(JReitCorporationId(building.j_reit_corporation_id)),
        })
        .collect::<Vec<_>>();

    Ok(JReitBuildingSearchResult {
        count: count as i64,
        j_reit_building_id_and_corporation_ids,
    })
}

// 一部譲渡は譲渡済みとして扱わない
fn get_transferred_transactions_join_subquery() -> SelectStatement {
    Query::select()
        .columns([
            j_reit_transactions::Column::JReitBuildingId,
            j_reit_transactions::Column::JReitCorporationId,
            j_reit_transactions::Column::TransactionDate,
        ])
        .from(j_reit_transactions::Entity)
        .and_where(
            Expr::col((
                j_reit_transactions::Entity,
                j_reit_transactions::Column::TransactionCategory,
            ))
            .eq(TransactionCategory::FullTransfer as i8),
        )
        .to_owned()
}

pub(crate) fn get_first_acquisitions_join_subquery() -> SelectStatement {
    Query::select()
        .columns([
            j_reit_transactions::Column::JReitBuildingId,
            j_reit_transactions::Column::JReitCorporationId,
            j_reit_transactions::Column::TransactionPrice,
            j_reit_transactions::Column::TransactionDate,
            j_reit_transactions::Column::LeasableArea,
        ])
        .columns([
            j_reit_appraisals::Column::CapRate,
            j_reit_appraisals::Column::AppraisalPrice,
        ])
        .from(j_reit_transactions::Entity)
        .join(
            LeftJoin,
            j_reit_appraisals::Entity,
            Expr::col((
                j_reit_transactions::Entity,
                j_reit_transactions::Column::JReitAppraisalId,
            ))
            .equals((j_reit_appraisals::Entity, j_reit_appraisals::Column::Id)),
        )
        .and_where(
            Expr::col((
                j_reit_transactions::Entity,
                j_reit_transactions::Column::TransactionCategory,
            ))
            .eq(TransactionCategory::InitialAcquisition as i8),
        )
        .to_owned()
}

pub(crate) fn get_latest_transactions_join_subquery() -> SelectStatement {
    Query::select()
        .columns([
            (
                Alias::new("tr"),
                j_reit_transactions::Column::JReitBuildingId,
            ),
            (
                Alias::new("tr"),
                j_reit_transactions::Column::JReitCorporationId,
            ),
            (Alias::new("tr"), j_reit_transactions::Column::LeasableArea),
            (
                Alias::new("tr"),
                j_reit_transactions::Column::TotalLeasableArea,
            ),
        ])
        .from_as(j_reit_transactions::Entity, Alias::new("tr"))
        .join_subquery(
            InnerJoin,
            Query::select()
                .column(j_reit_transactions::Column::JReitBuildingId)
                .column(j_reit_transactions::Column::JReitCorporationId)
                .expr_as(
                    j_reit_transactions::Column::TransactionDate.max(),
                    Alias::new("max_transaction_date"),
                )
                .from(j_reit_transactions::Entity)
                .group_by_col(j_reit_transactions::Column::JReitBuildingId)
                .group_by_col(j_reit_transactions::Column::JReitCorporationId)
                .to_owned(),
            Alias::new("latest"),
            all![
                Expr::col((
                    Alias::new("tr"),
                    j_reit_transactions::Column::JReitBuildingId,
                ))
                .equals((
                    Alias::new("latest"),
                    j_reit_transactions::Column::JReitBuildingId,
                )),
                Expr::col((
                    Alias::new("tr"),
                    j_reit_transactions::Column::JReitCorporationId,
                ))
                .equals((
                    Alias::new("latest"),
                    j_reit_transactions::Column::JReitCorporationId,
                )),
                Expr::col((
                    Alias::new("tr"),
                    j_reit_transactions::Column::TransactionDate,
                ))
                .equals((Alias::new("latest"), Alias::new("max_transaction_date"),)),
            ],
        )
        .to_owned()
}

pub(crate) fn get_latest_cap_rate_history_join_subquery() -> SelectStatement {
    Query::select()
        .columns([
            (
                Alias::new("latest_cap_rate_history"),
                j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId,
            ),
            (
                Alias::new("latest_cap_rate_history"),
                j_reit_mizuho_cap_rate_histories::Column::CapRate,
            ),
        ])
        .from_as(
            j_reit_mizuho_cap_rate_histories::Entity,
            Alias::new("latest_cap_rate_history"),
        )
        .join_subquery(
            InnerJoin,
            Query::select()
                .column(j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId)
                .expr_as(
                    j_reit_mizuho_cap_rate_histories::Column::ClosingDate.max(),
                    Alias::new("max_closing_date"),
                )
                .from(j_reit_mizuho_cap_rate_histories::Entity)
                .group_by_col(j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId)
                .to_owned(),
            Alias::new("latest"),
            all![
                Expr::col((
                    Alias::new("latest_cap_rate_history"),
                    j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId,
                ))
                .eq(Expr::col((
                    Alias::new("latest"),
                    j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId,
                ))),
                Expr::col((
                    Alias::new("latest_cap_rate_history"),
                    j_reit_mizuho_cap_rate_histories::Column::ClosingDate,
                ))
                .eq(Expr::col((
                    Alias::new("latest"),
                    Alias::new("max_closing_date"),
                ))),
            ],
        )
        .to_owned()
}

pub(crate) fn get_latest_appraisal_history_join_subquery() -> SelectStatement {
    Query::select()
        .columns([
            (
                Alias::new("latest_appraisal_history"),
                j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId,
            ),
            (
                Alias::new("latest_appraisal_history"),
                j_reit_mizuho_appraisal_histories::Column::AppraisalPrice,
            ),
        ])
        .from_as(
            j_reit_mizuho_appraisal_histories::Entity,
            Alias::new("latest_appraisal_history"),
        )
        .join_subquery(
            InnerJoin,
            Query::select()
                .column(j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId)
                .expr_as(
                    j_reit_mizuho_appraisal_histories::Column::AppraisalDate.max(),
                    Alias::new("max_appraisal_date"),
                )
                .from(j_reit_mizuho_appraisal_histories::Entity)
                .group_by_col(j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId)
                .to_owned(),
            Alias::new("latest"),
            all![
                Expr::col((
                    Alias::new("latest_appraisal_history"),
                    j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId,
                ))
                .eq(Expr::col((
                    Alias::new("latest"),
                    j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId,
                ))),
                Expr::col((
                    Alias::new("latest_appraisal_history"),
                    j_reit_mizuho_appraisal_histories::Column::AppraisalDate,
                ))
                .eq(Expr::col((
                    Alias::new("latest"),
                    Alias::new("max_appraisal_date"),
                )),),
            ],
        )
        .to_owned()
}

pub(crate) fn join_mizuho_id_mapping(query: &mut sea_orm::Select<j_reit_buildings::Entity>) {
    QueryTrait::query(query).join(
        InnerJoin,
        j_reit_mizuho_id_mappings::Entity,
        all![
            Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals((
                j_reit_mizuho_id_mappings::Entity,
                j_reit_mizuho_id_mappings::Column::JReitBuildingId,
            )),
            Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)).equals((
                j_reit_mizuho_id_mappings::Entity,
                j_reit_mizuho_id_mappings::Column::JReitCorporationId,
            ))
        ],
    );
}

pub(crate) fn join_latest_cap_rate_history(query: &mut sea_orm::Select<j_reit_buildings::Entity>) {
    QueryTrait::query(query).join_subquery(
        LeftJoin,
        get_latest_cap_rate_history_join_subquery(),
        Alias::new("joined_latest_cap_rate_history"),
        all![Expr::col((
            j_reit_mizuho_id_mappings::Entity,
            j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId
        ))
        .eq(Expr::col((
            Alias::new("joined_latest_cap_rate_history"),
            j_reit_mizuho_cap_rate_histories::Column::JReitMizuhoBuildingId,
        )),),],
    );
}

pub(crate) fn join_latest_appraisal_history(query: &mut sea_orm::Select<j_reit_buildings::Entity>) {
    QueryTrait::query(query).join_subquery(
        LeftJoin,
        get_latest_appraisal_history_join_subquery(),
        Alias::new("joined_latest_appraisal_history"),
        all![Expr::col((
            j_reit_mizuho_id_mappings::Entity,
            j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId
        ))
        .eq(Expr::col((
            Alias::new("joined_latest_appraisal_history"),
            j_reit_mizuho_appraisal_histories::Column::JReitMizuhoBuildingId,
        )),),],
    );
}
