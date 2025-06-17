pub(crate) mod search;
pub(crate) mod sort_and_pagination;

use crate::types::{
    ids::{JReitAppraisalId, JReitBuildingId, JReitCorporationId},
    j_reit_corporations::GraphQLJReitCorporation,
};
use async_graphql::{ComplexObject, InputObject, SimpleObject, ID};
use chrono::NaiveDate;
use common::types::TransactionCategory;
use sql_entities::j_reit_transactions;

use super::{j_reit_appraisals::GraphQLJReitAppraisal, GraphQLJReitBuilding};
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(crate) struct GraphQLJReitTransaction {
    id: ID,
    /// 戸数（住宅）または客室数（ホテル）
    pub(crate) leasable_units: Option<i64>,
    /// 土地所有形態
    land_ownership_type: Option<String>,
    /// 土地所有割合［%］
    land_ownership_ratio: Option<f64>,
    /// 建物所有形態
    building_ownership_type: Option<String>,
    /// 建物所有割合［%］
    building_ownership_ratio: Option<f64>,
    /// 取引先
    transaction_partner: Option<String>,
    /// 取引日
    pub(crate) transaction_date: NaiveDate,
    /// 取引価格
    transaction_price: Option<i64>,
    /// 取引種類
    pub(crate) transaction_category: TransactionCategory,
    /// 取引分の賃貸可能面積
    leasable_area: Option<f64>,
    /// 累計の賃貸可能面積
    total_leasable_area: Option<f64>,
    /// PM会社
    property_manager: Option<String>,
    /// PML調査会社
    pml_assessment_company: Option<String>,
    /// 信託受託者
    trustee: Option<String>,
    /// プレスリリース日
    press_release_date: Option<NaiveDate>,
    /// バルク取引かどうか
    is_bulk: bool,
    /// 按分後取引価格
    apportioned_transaction_price: Option<i64>,

    #[graphql(skip)]
    pub(crate) j_reit_building_id: JReitBuildingId,
    #[graphql(skip)]
    pub(crate) j_reit_corporation_id: JReitCorporationId,
    #[graphql(skip)]
    pub(crate) j_reit_appraisal_id: Option<JReitAppraisalId>,
}

#[ComplexObject]
impl GraphQLJReitTransaction {
    /// 建物
    pub(crate) async fn building(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitBuilding>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;
        let j_reit_building = dataloader.load_one(self.j_reit_building_id.clone()).await?;
        Ok(j_reit_building)
    }

    /// 鑑定情報
    pub(crate) async fn appraisal(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitAppraisal>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;
        let j_reit_appraisal = match self.j_reit_appraisal_id.clone() {
            Some(j_reit_appraisal_id) => dataloader.load_one(j_reit_appraisal_id).await?,
            None => None,
        };

        Ok(j_reit_appraisal)
    }

    /// 投資法人
    pub(crate) async fn corporation(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<GraphQLJReitCorporation>> {
        let dataloader = ctx.data::<crate::dataloader::DataLoader>()?;
        let j_reit_corporation = dataloader
            .load_one(self.j_reit_corporation_id.clone())
            .await?;
        Ok(j_reit_corporation)
    }
}

impl From<j_reit_transactions::Model> for GraphQLJReitTransaction {
    fn from(model: j_reit_transactions::Model) -> Self {
        Self {
            id: ID(model.id),
            j_reit_building_id: JReitBuildingId(model.j_reit_building_id),
            j_reit_corporation_id: JReitCorporationId(model.j_reit_corporation_id),
            leasable_units: model.leasable_units,
            land_ownership_type: model.land_ownership_type,
            land_ownership_ratio: model.land_ownership_ratio,
            building_ownership_type: model.building_ownership_type,
            building_ownership_ratio: model.building_ownership_ratio,
            transaction_partner: model.transaction_partner,
            transaction_date: model.transaction_date,
            transaction_price: model.transaction_price,
            apportioned_transaction_price: model.apportioned_transaction_price,
            transaction_category: TransactionCategory::from(model.transaction_category),
            leasable_area: model.leasable_area,
            total_leasable_area: model.total_leasable_area,
            property_manager: model.property_manager,
            pml_assessment_company: model.pml_assessment_company,
            trustee: model.trustee,
            press_release_date: model.press_release_date,
            is_bulk: model.is_bulk == 1,
            j_reit_appraisal_id: model.j_reit_appraisal_id.map(JReitAppraisalId),
        }
    }
}

#[derive(InputObject, Clone, Hash, Eq, PartialEq)]
pub(crate) struct JReitTransactionsByJReitBuildingIdWithCorporationId {
    pub(crate) j_reit_building_id: ID,
    pub(crate) j_reit_corporation_id: ID,
}
