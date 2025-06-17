use super::{
    ids::{
        CityId, JReitAppraisalHistoriesByJReitMizuhoBuildingId, JReitBuildingId,
        JReitCapRateHistoriesByJReitMizuhoBuildingId, JReitCorporationId,
        JReitFinancialsByJReitMizuhoBuildingId, JReitMizuhoBuildingIdByBuildingIdAndCorporationId,
        JReitPressReleasesByJReitMizuhoBuildingId, JReitTransactionsByJReitBuildingId,
    },
    j_reit_corporations::GraphQLJReitCorporation,
    j_reit_financials::GraphQLJReitFinancial,
    utils::to_decimal_2_digits,
};
use async_graphql::{ComplexObject, InputObject, SimpleObject, ID};
use chrono::NaiveDate;
use common::types::{JReitBuildingIdAndCorporationId, TransactionCategory};
use j_reit_appraisal_histories::GraphQLJReitAppraisalHistory;
use j_reit_cap_rate_histories::GraphQLJReitCapRateHistory;
use j_reit_press_releases::GraphQLJReitPressRelease;
use j_reit_transactions::{
    GraphQLJReitTransaction, JReitTransactionsByJReitBuildingIdWithCorporationId,
};
use sea_orm::prelude::Decimal;
use sql_entities::j_reit_buildings;

pub(crate) mod j_reit_appraisal_histories;
pub(crate) mod j_reit_appraisals;
pub(crate) mod j_reit_cap_rate_histories;
pub(crate) mod j_reit_press_releases;
pub(crate) mod j_reit_transactions;
pub(crate) mod search;
pub(crate) mod sort_and_pagination;

/// Jリート物件のデータ
/// 参考になるかも: https://nabra-estie.atlassian.net/wiki/spaces/AC/pages/1748174129/J-REIT
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(crate) struct GraphQLJReitBuilding {
    pub(crate) id: ID,
    // #[graphql(skip)]
    // pub(crate) building_id: Option<BuildingId>,
    /// オフィスビルID
    pub(crate) office_building_id: Option<ID>,
    /// 住居ビルID
    pub(crate) residential_building_id: Option<ID>,
    #[graphql(skip)]
    #[allow(dead_code)] // FIXME: fn city 実装したら allow dead_code 削除する
    pub(crate) city_id: CityId,
    /// 資産タイプ
    asset_type: GraphQLJReitBuildingAssetType,
    /// 建物基本データ
    building_spec: GraphQLJReitBuildingBuildingSpec,
    /// 土地基本データ
    land_spec: GraphQLJReitBuildingLandSpec,

    #[graphql(skip)]
    pub(crate) j_reit_corporation_id: Option<JReitCorporationId>,
}

#[ComplexObject]
impl GraphQLJReitBuilding {
    /// 決算データ
    /// firstとlastの両方が指定された場合、firstが優先される
    pub(crate) async fn financials(
        &self,
        ctx: &async_graphql::Context<'_>,
        first: Option<usize>,
        last: Option<usize>,
    ) -> async_graphql::Result<Vec<GraphQLJReitFinancial>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;

        let j_reit_mizuho_building_id = match &self.j_reit_corporation_id {
            Some(j_reit_corporation_id) => {
                dataloader
                    .load_one(JReitMizuhoBuildingIdByBuildingIdAndCorporationId {
                        j_reit_building_id: JReitBuildingId::from(self.id.clone()),
                        j_reit_corporation_id: j_reit_corporation_id.clone(),
                    })
                    .await?
            }
            None => None,
        };

        // Then use the Mizuho building ID to get the financials
        let mut j_reit_financials = match j_reit_mizuho_building_id {
            Some(Some(mizuho_id)) => dataloader
                .load_one(JReitFinancialsByJReitMizuhoBuildingId(mizuho_id))
                .await?
                .unwrap_or(Vec::new()),
            _ => Vec::new(),
        };
        j_reit_financials.sort_by_key(|t| t.fiscal_period.end_date);

        let j_reit_financials = match (first, last) {
            (Some(first), _) => j_reit_financials.iter().take(first).cloned().collect(),
            (None, Some(last)) => j_reit_financials
                .iter()
                .rev()
                .take(last)
                .rev()
                .cloned()
                .collect(),
            _ => j_reit_financials,
        };

        Ok(j_reit_financials)
    }

    /// 投資法人データ
    pub(crate) async fn j_reit_corporation(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitCorporation>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;
        let j_reit_corporation = match self.j_reit_corporation_id.clone() {
            Some(j_reit_corporation_id) => dataloader.load_one(j_reit_corporation_id).await?,
            None => None,
        };

        Ok(j_reit_corporation)
    }

    /// プレスリリースデータ
    pub(crate) async fn press_releases(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<GraphQLJReitPressRelease>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;

        let j_reit_mizuho_building_id = match &self.j_reit_corporation_id {
            Some(j_reit_corporation_id) => {
                dataloader
                    .load_one(JReitMizuhoBuildingIdByBuildingIdAndCorporationId {
                        j_reit_building_id: JReitBuildingId::from(self.id.clone()),
                        j_reit_corporation_id: j_reit_corporation_id.clone(),
                    })
                    .await?
            }
            None => None,
        };

        // Then use the Mizuho building ID to get the press releases
        let press_releases = match j_reit_mizuho_building_id {
            Some(Some(mizuho_id)) => dataloader
                .load_one(JReitPressReleasesByJReitMizuhoBuildingId(mizuho_id))
                .await?
                .unwrap_or(Vec::new()),
            _ => Vec::new(),
        };

        Ok(press_releases)
    }

    /// 鑑定評価の履歴
    pub(crate) async fn appraisal_histories(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<GraphQLJReitAppraisalHistory>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;

        let j_reit_mizuho_building_id = match &self.j_reit_corporation_id {
            Some(j_reit_corporation_id) => {
                dataloader
                    .load_one(JReitMizuhoBuildingIdByBuildingIdAndCorporationId {
                        j_reit_building_id: JReitBuildingId::from(self.id.clone()),
                        j_reit_corporation_id: j_reit_corporation_id.clone(),
                    })
                    .await?
            }
            None => None,
        };

        // Then use the Mizuho building ID to get the appraisal histories
        let mut appraisal_histories = match j_reit_mizuho_building_id {
            Some(Some(mizuho_id)) => dataloader
                .load_one(JReitAppraisalHistoriesByJReitMizuhoBuildingId(mizuho_id))
                .await?
                .unwrap_or(Vec::new()),
            _ => Vec::new(),
        };
        appraisal_histories.sort_by_key(|t| t.appraisal_date);

        Ok(appraisal_histories)
    }

    /// キャップレートの履歴
    pub(crate) async fn cap_rate_histories(
        &self,
        ctx: &async_graphql::Context<'_>,
        first: Option<usize>,
        last: Option<usize>,
    ) -> async_graphql::Result<Vec<GraphQLJReitCapRateHistory>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;

        let j_reit_mizuho_building_id = match &self.j_reit_corporation_id {
            Some(j_reit_corporation_id) => {
                dataloader
                    .load_one(JReitMizuhoBuildingIdByBuildingIdAndCorporationId {
                        j_reit_building_id: JReitBuildingId::from(self.id.clone()),
                        j_reit_corporation_id: j_reit_corporation_id.clone(),
                    })
                    .await?
            }
            None => None,
        };

        let mut cap_rate_histories: Vec<_> = match j_reit_mizuho_building_id {
            Some(Some(mizuho_id)) => dataloader
                .load_one(JReitCapRateHistoriesByJReitMizuhoBuildingId(mizuho_id))
                .await?
                .unwrap_or(Vec::new()),
            _ => Vec::new(),
        };

        cap_rate_histories.sort_by_key(|t| t.closing_date);

        if let Some(first_n) = first {
            cap_rate_histories.truncate(first_n);
        } else if let Some(last_n) = last {
            if last_n < cap_rate_histories.len() {
                let skip_count = cap_rate_histories.len() - last_n;
                let last_items: Vec<_> = cap_rate_histories
                    .iter()
                    .skip(skip_count)
                    .cloned()
                    .collect();
                cap_rate_histories = last_items;
            }
        }

        Ok(cap_rate_histories)
    }
    /* FIXME: 実装する
    /// 所在する町（「丸の内」など）
    pub(crate) async fn city(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLCity>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;
        let city = match &self.city_id {
            Some(city_id) => dataloader.load_one(*city_id).await?,
            None => None,
        };

        Ok(city)
    }
    */
    pub(crate) async fn transactions(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<GraphQLJReitTransaction>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;

        if let Some(corporation_id) = self.j_reit_corporation_id.to_owned() {
            let mut transactions = dataloader
                .load_one(JReitTransactionsByJReitBuildingIdWithCorporationId {
                    j_reit_building_id: self.id.clone(),
                    j_reit_corporation_id: ID::from(corporation_id),
                })
                .await?
                .unwrap_or(Vec::new());
            transactions.sort_by_key(|t| t.transaction_date);
            return Ok(transactions);
        }

        let mut transactions = dataloader
            .load_one(JReitTransactionsByJReitBuildingId(JReitBuildingId::from(
                self.id.clone(),
            )))
            .await?
            .unwrap_or(Vec::new());
        transactions.sort_by_key(|t| t.transaction_date);

        Ok(transactions)
    }

    /// 初回取得トランザクション
    pub(crate) async fn initial_acquisition(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitTransaction>> {
        let transactions = self.transactions(ctx).await?;

        Ok(transactions.into_iter().find(|t| {
            matches!(
                t.transaction_category,
                TransactionCategory::InitialAcquisition
            )
        }))
    }

    /// 最新のトランザクション
    pub(crate) async fn latest_transaction(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitTransaction>> {
        let mut transactions = self.transactions(ctx).await?;
        transactions.sort_by_key(|t| t.transaction_date);

        Ok(transactions.into_iter().last())
    }

    /// 譲渡トランザクション
    pub(crate) async fn transfer_transaction(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitTransaction>> {
        let transactions = self.transactions(ctx).await?;

        Ok(transactions
            .into_iter()
            .find(|t| matches!(t.transaction_category, TransactionCategory::FullTransfer)))
    }

    /// 最新の決算データ
    pub(crate) async fn latest_financial(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitFinancial>> {
        let financials = self.financials(ctx, None, Some(1)).await?;

        Ok(financials.into_iter().next())
    }
}

/// Jリート物件の資産タイプ
/// 複数の資産タイプを持つことがある（オフィス兼住宅など）
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitBuildingAssetType {
    /// オフィス
    is_office: bool,
    /// 商業
    is_retail: bool,
    /// ホテル
    is_hotel: bool,
    /// 物流施設
    is_logistic: bool,
    /// 住宅
    is_residential: bool,
    /// 医療施設
    is_health_care: bool,
    /// その他
    is_other: bool,
}

/// Jリート物件の建物基本データ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitBuildingBuildingSpec {
    /// 物件名
    name: String,
    /// 住所
    address: Option<String>,
    /// 緯度
    latitude: f64,
    /// 経度
    longitude: f64,
    /// 最寄駅
    nearest_station: Option<String>,
    /// 竣工年
    completed_year: Option<i64>,
    /// 竣工月
    completed_month: Option<i64>,
    /// 延床面積［坪］
    gross_floor_area: Option<Decimal>,
    /// 地下階数
    basement: Option<i64>,
    /// 地上階数
    groundfloor: Option<i64>,
    /// 構造
    structure: Option<String>,
    /// 間取（住宅）
    floor_plan: Option<String>,
}

/// Jリート物件の土地基本データ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitBuildingLandSpec {
    /// 敷地面積［坪］
    land: Option<Decimal>,
    /// 建蔽率
    building_coverage_ratio: Option<Decimal>,
    /// 容積率
    floor_area_ratio: Option<Decimal>,
}

/// Jリート物件の所有権情報
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitBuildingOwnership {
    /// 土地所有形態
    land_ownership_type: Option<String>,
    /// 土地所有割合［%］
    land_ownership_ratio: Option<Decimal>,
    /// 建物所有形態
    building_ownership_type: Option<String>,
    /// 建物所有割合［%］
    building_ownership_ratio: Option<Decimal>,
}

/// Jリート物件の取得情報
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitBuildingAcquisition {
    /// 売主
    initial_acquisition_seller: Option<String>,
    /// 取得日
    acquisition_date: Option<NaiveDate>,
    /// 取得価格
    acquisition_price: Option<i64>,
    /// 取引時の貸付有効面積［坪］あたりの取得価格
    acquisition_price_per_initial_net_leasable_area_total: Option<Decimal>,
    /// 取得時鑑定価格
    initial_appraised_price: Option<i64>,
    /// 取引時の貸付有効面積［坪］あたりの取得時鑑定価格
    initial_appraised_price_per_initial_net_leasable_area_total: Option<Decimal>,
    /// 取得時の貸付有効面積［坪］
    initial_net_leasable_area_total: Option<Decimal>,
    /// 取得時還元利回り（直接還元法）［%］
    /// 参考：https://www.ownersbook.jp/blog/basics_of_real_estate_investment/capitalization_approach/
    initial_cap_rate: Option<Decimal>,
    /// 取得時最終還元利回り（DCF法）［%］
    /// 参考：同上
    initial_terminal_cap_rate: Option<Decimal>,
    /// 取得時割引率（DCF法）
    /// 参考：同上
    initial_discounted_cap_rate: Option<Decimal>,
}

impl From<j_reit_buildings::Model> for GraphQLJReitBuilding {
    fn from(value: j_reit_buildings::Model) -> Self {
        let j_reit_buildings::Model {
            id,
            is_office,
            is_retail,
            is_hotel,
            is_logistic,
            is_residential,
            is_health_care,
            is_other,
            office_building_id,
            residential_building_id,
            name,
            address,
            city_id,
            latitude,
            longitude,
            nearest_station,
            completed_year,
            completed_month,
            gross_floor_area,
            basement,
            groundfloor,
            structure,
            floor_plan,
            land,
            building_coverage_ratio,
            floor_area_ratio,
            snowflake_deleted: _,
        } = value;

        Self {
            id: ID(id),
            // building_id: building_id.map(BuildingId),
            office_building_id: office_building_id.map(|num| ID(num.to_string())),
            residential_building_id: residential_building_id.map(|num| ID(num.to_string())),
            city_id: CityId(city_id),
            asset_type: GraphQLJReitBuildingAssetType {
                is_office: is_office == 1,
                is_retail: is_retail == 1,
                is_hotel: is_hotel == 1,
                is_logistic: is_logistic == 1,
                is_residential: is_residential == 1,
                is_health_care: is_health_care == 1,
                is_other: is_other == 1,
            },
            building_spec: GraphQLJReitBuildingBuildingSpec {
                name,
                address,
                latitude,
                longitude,
                nearest_station,
                completed_year,
                completed_month,
                gross_floor_area: gross_floor_area.map(to_decimal_2_digits),
                basement,
                groundfloor,
                structure,
                floor_plan,
            },
            land_spec: GraphQLJReitBuildingLandSpec {
                land: land.map(to_decimal_2_digits),
                building_coverage_ratio: building_coverage_ratio.map(to_decimal_2_digits),
                floor_area_ratio: floor_area_ratio.map(to_decimal_2_digits),
            },
            j_reit_corporation_id: None,
        }
    }
}

#[derive(InputObject, Clone, Hash, Eq, PartialEq)]
pub(crate) struct GraphQLJReitBuildingIdWithCorporationId {
    pub(crate) building_id: ID,
    pub(crate) corporation_id: ID,
}

impl From<GraphQLJReitBuildingIdWithCorporationId> for JReitBuildingIdAndCorporationId {
    fn from(value: GraphQLJReitBuildingIdWithCorporationId) -> Self {
        Self {
            building_id: value.building_id.0,
            corporation_id: value.corporation_id.0,
        }
    }
}
