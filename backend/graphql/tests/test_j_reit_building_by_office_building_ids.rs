mod common;
use common::*;
mod utils;
use utils::*;

use async_graphql::{value, Request, Result, Variables};
use pretty_assertions::assert_eq;
use serde_json::json;

#[tokio::test]
// 指定したオフィスビルIDに対して正しいJ-REITビル情報を取得できることを確認
async fn test_j_reit_building_by_office_building_ids_attributes() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    // テスト用のオフィスビルID
    let office_building_id_1 = 1111;
    let office_building_id_2 = 2222;
    let office_building_id_3 = 3333;
    let office_building_id_4 = 4444;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    create_j_reit_building3(&db).await?;
    // office_building_id_4 に対応するJ-REITビルは存在しない

    let request: Request = r#"
        query jReitBuildingByOfficeBuildingIds($ids: [ID!]!) {
            jReitBuildingByOfficeBuildingIds(officeBuildingIds: $ids) {
                id
                officeBuildingId
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": [office_building_id_1, office_building_id_2, office_building_id_3, office_building_id_4]
    });
    let request = request
        .variables(Variables::from_json(variables))
        .data(test_roles_market_research_login(true));
    let response = schema.execute(request).await;

    // Assert
    assert!(response.is_ok(), "{:?}", response.errors);
    let response_data = response.data.into_json().expect("response is not json");
    // office_building_id_3 はオフィスビルIDは設定されているがアセットタイプがオフィスでないため取得されない
    let expect = json!({
        "jReitBuildingByOfficeBuildingIds": [
            {"id": "test_id_1", "officeBuildingId": office_building_id_1.to_string()},
            {"id": "test_id_2", "officeBuildingId": office_building_id_2.to_string()},
        ]
    });
    assert!(is_equal_json_set(
        response_data,
        expect,
        "jReitBuildingByOfficeBuildingIds"
    ));

    Ok(())
}

#[tokio::test]
// ids が空の場合空のデータが返ることの確認
async fn test_j_reit_building_by_office_building_ids_empty_ids() -> Result<()> {
    let TestContext { db, schema } = TestContext::new().await?;
    setup_basic_test_data(&db).await?;

    create_j_reit_building1(&db).await?;
    create_j_reit_building2(&db).await?;
    create_j_reit_building3(&db).await?;

    let request: Request = r#"
        query jReitBuildingByOfficeBuildingIds($ids: [ID!]!) {
            jReitBuildingByOfficeBuildingIds(officeBuildingIds: $ids) {
                id
            }
        }
        "#
    .into();
    let variables = json!({
        "ids": []
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
            "jReitBuildingByOfficeBuildingIds": []
        }),
    );

    Ok(())
}
