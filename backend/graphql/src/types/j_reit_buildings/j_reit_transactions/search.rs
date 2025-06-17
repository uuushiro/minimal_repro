use async_graphql::{InputObject, SimpleObject, ID};
use chrono::NaiveDate;
use common::types::TransactionCategory;

use crate::types::{
    common::{
        search::{
            GraphQLSearchConditionAssetType, GraphQLSearchConditionLatLng,
            GraphQLSearchConditionLocation, GraphQLSearchConditionMinMax,
            GraphQLSearchConditionMinMaxDate, GraphQLSearchConditionMinMaxFloat,
            GraphQLSearchConditionStation, SearchConditionAssetType, SearchConditionLocation,
            SearchConditionStation,
        },
        sort_and_pagination::GraphQLOffsetPageInfo,
    },
    j_reit_buildings::j_reit_transactions::GraphQLJReitTransaction,
};

#[derive(InputObject)]
pub(crate) struct GraphQLSearchTransactionInput {
    /// 町ID
    pub(crate) city_ids: Option<Vec<ID>>, // 後で消す
    /// エリア
    pub(crate) location: Option<GraphQLSearchConditionLocation>,
    /// 駅
    station: Option<GraphQLSearchConditionStation>,
    /// アセットタイプ
    asset_type: Option<GraphQLSearchConditionAssetType>,
    /// 取引日の範囲
    pub(crate) transaction_date: Option<GraphQLSearchConditionMinMaxDate>,
    /// 取引価格の範囲
    pub(crate) transaction_price: Option<GraphQLSearchConditionMinMax>,
    /// 取引種類
    pub(crate) transaction_categories: Option<Vec<TransactionCategory>>,
    /// 竣工年の範囲
    pub(crate) completion_year: Option<GraphQLSearchConditionMinMax>,
    /// 延床面積［坪］の範囲
    pub(crate) gross_floor_area: Option<GraphQLSearchConditionMinMax>,
    /// プレスリリース日の範囲
    pub(crate) press_release_date: Option<GraphQLSearchConditionMinMaxDate>,
    /// バルク取引を含めるかどうか
    pub(crate) include_bulk: Option<bool>,
    /// 鑑定評価額の範囲
    pub(crate) appraisal_price: Option<GraphQLSearchConditionMinMax>,
    /// 鑑定利回り［%］の範囲
    pub(crate) appraisal_cap_rate: Option<GraphQLSearchConditionMinMaxFloat>,
    /// 投資法人ID
    pub(crate) j_reit_corporation_ids: Option<Vec<ID>>,
    /// 上場廃止を含めるかどうか
    pub(crate) include_delisted: Option<bool>,
    /// 按分後取引価格を使用するかどうか
    pub(crate) use_apportioned_price: Option<bool>,
    /// 緯度経度
    pub(crate) latitude_and_longitude: Option<GraphQLSearchConditionLatLng>,
}

pub(crate) struct SearchTransactionInput {
    pub(crate) city_ids: Option<Vec<String>>,
    pub(crate) location: Option<SearchConditionLocation>,
    pub(crate) station: Option<SearchConditionStation>,
    pub(crate) asset_type: SearchConditionAssetType,
    pub(crate) transaction_date_min: Option<NaiveDate>,
    pub(crate) transaction_date_max: Option<NaiveDate>,
    pub(crate) transaction_price_min: Option<i64>,
    pub(crate) transaction_price_max: Option<i64>,
    pub(crate) transaction_categories: Option<Vec<TransactionCategory>>,
    pub(crate) completion_year_min: Option<i64>,
    pub(crate) completion_year_max: Option<i64>,
    pub(crate) gross_floor_area_min: Option<i64>,
    pub(crate) gross_floor_area_max: Option<i64>,
    pub(crate) press_release_date_min: Option<NaiveDate>,
    pub(crate) press_release_date_max: Option<NaiveDate>,
    pub(crate) include_bulk: Option<bool>,
    pub(crate) appraisal_price_min: Option<i64>,
    pub(crate) appraisal_price_max: Option<i64>,
    pub(crate) appraisal_cap_rate_min: Option<f64>,
    pub(crate) appraisal_cap_rate_max: Option<f64>,
    pub(crate) j_reit_corporation_ids: Option<Vec<String>>,
    pub(crate) include_delisted: Option<bool>,
    pub(crate) use_apportioned_price: Option<bool>,
    pub(crate) latitude_and_longitude: Option<GraphQLSearchConditionLatLng>,
}

impl From<GraphQLSearchTransactionInput> for SearchTransactionInput {
    fn from(input: GraphQLSearchTransactionInput) -> Self {
        Self {
            city_ids: input
                .city_ids
                .map(|ids| ids.into_iter().map(|id| id.to_string()).collect()),
            location: input.location.and_then(|location| location.into()),
            station: input.station.and_then(|station| station.into()),
            asset_type: input.asset_type.into(),
            transaction_date_min: input.transaction_date.and_then(|date| date.min),
            transaction_date_max: input.transaction_date.and_then(|date| date.max),
            transaction_price_min: input.transaction_price.and_then(|price| price.min),
            transaction_price_max: input.transaction_price.and_then(|price| price.max),
            transaction_categories: input.transaction_categories,
            completion_year_min: input.completion_year.and_then(|year| year.min),
            completion_year_max: input.completion_year.and_then(|year| year.max),
            gross_floor_area_min: input.gross_floor_area.and_then(|area| area.min),
            gross_floor_area_max: input.gross_floor_area.and_then(|area| area.max),
            press_release_date_min: input.press_release_date.and_then(|date| date.min),
            press_release_date_max: input.press_release_date.and_then(|date| date.max),
            include_bulk: input.include_bulk,
            appraisal_price_min: input.appraisal_price.and_then(|price| price.min),
            appraisal_price_max: input.appraisal_price.and_then(|price| price.max),
            appraisal_cap_rate_min: input.appraisal_cap_rate.and_then(|rate| rate.min),
            appraisal_cap_rate_max: input.appraisal_cap_rate.and_then(|rate| rate.max),
            j_reit_corporation_ids: input
                .j_reit_corporation_ids
                .map(|ids| ids.into_iter().map(|id| id.to_string()).collect()),
            include_delisted: input.include_delisted,
            use_apportioned_price: input.use_apportioned_price,
            latitude_and_longitude: input.latitude_and_longitude,
        }
    }
}

#[derive(SimpleObject)]
pub(crate) struct GraphQLSearchTransactionResult {
    /// 検索結果
    pub(crate) nodes: Vec<GraphQLJReitTransaction>,
    /// ページネーション情報
    pub(crate) page_info: GraphQLOffsetPageInfo,
}
