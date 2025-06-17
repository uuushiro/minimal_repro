use async_graphql::{Enum, InputObject, SimpleObject};

/// 並び替えの順序
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum GraphQLSortOrder {
    /// 昇順
    Asc,
    /// 降順
    Desc,
}

impl From<GraphQLSortOrder> for sea_orm::Order {
    fn from(value: GraphQLSortOrder) -> Self {
        match value {
            GraphQLSortOrder::Asc => sea_orm::Order::Asc,
            GraphQLSortOrder::Desc => sea_orm::Order::Desc,
        }
    }
}

/// ページネーション設定（指定された順序で offset+1 件目から offset+limit 件目までを取得できる）
#[derive(InputObject, Clone, Copy)]
pub(crate) struct GraphQLPaginateCondition {
    /// オフセット
    pub(crate) offset: u64,
    /// 表示件数
    pub(crate) limit: u64,
}

/// offsetページネーションのページ情報
#[derive(SimpleObject)]
pub(crate) struct GraphQLOffsetPageInfo {
    /// ページ番号 (0-indexed)
    pub(crate) page: u64,
    /// 総ページ数
    pub(crate) total_pages: u64,
    /// 総件数
    pub(crate) total_count: u64,
}
