use async_graphql::{SimpleObject, ID};

#[derive(SimpleObject, Clone, Debug)]
pub struct GraphQLJReitIdMapping {
    pub j_reit_building_id: ID,
    pub j_reit_corporation_id: ID,
}

impl From<sql_entities::j_reit_mizuho_id_mappings::Model> for GraphQLJReitIdMapping {
    fn from(model: sql_entities::j_reit_mizuho_id_mappings::Model) -> Self {
        Self {
            j_reit_building_id: ID(model.j_reit_building_id),
            j_reit_corporation_id: ID(model.j_reit_corporation_id),
        }
    }
}
