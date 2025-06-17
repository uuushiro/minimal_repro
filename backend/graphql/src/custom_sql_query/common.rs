use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

// searchの結果をIDのみで返すための構造体（id: i64）
#[derive(FromQueryResult, Debug, Serialize, Deserialize)]
pub(super) struct IntegerId {
    pub(super) id: i64,
}

// searchの結果をIDのみで返すための構造体（id: String）
#[derive(FromQueryResult, Debug, Serialize, Deserialize)]
pub(super) struct StringId {
    pub(super) id: String,
}
