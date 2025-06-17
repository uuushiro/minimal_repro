mod common;

use ::common::types::TransactionCategory;
use common::*;
mod utils;
use utils::*;

use async_graphql::{value, Request, Result, Variables};
// use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::json;
use sql_entities::{
    j_reit_appraisals, j_reit_buildings, j_reit_mizuho_appraisal_histories,
    j_reit_mizuho_cap_rate_histories, j_reit_mizuho_financials, j_reit_transactions,
};

// 名称での検索の確認

#[tokio::test]
// 指定した文字列を含む名称の物件が取得できることの確認(1)
async fn test_search_j_reit_buildings_search_condition_name1() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "name": "新"
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定した文字列を含む名称の物件が取得できることの確認(2)
async fn test_search_j_reit_buildings_search_condition_name2() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "name": "丸の内"
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 名称を指定しない場合全件取得できることの確認
async fn test_search_j_reit_buildings_search_condition_name_is_null() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "name": null
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// j_reit_corporation_id での検索の確認

#[tokio::test]
// 指定した j_reit_corporation_id に紐づくデータが取得できることの確認
async fn test_search_j_reit_buildings_search_condition_j_reit_corporation_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let InsertTestDataStruct {
        j_reit_corporation_id_1,
        ..
    } = insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "jReitCorporationIds": [j_reit_corporation_id_1]
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// j_reit_corporation_idを指定しない場合全件取得できることの確認
async fn test_search_j_reit_buildings_search_condition_j_reit_corporation_ids_are_blanked(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {}
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 所在地（自治体）での検索の確認
// テストデータ再掲
// ビル1:東京都/港区/六本木
// ビル2:東京都/港区/赤坂
// ビル3:東京都/千代田区/丸の内
// ビル4:神奈川県/横浜市西区/みなとみらい

#[tokio::test]
// 指定された地域区分のどれかに該当するデータを取得できる（1）
async fn test_search_j_reit_buildings_search_condition_location1() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let InsertTestDataStruct {
        prefecture_id_kanagawa,
        city_id_roppongi,
        ..
    } = insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            // 神奈川県or六本木
            "location": {
                "prefectureIds": [prefecture_id_kanagawa],
                "cityIds": [city_id_roppongi]
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// IDが空などの場合は未指定とみなされて全件取得
async fn test_search_j_reit_buildings_search_condition_location_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();

    let variables = json!({
        "searchCondition": {
            "location": {
                "prefectureIds": [],
                "cityIds": []
            }
        }
    });

    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));

    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 緯度軽度での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合(1)
async fn test_search_j_reit_buildings_search_condition_lati_and_long1() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "latitudeAndLongitude": {
                "south": 35.015,
                "north": 35.025,
                "west": 139.015,
                "east": 139.025
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限と下限を両方指定した場合(2)
async fn test_search_j_reit_buildings_search_condition_lati_and_long2() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "latitudeAndLongitude": {
                "south": 35.005,
                "north": 35.035,
                "west": 139.005,
                "east": 139.035
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 3,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_lati_and_long_is_null() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "latitudeAndLongitude": null
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 竣工年での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_completed_year_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "completedYear": {
                "min": 1990,
                "max": 2010
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_completed_year_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "completedYear": {
                "min": 1990
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_completed_year_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "completedYear": {
                "max": 2010
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_completed_year_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "completedYear": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 敷地面積での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_land_area_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "landArea": {
                "min": 500,
                "max": 800
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_land_area_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "landArea": {
                "min": 500
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_land_area_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "landArea": {
                "max": 800
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_land_area_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "landArea": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 延床面積での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_gross_floor_area_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "grossFloorArea": {
                "min": 5000,
                "max": 8000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_gross_floor_area_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "grossFloorArea": {
                "min": 5000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_gross_floor_area_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "grossFloorArea": {
                "max": 8000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_gross_floor_area_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "grossFloorArea": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 賃貸可能面積での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_net_leasable_area_total_both() -> Result<()>
{
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "totalLeasableArea": {
                "min": 7900,
                "max": 7998,
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_net_leasable_area_total_set_min(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "totalLeasableArea": {
                "min": 7998
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_net_leasable_area_total_set_max(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "totalLeasableArea": {
                "max": 7998
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_net_leasable_area_total_is_blank(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "totalLeasableArea": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 複数の企業がそれぞれの賃貸可能面積をもつ場合
#[tokio::test]
async fn test_search_j_reit_buildings_with_different_leasable_areas() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;

    let InsertTestDataStruct {
        j_reit_corporation_id_1,
        j_reit_corporation_id_2,
        city_id_roppongi,
        ..
    } = insert_test_data(&db).await?;

    j_reit_buildings::ActiveModel {
        id: Set("building01".into()),
        name: Set("大阪ビル1".into()),
        is_office: Set(1),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        snowflake_deleted: Set(0),
        city_id: Set(city_id_roppongi),
        latitude: Set(35.0),
        longitude: Set(139.0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("new_transaction_id_1_1".into()),
        j_reit_building_id: Set("building01".into()),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        combined_transaction_id: Set(get_combined_transaction_id(
            "building01",
            &j_reit_corporation_id_1,
        )),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(datetime_utc(2022, 1, 1, 0).date_naive()),
        leasable_area: Set(Some(1000.0)),
        total_leasable_area: Set(Some(1000.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("new_transaction_id_1_2".into()),
        j_reit_building_id: Set("building01".into()),
        j_reit_corporation_id: Set(j_reit_corporation_id_2.clone()),
        combined_transaction_id: Set(get_combined_transaction_id(
            "building01",
            &j_reit_corporation_id_2,
        )),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        transaction_date: Set(datetime_utc(2023, 1, 1, 0).date_naive()),
        leasable_area: Set(Some(1500.0)),
        total_leasable_area: Set(Some(1500.0)),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let request: Request = r#"
        query SearchBuildings($searchCondition: GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                    jReitCorporation {
                        id
                    }
                }
            }
        }
    "#
    .into();

    let variables = json!({
        "searchCondition": {
            "name": "大阪ビル1",
            "totalLeasableArea": {
                "min": 1200
            }
        }
    });

    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    {
                        "id": "building01",
                        "jReitCorporation": {
                            "id": "test_company_id_2",
                        },
                    }
                ]
            }
        })
    );

    Ok(())
}

// 取得日での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_acquisition_date_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionDate": {
                "min": "2000-01-01",
                "max": "2004-01-01"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_acquisition_date_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionDate": {
                "min": "2011-01-01"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_acquisition_date_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionDate": {
                "max": "2015-01-01"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_acquisition_date_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionDate": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 取得価格での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_acquisition_price_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionPrice": {
                "min": 300_000_000,
                "max": 700_000_000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_acquisition_price_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionPrice": {
                "min": 300_000_000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_acquisition_price_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "acquisitionPrice": {
                "max": 700_000_000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_acquisition_price_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {}
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 鑑定価格での検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_appraised_price_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "appraisedPrice": {
                "min": 700_000_000,
                "max": 1_500_000_000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_appraised_price_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "appraisedPrice": {
                "min": 700_000_000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_appraised_price_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "appraisedPrice": {
                "max": 1_500_000_000
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_appraised_price_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "appraisedPrice": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 取得時キャップレートでの検索の確認

#[tokio::test]
#[ignore]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_initial_cap_rate_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "initialCapRate": {
                "min": 6.5,
                "max": 7.5
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
#[ignore]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_initial_cap_rate_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "initialCapRate": {
                "min": 6.5
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
#[ignore]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_initial_cap_rate_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "initialCapRate": {
                "max": 7.5
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 初回取引のないビルの取得時CRは検索できない
async fn test_search_j_reit_buildings_search_condition_initial_cap_rate_no_initial_transaction(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;

    let InsertTestDataStruct {
        j_reit_corporation_id_1,
        city_id_roppongi,
        ..
    } = insert_test_data(&db).await?;

    // 初回取引がないビルを作成
    j_reit_buildings::ActiveModel {
        id: Set("test_id_5".into()),
        name: Set("古いビル".into()),
        is_office: Set(1),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        city_id: Set(city_id_roppongi),
        latitude: Set(35.0),
        longitude: Set(139.0),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_5_1".into()),
        cap_rate: Set(Some(7.0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_5_1".into()),
        j_reit_building_id: Set("test_id_5".into()),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_5",
            &j_reit_corporation_id_1,
        )),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_5_1".into())),
        transaction_date: Set(datetime_utc(2024, 1, 1, 0).date_naive()),
        transaction_category: Set(TransactionCategory::FullTransfer as i8),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "initialCapRate": {
                "min": 6.5,
                "max": 7.5
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_initial_cap_rate_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "initialCapRate": {}
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 最新cap_rateでの検索の確認

#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_cap_rate_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "capRate": {
                "min": 4.0,
                "max": 6.0
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_1" }
                ]
            }
        })
    );

    Ok(())
}

// キャップレートでの検索の確認
#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_cap_rate_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {
            "capRate": {
                "min": 4.0
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_cap_rate_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {
            "capRate": {
                "max": 6.0
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_cap_rate_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {}
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 譲渡日での検索の確認
#[tokio::test]
// 上限と下限を両方指定した場合
async fn test_search_j_reit_buildings_search_condition_transfer_date_both() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {
            "transferDate": {
                "min": "2022-01-01",
                "max": "2023-01-01"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_transfer_date_set_max() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {
            "transferDate": {
                "max": "2023-01-01"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 下限のみ指定した場合
async fn test_search_j_reit_buildings_search_condition_transfer_date_set_min() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {
            "transferDate": {
                "min": "2022-01-01"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定しない場合（nullも含めて全件取得）
async fn test_search_j_reit_buildings_search_condition_transfer_date_is_blank() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
            searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                totalCount
                jReitBuildings {
                    id
                }
            }
        }
        "#
    .into();
    let variables = json!({
        "searchCondition": {
            "transferDate": null
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 条件を組み合わせた場合AND検索になることの確認
async fn test_search_j_reit_buildings_search_condition_combined() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "grossFloorArea": {
                "min": 6_000, // 1,2が該当
            },
            "appraisedPrice": {
                "max": 1_500_000_000 // 1,3が該当
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_1" }
                ]
            }
        })
    );

    Ok(())
}

// asset_type での検索の確認
// NOTE
// 設定されたアセットタイプは以下の通り
// ビル1:office、ビル2:residential、ビル3:hotel、ビル4:logistic

#[tokio::test]
// 指定した asset_type のデータが取得できることの確認(1)
async fn test_search_j_reit_buildings_search_condition_asset_type_1() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "assetType": {
                "isOffice": true,
                "isResidential": true,
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 指定した asset_type のデータが取得できることの確認(2)
async fn test_search_j_reit_buildings_search_condition_asset_type_2() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "assetType": {
                "isOffice": true,
                "isResidential": false,
                "isLogistic": true
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// asset_typeの条件を指定しない場合全件取得できることの確認
async fn test_search_j_reit_buildings_search_condition_asset_type_not_specified() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {}
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// asset_typeに実質的に無効な条件（falseのみ）を指定した場合全件取得できることの確認
async fn test_search_j_reit_buildings_search_condition_asset_type_all_false() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "assetType": {
                "isOffice": false,
                "isResidential": false,
                "isHotel": false,
                "isLogistic": false,
                "isRetail": false,
                "isHealthCare": false,
                "isOther": false
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 譲渡済/保有中の条件での検索の確認
// NOTE
// ビル1,2: 保有中、ビル3,4: 譲渡済

#[tokio::test]
#[ignore]
// 保有中のビルが検索できることの確認
async fn test_search_j_reit_buildings_search_condition_is_transferred_false() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "isTransferred": false
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 3,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
#[ignore]
// 譲渡済のビルが検索できることの確認
async fn test_search_j_reit_buildings_search_condition_is_transferred_true() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "isTransferred": true
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 1,
                "jReitBuildings": [
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 保有中/譲渡済の条件を指定しない場合全件取得できることの確認
async fn test_search_j_reit_buildings_search_condition_is_transferred_not_specified() -> Result<()>
{
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {}
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 上場中/上場廃止の条件での検索の確認
// NOTE
// ビル1,3: 上場中、ビル2,4: 上場廃止

#[tokio::test]
// 上場中のビルが検索できることの確認
async fn test_search_j_reit_buildings_search_condition_is_delisted_false() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "isDelisted": false
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上場廃止のビルが検索できることの確認
async fn test_search_j_reit_buildings_search_condition_is_delisted_true() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "isDelisted": true
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 上場中/上場廃止の条件を指定しない場合全件取得できることの確認
async fn test_search_j_reit_buildings_search_condition_is_delisted_null() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "isDelisted": null
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// ページネーションができることの確認

#[tokio::test]
// 基本的なページネーションの確認
async fn test_search_j_reit_buildings_pagination_basic() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    // ページネーション以外の条件は入れないので全件ヒットする
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "pagination": {
                "offset": 1,
                "limit": 2
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// offsetが件数を超える場合は配列が空になる
async fn test_search_j_reit_buildings_pagination_offset_exceeds_count() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "pagination": {
                "offset": 5,
                "limit": 2
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": []
            }
        })
    );

    Ok(())
}

#[tokio::test]
// offset+limitが件数を超える場合はデータがあるところまでで配列が打ち切られる（limitより少ない件数になる）
async fn test_search_j_reit_buildings_pagination_partial_results() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "pagination": {
                "offset": 3,
                "limit": 2
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_4" }
                ]
            }
        })
    );

    Ok(())
}

// 適切な権限がない場合は該当件数が0となることの確認

#[tokio::test]
async fn test_search_j_reit_buildings_no_permission() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {
            "appraisedPrice": {
                "min": 1
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(false)); // 権限なし
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 0,
                "jReitBuildings": []
            }
        })
    );

    Ok(())
}

// ソート条件の適用の確認

#[tokio::test]
// 取得価格の昇順
async fn test_search_j_reit_buildings_sort_acquisition_price_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_PRICE",
                "order": "ASC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_3" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_1" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 取得価格の昇順
async fn test_search_j_reit_buildings_sort_acquisition_price_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_PRICE",
                "order": "DESC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 取得価格の昇順
async fn test_search_j_reit_buildings_sort_acquisition_date_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_DATE",
                "order": "ASC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_1" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 取得価格の昇順
async fn test_search_j_reit_buildings_sort_acquisition_date_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_DATE",
                "order": "DESC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 延床面積の降順
async fn test_search_j_reit_buildings_sort_gross_floor_area_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "GROSS_FLOOR_AREA",
                "order": "DESC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 延床面積の昇順
async fn test_search_j_reit_buildings_sort_gross_floor_area_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "GROSS_FLOOR_AREA",
                "order": "ASC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_3" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_1" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 投資法人名の昇順
async fn test_search_j_reit_buildings_sort_j_reit_corporation_name_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "J_REIT_CORPORATION_NAME",
                "order": "ASC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" },
                    { "id" : "test_id_2" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 投資法人名の降順
async fn test_search_j_reit_buildings_sort_j_reit_corporation_name_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "J_REIT_CORPORATION_NAME",
                "order": "DESC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_2" },
                    { "id" : "test_id_4" },
                    { "id" : "test_id_1" },
                    { "id" : "test_id_3" }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 取得価格の昇順
async fn test_search_j_reit_buildings_sort_initial_net_leasable_area_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "INITIAL_LEASABLE_AREA",
                "order": "ASC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_3" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_1" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 取得価格の昇順
async fn test_search_j_reit_buildings_sort_initial_net_leasable_area_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
    query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!, $sortAndPagination: GraphQLJReitBuildingSortAndPaginateCondition) {
        searchJReitBuildingsPerCorporation(searchCondition: $searchCondition, sortAndPagination: $sortAndPagination) {
            totalCount
            jReitBuildings {
                id
            }
        }
    }
    "#
    .into();
    let variables = json!({
        "searchCondition": {},
        "sortAndPagination": {
            "sort": {
                "key": "ACQUISITION_PRICE",
                "order": "DESC"
            }
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 4,
                "jReitBuildings": [
                    { "id" : "test_id_1" },
                    { "id" : "test_id_2" },
                    { "id" : "test_id_3" },
                    { "id" : "test_id_4" } // None は最後
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// jReitCorporationフィールドが正しく取得できることの確認
async fn test_search_j_reit_buildings_j_reit_corporation_field() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let InsertTestDataStruct {
        j_reit_corporation_id_1,
        ..
    } = insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                        jReitCorporation {
                            id
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "jReitCorporationIds": [j_reit_corporation_id_1]
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    {
                        "id": "test_id_1",
                        "jReitCorporation": {
                            "id": j_reit_corporation_id_1
                        }
                    },
                    {
                        "id": "test_id_3",
                        "jReitCorporation": {
                            "id": j_reit_corporation_id_1
                        }
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
// 同じj_reit_buildings, j_reit_corporationsに複数のj_reit_transactionsが存在する場合でも1つのレコードしか返らないことを確認
async fn test_search_j_reit_buildings_duplicate_transactions() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    let InsertTestDataStruct {
        j_reit_corporation_id_1,
        ..
    } = insert_test_data(&db).await?;

    let request: Request = r#"
            query searchJReitBuildings($searchCondition:GraphQLSearchJReitBuildingCondition!) {
                searchJReitBuildingsPerCorporation(searchCondition: $searchCondition) {
                    totalCount
                    jReitBuildings {
                        id
                        transactions {
                            id
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "searchCondition": {
            "jReitCorporationIds": [j_reit_corporation_id_1]
        }
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchJReitBuildingsPerCorporation": {
                "totalCount": 2,
                "jReitBuildings": [
                    {
                        "id": "test_id_1",
                        "transactions": [
                            { "id": "test_transaction_id_1_1" },
                            { "id": "test_transaction_id_1_2" }
                        ]
                    },
                    {
                        "id": "test_id_3",
                        "transactions": [
                            { "id": "test_transaction_id_3_1" },
                            { "id": "test_transaction_id_3_2" }
                        ]
                    }
                ]
            }
        })
    );

    Ok(())
}

struct InsertTestDataStruct {
    j_reit_corporation_id_1: String,
    j_reit_corporation_id_2: String,
    prefecture_id_kanagawa: i64,
    city_id_roppongi: i64,
}

async fn insert_test_data(db: &DatabaseConnection) -> Result<InsertTestDataStruct> {
    setup_basic_test_data(db).await?;

    let j_reit_corporation_id_1 =
        insert_test_j_reit_corporation(db, "A0法人".into(), "test_company_id_1".into(), 0).await;
    let j_reit_corporation_id_2 =
        insert_test_j_reit_corporation(db, "B0法人".into(), "test_company_id_2".into(), 1).await;
    let j_reit_corporation_id_3 =
        insert_test_j_reit_corporation(db, "A1法人".into(), "test_company_id_3".into(), 1).await;

    // let building_id_1 = insert_test_building(&db, None, None, TEST_CITY_ID_MARUNOUCHI).await;
    // let building_id_2 = insert_test_building(&db, None, None, TEST_CITY_ID_MARUNOUCHI).await;
    // let building_id_3 = insert_test_building(&db, None, None, TEST_CITY_ID_MARUNOUCHI).await;
    // let building_id_4 = insert_test_building(&db, None, None, TEST_CITY_ID_MARUNOUCHI).await;

    // 取得価格等について、ビル1>ビル2>ビル3の順に設定し、ビル4はnullにする
    // アセットタイプはビル1:office、ビル2:residential、ビル3:hotel、ビル4:logisticとする
    // また、ビル1,2は保有中、ビル3,4は譲渡済とする
    // ビル1,3は上場中、ビル2,4は上場廃止とする
    // 所在地は以下の通りに設定
    // J-REITビル1:東京都/港区/六本木
    // J-REITビル2:東京都/港区/赤坂
    // J-REITビル3:東京都/千代田区/丸の内
    // J-REITビル4:神奈川県/横浜市西区/みなとみらい

    // 東京都、千代田区、丸の内はテスト用のbasic_dataとして自動で作成されるので、残りの地域区分を作成
    let prefecture_id_kanagawa = insert_test_prefecture(db).await;
    let designated_city_id_yokohama = insert_test_designated_city(db, prefecture_id_kanagawa).await;
    let ward_id_yokohama = insert_test_ward(db, prefecture_id_kanagawa).await;
    insert_test_designated_city_ward(db, designated_city_id_yokohama, ward_id_yokohama).await;
    let city_id_minatomirai = insert_test_city(db, ward_id_yokohama).await;
    let ward_id_minato = insert_test_ward(db, TEST_PREFECTURE_ID_TOKYO).await;
    let city_id_roppongi = insert_test_city(db, ward_id_minato).await;
    let city_id_akasaka = insert_test_city(db, ward_id_minato).await;

    // j-reitビル 1
    j_reit_buildings::ActiveModel {
        id: Set("test_id_1".into()),
        // building_id: Set(Some(building_id_1)),
        // 検索条件の検証用のデータ
        name: Set("日比谷新丸の内ビル".into()),
        completed_year: Set(Some(2020)),
        land: Set(Some(1000.0)),
        gross_floor_area: Set(Some(10_000.0)),
        // appraised_price: Set(Some(2_000_000_000i64)),
        is_office: Set(1),
        city_id: Set(city_id_roppongi),
        latitude: Set(35.03),
        longitude: Set(139.03),
        // is_delisted: Set(0),
        // 以下はnot nullなので設定（確認はしない）
        is_residential: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        // reit: Set("test".into()),
        // mizuho_reit_id: Set("test_mizuho_reit_id".into()),
        // created_at: Set(datetime_utc(2024, 1, 1, 0)),
        // updated_at: Set(datetime_utc(2024, 1, 2, 0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // j-reitビル 2
    j_reit_buildings::ActiveModel {
        id: Set("test_id_2".into()),
        // building_id: Set(Some(building_id_2)),
        // 検索条件の検証用のデータ
        name: Set("新赤坂ビル".into()),
        completed_year: Set(Some(2000)),
        land: Set(Some(600.0)),
        gross_floor_area: Set(Some(6_000.0)),
        // appraised_price: Set(Some(1_000_000_000i64)),
        is_residential: Set(1),
        city_id: Set(city_id_akasaka),
        latitude: Set(35.02),
        longitude: Set(139.02),
        // is_delisted: Set(1),
        // 以下はnot nullなので設定（確認はしない）
        is_office: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        // reit: Set("test".into()),
        // mizuho_reit_id: Set("test_mizuho_reit_id".into()),
        // created_at: Set(datetime_utc(2024, 1, 1, 0)),
        // updated_at: Set(datetime_utc(2024, 1, 2, 0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // j-reitビル 3
    j_reit_buildings::ActiveModel {
        id: Set("test_id_3".into()),
        // building_id: Set(Some(building_id_3)),
        // 検索条件の検証用のデータ
        name: Set("丸の内ビル".into()),
        completed_year: Set(Some(1980)),
        land: Set(Some(300.0)),
        gross_floor_area: Set(Some(3_000.0)),
        // appraised_price: Set(Some(500_000_000i64)),
        is_hotel: Set(1),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        latitude: Set(35.01),
        longitude: Set(139.01),
        // is_delisted: Set(0),
        // 以下はnot nullなので設定（確認はしない）
        is_office: Set(0),
        is_residential: Set(0),
        is_logistic: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        // reit: Set("test".into()),
        // mizuho_reit_id: Set("test_mizuho_reit_id".into()),
        // created_at: Set(datetime_utc(2024, 1, 1, 0)),
        // updated_at: Set(datetime_utc(2024, 1, 2, 0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // j-reitビル 4
    j_reit_buildings::ActiveModel {
        id: Set("test_id_4".into()),
        // building_id: Set(Some(building_id_4)),
        // 検索条件の検証用のデータ
        name: Set("テストビル".into()),
        completed_year: Set(None),
        land: Set(None),
        gross_floor_area: Set(None),
        // appraised_price: Set(None),
        is_logistic: Set(1),
        city_id: Set(city_id_minatomirai),
        latitude: Set(35.0),
        longitude: Set(139.0),
        // is_delisted: Set(1),
        // 以下はnot nullなので設定（確認はしない）
        is_office: Set(0),
        is_residential: Set(0),
        is_hotel: Set(0),
        is_retail: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        // reit: Set("test".into()),
        // mizuho_reit_id: Set("test_mizuho_reit_id".into()),
        // created_at: Set(datetime_utc(2024, 1, 1, 0)),
        // updated_at: Set(datetime_utc(2024, 1, 2, 0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    insert_test_appraisals(db).await?;
    insert_test_transactions(
        db,
        j_reit_corporation_id_1.clone(),
        j_reit_corporation_id_2.clone(),
        j_reit_corporation_id_3.clone(),
    )
    .await?;
    insert_test_financials(
        db,
        j_reit_corporation_id_1.clone(),
        j_reit_corporation_id_2.clone(),
    )
    .await?;

    Ok(InsertTestDataStruct {
        j_reit_corporation_id_1,
        j_reit_corporation_id_2,
        prefecture_id_kanagawa,
        city_id_roppongi,
    })
}

async fn insert_test_transactions(
    db: &DatabaseConnection,
    j_reit_corporation_id_1: String,
    j_reit_corporation_id_2: String,
    j_reit_corporation_id_3: String,
) -> Result<()> {
    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_1_1".into()),
        j_reit_building_id: Set("test_id_1".into()),
        transaction_date: Set(datetime_utc(2020, 6, 1, 0).date_naive()),
        transaction_price: Set(Some(1_000_000_000i64)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        leasable_area: Set(Some(7001.0)),
        total_leasable_area: Set(Some(7001.0)),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_1_1".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id_1,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_1_2".into()),
        j_reit_building_id: Set("test_id_1".into()),
        transaction_date: Set(datetime_utc(2025, 6, 1, 0).date_naive()),
        transaction_price: Set(Some(700_000_000i64)),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        leasable_area: Set(Some(200.0)),
        total_leasable_area: Set(Some(7201.0)),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_1_2".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_1",
            &j_reit_corporation_id_1,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_2_1".into()),
        j_reit_building_id: Set("test_id_2".into()),
        transaction_date: Set(datetime_utc(2010, 6, 1, 0).date_naive()),
        transaction_price: Set(Some(700_000_000i64)),
        transaction_category: Set(TransactionCategory::AdditionalAcquisition as i8),
        leasable_area: Set(Some(8000.0)),
        total_leasable_area: Set(Some(12000.0)),
        j_reit_corporation_id: Set(j_reit_corporation_id_2.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_2_1".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_2",
            &j_reit_corporation_id_2,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_2_2".into()),
        j_reit_building_id: Set("test_id_2".into()),
        transaction_date: Set(datetime_utc(2001, 1, 1, 0).date_naive()),
        transaction_price: Set(Some(500_000_000i64)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        leasable_area: Set(Some(4000.0)),
        total_leasable_area: Set(Some(4000.0)),
        j_reit_corporation_id: Set(j_reit_corporation_id_2.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_2_1".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_2",
            &j_reit_corporation_id_2,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_3_1".into()),
        j_reit_building_id: Set("test_id_3".into()),
        transaction_date: Set(datetime_utc(2014, 12, 31, 0).date_naive()),
        transaction_price: Set(Some(200_000_000i64)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        leasable_area: Set(Some(3999.0)),
        total_leasable_area: Set(Some(3999.0)),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_3_1".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_3",
            &j_reit_corporation_id_1,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_3_2".into()),
        j_reit_building_id: Set("test_id_3".into()),
        transaction_date: Set(datetime_utc(2025, 1, 1, 0).date_naive()),
        transaction_price: Set(Some(200_000_000i64)),
        transaction_category: Set(TransactionCategory::PartialTransfer as i8),
        leasable_area: Set(Some(3999.0)),
        total_leasable_area: Set(Some(7998.0)),
        j_reit_corporation_id: Set(j_reit_corporation_id_1.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_3_2".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_3",
            &j_reit_corporation_id_1,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("test_transaction_id_4".into()),
        j_reit_building_id: Set("test_id_4".into()),
        transaction_date: Set(datetime_utc(2022, 12, 1, 0).date_naive()),
        transaction_price: Set(Some(0)),
        transaction_category: Set(TransactionCategory::FullTransfer as i8),
        j_reit_corporation_id: Set(j_reit_corporation_id_3.clone()),
        j_reit_appraisal_id: Set(Some("test_appraisal_id_4".into())),
        snowflake_deleted: Set(0),
        combined_transaction_id: Set(get_combined_transaction_id(
            "test_id_4",
            &j_reit_corporation_id_3,
        )),
        is_bulk: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

#[allow(dead_code)]
async fn insert_test_financials(
    db: &DatabaseConnection,
    j_reit_corporation_id_1: String,
    j_reit_corporation_id_2: String,
) -> Result<()> {
    // 最新のレコードが検索対象なため、古いデータに引っかからないか確認するためのデータも作成する

    // transactions と紐付ける corporation を合わせること
    let j_reit_mizuho_building_id_1 =
        insert_test_mizuho_id_mapping(db, "test_id_1".into(), j_reit_corporation_id_1.clone())
            .await;
    let j_reit_mizuho_building_id_2 =
        insert_test_mizuho_id_mapping(db, "test_id_2".into(), j_reit_corporation_id_2.clone())
            .await;
    let j_reit_mizuho_building_id_3 =
        insert_test_mizuho_id_mapping(db, "test_id_3".into(), j_reit_corporation_id_1.clone())
            .await;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_financial_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(Some(7.0)),
        appraisal_price: Set(Some(2_000_000_000i64)),
        fiscal_period_start_date: Set(datetime_utc(2020, 4, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2021, 3, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(365),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_financial_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(Some(1.0)),
        appraisal_price: Set(Some(0i64)),
        fiscal_period_start_date: Set(datetime_utc(2010, 4, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2011, 3, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(365),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_financial_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        cap_rate: Set(Some(5.0)),
        appraisal_price: Set(Some(1_000_000_000i64)),
        fiscal_period_start_date: Set(datetime_utc(2010, 4, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2011, 3, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(365),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_financial_id_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        cap_rate: Set(Some(3.0)),
        appraisal_price: Set(Some(500_000_000i64)),
        fiscal_period_start_date: Set(datetime_utc(2020, 4, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2021, 3, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(365),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_mizuho_financials::ActiveModel {
        id: Set("test_financial_id_5".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        cap_rate: Set(Some(10.0)),
        appraisal_price: Set(Some(0i64)),
        fiscal_period_start_date: Set(datetime_utc(2010, 4, 1, 0).date_naive()),
        fiscal_period_end_date: Set(datetime_utc(2011, 3, 31, 0).date_naive()),
        fiscal_period_operating_day: Set(365),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(5.0),
        closing_date: Set(naive_date(2020, 4, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        cap_rate: Set(5.0),
        closing_date: Set(naive_date(2025, 1, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        cap_rate: Set(10.0),
        closing_date: Set(naive_date(2020, 6, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;
    j_reit_mizuho_cap_rate_histories::ActiveModel {
        id: Set("test_cap_rate_history_id_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        cap_rate: Set(1.0),
        closing_date: Set(naive_date(2020, 6, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_1".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        appraisal_price: Set(2_000_000_000i64),
        appraisal_date: Set(naive_date(2020, 6, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;
    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_2".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_1.clone()),
        appraisal_price: Set(500_000_000i64),
        appraisal_date: Set(naive_date(2025, 1, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;
    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_3".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_2.clone()),
        appraisal_price: Set(2_000_000_000i64),
        appraisal_date: Set(naive_date(2020, 6, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;
    j_reit_mizuho_appraisal_histories::ActiveModel {
        id: Set("test_appraisal_history_id_4".into()),
        j_reit_mizuho_building_id: Set(j_reit_mizuho_building_id_3.clone()),
        appraisal_price: Set(1_000_000_000i64),
        appraisal_date: Set(naive_date(2020, 6, 1)),
        snowflake_deleted: Set(0),
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn insert_test_appraisals(db: &DatabaseConnection) -> Result<()> {
    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_1_1".into()),
        cap_rate: Set(Some(8.0)),
        appraisal_price: Set(Some(2_000_000_000i64)),
        appraisal_date: Set(Some(datetime_utc(2020, 6, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_1_2".into()),
        cap_rate: Set(Some(7.0)),
        appraisal_price: Set(Some(2_000_000_001i64)),
        appraisal_date: Set(Some(datetime_utc(2024, 6, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_2_1".into()),
        cap_rate: Set(Some(7.0)),
        appraisal_price: Set(Some(1_000_000_000i64)),
        appraisal_date: Set(Some(datetime_utc(2020, 6, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_2_2".into()),
        cap_rate: Set(Some(7.0)),
        appraisal_price: Set(Some(0i64)),
        appraisal_date: Set(Some(datetime_utc(2025, 1, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_3_1".into()),
        cap_rate: Set(Some(6.0)),
        appraisal_price: Set(Some(500_000_000i64)),
        appraisal_date: Set(Some(datetime_utc(2020, 6, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_3_2".into()),
        cap_rate: Set(Some(6.0)),
        appraisal_price: Set(Some(0i64)),
        appraisal_date: Set(Some(datetime_utc(2024, 1, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    j_reit_appraisals::ActiveModel {
        id: Set("test_appraisal_id_4".into()),
        cap_rate: Set(None),
        appraisal_price: Set(Some(100_000_000i64)),
        appraisal_date: Set(Some(datetime_utc(2020, 6, 1, 0).date_naive())),
        appraisal_company: Set(Some("test_appraisal_company".into())),
        snowflake_deleted: Set(0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}
