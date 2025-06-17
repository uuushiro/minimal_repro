use crate::types::ids::JReitMizuhoBuildingId;
use async_graphql::{SimpleObject, ID};
use chrono::NaiveDate;
use sql_entities::j_reit_mizuho_appraisal_histories;

#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitAppraisalHistory {
    id: ID,
    #[graphql(skip)]
    pub(crate) j_reit_mizuho_building_id: JReitMizuhoBuildingId,
    /// 鑑定日
    pub(crate) appraisal_date: NaiveDate,
    /// 鑑定価格［円］
    appraisal_price: i64,
}

impl From<j_reit_mizuho_appraisal_histories::Model> for GraphQLJReitAppraisalHistory {
    fn from(model: j_reit_mizuho_appraisal_histories::Model) -> Self {
        Self {
            id: ID(model.id),
            j_reit_mizuho_building_id: JReitMizuhoBuildingId(model.j_reit_mizuho_building_id),
            appraisal_date: model.appraisal_date,
            appraisal_price: model.appraisal_price,
        }
    }
}
