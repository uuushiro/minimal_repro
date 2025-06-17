use async_graphql::Result;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use sea_query::SimpleExpr;
use sql_entities::j_reit_corporations;

use crate::types::j_reit_corporations::GraphQLJReitCorporation;

/// Jリート投資法人のレコードを上場廃止の有無に応じて取得する
pub(crate) async fn get_j_reit_corporations(
    db: &DatabaseConnection,
    include_delisted: bool,
) -> Result<Vec<GraphQLJReitCorporation>> {
    let j_reit_corporations = j_reit_corporations::Entity::find()
        .filter(
            Condition::any()
                .add(SimpleExpr::from(include_delisted))
                .add(j_reit_corporations::Column::IsDelisted.eq(Some(0))),
        )
        .all(db)
        .await?
        .into_iter()
        .map(|corporation| corporation.into())
        .collect();

    Ok(j_reit_corporations)
}
