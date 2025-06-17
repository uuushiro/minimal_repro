use async_graphql::Result;
use common::constants::METERS_PER_MINUTE_ON_FOOT;
use sea_orm::RelationTrait;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType::LeftJoin, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, Value,
};
use sea_query::{Expr, ExprTrait, NullOrdering, SimpleExpr};
use sql_entities::{
    cities, j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_transactions,
    stations, wards,
};

use crate::types::{
    common::sort_and_pagination::{GraphQLOffsetPageInfo, GraphQLPaginateCondition},
    j_reit_buildings::j_reit_transactions::{
        search::{GraphQLSearchTransactionResult, SearchTransactionInput},
        sort_and_pagination::{JReitTransactionSortKeyColumn, SortJReitTransactionCondition},
        GraphQLJReitTransaction,
    },
};

pub(crate) async fn search_transactions(
    db: &DatabaseConnection,
    search_condition: SearchTransactionInput,
    sort_condition: SortJReitTransactionCondition,
    pagination_condition: Option<GraphQLPaginateCondition>,
) -> Result<GraphQLSearchTransactionResult> {
    let mut query = j_reit_transactions::Entity::find()
        .distinct()
        .inner_join(j_reit_buildings::Entity)
        .inner_join(j_reit_corporations::Entity);

    let mut join_appraisals = false;
    // joins
    if search_condition.appraisal_price_min.is_some()
        || search_condition.appraisal_price_max.is_some()
        || search_condition.appraisal_cap_rate_min.is_some()
        || search_condition.appraisal_cap_rate_max.is_some()
    {
        query = query.left_join(j_reit_appraisals::Entity);
        join_appraisals = true;
    }

    // filters

    // TODO: locationリリース後にcity_idsを消す
    if let Some(city_ids) = search_condition.city_ids {
        query = query.filter(j_reit_buildings::Column::CityId.is_in(city_ids));
    }
    if let Some(location) = search_condition.location {
        query = query
            .join(LeftJoin, j_reit_buildings::Relation::Cities.def())
            .join(LeftJoin, cities::Relation::Wards.def())
            .filter(
                Condition::any()
                    .add(j_reit_buildings::Column::CityId.is_in(location.city_ids))
                    .add(cities::Column::WardId.is_in(location.ward_ids))
                    .add(wards::Column::PrefectureId.is_in(location.prefecture_ids)),
            )
    }
    if let Some(lat_lng) = search_condition.latitude_and_longitude {
        query = query.filter(
            Condition::all()
                .add(j_reit_buildings::Column::Latitude.between(lat_lng.south, lat_lng.north))
                .add(j_reit_buildings::Column::Longitude.between(lat_lng.west, lat_lng.east)),
        );
    }
    if let Some(station) = search_condition.station {
        // TODO: テスト実装
        QueryTrait::query(&mut query)
            .inner_join(stations::Entity, Expr::cust("ST_Distance_Sphere(Point(j_reit_buildings.longitude, j_reit_buildings.latitude), Point(stations.longitude, stations.latitude))").between(station.min_time * METERS_PER_MINUTE_ON_FOOT, station.max_time * METERS_PER_MINUTE_ON_FOOT))
            .and_where(stations::Column::Id.is_in(station.station_ids));
    }
    if let Some(transaction_date_min) = search_condition.transaction_date_min {
        query =
            query.filter(j_reit_transactions::Column::TransactionDate.gte(transaction_date_min));
    }
    if let Some(transaction_date_max) = search_condition.transaction_date_max {
        query =
            query.filter(j_reit_transactions::Column::TransactionDate.lte(transaction_date_max));
    }
    if let Some(transaction_price_min) = search_condition.transaction_price_min {
        query = if search_condition.include_bulk == Some(true)
            && search_condition.use_apportioned_price == Some(true)
        {
            query.filter(
                Condition::any()
                    .add(
                        j_reit_transactions::Column::IsBulk.eq(1).and(
                            j_reit_transactions::Column::ApportionedTransactionPrice
                                .gte(transaction_price_min),
                        ),
                    )
                    .add(j_reit_transactions::Column::IsBulk.eq(0).and(
                        j_reit_transactions::Column::TransactionPrice.gte(transaction_price_min),
                    )),
            )
        } else {
            query.filter(j_reit_transactions::Column::TransactionPrice.gte(transaction_price_min))
        }
    }
    if let Some(transaction_price_max) = search_condition.transaction_price_max {
        query = if search_condition.include_bulk == Some(true)
            && search_condition.use_apportioned_price == Some(true)
        {
            query.filter(
                Condition::any()
                    .add(
                        j_reit_transactions::Column::IsBulk.eq(1).and(
                            j_reit_transactions::Column::ApportionedTransactionPrice
                                .lte(transaction_price_max),
                        ),
                    )
                    .add(j_reit_transactions::Column::IsBulk.eq(0).and(
                        j_reit_transactions::Column::TransactionPrice.lte(transaction_price_max),
                    )),
            )
        } else {
            query.filter(j_reit_transactions::Column::TransactionPrice.lte(transaction_price_max))
        }
    }
    if let Some(transaction_categories) = search_condition.transaction_categories {
        let category_values: Vec<i8> = transaction_categories
            .into_iter()
            .map(|c| c as i8)
            .collect();
        query =
            query.filter(j_reit_transactions::Column::TransactionCategory.is_in(category_values));
    }
    if let Some(completion_year_min) = search_condition.completion_year_min {
        query = query.filter(j_reit_buildings::Column::CompletedYear.gte(completion_year_min));
    }
    if let Some(completion_year_max) = search_condition.completion_year_max {
        query = query.filter(j_reit_buildings::Column::CompletedYear.lte(completion_year_max));
    }
    if let Some(gross_floor_area_min) = search_condition.gross_floor_area_min {
        query = query.filter(j_reit_buildings::Column::GrossFloorArea.gte(gross_floor_area_min));
    }
    if let Some(gross_floor_area_max) = search_condition.gross_floor_area_max {
        query = query.filter(j_reit_buildings::Column::GrossFloorArea.lte(gross_floor_area_max));
    }
    if let Some(press_release_date_min) = search_condition.press_release_date_min {
        query =
            query.filter(j_reit_transactions::Column::PressReleaseDate.gte(press_release_date_min));
    }
    if let Some(press_release_date_max) = search_condition.press_release_date_max {
        query =
            query.filter(j_reit_transactions::Column::PressReleaseDate.lte(press_release_date_max));
    }
    if search_condition.include_bulk == Some(false) {
        query = query.filter(j_reit_transactions::Column::IsBulk.eq(0));
    }
    if let Some(appraisal_price_min) = search_condition.appraisal_price_min {
        query = query.filter(j_reit_appraisals::Column::AppraisalPrice.gte(appraisal_price_min));
    }
    if let Some(appraisal_price_max) = search_condition.appraisal_price_max {
        query = query.filter(j_reit_appraisals::Column::AppraisalPrice.lte(appraisal_price_max));
    }
    if let Some(appraisal_cap_rate_min) = search_condition.appraisal_cap_rate_min {
        query = query.filter(j_reit_appraisals::Column::CapRate.gte(appraisal_cap_rate_min));
    }
    if let Some(appraisal_cap_rate_max) = search_condition.appraisal_cap_rate_max {
        query = query.filter(j_reit_appraisals::Column::CapRate.lte(appraisal_cap_rate_max));
    }
    if let Some(j_reit_corporation_ids) = search_condition.j_reit_corporation_ids {
        if !j_reit_corporation_ids.is_empty() {
            query = query.filter(
                j_reit_transactions::Column::JReitCorporationId.is_in(j_reit_corporation_ids),
            );
        }
    }
    if search_condition.include_delisted != Some(true) {
        query = query.filter(j_reit_corporations::Column::IsDelisted.eq(0));
    }

    // アセットタイプ検索（or条件）
    query = query.filter(
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
    );

    if let JReitTransactionSortKeyColumn::Appraisals(column) = sort_condition.key {
        if !join_appraisals {
            query = query.left_join(j_reit_appraisals::Entity);
        }
        query = query.column(column);
    }
    query = query.order_by_with_nulls(sort_condition.key, sort_condition.order, NullOrdering::Last);

    let count = query.clone().count(db).await?;

    let (offset, limit) = if let Some(pagination) = pagination_condition {
        (pagination.offset, pagination.limit)
    } else {
        (0, 10) // デフォルトは最初の10件
    };
    query = query.offset(offset).limit(limit);

    let transactions = query
        .all(db)
        .await?
        .into_iter()
        .map(GraphQLJReitTransaction::from)
        .collect();

    let page_info = GraphQLOffsetPageInfo {
        page: offset / limit,
        total_pages: count.div_ceil(limit),
        total_count: count,
    };

    Ok(GraphQLSearchTransactionResult {
        nodes: transactions,
        page_info,
    })
}
