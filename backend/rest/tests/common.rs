use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend};
use sql_entities::prelude::*;

async fn setup_test_database() -> anyhow::Result<DatabaseConnection> {
    let db = Database::connect("sqlite::memory:").await?;
    let schema = sea_orm::Schema::new(DbBackend::Sqlite);
    let statements = [
        schema.create_table_from_entity(Feedbacks),
        schema.create_table_from_entity(FreeTrials),
        schema.create_table_from_entity(JReitCorporations),
        schema.create_table_from_entity(JReitBuildings),
        schema.create_table_from_entity(JReitMizuhoPressReleases),
        schema.create_table_from_entity(JReitMizuhoAppraisalHistories),
        schema.create_table_from_entity(JReitMizuhoCapRateHistories),
        schema.create_table_from_entity(JReitMizuhoFinancials),
        schema.create_table_from_entity(JReitTransactions),
        schema.create_table_from_entity(JReitAppraisals),
        schema.create_table_from_entity(JReitMizuhoIdMappings),
        schema.create_table_from_entity(Prefectures),
        schema.create_table_from_entity(DesignatedCities),
        schema.create_table_from_entity(DesignatedCityWards),
        schema.create_table_from_entity(Wards),
        schema.create_table_from_entity(Cities),
    ];

    for stmt in statements.iter() {
        db.execute(db.get_database_backend().build(stmt)).await?;
    }
    Ok(db)
}

pub struct TestContext {
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new() -> anyhow::Result<Self> {
        let db = setup_test_database().await?;
        Ok(Self { db })
    }
}
