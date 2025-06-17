use crate::common::TEST_CITY_ID_MARUNOUCHI;
use chrono::{NaiveDate, Utc};
use common::types::TransactionCategory;
use proto::{DataRoles, MarketResearch, MarketResearchRoles, MessageField, ProductRoles, Roles};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng, Rng,
};
use sea_orm::prelude::DateTimeUtc;
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveModelTrait, Set};
use sql_entities::{
    cities, designated_cities, designated_city_wards, j_reit_appraisals, j_reit_buildings,
    j_reit_corporations, j_reit_mizuho_id_mappings, j_reit_transactions, prefectures, wards,
};

/// market_researchのログイン権限を指定してRolesを作成する
#[allow(dead_code)]
pub fn test_roles_market_research_login(market_research_login: bool) -> Roles {
    let market_research_roles = MessageField::some(MarketResearch {
        login: market_research_login,
        ..Default::default()
    });
    let product_roles = MessageField::some(ProductRoles {
        market_research: market_research_roles,
        ..Default::default()
    });
    let market_research_roles_old = MessageField::some(MarketResearchRoles {
        login: market_research_login,
        ..Default::default()
    });
    Roles {
        market_research: market_research_roles_old,
        products: product_roles,
        ..Default::default()
    }
}

#[allow(dead_code)]
pub async fn insert_test_j_reit_corporation(
    db: &DatabaseConnection,
    name: String,
    id: String,
    is_delisted: i8,
) -> String {
    j_reit_corporations::ActiveModel {
        id: Set(id.clone()),
        name: Set(name),
        is_delisted: Set(is_delisted),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test j_reit corporation");
    id
}

#[allow(dead_code)]
pub async fn insert_test_j_reit_transaction(
    db: &DatabaseConnection,
    j_reit_building_id: String,
    j_reit_corporation_id: String,
) {
    let mut rng = rng();
    let id: String = Alphanumeric.sample_string(&mut rng, 32);
    let j_reit_appraisal_id = insert_test_appraisal(db, id.clone()).await;

    j_reit_transactions::ActiveModel {
        id: Set(id.clone()),
        j_reit_building_id: Set(j_reit_building_id.clone()),
        j_reit_corporation_id: Set(j_reit_corporation_id.clone()),
        j_reit_appraisal_id: Set(Some(j_reit_appraisal_id)),
        transaction_date: Set(datetime_utc(2025, 1, 1, 0).date_naive()),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            &j_reit_building_id,
            &j_reit_corporation_id,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert test j_reit transaction");
}

#[allow(dead_code)]
pub async fn insert_test_j_reit_building(db: &DatabaseConnection, id: String) -> String {
    let j_reit_building = sql_entities::j_reit_buildings::ActiveModel {
        id: Set(id.clone()),
        name: Set("test default".into()),
        is_office: Set(0),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    j_reit_building
        .insert(db)
        .await
        .expect("Failed to insert test j_reit building");
    id
}

// DateTimeUtcの作成（入力値に使う）
#[allow(dead_code)]
pub fn datetime_utc(year: i32, month: i32, day: i32, hour: i32) -> DateTimeUtc {
    let datetime = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .expect("Failed to create NaiveDate")
        .and_hms_opt(hour as u32, 0, 0)
        .expect("Failed to create NaiveDateTime");
    DateTimeUtc::from_naive_utc_and_offset(datetime, Utc)
}

// DateTimeUtcの出力値の文字列形式
#[allow(dead_code)]
pub fn datetime_response(year: i32, month: i32, day: i32, hour: i32) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:00:00+00:00",
        year, month, day, hour
    )
}

// NaiveDateの作成（入力値に使う）
#[allow(dead_code)]
pub fn naive_date(year: i32, month: i32, day: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).expect("Failed to create NaiveDate")
}

#[allow(dead_code)]
pub async fn create_j_reit_building1(
    db: &sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    j_reit_buildings::ActiveModel {
        id: Set("test_id_1".into()),
        is_office: Set(1), // true
        is_retail: Set(0), // false
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_residential: Set(1),
        is_health_care: Set(0),
        is_other: Set(0),
        office_building_id: Set(Some(1111)),
        residential_building_id: Set(Some(2222)),
        name: Set("estieビル".into()),
        address: Set(Some("東京都千代田区丸の内1-1".into())),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        nearest_station: Set(Some("東京駅徒歩3分".into())),
        completed_year: Set(Some(2000)),
        completed_month: Set(Some(1)),
        gross_floor_area: Set(Some(1000.0)),
        basement: Set(Some(2)),
        groundfloor: Set(Some(23)),
        structure: Set(Some("SRC".into())),
        floor_plan: Set(Some("1K:30".into())),
        land: Set(Some(500.0)),
        building_coverage_ratio: Set(Some(70.0)),
        floor_area_ratio: Set(Some(300.0)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn create_j_reit_building2(
    db: &sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    j_reit_buildings::ActiveModel {
        id: Set("test_id_2".into()),
        completed_year: Set(Some(1999)),
        completed_month: Set(Some(1)),
        // // 以下はnot nullなので設定（確認はしない）
        name: Set("estie第２ビル".into()),
        is_office: Set(1),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        office_building_id: Set(Some(2222)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn create_j_reit_building3(
    db: &sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    j_reit_buildings::ActiveModel {
        id: Set("test_id_3".into()),
        completed_year: Set(Some(1999)),
        completed_month: Set(Some(1)),
        // // 以下はnot nullなので設定（確認はしない）
        name: Set("estie第３倉庫".into()),
        is_office: Set(0),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(1),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.0),
        longitude: Set(139.0),
        office_building_id: Set(Some(3333)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn create_j_reit_transactions(
    db: &sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    let j_reit_corporation_id =
        insert_test_j_reit_corporation(db, "法人A".into(), "test_company_id_1".into(), 0).await;
    let j_reit_appraisal_1_id = insert_test_appraisal(db, "test_appraisal_id_1".into()).await;
    let j_reit_appraisal_2_id = insert_test_appraisal(db, "test_appraisal_id_2".into()).await;
    let j_reit_appraisal_3_id = insert_test_appraisal(db, "test_appraisal_id_3".into()).await;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_1".into()),
        j_reit_building_id: Set("test_id_1".into()),
        leasable_units: Set(Some(40)),
        land_ownership_type: Set(Some("所有権".into())),
        land_ownership_ratio: Set(Some(100.0)),
        building_ownership_type: Set(Some("全体所有".into())),
        building_ownership_ratio: Set(Some(100.0)),
        transaction_partner: Set(Some("売主A".to_string())),
        transaction_date: Set(datetime_utc(2021, 1, 1, 0).date_naive()),
        transaction_price: Set(Some(1000000000)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        leasable_area: Set(Some(100.0)),
        property_manager: Set(Some("estie PM".into())),
        pml_assessment_company: Set(Some("estie調査会社".into())),
        j_reit_corporation_id: Set(j_reit_corporation_id.clone()),
        j_reit_appraisal_id: Set(Some(j_reit_appraisal_1_id)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id,
        )),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_2".into()),
        j_reit_building_id: Set("test_id_1".into()),
        leasable_units: Set(Some(60)),
        land_ownership_ratio: Set(Some(50.0)),
        building_ownership_ratio: Set(Some(33.3)),
        leasable_area: Set(Some(200.0)),
        transaction_date: Set(datetime_utc(2021, 1, 1, 0).date_naive()),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        j_reit_corporation_id: Set(j_reit_corporation_id.clone()),
        j_reit_appraisal_id: Set(Some(j_reit_appraisal_2_id)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id,
        )),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_3".into()),
        j_reit_building_id: Set("test_id_1".into()),
        leasable_units: Set(Some(0)),
        land_ownership_type: Set(Some("所有権".into())),
        land_ownership_ratio: Set(Some(33.3)),
        leasable_area: Set(Some(500.0)),
        transaction_date: Set(datetime_utc(2023, 1, 1, 0).date_naive()),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        j_reit_corporation_id: Set(j_reit_corporation_id.clone()),
        j_reit_appraisal_id: Set(Some(j_reit_appraisal_3_id)),
        snowflake_deleted: Set(0),
        is_bulk: Set(1),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id,
        )),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn insert_test_prefecture(db: &DatabaseConnection) -> i64 {
    let mut rng = rng();
    let id: i64 = rng.random(); // 乱数生成
    prefectures::ActiveModel {
        id: Set(id),
        name: Set("test default".to_string()),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test prefecture");
    id
}

#[allow(dead_code)]
pub async fn insert_test_designated_city(db: &DatabaseConnection, prefecture_id: i64) -> i64 {
    let mut rng = rng();
    let id: i64 = rng.random(); // 乱数生成
    designated_cities::ActiveModel {
        id: Set(id),
        name: Set("test default".to_string()),
        prefecture_id: Set(prefecture_id),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test designated city");
    id
}

#[allow(dead_code)]
pub async fn insert_test_ward(db: &DatabaseConnection, prefecture_id: i64) -> i64 {
    let mut rng = rng();
    let id: i64 = rng.random(); // 乱数生成
    wards::ActiveModel {
        id: Set(id),
        name: Set("test default".to_string()),
        prefecture_id: Set(prefecture_id),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test ward");
    id
}

#[allow(dead_code)]
pub async fn insert_test_designated_city_ward(
    db: &DatabaseConnection,
    designated_city_id: i64,
    ward_id: i64,
) -> i64 {
    let mut rng = rng();
    let id: i64 = rng.random(); // 乱数生成
    designated_city_wards::ActiveModel {
        id: Set(id),
        designated_city_id: Set(designated_city_id),
        ward_id: Set(ward_id),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test designated city ward");
    id
}

#[allow(dead_code)]
pub async fn insert_test_city(db: &DatabaseConnection, ward_id: i64) -> i64 {
    let mut rng = rng();
    let id: i64 = rng.random(); // 乱数生成
    cities::ActiveModel {
        id: Set(id),
        name: Set(Some("test default".to_string())),
        ward_id: Set(ward_id),
        latitude: Set(35.0),
        longitude: Set(139.0),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test city");
    id
}

#[allow(dead_code)]
pub async fn insert_test_appraisal(db: &DatabaseConnection, id: String) -> String {
    j_reit_appraisals::ActiveModel {
        id: Set(id.clone()),
        appraisal_price: Set(Some(1_000_000)),
        appraisal_date: Set(Some(naive_date(2021, 1, 1))),
        appraisal_company: Set(Some("test company default".into())),
        cap_rate: Set(Some(5.0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert test appraisal");
    id
}

#[allow(dead_code)]
pub async fn insert_test_mizuho_id_mapping(
    db: &DatabaseConnection,
    j_reit_building_id: String,
    j_reit_corporation_id: String,
) -> String {
    let mut rng = rng();
    let idx: i64 = rng.random(); // 乱数生成
    let id: String = format!("test_mizuho_id_mapping_id_{}", idx);

    j_reit_mizuho_id_mappings::ActiveModel {
        j_reit_mizuho_building_id: Set(id.clone()),
        j_reit_building_id: Set(j_reit_building_id),
        j_reit_corporation_id: Set(j_reit_corporation_id),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await
    .expect("Failed to insert test mizuho_id_mappings");
    id
}

#[allow(dead_code)]
pub fn get_combined_transaction_id(
    j_reit_building_id: &str,
    j_reit_corporation_id: &str,
) -> String {
    format!("{}-{}", j_reit_building_id, j_reit_corporation_id)
}

#[allow(dead_code)]
pub fn make_roles_j_reit_premium(premium: bool) -> Roles {
    let mut roles = Roles::default();
    let data_roles = DataRoles {
        j_reit_premium: premium,
        ..Default::default()
    };
    roles.data = MessageField::from_option(Some(data_roles));
    roles
}
