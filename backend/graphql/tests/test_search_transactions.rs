mod common;

use ::common::types::TransactionCategory;
use common::*;
mod utils;

use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::json;
use sql_entities::{
    cities, j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_transactions,
    prefectures, wards,
};
use utils::naive_date;

// filter
#[tokio::test]
async fn test_search_transactions_filter_by_city_id() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
            searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                nodes {
                    id
                }
            }
        }
    "#
    .into();
    let variables = json!({
        "input": {
            "cityIds": [TEST_CITY_ID_MARUNOUCHI]
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                    },
                    {
                        "id": "transaction1_1",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_by_location() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let mut prefecture = prefectures::ActiveModel {
        id: Set(8),
        name: Set("兵庫県".to_string()),
        snowflake_deleted: Set(0),
    };
    prefecture.clone().insert(&db).await?;

    prefecture.id = Set(9);
    prefecture.name = Set("奈良県".to_string());
    prefecture.clone().insert(&db).await?;

    let mut wards = wards::ActiveModel {
        id: Set(81),
        name: Set("西宮市".to_string()),
        snowflake_deleted: Set(0),
        prefecture_id: Set(8),
        ..Default::default()
    };
    wards.clone().insert(&db).await?;

    wards.id = Set(82);
    wards.name = Set("西宮市".to_string());
    wards.prefecture_id = Set(8);
    wards.clone().insert(&db).await?;

    wards.id = Set(91);
    wards.name = Set("奈良市".to_string());
    wards.prefecture_id = Set(9);
    wards.clone().insert(&db).await?;

    let mut cities = cities::ActiveModel {
        id: Set(811),
        name: Set(Some("北口".to_string())),
        snowflake_deleted: Set(0),
        ward_id: Set(81),
        latitude: Set(34.710556),
        longitude: Set(135.386944),
    };
    cities.clone().insert(&db).await?;

    cities.id = Set(812);
    cities.name = Set(Some("南口".to_string()));
    cities.ward_id = Set(81);
    cities.clone().insert(&db).await?;

    cities.id = Set(821);
    cities.name = Set(Some("枡井".to_string()));
    cities.ward_id = Set(82);
    cities.clone().insert(&db).await?;

    cities.id = Set(911);
    cities.name = Set(Some("大和西大寺".to_string()));
    cities.ward_id = Set(91);
    cities.insert(&db).await?;

    let mut j_reit_building = j_reit_buildings::ActiveModel {
        id: Set("811".to_string()),
        name: Set("ビル811".to_string()),
        city_id: Set(811),
        is_office: Set(1),
        is_retail: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_residential: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        latitude: Set(35.6804),
        longitude: Set(139.761),
        snowflake_deleted: Set(0),
        completed_year: Set(Some(2020)),
        gross_floor_area: Set(Some(500.0)),
        ..Default::default()
    };
    j_reit_building.clone().insert(&db).await?;

    j_reit_building.id = Set("812".to_string());
    j_reit_building.name = Set("ビル812".to_string());
    j_reit_building.city_id = Set(812);
    j_reit_building.clone().insert(&db).await?;

    j_reit_building.id = Set("821".to_string());
    j_reit_building.name = Set("ビル821".to_string());
    j_reit_building.city_id = Set(821);
    j_reit_building.clone().insert(&db).await?;

    j_reit_building.id = Set("911".to_string());
    j_reit_building.name = Set("ビル911".to_string());
    j_reit_building.city_id = Set(911);
    j_reit_building.clone().insert(&db).await?;

    let mut j_reit_transaction = j_reit_transactions::ActiveModel {
        id: Set("transaction811".to_string()),
        j_reit_building_id: Set("811".to_string()),
        j_reit_corporation_id: Set("corporation1".to_string()),
        combined_transaction_id: Set("combined_transaction_id_811".to_string()),
        transaction_date: Set(naive_date(2025, 1, 1)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        is_bulk: Set(0),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    j_reit_transaction.clone().insert(&db).await?;

    j_reit_transaction.id = Set("transaction812".to_string());
    j_reit_transaction.j_reit_building_id = Set("812".to_string());
    j_reit_transaction.clone().insert(&db).await?;

    j_reit_transaction.id = Set("transaction821".to_string());
    j_reit_transaction.j_reit_building_id = Set("821".to_string());
    j_reit_transaction.clone().insert(&db).await?;

    j_reit_transaction.id = Set("transaction911".to_string());
    j_reit_transaction.j_reit_building_id = Set("911".to_string());
    j_reit_transaction.clone().insert(&db).await?;

    let request: Request = r#"
        query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
            searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                nodes {
                    id
                }
            }
        }
    "#
    .into();
    let variables = json!({
        "input": {
            "location": {
                "cityIds": [
                    811,
                ],
                "wardIds": [
                    82
                ],
                "prefectureIds": [
                    9
                ]
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction811",
                    },
                    {
                        "id": "transaction821",
                    },
                    {
                        "id": "transaction911",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_latitude_and_longitude() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
        query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
            searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                nodes {
                    id
                }
            }
        }
    "#
    .into();
    let variables = json!({
        "input": {
            "latitudeAndLongitude": {
                "north": 36.0,
                "south": 35.0,
                "east": 140.0,
                "west": 139.0
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                    },
                    {
                        "id": "transaction1_1",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_transaction_date() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionDate
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "transactionDate": {
                "min": "2025-01-01",
                "max": "2025-02-28"
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                        "transactionDate": "2025-01-01",
                    },
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_completion_year() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        building {
                            buildingSpec {
                                completedYear
                            }
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "completionYear": {
                "min": 2020,
                "max": 2025
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                        "building": {
                            "buildingSpec": {
                                "completedYear": 2020
                            }
                        }
                    },
                    {
                        "id": "transaction1_1",
                        "building": {
                            "buildingSpec": {
                                "completedYear": 2020
                            }
                        }
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_asset_type() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!) {
                searchTransactions(input: $input) {
                    nodes {
                        id
                        building {
                            assetType {
                                isOffice
                                isRetail
                                isHotel
                                isLogistic
                                isResidential
                                isHealthCare
                                isOther
                            }
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "assetType": {
                "isOffice": false,
                "isRetail": false,
                "isHotel": false,
                "isLogistic": false,
                "isResidential": true,
                "isHealthCare": false,
                "isOther": false
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                        "building": {
                            "assetType": {
                                "isOffice": false,
                                "isRetail": false,
                                "isHotel": false,
                                "isLogistic": false,
                                "isResidential": true,
                                "isHealthCare": false,
                                "isOther": false
                            }
                        }
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_press_release_date() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        pressReleaseDate
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "pressReleaseDate": {
                "min": "2025-01-01",
                "max": "2025-02-28"
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                        "pressReleaseDate": "2025-01-01",
                    },
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_transaction_category() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;
    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionCategory
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "transactionCategories": [
                "PARTIAL_TRANSFER",
                "FULL_TRANSFER"
            ]
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                        "transactionCategory": "PARTIAL_TRANSFER"
                    },
                    {
                        "id": "transaction2_1",
                        "transactionCategory": "FULL_TRANSFER"
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_gross_floor_area() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        building {
                            buildingSpec {
                                grossFloorArea
                            }
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "grossFloorArea": {
                "min": 1000,
                "max": 5000
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                        "building": {
                            "buildingSpec": {
                                "grossFloorArea": "2000.00"
                            }
                        }
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_include_bulk_true() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        isBulk
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "includeBulk": true
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                        "isBulk": true
                    },
                    {
                        "id": "transaction1_1",
                        "isBulk": false
                    },
                    {
                        "id": "transaction2_1",
                        "isBulk": true
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_include_bulk_false() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        isBulk
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "includeBulk": false
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                        "isBulk": false
                    },
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_appraisal_price() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        appraisal {
                            appraisalPrice
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "appraisalPrice": {
                "min": 1500000000u32,
                "max": 2500000000u32
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                        "appraisal": {
                            "appraisalPrice": 2000000000
                        }
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_appraisal_cap_rate() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    // キャップレートの範囲で検索するテスト
    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        appraisal {
                            capRate
                        }
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "appraisalCapRate": {
                "min": 4.5,
                "max": 6.0
            }
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                        "appraisal": {
                            "capRate": 5.0
                        }
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_j_reit_corporation_id() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {
            "jReitCorporationIds": ["corporation1"]
        },
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2"
                    },
                    {
                        "id": "transaction1_1"
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_include_is_delisted() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    j_reit_corporations::ActiveModel {
        id: Set("delisted_corporation".to_string()),
        name: Set("上場廃止法人".to_string()),
        is_delisted: Set(1),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_corporations::ActiveModel {
        id: Set("listed_corporation".to_string()),
        name: Set("上場中法人".to_string()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("delisted_transaction".to_string()),
        j_reit_building_id: Set("building1".to_string()),
        j_reit_corporation_id: Set("delisted_corporation".to_string()),
        combined_transaction_id: Set("building1-delisted_corporation".to_string()),
        transaction_date: Set(naive_date(2090, 1, 1)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        press_release_date: Set(Some(naive_date(2090, 1, 1))),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    j_reit_transactions::ActiveModel {
        id: Set("listed_transaction".to_string()),
        j_reit_building_id: Set("building1".to_string()),
        j_reit_corporation_id: Set("listed_corporation".to_string()),
        combined_transaction_id: Set("building1-listed_corporation".to_string()),
        transaction_date: Set(naive_date(2090, 1, 1)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        press_release_date: Set(Some(naive_date(2090, 1, 1))),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let request_str = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#;

    // 上場廃止を含める
    let variables = json!({
        "input": {
            "transactionDate": {
                "min": "2090-01-01",
                "max": "2090-01-01"
            },
            "includeDelisted": true
        },
    });
    let request: Request = request_str.into();
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "delisted_transaction"
                    },
                    {
                        "id": "listed_transaction"
                    }
                ]
            }
        })
    );

    // 上場廃止を含めない
    let variables = json!({
        "input": {
            "transactionDate": {
                "min": "2090-01-01",
                "max": "2090-01-01"
            },
            "includeDelisted": false
        },
    });
    let request: Request = request_str.into();
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "listed_transaction"
                    }
                ]
            }
        })
    );

    // デフォルトも含めない
    let variables = json!({
        "input": {
            "transactionDate": {
                "min": "2090-01-01",
                "max": "2090-01-01"
            },
        },
    });
    let request: Request = request_str.into();
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "listed_transaction"
                    }
                ]
            }
        })
    );
    Ok(())
}

// order
#[tokio::test]
async fn test_search_transactions_sort_by_transaction_date_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionDate
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "TRANSACTION_DATE",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                        "transactionDate": "2024-12-01",
                    },
                    {
                        "id": "transaction1_1",
                        "transactionDate": "2025-01-01",
                    },
                    {
                        "id": "transaction1_2",
                        "transactionDate": "2025-05-15",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_transaction_date_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionDate
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "TRANSACTION_DATE",
                "order": "DESC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                        "transactionDate": "2025-05-15",
                    },
                    {
                        "id": "transaction1_1",
                        "transactionDate": "2025-01-01",
                    },
                    {
                        "id": "transaction2_1",
                        "transactionDate": "2024-12-01",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_transaction_category_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "TRANSACTION_CATEGORY",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction1_2",
                    },
                    {
                        "id": "transaction2_1",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_transaction_category_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "TRANSACTION_CATEGORY",
                "order": "DESC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_2",
                    },
                    {
                        "id": "transaction1_1",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_transaction_price_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionPrice
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "TRANSACTION_PRICE",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_2",
                        "transactionPrice": 1000000000
                    },
                    {
                        "id": "transaction2_1",
                        "transactionPrice": 2000000000
                    },
                    {
                        "id": "transaction1_1",
                        "transactionPrice": 3000000000u32
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_transaction_price_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionPrice
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "TRANSACTION_PRICE",
                "order": "DESC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                        "transactionPrice": 3000000000u32
                    },
                    {
                        "id": "transaction2_1",
                        "transactionPrice": 2000000000
                    },
                    {
                        "id": "transaction1_2",
                        "transactionPrice": 1000000000
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_press_release_date_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "PRESS_RELEASE_DATE",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction1_2",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_press_release_date_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "PRESS_RELEASE_DATE",
                "order": "DESC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_2",
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_appraisal_price_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "APPRAISAL_PRICE",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction1_2",
                    }
                ]
            }
        })
    );
    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_appraisal_price_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "APPRAISAL_PRICE",
                "order": "DESC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_2",
                    },
                ]
            }
        })
    );
    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_appraisal_cap_rate_asc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "APPRAISAL_CAP_RATE",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_2",
                    }
                ]
            }
        })
    );
    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_appraisal_cap_rate_desc() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "APPRAISAL_CAP_RATE",
                "order": "DESC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                    },
                    {
                        "id": "transaction1_1",
                    },
                    {
                        "id": "transaction1_2",
                    },
                ]
            }
        })
    );
    Ok(())
}

#[tokio::test]
async fn test_search_transactions_pagination() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                    }
                    pageInfo {
                        totalCount
                        page
                        totalPages
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "pagination": {
                "offset": 2,
                "limit": 2
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                    }
                ],
                "pageInfo": {
                    "totalCount": 3,
                    "page": 1,
                    "totalPages": 2
                }
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_sort_by_apportioned_transaction_price() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request: Request = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionPrice
                        apportionedTransactionPrice
                    }
                }
            }
            "#
    .into();
    let variables = json!({
        "input": {},
        "sortAndPagination": {
            "sort": {
                "key": "APPORTIONED_TRANSACTION_PRICE",
                "order": "ASC"
            }
        }
    });
    let request = request.variables(Variables::from_json(variables));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    assert_eq!(
        response.data,
        value!({
            "searchTransactions": {
                "nodes": [
                    {
                        "id": "transaction2_1",
                        "transactionPrice": 2000000000,
                        "apportionedTransactionPrice": 160000000
                    },
                    {
                        "id": "transaction1_2",
                        "transactionPrice": 1000000000,
                        "apportionedTransactionPrice": 500000000
                    },
                    {
                        "id": "transaction1_1",
                        "transactionPrice": 3000000000i64,
                        "apportionedTransactionPrice": null,
                    }
                ]
            }
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_transaction_price_with_use_apportioned_price(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request_query = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionPrice
                        apportionedTransactionPrice
                    }
                }
            }
            "#;

    {
        let variables = json!({
            "input": {
                "transactionPrice": {
                    "min": 100000000,
                    "max": 160000000
                },
                "includeBulk": true,
                "useApportionedPrice": true
            },
        });
        let request =
            Into::<Request>::into(request_query).variables(Variables::from_json(variables));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);
        assert_eq!(
            response.data,
            value!({
                "searchTransactions": {
                    "nodes": [
                        {
                            "id": "transaction2_1",
                            "transactionPrice": 2000000000,
                            "apportionedTransactionPrice": 160000000
                        }
                    ]
                }
            })
        );
    }

    {
        let variables = json!({
            "input": {
                "transactionPrice": {
                    "min": 500000000,
                    "max": 3000000000i64
                },
                "includeBulk": true,
                "useApportionedPrice": true
            },
        });
        let request =
            Into::<Request>::into(request_query).variables(Variables::from_json(variables));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);
        assert_eq!(
            response.data,
            value!({
                "searchTransactions": {
                    "nodes": [
                        {
                            "id": "transaction1_2",
                            "transactionPrice": 1000000000,
                            "apportionedTransactionPrice": 500000000
                        },
                        {
                            "id": "transaction1_1",
                            "transactionPrice": 3000000000i64,
                            "apportionedTransactionPrice": null
                        }
                    ]
                }
            })
        );
    }

    {
        let variables = json!({
            "input": {
                "transactionPrice": {
                    "min": 500000000,
                    "max": 3000000000i64
                },
                "includeBulk": false,
                "useApportionedPrice": true
            },
        });
        let request =
            Into::<Request>::into(request_query).variables(Variables::from_json(variables));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);
        assert_eq!(
            response.data,
            value!({
                "searchTransactions": {
                    "nodes": [
                        {
                            "id": "transaction1_1",
                            "transactionPrice": 3000000000i64,
                            "apportionedTransactionPrice": null
                        }
                    ]
                }
            })
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_search_transactions_filter_by_transaction_price_without_use_apportioned_price(
) -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;
    insert_test_data(&db).await?;

    let request_query = r#"
            query searchTransactions($input: GraphQLSearchTransactionInput!, $sortAndPagination: GraphQLJReitTransactionSortAndPaginateCondition) {
                searchTransactions(input: $input, sortAndPagination: $sortAndPagination) {
                    nodes {
                        id
                        transactionPrice
                        apportionedTransactionPrice
                    }
                }
            }
            "#;

    {
        let variables = json!({
            "input": {
                "transactionPrice": {
                    "min": 2000000000,
                    "max": 3000000000i64
                },
                "includeBulk": true,
                "useApportionedPrice": false
            },
        });
        let request =
            Into::<Request>::into(request_query).variables(Variables::from_json(variables));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);
        assert_eq!(
            response.data,
            value!({
                "searchTransactions": {
                    "nodes": [
                        {
                            "id": "transaction1_1",
                            "transactionPrice": 3000000000i64,
                            "apportionedTransactionPrice": null
                        },
                        {
                            "id": "transaction2_1",
                            "transactionPrice": 2000000000,
                            "apportionedTransactionPrice": 160000000
                        },
                    ]
                }
            })
        );
    }

    {
        let variables = json!({
            "input": {
                "transactionPrice": {
                    "min": 2000000000,
                    "max": 3000000000i64
                },
                "includeBulk": false,
                "useApportionedPrice": false
            },
        });
        let request =
            Into::<Request>::into(request_query).variables(Variables::from_json(variables));
        let response = schema.execute(request).await;

        // Assert
        assert!(response.is_ok(), "{:?}", response.errors);
        assert_eq!(
            response.data,
            value!({
                "searchTransactions": {
                    "nodes": [
                        {
                            "id": "transaction1_1",
                            "transactionPrice": 3000000000i64,
                            "apportionedTransactionPrice": null
                        },
                    ]
                }
            })
        );
    }

    Ok(())
}

async fn insert_test_data(db: &DatabaseConnection) -> Result<()> {
    // J-REIT法人のデータを挿入
    let corporation1 = j_reit_corporations::ActiveModel {
        id: Set("corporation1".to_string()),
        name: Set("テスト法人1".to_string()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    };
    corporation1.insert(db).await?;

    let corporation2 = j_reit_corporations::ActiveModel {
        id: Set("corporation2".to_string()),
        name: Set("テスト法人2".to_string()),
        is_delisted: Set(0),
        snowflake_deleted: Set(0),
    };
    corporation2.insert(db).await?;

    // J-REIT物件のデータを挿入
    let building1: j_reit_buildings::ActiveModel = j_reit_buildings::ActiveModel {
        id: Set("building1".to_string()),
        name: Set("テストビル1".to_string()),
        city_id: Set(TEST_CITY_ID_MARUNOUCHI),
        is_office: Set(1),
        is_retail: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_residential: Set(0),
        is_health_care: Set(0),
        is_other: Set(0),
        latitude: Set(35.6804),
        longitude: Set(139.761),
        snowflake_deleted: Set(0),
        completed_year: Set(Some(2020)),
        gross_floor_area: Set(Some(500.0)),
        ..Default::default()
    };
    building1.insert(db).await?;

    let building2 = j_reit_buildings::ActiveModel {
        id: Set("building2".to_string()),
        name: Set("テストビル2".to_string()),
        city_id: Set(TEST_CITY_ID_ODORI_HIGASHI),
        is_office: Set(0),
        is_retail: Set(0),
        is_hotel: Set(0),
        is_logistic: Set(0),
        is_residential: Set(1),
        is_health_care: Set(0),
        is_other: Set(0),
        latitude: Set(43.063),
        longitude: Set(141.365),
        snowflake_deleted: Set(0),
        completed_year: Set(Some(2018)),
        gross_floor_area: Set(Some(2000.0)),
        ..Default::default()
    };
    building2.insert(db).await?;

    // J-REIT鑑定のデータを挿入
    let appraisal1 = j_reit_appraisals::ActiveModel {
        id: Set("appraisal1".to_string()),
        appraisal_price: Set(Some(2000000000)),
        cap_rate: Set(Some(4.0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    appraisal1.insert(db).await?;

    let appraisal2 = j_reit_appraisals::ActiveModel {
        id: Set("appraisal2".to_string()),
        appraisal_price: Set(Some(1000000000)),
        cap_rate: Set(Some(5.0)),
        snowflake_deleted: Set(0),
        ..Default::default()
    };
    appraisal2.insert(db).await?;

    // J-REIT取引のデータを挿入
    let transaction1_1 = j_reit_transactions::ActiveModel {
        id: Set("transaction1_1".to_string()),
        j_reit_building_id: Set("building1".to_string()),
        j_reit_corporation_id: Set("corporation1".to_string()),
        combined_transaction_id: Set("building1-corporation1".to_string()),
        transaction_date: Set(naive_date(2025, 1, 1)),
        transaction_price: Set(Some(3000000000)),
        transaction_category: Set(TransactionCategory::InitialAcquisition as i8),
        leasable_area: Set(Some(1000.0)),
        j_reit_appraisal_id: Set(Some("appraisal1".to_string())),
        snowflake_deleted: Set(0),
        is_bulk: Set(0),
        apportioned_transaction_price: Set(None),
        press_release_date: Set(Some(naive_date(2025, 1, 1))),
        ..Default::default()
    };
    transaction1_1.insert(db).await?;

    let transaction1_2 = j_reit_transactions::ActiveModel {
        id: Set("transaction1_2".to_string()),
        j_reit_building_id: Set("building1".to_string()),
        j_reit_corporation_id: Set("corporation1".to_string()),
        combined_transaction_id: Set("building1-corporation1".to_string()),
        transaction_date: Set(naive_date(2025, 5, 15)),
        transaction_price: Set(Some(1000000000)),
        transaction_category: Set(TransactionCategory::PartialTransfer as i8),
        leasable_area: Set(Some(1000.0)),
        j_reit_appraisal_id: Set(None),
        snowflake_deleted: Set(0),
        is_bulk: Set(1),
        apportioned_transaction_price: Set(Some(500000000)),
        press_release_date: Set(None),
        ..Default::default()
    };
    transaction1_2.insert(db).await?;

    let transaction2_1 = j_reit_transactions::ActiveModel {
        id: Set("transaction2_1".to_string()),
        j_reit_building_id: Set("building2".to_string()),
        j_reit_corporation_id: Set("corporation2".to_string()),
        combined_transaction_id: Set("building2-corporation2".to_string()),
        transaction_date: Set(naive_date(2024, 12, 1)),
        transaction_price: Set(Some(2000000000)),
        transaction_category: Set(TransactionCategory::FullTransfer as i8),
        leasable_area: Set(Some(2000.0)),
        j_reit_appraisal_id: Set(Some("appraisal2".to_string())),
        snowflake_deleted: Set(0),
        is_bulk: Set(1),
        apportioned_transaction_price: Set(Some(160000000)),
        press_release_date: Set(Some(naive_date(2024, 12, 1))),
        ..Default::default()
    };
    transaction2_1.insert(db).await?;

    Ok(())
}
