use async_graphql::{SimpleObject, ID};
use chrono::NaiveDate;
use sea_orm::prelude::Decimal;
use sql_entities::j_reit_mizuho_financials;

use super::{ids::JReitMizuhoBuildingId, utils::to_decimal_2_digits};

/// Jリート物件の決算データ
/// 参考になるかも: https://nabra-estie.atlassian.net/wiki/spaces/AC/pages/1846937864
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancial {
    pub(crate) id: ID,
    #[graphql(skip)]
    pub(crate) j_reit_mizuho_building_id: JReitMizuhoBuildingId,
    /// 決算期に関するデータ
    pub(crate) fiscal_period: GraphQLJReitFinancialFiscalPeriod,
    /// 収入に関するデータ
    income: GraphQLJReitFinancialIncome,
    /// 支出に関するデータ
    expense: GraphQLJReitFinancialExpense,
    /// 収支に関するデータ
    balance: GraphQLJReitFinancialBalance,
    /// 賃貸状況に関するデータ
    leasing: GraphQLJReitFinancialLeasing,
    /// 物件の鑑定データ
    appraisal: GraphQLJReitFinancialAppraisal,
    /// 運用指標データ
    indicators: GraphQLJReitFinancialIndicators,
    /// 取得額［円］
    acquisition_price: Option<i64>,
    /// 帳簿価格［円］
    book_value: Option<i64>,
    /// 敷金残高［円］
    security_deposit_balance: Option<i64>,
    /// 固定資産税予定額［円］
    scheduled_property_tax: Option<i64>,
}

/// 決算期
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialFiscalPeriod {
    /// 名称（「第3期」など）
    name: Option<String>,
    /// 開始日
    pub(crate) start_date: NaiveDate,
    /// 終了日
    pub(crate) end_date: NaiveDate,
    /// 運用日数
    operating_days: i64,
}

/// 収入に関するデータ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialIncome {
    /// 賃料収入
    rent: Option<i64>,
    /// 駐車料収入
    parking: Option<i64>,
    /// 共益費収入
    cam_fee: Option<i64>,
    /// その他の賃料収入
    other_rental_income: Option<i64>,
    /// その他の収入
    other: Option<i64>,
}

/// 支出に関するデータ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialExpense {
    /// 管理委託料
    property_management: Option<i64>,
    /// 保守点検料
    maintenance: Option<i64>,
    /// 給水光熱費
    utility: Option<i64>,
    /// 警備委託料
    security: Option<i64>,
    /// 修繕費
    repair: Option<i64>,
    /// 清掃費
    cleaning: Option<i64>,
    /// 損害保険料
    insurance: Option<i64>,
    /// 固定資産税等
    real_estate_tax: Option<i64>,
    /// 共益費支出
    cam_fee: Option<i64>,
    /// その他の支出
    other: Option<i64>,
    /// 資本的支出
    capital_expenditure: Option<i64>,
}

/// 収支に関するデータ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialBalance {
    /// NOI
    net_operating_income: Option<i64>,
    /// 減価償却費
    depreciation: Option<i64>,
    /// 利益（NOI-減価償却費）
    net_income: Option<i64>,
    /// フリーキャッシュフロー
    free_cash_flow: Option<i64>,
}

/// 賃貸状況に関するデータ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialLeasing {
    /// 稼働率
    occupancy_rate: Option<Decimal>,
    /// 賃貸先数
    number_of_tenants: Option<i64>,
    /// 賃貸可能面積［坪］
    net_leasable_area_total: Option<Decimal>,
}

/// 物件の鑑定データ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialAppraisal {
    /// 鑑定価格
    appraisal_price: Option<i64>,
    /// 直接還元法 収益価格［円］
    direct_capitalization_price: Option<i64>,
    /// 直接還元法 利回り［%］
    cap_rate: Option<Decimal>,
    /// DCF法 収益価格［円］
    discount_cash_flow_price: Option<i64>,
    /// DCF法 割引率［%］
    discount_rate: Option<Decimal>,
    /// DCF法 最終還元利回り［%］
    terminal_cap_rate: Option<Decimal>,
    /// 鑑定キャップレート（削除予定）［%］
    appraisal_cap_rate: Option<Decimal>,
    /// 鑑定割引率（削除予定）［%］
    appraisal_discount_rate: Option<Decimal>,
}

/// 運用指標データ
#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitFinancialIndicators {
    /// 貸室賃料収入単価［円/月・坪］
    rental_income_per_tsubo: Option<i64>,
    /// 直近1年間のNOI［円］
    year_to_date_net_operating_income: Option<i64>,
    /// NOI利回り（NOI/取得価格の年換算）［%］
    net_operating_income_yield: Option<Decimal>,
    /// 純収益利回り（年換算）［%］
    net_cash_flow_cap_rate: Option<Decimal>,
}

impl From<j_reit_mizuho_financials::Model> for GraphQLJReitFinancial {
    fn from(value: j_reit_mizuho_financials::Model) -> Self {
        let j_reit_mizuho_financials::Model {
            id,
            j_reit_mizuho_building_id,
            fiscal_period,
            fiscal_period_start_date,
            fiscal_period_end_date,
            fiscal_period_operating_day,
            rental_income,
            parking_income,
            common_area_charge,
            other_rental_income,
            other_income,
            property_management_fee,
            maintenance_fee,
            utility_cost,
            security_fee,
            repair_cost,
            cleaning_fee,
            insurance_cost,
            real_estate_tax,
            common_area_expense,
            other_operating_expense,
            capital_expenditure,
            net_operating_income,
            depriciation,
            net_income,
            free_cash_flow,
            occupancy_rate,
            number_of_tenants,
            security_deposit_balance,
            appraisal_price,
            appraisal_cap_rate,
            appraisal_discount_rate,
            acquisition_price,
            scheduled_property_tax,
            net_leasable_area_total,
            year_to_date_net_operating_income,
            rental_income_per_tsubo,
            net_operating_income_yield,
            net_cash_flow_cap_rate,
            book_value,
            direct_capitalization_price,
            discount_cash_flow_price,
            cap_rate,
            discount_rate,
            terminal_cap_rate,
            snowflake_deleted: _,
        } = value.clone();
        Self {
            id: ID(id),
            j_reit_mizuho_building_id: JReitMizuhoBuildingId(j_reit_mizuho_building_id),
            fiscal_period: GraphQLJReitFinancialFiscalPeriod {
                name: fiscal_period,
                start_date: fiscal_period_start_date,
                end_date: fiscal_period_end_date,
                operating_days: fiscal_period_operating_day,
            },
            income: GraphQLJReitFinancialIncome {
                rent: rental_income,
                parking: parking_income,
                cam_fee: common_area_charge,
                other_rental_income,
                other: other_income,
            },
            expense: GraphQLJReitFinancialExpense {
                property_management: property_management_fee,
                maintenance: maintenance_fee,
                utility: utility_cost,
                security: security_fee,
                repair: repair_cost,
                cleaning: cleaning_fee,
                insurance: insurance_cost,
                real_estate_tax,
                cam_fee: common_area_expense,
                other: other_operating_expense,
                capital_expenditure,
            },
            balance: GraphQLJReitFinancialBalance {
                net_operating_income,
                depreciation: depriciation,
                net_income,
                free_cash_flow,
            },
            leasing: GraphQLJReitFinancialLeasing {
                occupancy_rate: occupancy_rate.map(to_decimal_2_digits),
                number_of_tenants,
                net_leasable_area_total: net_leasable_area_total.map(to_decimal_2_digits),
            },
            appraisal: GraphQLJReitFinancialAppraisal {
                appraisal_price,
                direct_capitalization_price,
                cap_rate: cap_rate.map(to_decimal_2_digits),
                discount_cash_flow_price,
                discount_rate: discount_rate.map(to_decimal_2_digits),
                terminal_cap_rate: terminal_cap_rate.map(to_decimal_2_digits),
                appraisal_cap_rate: appraisal_cap_rate.map(to_decimal_2_digits),
                appraisal_discount_rate: appraisal_discount_rate.map(to_decimal_2_digits),
            },
            indicators: GraphQLJReitFinancialIndicators {
                rental_income_per_tsubo,
                year_to_date_net_operating_income,
                net_operating_income_yield: net_operating_income_yield.map(to_decimal_2_digits),
                net_cash_flow_cap_rate: net_cash_flow_cap_rate.map(to_decimal_2_digits),
            },
            acquisition_price,
            book_value,
            security_deposit_balance,
            scheduled_property_tax,
        }
    }
}
