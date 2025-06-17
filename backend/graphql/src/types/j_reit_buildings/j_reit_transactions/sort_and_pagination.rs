use async_graphql::{Enum, InputObject};
use sea_orm::IntoSimpleExpr;
use sea_query::{IntoColumnRef, IntoIden, SimpleExpr};
use sql_entities::{j_reit_appraisals, j_reit_transactions};

use crate::types::common::sort_and_pagination::{GraphQLPaginateCondition, GraphQLSortOrder};

/// J-REIT取引の並び替えとページネーションの条件
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLJReitTransactionSortAndPaginateCondition {
    /// 並び替え条件
    pub(crate) sort: Option<GraphQLJReitTransactionSortCondition>,
    /// ページネーション条件
    pub(crate) pagination: Option<GraphQLPaginateCondition>,
}

/// J-REIT取引の並び替え条件
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLJReitTransactionSortCondition {
    /// 並び替えのキー
    key: GraphQLJReitTransactionSortKey,
    /// 並び替えの順序
    order: GraphQLSortOrder,
}

/// J-REIT取引の並び替えキー
#[derive(Enum, Clone, Copy, PartialEq, Eq)]
enum GraphQLJReitTransactionSortKey {
    /// 取引日
    TransactionDate,
    /// 取引種類
    TransactionCategory,
    /// 取引価格
    TransactionPrice,
    /// プレスリリース日
    PressReleaseDate,
    /// 取引時鑑定評価額
    AppraisalPrice,
    /// 取引時鑑定キャップレート
    AppraisalCapRate,
    /// 按分後取引価格
    ApportionedTransactionPrice,
}

#[derive(Clone, Debug)]
pub(crate) enum JReitTransactionSortKeyColumn {
    Transactions(j_reit_transactions::Column),
    Appraisals(j_reit_appraisals::Column),
}

impl IntoIden for JReitTransactionSortKeyColumn {
    fn into_iden(self) -> sea_query::DynIden {
        match self {
            JReitTransactionSortKeyColumn::Transactions(column) => column.into_iden(),
            JReitTransactionSortKeyColumn::Appraisals(column) => column.into_iden(),
        }
    }
}

impl IntoSimpleExpr for JReitTransactionSortKeyColumn {
    fn into_simple_expr(self) -> SimpleExpr {
        SimpleExpr::Column(self.into_iden().into_column_ref())
    }
}

impl From<GraphQLJReitTransactionSortKey> for JReitTransactionSortKeyColumn {
    fn from(value: GraphQLJReitTransactionSortKey) -> Self {
        match value {
            GraphQLJReitTransactionSortKey::TransactionDate => {
                JReitTransactionSortKeyColumn::Transactions(
                    j_reit_transactions::Column::TransactionDate,
                )
            }
            GraphQLJReitTransactionSortKey::TransactionCategory => {
                JReitTransactionSortKeyColumn::Transactions(
                    j_reit_transactions::Column::TransactionCategory,
                )
            }
            GraphQLJReitTransactionSortKey::TransactionPrice => {
                JReitTransactionSortKeyColumn::Transactions(
                    j_reit_transactions::Column::TransactionPrice,
                )
            }
            GraphQLJReitTransactionSortKey::PressReleaseDate => {
                JReitTransactionSortKeyColumn::Transactions(
                    j_reit_transactions::Column::PressReleaseDate,
                )
            }
            GraphQLJReitTransactionSortKey::AppraisalPrice => {
                JReitTransactionSortKeyColumn::Appraisals(j_reit_appraisals::Column::AppraisalPrice)
            }
            GraphQLJReitTransactionSortKey::AppraisalCapRate => {
                JReitTransactionSortKeyColumn::Appraisals(j_reit_appraisals::Column::CapRate)
            }
            GraphQLJReitTransactionSortKey::ApportionedTransactionPrice => {
                JReitTransactionSortKeyColumn::Transactions(
                    j_reit_transactions::Column::ApportionedTransactionPrice,
                )
            }
        }
    }
}

pub(crate) struct SortJReitTransactionCondition {
    pub(crate) key: JReitTransactionSortKeyColumn,
    pub(crate) order: sea_orm::Order,
}

impl From<Option<GraphQLJReitTransactionSortCondition>> for SortJReitTransactionCondition {
    fn from(value: Option<GraphQLJReitTransactionSortCondition>) -> Self {
        if let Some(value) = value {
            let key = value.key.into();
            let order = value.order.into();
            Self { key, order }
        } else {
            // デフォルトは取引日の降順
            Self {
                key: JReitTransactionSortKeyColumn::Transactions(
                    j_reit_transactions::Column::TransactionDate,
                ),
                order: sea_orm::Order::Desc,
            }
        }
    }
}
