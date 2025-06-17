use super::GraphQLLoader;
use crate::types::{
    ids::{JReitBuildingByOfficeBuildingId, JReitBuildingId, OfficeBuildingId},
    j_reit_buildings::GraphQLJReitBuilding,
};
use async_graphql::{dataloader::Loader, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sql_entities::j_reit_buildings;
use std::collections::HashMap;

#[async_trait::async_trait]
impl Loader<JReitBuildingByOfficeBuildingId> for GraphQLLoader {
    type Value = Option<GraphQLJReitBuilding>;
    type Error = async_graphql::Error;

    async fn load(
        &self,
        keys: &[JReitBuildingByOfficeBuildingId],
    ) -> Result<HashMap<JReitBuildingByOfficeBuildingId, Self::Value>> {
        // office_building_idから該当するj_reit_building_idsを取得
        let office_building_ids = keys.iter().map(|key| key.0 .0);
        let j_reit_buildings = j_reit_buildings::Entity::find()
            .filter(j_reit_buildings::Column::OfficeBuildingId.is_in(office_building_ids))
            // 直接指定されてはいないが、オフィスビルIDでの取得なのでオフィスビルの物件のみ取得する
            // データの不整合により、オフィスでないのにオフィスビルIDが入っている場合があるための対応
            .filter(j_reit_buildings::Column::IsOffice.eq(1))
            .all(&self.db)
            .await?;
        let j_reit_building_ids = j_reit_buildings
            .iter()
            .map(|building| JReitBuildingId(building.id.clone()))
            .collect::<Vec<_>>();

        // j_reit_building_idsからj_reit_buildingsを取得
        let j_reit_buildings = self
            .load(&j_reit_building_ids)
            .await?
            .into_values()
            .collect::<Vec<_>>();

        // office_building_idをキーにしたHashMapを作成
        let mut j_reit_buildings_by_office_building_id: HashMap<
            JReitBuildingByOfficeBuildingId,
            Option<GraphQLJReitBuilding>,
        > = HashMap::new();
        for j_reit_building in j_reit_buildings {
            let office_building_id = match j_reit_building.office_building_id {
                Some(ref office_building_id) => Ok(office_building_id.clone()),
                None => Err(async_graphql::Error::new(
                    "graphql::Loader<JReitBuildingByOfficeBuildingId>::office_building_id is None",
                )),
            }?;
            let office_building_id = OfficeBuildingId::try_from(office_building_id)?;
            j_reit_buildings_by_office_building_id.insert(
                JReitBuildingByOfficeBuildingId(office_building_id),
                Some(j_reit_building),
            );
        }

        // j_reit_buildingが存在しない場合はNoneを入れる
        for key in keys {
            j_reit_buildings_by_office_building_id
                .entry(*key)
                .or_insert(None);
        }

        Ok(j_reit_buildings_by_office_building_id)
    }
}
