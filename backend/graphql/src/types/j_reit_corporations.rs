use async_graphql::{SimpleObject, ID};

use sql_entities::j_reit_corporations;

/// J-REITの投資法人情報
#[derive(SimpleObject, Clone, Debug)]
pub(crate) struct GraphQLJReitCorporation {
    /// 投資法人ID
    pub(crate) id: ID,
    /// 名称
    name: String,
    /// 上場廃止しているか否か
    is_delisted: bool,
}

impl From<j_reit_corporations::Model> for GraphQLJReitCorporation {
    fn from(value: j_reit_corporations::Model) -> Self {
        let j_reit_corporations::Model {
            id,
            name,
            is_delisted,
            snowflake_deleted: _,
        } = value.clone();
        Self {
            id: ID::from(id),
            name,
            is_delisted: is_delisted == 1,
        }
    }
}
