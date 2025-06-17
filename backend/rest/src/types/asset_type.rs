use sql_entities::j_reit_buildings;

pub(crate) struct AssetType {
    is_office: bool,
    is_residential: bool,
    is_retail: bool,
    is_logistic: bool,
    is_hotel: bool,
    is_health_care: bool,
    is_other: bool,
}

impl AssetType {
    pub(crate) fn format(&self) -> String {
        let asset_type_conds = [
            (self.is_office, "オフィス"),
            (self.is_residential, "住宅"),
            (self.is_retail, "商業"),
            (self.is_logistic, "物流"),
            (self.is_hotel, "ホテル"),
            (self.is_health_care, "ヘルスケア"),
            (self.is_other, "その他"),
            // (asset_type.is_land, "開発用地")
        ];

        asset_type_conds
            .iter()
            .filter_map(|&(condition, asset_name)| if condition { Some(asset_name) } else { None })
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl From<&j_reit_buildings::Model> for AssetType {
    fn from(value: &j_reit_buildings::Model) -> Self {
        Self {
            is_office: value.is_office == 1,
            is_residential: value.is_residential == 1,
            is_retail: value.is_retail == 1,
            is_logistic: value.is_logistic == 1,
            is_hotel: value.is_hotel == 1,
            is_health_care: value.is_health_care == 1,
            is_other: value.is_other == 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_asset_type() {
        let asset_type = AssetType {
            is_office: true,
            is_residential: false,
            is_retail: true,
            is_logistic: false,
            is_hotel: true,
            is_health_care: false,
            is_other: false,
        };
        assert_eq!(asset_type.format(), "オフィス,商業,ホテル");
    }
}
