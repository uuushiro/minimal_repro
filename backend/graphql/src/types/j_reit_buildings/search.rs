use async_graphql::{InputObject, SimpleObject};
use chrono::NaiveDate;

use crate::types::{
    common::search::{
        GraphQLSearchConditionAssetType, GraphQLSearchConditionLatLng,
        GraphQLSearchConditionLocation, GraphQLSearchConditionMinMax,
        GraphQLSearchConditionMinMaxDate, GraphQLSearchConditionMinMaxFloat,
        GraphQLSearchConditionStation, SearchConditionAssetType, SearchConditionLocation,
        SearchConditionStation,
    },
    j_reit_buildings::GraphQLJReitBuilding,
};

/// J-REIT物件の検索結果
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLSearchJReitBuildingResult {
    /// 該当のJ-REIT物件
    j_reit_buildings: Vec<GraphQLJReitBuilding>,
    /// ヒットした件数
    total_count: i64,
}

impl GraphQLSearchJReitBuildingResult {
    pub(crate) fn new(count: i64, j_reit_buildings: Vec<GraphQLJReitBuilding>) -> Self {
        Self {
            j_reit_buildings,
            total_count: count,
        }
    }
}

/// Jリート物件の検索条件
#[derive(InputObject, Clone)]
pub(crate) struct GraphQLSearchJReitBuildingCondition {
    /// 建物名
    name: Option<String>,
    /// 投資法人ID
    j_reit_corporation_ids: Option<Vec<String>>,
    /// 所在地（都道府県、市区町村、町名）
    /// 設定時点で約1%ほどcity_idがnullのデータが存在し、それらはこの条件では検索できない
    location: Option<GraphQLSearchConditionLocation>,
    /// 駅
    station: Option<GraphQLSearchConditionStation>,
    /// 緯度軽度
    /// 設定する場合は緯度軽度、最大最小の全ての値を設定する必要がある
    latitude_and_longitude: Option<GraphQLSearchConditionLatLng>,
    /// 竣工年
    completed_year: Option<GraphQLSearchConditionMinMax>,
    /// 敷地面積（坪）
    land_area: Option<GraphQLSearchConditionMinMax>,
    /// 延床面積（坪）
    gross_floor_area: Option<GraphQLSearchConditionMinMax>,
    /// 賃貸可能面積（坪）
    total_leasable_area: Option<GraphQLSearchConditionMinMax>,
    /// 取引日
    acquisition_date: Option<GraphQLSearchConditionMinMaxDate>,
    /// 取得価格
    acquisition_price: Option<GraphQLSearchConditionMinMax>,
    /// 鑑定額
    appraised_price: Option<GraphQLSearchConditionMinMax>,
    /// 取得時キャップレート
    initial_cap_rate: Option<GraphQLSearchConditionMinMaxFloat>,
    /// 最新キャップレート
    cap_rate: Option<GraphQLSearchConditionMinMaxFloat>,
    /// 譲渡日
    transfer_date: Option<GraphQLSearchConditionMinMaxDate>,
    /// アセットタイプ（or検索、1件でもtrueがある場合trueになっているアセットのどれかに合致する物件を返す）
    asset_type: Option<GraphQLSearchConditionAssetType>,
    /// 譲渡済みか否か（trueで譲渡済みのみ、falseで保有中のみ、Noneでどちらも）
    is_transferred: Option<bool>,
    /// 上場廃止か否か（trueで上場廃止のみ、falseで上場中のみ、Noneでどちらも）
    is_delisted: Option<bool>,
}

// J-REIT物件の検索条件（正規化後）
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct SearchJReitBuildingCondition {
    // 建物名
    pub(crate) name: Option<String>,
    // 投資法人ID
    pub(crate) j_reit_corporation_ids: Option<Vec<String>>,
    // 所在地（都道府県、市区町村、町名）
    pub(crate) location: Option<SearchConditionLocation>,
    // 駅
    pub(crate) station: Option<SearchConditionStation>,
    // 緯度軽度
    pub(crate) latitude_and_longitude: Option<GraphQLSearchConditionLatLng>,
    // 竣工年
    pub(crate) completed_year_min: Option<i64>,
    pub(crate) completed_year_max: Option<i64>,
    // 敷地面積（坪）
    pub(crate) land_area_min: Option<i64>,
    pub(crate) land_area_max: Option<i64>,
    // 延床面積（坪）
    pub(crate) gross_floor_area_min: Option<i64>,
    pub(crate) gross_floor_area_max: Option<i64>,
    // 賃貸可能面積（坪）
    pub(crate) total_leasable_area_min: Option<i64>,
    pub(crate) total_leasable_area_max: Option<i64>,
    // 取引日
    pub(crate) acquisition_date_min: Option<NaiveDate>,
    pub(crate) acquisition_date_max: Option<NaiveDate>,
    // 取得価格
    pub(crate) acquisition_price_min: Option<i64>,
    pub(crate) acquisition_price_max: Option<i64>,
    // 鑑定額
    pub(crate) appraised_price_min: Option<i64>,
    pub(crate) appraised_price_max: Option<i64>,
    // 取得時キャップレート
    pub(crate) initial_cap_rate_min: Option<f64>,
    pub(crate) initial_cap_rate_max: Option<f64>,
    // 最新キャップレート
    pub(crate) cap_rate_min: Option<f64>,
    pub(crate) cap_rate_max: Option<f64>,
    /// 譲渡日
    pub(crate) transfer_date_min: Option<NaiveDate>,
    pub(crate) transfer_date_max: Option<NaiveDate>,
    // アセットタイプ
    pub(crate) asset_type: SearchConditionAssetType,
    // 譲渡済みか否か
    pub(crate) include_is_transferred_true: bool,
    pub(crate) include_is_transferred_false: bool,
    // 上場廃止か否か
    pub(crate) include_is_delisted_true: bool,
    pub(crate) include_is_delisted_false: bool,
}

impl From<GraphQLSearchJReitBuildingCondition> for SearchJReitBuildingCondition {
    fn from(input: GraphQLSearchJReitBuildingCondition) -> Self {
        let GraphQLSearchJReitBuildingCondition {
            name,
            j_reit_corporation_ids,
            location,
            station,
            latitude_and_longitude,
            completed_year,
            land_area,
            gross_floor_area,
            total_leasable_area,
            acquisition_date,
            acquisition_price,
            appraised_price,
            initial_cap_rate,
            cap_rate,
            transfer_date,
            asset_type,
            is_transferred,
            is_delisted,
        } = input;

        // 空文字の場合はNoneに変換
        let name = name.filter(|name| !name.is_empty());

        let location = location.and_then(|l| l.into());
        let station = station.and_then(|s| s.into());

        let (include_is_transferred_true, include_is_transferred_false) = match is_transferred {
            Some(true) => (true, false),
            Some(false) => (false, true),
            None => (true, true),
        };
        let (include_is_delisted_true, include_is_delisted_false) = match is_delisted {
            Some(true) => (true, false),
            Some(false) => (false, true),
            None => (true, true),
        };

        SearchJReitBuildingCondition {
            name,
            j_reit_corporation_ids,
            location,
            station,
            latitude_and_longitude,
            completed_year_min: completed_year.and_then(|c| c.min),
            completed_year_max: completed_year.and_then(|c| c.max),
            land_area_min: land_area.and_then(|a| a.min),
            land_area_max: land_area.and_then(|a| a.max),
            gross_floor_area_min: gross_floor_area.and_then(|a| a.min),
            gross_floor_area_max: gross_floor_area.and_then(|a| a.max),
            total_leasable_area_min: total_leasable_area.and_then(|a| a.min),
            total_leasable_area_max: total_leasable_area.and_then(|a| a.max),
            acquisition_date_min: acquisition_date.and_then(|a| a.min),
            acquisition_date_max: acquisition_date.and_then(|a| a.max),
            acquisition_price_min: acquisition_price.and_then(|a| a.min),
            acquisition_price_max: acquisition_price.and_then(|a| a.max),
            appraised_price_min: appraised_price.and_then(|a| a.min),
            appraised_price_max: appraised_price.and_then(|a| a.max),
            initial_cap_rate_min: initial_cap_rate.and_then(|r| r.min),
            initial_cap_rate_max: initial_cap_rate.and_then(|r| r.max),
            cap_rate_min: cap_rate.and_then(|r| r.min),
            cap_rate_max: cap_rate.and_then(|r| r.max),
            transfer_date_min: transfer_date.and_then(|t| t.min),
            transfer_date_max: transfer_date.and_then(|t| t.max),
            asset_type: asset_type.into(),
            include_is_transferred_true,
            include_is_transferred_false,
            include_is_delisted_true,
            include_is_delisted_false,
        }
    }
}
