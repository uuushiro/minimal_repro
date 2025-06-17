use crate::types::ids::JReitMizuhoBuildingId;
use crate::types::utils::to_decimal_2_digits;
use async_graphql::{SimpleObject, ID};
use chrono::NaiveDate;
use sea_orm::prelude::Decimal;
use sql_entities::j_reit_mizuho_cap_rate_histories;

#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitCapRateHistory {
    id: ID,
    /// キャップレート［%］
    cap_rate: Decimal,
    /// 決算期の締め日（初回は初回取得日）
    pub(crate) closing_date: NaiveDate,

    #[graphql(skip)]
    pub(crate) j_reit_mizuho_building_id: JReitMizuhoBuildingId,
}

impl From<j_reit_mizuho_cap_rate_histories::Model> for GraphQLJReitCapRateHistory {
    fn from(model: j_reit_mizuho_cap_rate_histories::Model) -> Self {
        Self {
            id: ID(model.id),
            cap_rate: to_decimal_2_digits(model.cap_rate),
            closing_date: model.closing_date,
            j_reit_mizuho_building_id: JReitMizuhoBuildingId(model.j_reit_mizuho_building_id),
        }
    }
}
