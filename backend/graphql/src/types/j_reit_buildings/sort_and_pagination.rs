use async_graphql::{Enum, InputObject};
use sea_query::IntoIden;
use sql_entities::{
    j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_mizuho_appraisal_histories,
    j_reit_mizuho_cap_rate_histories, j_reit_transactions,
};

use crate::types::common::sort_and_pagination::{GraphQLPaginateCondition, GraphQLSortOrder};

/// J-REIT物件の並び替えとページネーションの条件
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLJReitBuildingSortAndPaginateCondition {
    /// 並び替え条件
    pub(crate) sort: Option<GraphQLJReitBuildingSortCondition>,
    /// ページネーション条件
    pub(crate) pagination: Option<GraphQLPaginateCondition>,
}

/// J-REIT物件の並び替え条件
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLJReitBuildingSortCondition {
    /// 並び替えのキー
    key: GraphQLJReitBuildingSortKey,
    /// 並び替えの順序
    order: GraphQLSortOrder,
}

/// J-REIT物件の並び替えキー
#[derive(Enum, Clone, Copy, PartialEq, Eq)]
enum GraphQLJReitBuildingSortKey {
    /// 竣工年
    CompletedYear,
    /// 敷地面積
    LandArea,
    /// 延床面積
    GrossFloorArea,
    /// 賃貸可能面積
    TotalLeasableArea,
    /// 初回取引時賃貸可能面積
    InitialLeasableArea,
    /// キャップレート
    CapRate,
    /// 取得時キャップレート
    InitialCapRate,
    /// 鑑定価格
    AppraisedPrice,
    /// 取得時鑑定価格
    InitialAppraisedPrice,
    /// 取得価格
    AcquisitionPrice,
    /// 取得日
    AcquisitionDate,
    /// 投資法人名
    JReitCorporationName,
}

#[derive(Clone, Debug)]
pub(crate) struct SortJReitBuildingCondition {
    pub(crate) key_in_j_reit_buildings: Option<j_reit_buildings::Column>,
    pub(crate) key_in_j_reit_corporations: Option<j_reit_corporations::Column>,
    pub(crate) key_in_first_acquisitions: Option<TransactionOrAppraisalColumn>,
    pub(crate) key_in_latest_transactions: Option<j_reit_transactions::Column>,
    pub(crate) key_in_latest_appraisal_history: Option<j_reit_mizuho_appraisal_histories::Column>,
    pub(crate) key_in_latest_cap_rate_history: Option<j_reit_mizuho_cap_rate_histories::Column>,
    pub(crate) order: sea_orm::Order,
}

#[derive(Clone, Debug)]
pub(crate) enum TransactionOrAppraisalColumn {
    Transactions(j_reit_transactions::Column),
    Appraisals(j_reit_appraisals::Column),
}

impl IntoIden for TransactionOrAppraisalColumn {
    fn into_iden(self) -> sea_query::DynIden {
        match self {
            TransactionOrAppraisalColumn::Transactions(column) => column.into_iden(),
            TransactionOrAppraisalColumn::Appraisals(column) => column.into_iden(),
        }
    }
}

impl Default for SortJReitBuildingCondition {
    fn default() -> Self {
        Self {
            key_in_j_reit_buildings: None,
            key_in_j_reit_corporations: None,
            key_in_first_acquisitions: None,
            key_in_latest_transactions: None,
            key_in_latest_appraisal_history: None,
            key_in_latest_cap_rate_history: None,
            order: sea_orm::Order::Asc,
        }
    }
}

impl From<Option<GraphQLJReitBuildingSortCondition>> for SortJReitBuildingCondition {
    fn from(value: Option<GraphQLJReitBuildingSortCondition>) -> Self {
        if let Some(value) = value {
            let mut condition = Self::default();
            match value.key {
                GraphQLJReitBuildingSortKey::CompletedYear => {
                    condition.key_in_j_reit_buildings =
                        Some(j_reit_buildings::Column::CompletedYear);
                }
                GraphQLJReitBuildingSortKey::LandArea => {
                    condition.key_in_j_reit_buildings = Some(j_reit_buildings::Column::Land);
                }
                GraphQLJReitBuildingSortKey::GrossFloorArea => {
                    condition.key_in_j_reit_buildings =
                        Some(j_reit_buildings::Column::GrossFloorArea);
                }
                GraphQLJReitBuildingSortKey::InitialLeasableArea => {
                    // TODO: 初回取引時賃貸可能面積は今は使われていない？
                    condition.key_in_first_acquisitions =
                        Some(TransactionOrAppraisalColumn::Transactions(
                            j_reit_transactions::Column::LeasableArea,
                        ));
                }
                GraphQLJReitBuildingSortKey::CapRate => {
                    condition.key_in_latest_cap_rate_history =
                        Some(j_reit_mizuho_cap_rate_histories::Column::CapRate);
                }
                GraphQLJReitBuildingSortKey::InitialCapRate => {
                    condition.key_in_first_acquisitions =
                        Some(TransactionOrAppraisalColumn::Appraisals(
                            j_reit_appraisals::Column::CapRate,
                        ));
                }
                GraphQLJReitBuildingSortKey::AppraisedPrice => {
                    condition.key_in_latest_appraisal_history =
                        Some(j_reit_mizuho_appraisal_histories::Column::AppraisalPrice);
                }
                GraphQLJReitBuildingSortKey::InitialAppraisedPrice => {
                    condition.key_in_first_acquisitions =
                        Some(TransactionOrAppraisalColumn::Appraisals(
                            j_reit_appraisals::Column::AppraisalPrice,
                        ));
                }
                GraphQLJReitBuildingSortKey::AcquisitionPrice => {
                    condition.key_in_first_acquisitions =
                        Some(TransactionOrAppraisalColumn::Transactions(
                            j_reit_transactions::Column::TransactionPrice,
                        ));
                }
                GraphQLJReitBuildingSortKey::AcquisitionDate => {
                    condition.key_in_first_acquisitions =
                        Some(TransactionOrAppraisalColumn::Transactions(
                            j_reit_transactions::Column::TransactionDate,
                        ))
                }
                GraphQLJReitBuildingSortKey::JReitCorporationName => {
                    condition.key_in_j_reit_corporations = Some(j_reit_corporations::Column::Name);
                }
                GraphQLJReitBuildingSortKey::TotalLeasableArea => {
                    condition.key_in_latest_transactions =
                        Some(j_reit_transactions::Column::TotalLeasableArea);
                }
            };
            condition.order = value.order.into();
            condition
        } else {
            Self::default()
        }
    }
}
