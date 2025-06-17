mod custom_sql_query;
mod dataloader;
pub mod metadata;
mod mutation;
mod query;
mod types;
mod utils;

use async_graphql::extensions::Tracing;
use async_graphql::EmptySubscription;
use dataloader::dataloader;
use mutation::MutationRoot;
use query::QueryRoot;
use sea_orm::DatabaseConnection;

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn sdl() -> String {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .finish()
        .sdl()
}

pub fn schema(db: DatabaseConnection) -> Schema {
    let dataloader = dataloader(db.clone());
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(db)
        .data(dataloader)
        .extension(Tracing)
        .finish()
}
