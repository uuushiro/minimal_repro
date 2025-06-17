use async_graphql::Result;
use graphql::{schema, Schema};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, DbBackend};
use serde_json::Value;
use sql_entities::prelude::*;
use sql_entities::{cities, prefectures, wards};

async fn setup_test_database() -> Result<DatabaseConnection> {
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
        schema.create_table_from_entity(SavedBuildingSearchParams),
        schema.create_table_from_entity(SavedTransactionSearchParams),
    ];

    for stmt in statements.iter() {
        db.execute(db.get_database_backend().build(stmt)).await?;
    }
    Ok(db)
}

#[allow(dead_code)]
pub struct TestContext {
    pub db: DatabaseConnection,
    pub schema: Schema,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        let db = setup_test_database().await?;
        let schema = schema(db.clone());
        Ok(Self { db, schema })
    }
}

/// graphqlの返り値を順不同で一致判定する
/// 返り値のネストされた中身が配列で順不同の場合は対応できないのでad-hocに対応する
#[allow(dead_code, clippy::unwrap_used)]
pub fn is_equal_json_set(result: Value, expected: Value, key: &str) -> bool {
    let result_array = result[key].as_array().unwrap();
    let expected_array = expected[key].as_array().unwrap();
    if result_array.len() != expected_array.len() {
        return false;
    }
    for expected_body in expected_array {
        if !result_array.contains(expected_body) {
            return false;
        }
    }
    true
}

// テスト用の自治体・路線データ
// 関東->東京都->千代田区->丸の内
// JR->山手線->東京駅
// 東京メトロ->丸の内線->東京駅・大手町駅
pub const TEST_PREFECTURE_ID_TOKYO: i64 = 12;
pub const TEST_WARD_ID_CHIYODA: i64 = 1899;
pub const TEST_CITY_ID_MARUNOUCHI: i64 = 29918;
pub const TEST_PREFECTURE_ID_HOKKAIDO: i64 = 47;
pub const TEST_WARD_ID_SAPPORO_CHUO: i64 = 1601;
pub const TEST_CITY_ID_ODORI_HIGASHI: i64 = 6773;

#[allow(dead_code)]
pub async fn setup_basic_test_data(db: &DatabaseConnection) -> Result<()> {
    prefectures::ActiveModel {
        id: Set(TEST_PREFECTURE_ID_TOKYO),
        name: Set("東京都".into()),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    wards::ActiveModel {
        id: Set(TEST_WARD_ID_CHIYODA),
        prefecture_id: Set(TEST_PREFECTURE_ID_TOKYO),
        name: Set("千代田区".into()),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    cities::ActiveModel {
        id: Set(TEST_CITY_ID_MARUNOUCHI),
        ward_id: Set(TEST_WARD_ID_CHIYODA),
        name: Set(Some("丸の内".into())),
        latitude: Set(35.6804),
        longitude: Set(139.761),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    prefectures::ActiveModel {
        id: Set(TEST_PREFECTURE_ID_HOKKAIDO),
        name: Set("北海道".into()),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    wards::ActiveModel {
        id: Set(TEST_WARD_ID_SAPPORO_CHUO),
        prefecture_id: Set(TEST_PREFECTURE_ID_HOKKAIDO),
        name: Set("札幌市中央区".into()),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    cities::ActiveModel {
        id: Set(TEST_CITY_ID_ODORI_HIGASHI),
        ward_id: Set(TEST_WARD_ID_SAPPORO_CHUO),
        name: Set(Some("大通東".into())),
        latitude: Set(43.063),
        longitude: Set(141.365),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    Ok(())
}
