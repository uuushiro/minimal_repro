use async_graphql::{SimpleObject, ID};
use chrono::NaiveDate;
use sql_entities::j_reit_appraisals;

#[derive(SimpleObject, Clone)]
pub(crate) struct GraphQLJReitAppraisal {
    pub(crate) id: ID,
    /// 鑑定評価額
    appraisal_price: Option<i64>,
    /// 鑑定日
    appraisal_date: Option<NaiveDate>,
    /// 鑑定会社
    appraisal_company: Option<String>,
    /// 収益価格
    net_income: Option<i64>,
    /// 直接還元法 収益価格
    direct_capitalization_price: Option<i64>,
    /// 運営収益
    operating_revenue: Option<i64>,
    /// 可能総収益
    potential_gross_income: Option<i64>,
    /// 運営費用
    operating_costs: Option<i64>,
    /// 維持管理費
    maintenance_cost: Option<i64>,
    /// 水道光熱費
    utility_cost: Option<i64>,
    /// 修繕費
    repair_cost: Option<i64>,
    /// テナント募集費用等
    tenant_acquisition_costs: Option<i64>,
    /// プロパティマネジメントフィー
    property_management_fee: Option<i64>,
    /// その他費用
    other_operating_expense: Option<i64>,
    /// NOI
    net_operating_income: Option<i64>,
    /// 一時金の運用益
    temporary_funds_profit: Option<i64>,
    /// 資本的支出
    capital_expenditure: Option<i64>,
    /// 純収益（NCF）
    net_cash_flow: Option<i64>,
    /// 直接還元法利回り［%］
    cap_rate: Option<f64>,
    /// DCF法 収益価格
    discount_cash_flow_price: Option<i64>,
    /// DCF法 割引率［%］
    discount_rate: Option<f64>,
    /// DCF法 最終還元利回り［%］
    terminal_cap_rate: Option<f64>,
    /// 積算価格
    price_by_cost_approach: Option<i64>,
    /// 建物の積算価格
    building_price_by_cost_approach: Option<i64>,
    /// 土地の積算価格
    land_price_by_cost_approach: Option<i64>,
}

impl From<j_reit_appraisals::Model> for GraphQLJReitAppraisal {
    fn from(model: j_reit_appraisals::Model) -> Self {
        Self {
            id: ID(model.id),
            appraisal_price: model.appraisal_price,
            appraisal_date: model.appraisal_date,
            appraisal_company: model.appraisal_company,
            net_income: model.net_income,
            direct_capitalization_price: model.direct_capitalization_price,
            operating_revenue: model.operating_revenue,
            potential_gross_income: model.potential_gross_income,
            operating_costs: model.operating_costs,
            maintenance_cost: model.maintenance_cost,
            utility_cost: model.utility_cost,
            repair_cost: model.repair_cost,
            tenant_acquisition_costs: model.tenant_acquisition_costs,
            property_management_fee: model.property_management_fee,
            other_operating_expense: model.other_operating_expense,
            net_operating_income: model.net_operating_income,
            temporary_funds_profit: model.temporary_funds_profit,
            capital_expenditure: model.capital_expenditure,
            net_cash_flow: model.net_cash_flow,
            cap_rate: model.cap_rate,
            discount_cash_flow_price: model.discount_cash_flow_price,
            discount_rate: model.discount_rate,
            terminal_cap_rate: model.terminal_cap_rate,
            price_by_cost_approach: model.price_by_cost_approach,
            building_price_by_cost_approach: model.building_price_by_cost_approach,
            land_price_by_cost_approach: model.land_price_by_cost_approach,
        }
    }
}
