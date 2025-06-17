use async_graphql::Enum;
use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Hash)]
pub struct JReitBuildingIdAndCorporationId {
    pub building_id: String,
    pub corporation_id: String,
}

impl JReitBuildingIdAndCorporationId {
    pub fn combined_transaction_id(&self) -> String {
        format!("{}-{}", self.building_id, self.corporation_id)
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TransactionCategory {
    InitialAcquisition,
    AdditionalAcquisition,
    PartialTransfer,
    FullTransfer,
}

impl From<i8> for TransactionCategory {
    fn from(value: i8) -> Self {
        match value {
            0 => TransactionCategory::InitialAcquisition,
            1 => TransactionCategory::AdditionalAcquisition,
            2 => TransactionCategory::PartialTransfer,
            3 => TransactionCategory::FullTransfer,
            // dbtテストによって起きないことを保証しているが、安全のため「初回取得」に寄せる
            _ => TransactionCategory::InitialAcquisition,
        }
    }
}
