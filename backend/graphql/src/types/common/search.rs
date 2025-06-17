use crate::utils::id_converter::try_parse_graphql_ids_to_i64_vec_option;
use async_graphql::{InputObject, ID};
use chrono::NaiveDate;

// 最小値と最大値の検索条件（入力用）
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLSearchConditionMinMax {
    pub(crate) min: Option<i64>,
    pub(crate) max: Option<i64>,
}

// 最小値と最大値の検索条件（小数）（入力用）
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLSearchConditionMinMaxFloat {
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
}

// 最小値と最大値の検索条件（日付）（入力用）
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLSearchConditionMinMaxDate {
    pub(crate) min: Option<NaiveDate>,
    pub(crate) max: Option<NaiveDate>,
}

/// 緯度軽度の検索条件
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLSearchConditionLatLng {
    /// 北端の緯度
    pub(crate) north: f64,
    /// 南端の緯度
    pub(crate) south: f64,
    /// 東端の経度
    pub(crate) east: f64,
    /// 西端の経度
    pub(crate) west: f64,
}

/// 所在地（自治体）の検索条件
#[derive(InputObject, Clone)]
pub(crate) struct GraphQLSearchConditionLocation {
    prefecture_ids: Option<Vec<ID>>,
    ward_ids: Option<Vec<ID>>,
    city_ids: Option<Vec<ID>>,
}

// 所在地（自治体）の検索条件（正規化後）
// 入力は使いやすさのためにOptionだが、処理の際にはNoneにならないように扱う
#[derive(Clone)]
pub(crate) struct SearchConditionLocation {
    pub(crate) prefecture_ids: Vec<i64>,
    pub(crate) ward_ids: Vec<i64>,
    pub(crate) city_ids: Vec<i64>,
}

impl From<GraphQLSearchConditionLocation> for Option<SearchConditionLocation> {
    fn from(input: GraphQLSearchConditionLocation) -> Self {
        let prefecture_ids_vec: Vec<ID> = input.prefecture_ids.unwrap_or_default();
        let prefecture_ids: Vec<i64> =
            try_parse_graphql_ids_to_i64_vec_option(&prefecture_ids_vec)?;
        let ward_ids_vec: Vec<ID> = input.ward_ids.unwrap_or_default();
        let ward_ids: Vec<i64> = try_parse_graphql_ids_to_i64_vec_option(&ward_ids_vec)?;
        let city_ids_vec: Vec<ID> = input.city_ids.unwrap_or_default();
        let city_ids: Vec<i64> = try_parse_graphql_ids_to_i64_vec_option(&city_ids_vec)?;

        // 全て空の場合はこの検索条件自体が指定されていないのでNoneにする
        if prefecture_ids.is_empty() && ward_ids.is_empty() && city_ids.is_empty() {
            None
        } else {
            Some(SearchConditionLocation {
                prefecture_ids,
                ward_ids,
                city_ids,
            })
        }
    }
}

// 駅の検索条件（入力用）
#[derive(InputObject, Clone)]
pub(crate) struct GraphQLSearchConditionStation {
    station_ids: Vec<ID>,
    /// 徒歩分数（最大）
    max_time: i64,
    /// 徒歩分数（最小）
    min_time: Option<i64>,
}

// 駅の検索条件（正規化後）
// 入力は使いやすさのためにOptionだが、処理の際にはNoneにならないように扱う
#[derive(Clone)]
pub(crate) struct SearchConditionStation {
    pub(crate) station_ids: Vec<i64>,
    pub(crate) max_time: i64,
    pub(crate) min_time: i64,
}

impl From<GraphQLSearchConditionStation> for Option<SearchConditionStation> {
    fn from(input: GraphQLSearchConditionStation) -> Self {
        let station_ids: Vec<i64> = try_parse_graphql_ids_to_i64_vec_option(&input.station_ids)?;

        // 駅IDが空の場合はこの検索条件自体が指定されていないのでNoneにする
        if station_ids.is_empty() {
            None
        } else {
            Some(SearchConditionStation {
                station_ids,
                max_time: input.max_time,
                min_time: input.min_time.unwrap_or(0),
            })
        }
    }
}

/// Jリート物件のアセットタイプ検索条件
/// 全て未指定またはfalseの場合は全てのアセットタイプを含む検索になる
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLSearchConditionAssetType {
    pub(crate) is_office: Option<bool>,
    pub(crate) is_retail: Option<bool>,
    pub(crate) is_hotel: Option<bool>,
    pub(crate) is_logistic: Option<bool>,
    pub(crate) is_residential: Option<bool>,
    pub(crate) is_health_care: Option<bool>,
    pub(crate) is_other: Option<bool>,
}

impl GraphQLSearchConditionAssetType {
    // 1つでも絞り込み条件が設定されていればfalse
    pub(crate) fn is_empty(&self) -> bool {
        self.is_office != Some(true)
            && self.is_retail != Some(true)
            && self.is_hotel != Some(true)
            && self.is_logistic != Some(true)
            && self.is_residential != Some(true)
            && self.is_health_care != Some(true)
            && self.is_other != Some(true)
    }
}

#[derive(Clone)]
pub(crate) struct SearchConditionAssetType {
    pub(crate) include_is_office: bool,
    pub(crate) include_is_retail: bool,
    pub(crate) include_is_hotel: bool,
    pub(crate) include_is_logistic: bool,
    pub(crate) include_is_residential: bool,
    pub(crate) include_is_health_care: bool,
    pub(crate) include_is_other: bool,
}

impl From<Option<GraphQLSearchConditionAssetType>> for SearchConditionAssetType {
    fn from(input: Option<GraphQLSearchConditionAssetType>) -> Self {
        // 全てのアセットタイプを含むという条件（条件が設定されていない場合これにマッピングする）
        let include_all_condition = SearchConditionAssetType {
            include_is_office: true,
            include_is_retail: true,
            include_is_hotel: true,
            include_is_logistic: true,
            include_is_residential: true,
            include_is_health_care: true,
            include_is_other: true,
        };

        match input {
            // 条件が設定されていない場合はアセットタイプでの絞り込みはしないので、全てのアセットタイプを含むという条件になる
            None => include_all_condition,
            Some(input) => {
                // 条件が設定されていない場合はアセットタイプでの絞り込みはしないので、全てのアセットタイプを含むという条件になる
                if input.is_empty() {
                    return include_all_condition;
                }

                // そうでない場合は、指定されたアセットタイプのみを含む条件になる
                let GraphQLSearchConditionAssetType {
                    is_office,
                    is_retail,
                    is_hotel,
                    is_logistic,
                    is_residential,
                    is_health_care,
                    is_other,
                } = input;

                Self {
                    include_is_office: is_office.unwrap_or(false),
                    include_is_retail: is_retail.unwrap_or(false),
                    include_is_hotel: is_hotel.unwrap_or(false),
                    include_is_logistic: is_logistic.unwrap_or(false),
                    include_is_residential: is_residential.unwrap_or(false),
                    include_is_health_care: is_health_care.unwrap_or(false),
                    include_is_other: is_other.unwrap_or(false),
                }
            }
        }
    }
}
