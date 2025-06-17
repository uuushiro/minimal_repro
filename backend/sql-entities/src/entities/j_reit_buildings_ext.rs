use crate::{j_reit_buildings, j_reit_corporations, j_reit_transactions};
use sea_orm::entity::prelude::*;

impl Related<j_reit_corporations::Entity> for j_reit_buildings::Entity {
    fn to() -> RelationDef {
        j_reit_transactions::Relation::JReitCorporations.def()
    }

    fn via() -> Option<RelationDef> {
        Some(j_reit_transactions::Relation::JReitBuildings.def().rev())
    }
}
