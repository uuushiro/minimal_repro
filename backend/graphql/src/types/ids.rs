use async_graphql::ID;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) struct CityId(pub(crate) i64);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) struct OfficeBuildingId(pub(crate) i64);

impl TryFrom<ID> for OfficeBuildingId {
    type Error = async_graphql::Error;

    fn try_from(id: ID) -> Result<Self, Self::Error> {
        id.as_str()
            .parse::<i64>()
            .map(Self)
            .map_err(|_| async_graphql::Error::new("Invalid OfficeBuildingId"))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitBuildingId(pub(crate) String);

impl From<ID> for JReitBuildingId {
    fn from(id: ID) -> Self {
        Self(id.to_string())
    }
}

impl From<JReitBuildingId> for ID {
    fn from(value: JReitBuildingId) -> Self {
        ID(value.0)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitMizuhoBuildingId(pub(crate) String);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) struct JReitBuildingByOfficeBuildingId(pub(crate) OfficeBuildingId);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitFinancialsByJReitMizuhoBuildingId(pub(crate) JReitMizuhoBuildingId);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitCorporationId(pub(crate) String);

impl From<ID> for JReitCorporationId {
    fn from(id: ID) -> Self {
        Self(id.to_string())
    }
}

impl From<JReitCorporationId> for ID {
    fn from(value: JReitCorporationId) -> Self {
        ID(value.0)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitTransactionsByJReitBuildingId(pub(crate) JReitBuildingId);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitPressReleasesByJReitMizuhoBuildingId(pub(crate) JReitMizuhoBuildingId);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitAppraisalId(pub(crate) String);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitAppraisalHistoriesByJReitMizuhoBuildingId(pub(crate) JReitMizuhoBuildingId);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitCapRateHistoriesByJReitBuildingId(pub(crate) JReitBuildingId);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, Debug)]
pub(crate) struct JReitCapRateHistoriesByJReitMizuhoBuildingId(pub(crate) JReitMizuhoBuildingId);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct JReitMizuhoBuildingIdByBuildingIdAndCorporationId {
    pub j_reit_building_id: JReitBuildingId,
    pub j_reit_corporation_id: JReitCorporationId,
}
