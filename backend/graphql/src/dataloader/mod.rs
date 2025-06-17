mod j_reit_appraisal_histories;
mod j_reit_appraisals;
mod j_reit_buildings;
mod j_reit_buildings_by_office_building_id;
mod j_reit_buildings_per_corporation;
mod j_reit_cap_rate_histories;
mod j_reit_corporations;
mod j_reit_financials;
mod j_reit_mizuho_building_id;
mod j_reit_mizuho_press_releases;
mod j_reit_transactions;

use sea_orm::DatabaseConnection;

pub(super) type DataLoader = async_graphql::dataloader::DataLoader<GraphQLLoader>;
pub(super) fn dataloader(db: DatabaseConnection) -> DataLoader {
    DataLoader::new(GraphQLLoader { db }, tokio::spawn)
}

#[allow(dead_code)]
pub(super) struct GraphQLLoader {
    pub(crate) db: DatabaseConnection,
}
