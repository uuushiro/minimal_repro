use crate::formatter::{
    format_area_f64_in_square_meter, format_completed_year_month, format_float, format_integer,
    format_j_reit_corporation_name, format_native_date,
};
use crate::roles::get_roles;
use crate::types::asset_type::AssetType;
use actix_web::{post, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use auth0_jwt_validator::Auth0JwtValidator;
use chrono::NaiveDate;
use common::types::{JReitBuildingIdAndCorporationId, TransactionCategory};
use regex::Regex;
use rust_xlsxwriter::{
    ColNum, Format, FormatAlign, IntoExcelData, RowNum, Workbook, Worksheet, XlsxError,
};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::{
    all, Alias, Asterisk, CommonTableExpression, OverStatement, Query, WindowStatement, WithClause,
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    Order, QueryFilter, QueryOrder,
};
use serde::Deserialize;
use sql_entities::{
    j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_mizuho_financials,
    j_reit_mizuho_id_mappings, j_reit_transactions,
};
use std::collections::HashMap;

#[derive(Deserialize)]
struct ExportExcelRequest {
    ids: Vec<JReitBuildingIdAndCorporationId>,
}

#[post("/api/rest/buildings/excel/v2")]
pub async fn excel_route(
    db: web::Data<DatabaseConnection>,
    request: web::Json<ExportExcelRequest>,
    auth: BearerAuth,
    validator: web::Data<Auth0JwtValidator>,
) -> actix_web::Result<HttpResponse> {
    let roles = get_roles(auth, validator).await?;
    if !roles.data.j_reit_premium {
        return Err(actix_web::error::ErrorForbidden("Permission denied"));
    }

    let db = db.get_ref();

    let j_reit_buildings = fetch_buildings_for_excel(db, &request.ids).await?;
    let financials_grouped_by_mizuho_id =
        fetch_financials_grouped_by_mizuho_id(db, &j_reit_buildings).await?;

    let excel = generate_excel(j_reit_buildings, financials_grouped_by_mizuho_id);
    match excel {
        Ok(excel) => Ok(HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .append_header((
                "Content-Disposition",
                "attachment; filename=\"J-REIT_comps.xlsx\"",
            ))
            .body(excel)),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

pub async fn fetch_financials_grouped_by_mizuho_id(
    db: &DatabaseConnection,
    j_reit_buildings: &[JReitBuildingsForExcel],
) -> Result<HashMap<String, Vec<j_reit_mizuho_financials::Model>>, actix_web::Error> {
    let mut financials_grouped_by_mizuho_id: HashMap<String, Vec<j_reit_mizuho_financials::Model>> =
        HashMap::new();

    let financials = j_reit_mizuho_financials::Entity::find()
        .filter(
            j_reit_mizuho_financials::Column::JReitMizuhoBuildingId.is_in(
                j_reit_buildings
                    .iter()
                    .map(|b| b.j_reit_mizuho_building_id.clone()),
            ),
        )
        .order_by_asc(j_reit_mizuho_financials::Column::FiscalPeriodEndDate)
        .all(db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    for financial in financials {
        let mizuho_id = financial.j_reit_mizuho_building_id.clone();
        financials_grouped_by_mizuho_id
            .entry(mizuho_id)
            .or_default()
            .push(financial);
    }

    Ok(financials_grouped_by_mizuho_id)
}

pub async fn fetch_buildings_for_excel(
    db: &DatabaseConnection,
    ids: &[JReitBuildingIdAndCorporationId],
) -> Result<Vec<JReitBuildingsForExcel>, actix_web::Error> {
    let with_clause = WithClause::new()
        // filtered_transactions
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(j_reit_transactions::Entity)
                        .column((j_reit_transactions::Entity, Asterisk))
                        .and_where(
                            j_reit_transactions::Column::CombinedTransactionId
                                .is_in(ids.iter().map(|id| id.combined_transaction_id())),
                        )
                        .to_owned(),
                )
                .table_name(Alias::new("filtered_transactions"))
                .to_owned(),
        )
        // distinct_filtered_transactions
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(Alias::new("filtered_transactions"))
                        .columns([
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ),
                        ])
                        .distinct()
                        .to_owned(),
                )
                .table_name(Alias::new("distinct_filtered_transactions"))
                .to_owned(),
        )
        // initial_appraisals
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(j_reit_appraisals::Entity)
                        .column((j_reit_appraisals::Entity, Asterisk))
                        .columns([
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ),
                        ])
                        .inner_join(
                            Alias::new("filtered_transactions"),
                            all![
                                Expr::col((
                                    Alias::new("filtered_transactions"),
                                    j_reit_transactions::Column::JReitAppraisalId,
                                ))
                                .equals(
                                    (j_reit_appraisals::Entity, j_reit_appraisals::Column::Id,)
                                ),
                                Expr::col((
                                    Alias::new("filtered_transactions"),
                                    j_reit_transactions::Column::TransactionCategory,
                                ))
                                .eq(TransactionCategory::InitialAcquisition as i8),
                            ],
                        )
                        .to_owned(),
                )
                .table_name(Alias::new("initial_appraisals"))
                .to_owned(),
        )
        // initial_acquisitions
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(Alias::new("filtered_transactions"))
                        .columns([
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitAppraisalId,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::TransactionDate,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::TransactionPrice,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::LeasableArea,
                            ),
                        ])
                        .and_where(
                            Expr::col((
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::TransactionCategory,
                            ))
                            .eq(TransactionCategory::InitialAcquisition as i8),
                        )
                        .to_owned(),
                )
                .table_name(Alias::new("initial_acquisitions"))
                .to_owned(),
        )
        // mizuho_id_mappings
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(j_reit_mizuho_id_mappings::Entity)
                        .columns([
                            (
                                j_reit_mizuho_id_mappings::Entity,
                                j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId,
                            ),
                            (
                                j_reit_mizuho_id_mappings::Entity,
                                j_reit_mizuho_id_mappings::Column::JReitBuildingId,
                            ),
                            (
                                j_reit_mizuho_id_mappings::Entity,
                                j_reit_mizuho_id_mappings::Column::JReitCorporationId,
                            ),
                        ])
                        .inner_join(
                            Alias::new("distinct_filtered_transactions"),
                            all![
                                Expr::col((
                                    j_reit_mizuho_id_mappings::Entity,
                                    j_reit_mizuho_id_mappings::Column::JReitBuildingId,
                                ))
                                .equals((
                                    Alias::new("distinct_filtered_transactions"),
                                    j_reit_transactions::Column::JReitBuildingId,
                                )),
                                Expr::col((
                                    j_reit_mizuho_id_mappings::Entity,
                                    j_reit_mizuho_id_mappings::Column::JReitCorporationId,
                                ))
                                .equals((
                                    Alias::new("distinct_filtered_transactions"),
                                    j_reit_transactions::Column::JReitCorporationId,
                                )),
                            ],
                        )
                        .to_owned(),
                )
                .table_name(Alias::new("mizuho_id_mappings"))
                .to_owned(),
        )
        // ranked_latest_transactions
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(j_reit_transactions::Entity)
                        .column((j_reit_transactions::Entity, Asterisk))
                        .expr_window_as(
                            Expr::cust("ROW_NUMBER()"),
                            WindowStatement::new()
                                .partition_by_columns([
                                    j_reit_transactions::Column::JReitBuildingId,
                                    j_reit_transactions::Column::JReitCorporationId,
                                ])
                                .order_by(j_reit_transactions::Column::TransactionDate, Order::Desc)
                                .to_owned(),
                            Alias::new("rn_latest_acquisitions"),
                        )
                        .to_owned(),
                )
                .table_name(Alias::new("ranked_latest_transactions"))
                .to_owned(),
        )
        // latest_transactions
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(Alias::new("ranked_latest_transactions"))
                        .columns([
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::JReitBuildingId,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::JReitCorporationId,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::TotalLeasableArea,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::TransactionCategory,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::TransactionDate,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::BuildingOwnershipRatio,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::BuildingOwnershipType,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::LandOwnershipType,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::LandOwnershipRatio,
                            ),
                        ])
                        .and_where(Expr::col(Alias::new("rn_latest_acquisitions")).eq(1))
                        .to_owned(),
                )
                .table_name(Alias::new("latest_transactions"))
                .to_owned(),
        )
        // ranked_latest_financials
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(j_reit_mizuho_financials::Entity)
                        .column((j_reit_mizuho_financials::Entity, Asterisk))
                        .columns([
                            (
                                Alias::new("mizuho_id_mappings"),
                                j_reit_mizuho_id_mappings::Column::JReitBuildingId,
                            ),
                            (
                                Alias::new("mizuho_id_mappings"),
                                j_reit_mizuho_id_mappings::Column::JReitCorporationId,
                            ),
                        ])
                        .inner_join(
                            Alias::new("mizuho_id_mappings"),
                            Expr::col((
                                j_reit_mizuho_financials::Entity,
                                j_reit_mizuho_financials::Column::JReitMizuhoBuildingId,
                            ))
                            .equals((
                                Alias::new("mizuho_id_mappings"),
                                j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId,
                            )),
                        )
                        .expr_window_as(
                            Expr::cust("ROW_NUMBER()"),
                            WindowStatement::new()
                                .partition_by_columns([(
                                    j_reit_mizuho_financials::Entity,
                                    j_reit_mizuho_financials::Column::JReitMizuhoBuildingId,
                                )])
                                .order_by(
                                    j_reit_mizuho_financials::Column::FiscalPeriodEndDate,
                                    Order::Desc,
                                )
                                .to_owned(),
                            Alias::new("rn_latest_financials"),
                        )
                        .to_owned(),
                )
                .table_name(Alias::new("ranked_latest_financials"))
                .to_owned(),
        )
        // latest_financials
        .cte(
            CommonTableExpression::new()
                .query(
                    Query::select()
                        .from(Alias::new("ranked_latest_financials"))
                        .columns([
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::CapRate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::FiscalPeriodEndDate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::AppraisalPrice,
                            ),
                        ])
                        .columns([
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_id_mappings::Column::JReitBuildingId,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_id_mappings::Column::JReitCorporationId,
                            ),
                        ])
                        .and_where(Expr::col(Alias::new("rn_latest_financials")).eq(1))
                        .to_owned(),
                )
                .table_name(Alias::new("latest_financials"))
                .to_owned(),
        )
        .to_owned();

    let query = Query::select()
        .from(j_reit_buildings::Entity)
        .join(
            JoinType::InnerJoin,
            Alias::new("distinct_filtered_transactions"),
            Expr::col((
                Alias::new("distinct_filtered_transactions"),
                j_reit_transactions::Column::JReitBuildingId,
            ))
            .equals((j_reit_buildings::Entity, j_reit_buildings::Column::Id)),
        )
        .join(
            JoinType::InnerJoin,
            j_reit_corporations::Entity,
            Expr::col((
                Alias::new("distinct_filtered_transactions"),
                j_reit_transactions::Column::JReitCorporationId,
            ))
            .equals((j_reit_corporations::Entity, j_reit_corporations::Column::Id)),
        )
        .left_join(
            Alias::new("initial_acquisitions"),
            Expr::col((
                Alias::new("distinct_filtered_transactions"),
                j_reit_transactions::Column::JReitBuildingId,
            ))
            .equals((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::JReitBuildingId,
            )),
        )
        .left_join(
            Alias::new("initial_appraisals"),
            all![
                Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals((
                    Alias::new("initial_appraisals"),
                    Alias::new("j_reit_building_id"),
                )),
                Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)).equals((
                    Alias::new("initial_appraisals"),
                    Alias::new("j_reit_corporation_id")
                )),
            ],
        )
        .left_join(
            Alias::new("latest_transactions"),
            all![
                Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals((
                    Alias::new("latest_transactions"),
                    Alias::new("j_reit_building_id"),
                )),
                Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)).equals((
                    Alias::new("latest_transactions"),
                    Alias::new("j_reit_corporation_id")
                )),
            ],
        )
        .left_join(
            Alias::new("latest_financials"),
            all![
                Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals((
                    Alias::new("latest_financials"),
                    Alias::new("j_reit_building_id"),
                )),
                Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)).equals((
                    Alias::new("latest_financials"),
                    Alias::new("j_reit_corporation_id")
                )),
            ],
        )
        .left_join(
            Alias::new("mizuho_id_mappings"),
            all![
                Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals((
                    Alias::new("mizuho_id_mappings"),
                    j_reit_mizuho_id_mappings::Column::JReitBuildingId,
                )),
                Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)).equals((
                    Alias::new("mizuho_id_mappings"),
                    j_reit_mizuho_id_mappings::Column::JReitCorporationId
                )),
            ],
        )
        // initial_appraisals
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::CapRate,
            )),
            Alias::new("appraisal_cap_rate"),
        )
        // j_reit_corporations
        .columns([j_reit_corporations::Column::IsDelisted])
        // latest_transactions
        .expr(Expr::col((Alias::new("latest_transactions"), Asterisk)))
        // corporations
        .expr_as(
            Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)),
            Alias::new("corporation_id"),
        )
        .expr_as(
            Expr::col((
                j_reit_corporations::Entity,
                j_reit_corporations::Column::Name,
            )),
            Alias::new("corporation_name"),
        )
        // initial_acquisitions
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::TransactionDate,
            )),
            Alias::new("initial_acquisition_transaction_date"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::TransactionPrice,
            )),
            Alias::new("initial_acquisition_transaction_price"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::LeasableArea,
            )),
            Alias::new("initial_acquisition_leasable_area"),
        )
        // latest_financials
        .expr_as(
            Expr::col((
                Alias::new("latest_financials"),
                j_reit_mizuho_financials::Column::CapRate,
            )),
            Alias::new("latest_financial_cap_rate"),
        )
        .expr_as(
            Expr::col((
                Alias::new("latest_financials"),
                j_reit_mizuho_financials::Column::FiscalPeriodEndDate,
            )),
            Alias::new("latest_financial_fiscal_period_end_date"),
        )
        .expr_as(
            Expr::col((
                Alias::new("latest_financials"),
                j_reit_mizuho_financials::Column::AppraisalPrice,
            )),
            Alias::new("latest_financial_appraisal_price"),
        )
        // j_reit_buildings
        .expr(Expr::col((j_reit_buildings::Entity, Asterisk)))
        // j_reit_mizuho_id_mappings
        .column(j_reit_mizuho_id_mappings::Column::JReitMizuhoBuildingId)
        .to_owned()
        .with(with_clause)
        .to_owned();

    let j_reit_buildings =
        JReitBuildingsForExcel::find_by_statement(db.get_database_backend().build(&query))
            .all(db)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(reorder_buildings(j_reit_buildings, ids))
}

fn reorder_buildings(
    j_reit_buildings: Vec<JReitBuildingsForExcel>,
    ids: &[JReitBuildingIdAndCorporationId],
) -> Vec<JReitBuildingsForExcel> {
    let mut map = HashMap::new();
    j_reit_buildings.into_iter().for_each(|b| {
        map.insert(
            JReitBuildingIdAndCorporationId {
                building_id: b.j_reit_building.id.clone(),
                corporation_id: b.corporation_id.clone(),
            },
            b,
        );
    });
    ids.iter().filter_map(|id| map.remove(id)).collect()
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct InitialAcquisition {
    pub initial_acquisition_transaction_date: Option<NaiveDate>,
    pub initial_acquisition_transaction_price: Option<i64>,
    pub initial_acquisition_leasable_area: Option<f64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct JReitAppraisal {
    pub appraisal_cap_rate: Option<f64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct LatestTransaction {
    pub total_leasable_area: Option<f64>,
    pub transaction_category: Option<i8>,
    pub transaction_date: Option<NaiveDate>,
    pub building_ownership_ratio: Option<f64>,
    pub building_ownership_type: Option<String>,
    pub land_ownership_type: Option<String>,
    pub land_ownership_ratio: Option<f64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct LatestFinancial {
    pub latest_financial_cap_rate: Option<f64>,
    pub latest_financial_fiscal_period_end_date: NaiveDate,
    pub latest_financial_appraisal_price: Option<i64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct JReitBuildingsForExcel {
    #[sea_orm(nested)]
    pub j_reit_building: j_reit_buildings::Model,
    #[sea_orm(nested)]
    pub initial_acquisition: Option<InitialAcquisition>,
    #[sea_orm(nested)]
    pub j_reit_appraisal: Option<JReitAppraisal>,
    #[sea_orm(nested)]
    pub latest_transaction: Option<LatestTransaction>,
    #[sea_orm(nested)]
    pub latest_financial: Option<LatestFinancial>,

    pub is_delisted: i8,
    pub corporation_id: String,
    pub corporation_name: String,
    pub j_reit_mizuho_building_id: Option<String>,
}

fn generate_excel(
    j_reit_buildings: Vec<JReitBuildingsForExcel>,
    financials_grouped_by_mizuho_id: HashMap<String, Vec<j_reit_mizuho_financials::Model>>,
) -> anyhow::Result<Vec<u8>> {
    let mut workbook = Workbook::new();
    let format = Format::new()
        .set_font_name("Meiryo UI")
        .set_font_size(9)
        .set_align(FormatAlign::Center);
    let buildings_sheet = build_buildings_sheet(&j_reit_buildings, &format)?;
    workbook.push_worksheet(buildings_sheet);
    let mut names_map = HashMap::new();
    let blank_vec = vec![];
    for building in j_reit_buildings {
        if let Some(j_reit_mizuho_building_id) = building.j_reit_mizuho_building_id.clone() {
            let financials = financials_grouped_by_mizuho_id
                .get(&j_reit_mizuho_building_id)
                .unwrap_or(&blank_vec);
            let financials_sheet =
                build_financials_sheet(&building, financials, &format, &mut names_map)?;
            workbook.push_worksheet(financials_sheet);
        }
    }

    let result = workbook.save_to_buffer()?;
    Ok(result)
}

trait CellType<T> {
    fn write_name(
        &self,
        worksheet: &mut Worksheet,
        row: RowNum,
        col: ColNum,
    ) -> Result<(), XlsxError>;

    fn write_data(
        &self,
        worksheet: &mut Worksheet,
        data: &T,
        row: RowNum,
        col: ColNum,
    ) -> Result<(), XlsxError>;
}
struct CellTypeImpl<'a, T, D: IntoExcelData> {
    name: &'a str,
    get_field_value: fn(&T) -> D,
    format: &'a Format,
}
impl<T, D: IntoExcelData> CellType<T> for CellTypeImpl<'_, T, D> {
    fn write_name(
        &self,
        worksheet: &mut Worksheet,
        row: RowNum,
        col: ColNum,
    ) -> Result<(), XlsxError> {
        worksheet.write_with_format(row, col, self.name, self.format)?;
        Ok(())
    }

    fn write_data(
        &self,
        worksheet: &mut Worksheet,
        data: &T,
        row: RowNum,
        col: ColNum,
    ) -> Result<(), XlsxError> {
        let value = (self.get_field_value)(data);
        worksheet.write_with_format(row, col, value, self.format)?;
        Ok(())
    }
}

fn build_buildings_sheet(
    data: &Vec<JReitBuildingsForExcel>,
    default_format: &Format,
) -> Result<Worksheet, XlsxError> {
    let number_format = &default_format.clone().set_num_format("#,##0");
    let decimal_format = &default_format.clone().set_num_format("#,##0.00");
    let percent_format = &default_format.clone().set_num_format("0.0#%");
    let cell_types: Vec<Box<dyn CellType<JReitBuildingsForExcel>>> = vec![
        Box::new(CellTypeImpl {
            name: "物件名",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building.j_reit_building.name.clone()
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "住所",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building.j_reit_building.address.clone()
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "最寄り駅",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building.j_reit_building.nearest_station.clone()
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "アセットタイプ",
            get_field_value: |building: &JReitBuildingsForExcel| {
                AssetType::from(&building.j_reit_building).format()
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "投資法人",
            get_field_value: |building: &JReitBuildingsForExcel| {
                format_j_reit_corporation_name(
                    &building.corporation_name,
                    building.is_delisted == 1,
                )
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "竣工年",
            get_field_value: |building: &JReitBuildingsForExcel| {
                format_completed_year_month(
                    &building.j_reit_building.completed_year,
                    &building.j_reit_building.completed_month,
                )
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "敷地面積［㎡］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .j_reit_building
                    .land
                    .map(format_area_f64_in_square_meter)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "延床面積［㎡］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .j_reit_building
                    .gross_floor_area
                    .map(format_area_f64_in_square_meter)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "賃貸可能面積［㎡］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_transaction
                    .as_ref()
                    .and_then(|t| t.total_leasable_area)
                    .map(format_area_f64_in_square_meter)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "敷地面積［坪］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building.j_reit_building.land.map(format_float)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "延床面積［坪］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building.j_reit_building.gross_floor_area.map(format_float)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "賃貸可能面積［坪］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_transaction
                    .as_ref()
                    .and_then(|t| t.total_leasable_area)
                    .map(format_float)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "地上階",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .j_reit_building
                    .groundfloor
                    .map(|floors| floors.to_string())
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "地下階",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .j_reit_building
                    .basement
                    .map(|floors| floors.to_string())
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "初回取得日",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .initial_acquisition
                    .as_ref()
                    .and_then(|t| t.initial_acquisition_transaction_date)
                    .map(|date| format_native_date(&date))
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "取引価格［円］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .initial_acquisition
                    .as_ref()
                    .and_then(|a| a.initial_acquisition_transaction_price)
                    .map(format_integer)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "取得時キャップレート（直接還元法）",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .j_reit_appraisal
                    .as_ref()
                    .and_then(|a| a.appraisal_cap_rate)
                    .map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "最新期末鑑定日",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_financial
                    .as_ref()
                    .map(|a| format_native_date(&a.latest_financial_fiscal_period_end_date))
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "最新期末鑑定評価額",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_financial
                    .as_ref()
                    .and_then(|a| a.latest_financial_appraisal_price)
                    .map(format_integer)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "最新期末鑑定キャップレート（直接還元法）",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_financial
                    .as_ref()
                    .and_then(|a| a.latest_financial_cap_rate)
                    .map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "土地坪単価（取引価格 / 敷地面積）［千円／坪］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                calc_price_per_land(
                    building
                        .initial_acquisition
                        .as_ref()
                        .and_then(|a| a.initial_acquisition_transaction_price),
                    building.j_reit_building.land,
                )
                .map(|price| price / 1000.0)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "専有坪単価（取引価格 / 取得時賃貸可能面積）［千円／坪］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                if let Some(initial_acquisition) = building.initial_acquisition.as_ref() {
                    calc_price_per_area(
                        initial_acquisition.initial_acquisition_transaction_price,
                        initial_acquisition.initial_acquisition_leasable_area,
                    )
                    .map(|price| price / 1000.0)
                } else {
                    None
                }
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "一種単価（土地坪単価 / 容積率）［千円／坪］",
            get_field_value: |building: &JReitBuildingsForExcel| {
                if let Some(initial_acquisition) = building.initial_acquisition.as_ref() {
                    calc_price_per_full_land(
                        initial_acquisition.initial_acquisition_transaction_price,
                        initial_acquisition.initial_acquisition_leasable_area,
                        building.j_reit_building.floor_area_ratio,
                    )
                    .map(|price| price / 1000.0)
                } else {
                    None
                }
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "建物所有形態",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_transaction
                    .as_ref()
                    .and_then(|t| t.building_ownership_type.clone())
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "建物所有割合",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_transaction
                    .as_ref()
                    .and_then(|t| t.building_ownership_ratio)
                    .map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "土地所有形態",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_transaction
                    .as_ref()
                    .and_then(|t| t.land_ownership_type.clone())
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "土地所有割合",
            get_field_value: |building: &JReitBuildingsForExcel| {
                building
                    .latest_transaction
                    .as_ref()
                    .and_then(|t| t.land_ownership_ratio)
                    .map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "保有状況",
            get_field_value: |building: &JReitBuildingsForExcel| {
                let latest_transaction = building.latest_transaction.as_ref();
                let latest_transfer = latest_transaction.as_ref().filter(|transaction| {
                    transaction.transaction_category
                        == Some(TransactionCategory::FullTransfer as i8)
                });
                if latest_transfer
                    .as_ref()
                    .and_then(|t| t.transaction_date)
                    .is_some()
                {
                    "譲渡済み".to_string()
                } else {
                    "保有中".to_string()
                }
            },
            format: default_format,
        }),
    ];

    let mut worksheet = Worksheet::new();

    worksheet.set_name("JREIT-comps")?;

    // header
    worksheet.set_column_width(0, 35)?;
    worksheet.write_with_format(0, 0, "No", default_format)?;
    for (index, cell) in (1u32..).zip(&cell_types) {
        cell.write_name(&mut worksheet, index, 0)?
    }

    // body
    for (col, building) in (1u16..).zip(data) {
        worksheet.set_column_width(col, 35)?;
        worksheet.write_with_format(0, col, col, default_format)?;
        for (row, cell) in (1u32..).zip(&cell_types) {
            cell.write_data(&mut worksheet, building, row, col)?
        }
    }

    Ok(worksheet)
}

// 土地坪単価
fn calc_price_per_land(price: Option<i64>, land: Option<f64>) -> Option<f64> {
    match (price, land) {
        (Some(price), Some(land)) if land > 0.0 => Some(price as f64 / land),
        (_, _) => None,
    }
}

fn calc_price_per_full_land(
    price: Option<i64>,
    land: Option<f64>,
    floor_area_ratio: Option<f64>,
) -> Option<f64> {
    let maybe_price_per_land = calc_price_per_land(price, land);
    match (maybe_price_per_land, floor_area_ratio) {
        (Some(price_per_land), Some(floor_area_ratio)) if floor_area_ratio > 0.0 => {
            Some(price_per_land / (floor_area_ratio / 100.0))
        }
        (_, _) => None,
    }
}

const MILLION: f64 = 1_000_000.0;

fn build_financials_sheet(
    building: &JReitBuildingsForExcel,
    financials: &Vec<j_reit_mizuho_financials::Model>,
    default_format: &Format,
    names_map: &mut HashMap<String, u64>,
) -> Result<Worksheet, XlsxError> {
    let number_format = &default_format.clone().set_num_format("#,##0");
    let decimal_format = &default_format.clone().set_num_format("#,##0.00");
    let percent_format = &default_format.clone().set_num_format("0.00%");
    let cell_types: Vec<Box<dyn CellType<j_reit_mizuho_financials::Model>>> = vec![
        Box::new(CellTypeImpl {
            name: "決算期",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.fiscal_period.clone()
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "運用期間 自",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                format_native_date(&financial.fiscal_period_start_date)
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "運用期間 至",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                format_native_date(&financial.fiscal_period_end_date)
            },
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "",
            get_field_value: |_| "".to_string(),
            format: default_format,
        }),
        Box::new(CellTypeImpl {
            name: "収入計［百万円］",
            get_field_value: calc_total_income,
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "室料収入",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.rental_income.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "共益費収入",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .common_area_charge
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "駐車場収入",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.parking_income.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "その他の賃料収入",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .other_rental_income
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "その他の収入",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.other_income.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "支出計［百万円］",
            get_field_value: calc_total_expense,
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "管理委託料",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .property_management_fee
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "保守点検料",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .maintenance_fee
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "給水光熱費",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.utility_cost.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "警備委託料",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.security_fee.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "修繕費",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.repair_cost.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "清掃費",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.cleaning_fee.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "損害保険料",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.insurance_cost.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "固定資産税等",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .real_estate_tax
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "共益費",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .common_area_expense
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "その他支出",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .other_operating_expense
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "NOI［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .net_operating_income
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "減価償却費［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.depriciation.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "利益［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.net_income.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "資本的支出［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .capital_expenditure
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "FCF［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.free_cash_flow.map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "稼働率",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.occupancy_rate.map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "賃貸先数",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.number_of_tenants
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "敷金残高［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .security_deposit_balance
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "鑑定評価額［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .appraisal_price
                    .map(|price| price as f64 / MILLION)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "専有坪単価（鑑定評価額 / 賃貸可能面積）［千円／坪］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                calc_price_per_area(financial.appraisal_price, financial.net_leasable_area_total)
                    .map(|price| price / 1000.0)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "鑑定キャップレート",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.appraisal_cap_rate.map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "取得額［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .acquisition_price
                    .map(|price| price as f64 / MILLION)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "固定資産税予定額［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .scheduled_property_tax
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "賃貸可能面積［坪］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.net_leasable_area_total
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "直近1年間NOI［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .year_to_date_net_operating_income
                    .map(|price| price as f64 / MILLION)
            },
            format: decimal_format,
        }),
        Box::new(CellTypeImpl {
            name: "貸室賃料収入単価［千円／月坪］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .rental_income_per_tsubo
                    .map(|price| price as f64 / 1000.0)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "帳簿価額［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.book_value.map(|price| price as f64 / MILLION)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "直接還元価格［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .direct_capitalization_price
                    .map(|price| price as f64 / MILLION)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "直接還元利回り",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.cap_rate.map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "DCF価格［百万円］",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial
                    .discount_cash_flow_price
                    .map(|price| price as f64 / MILLION)
            },
            format: number_format,
        }),
        Box::new(CellTypeImpl {
            name: "割引率",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.discount_rate.map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "最終還元利回り",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.terminal_cap_rate.map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "NOI利回り",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.net_operating_income_yield
            },
            format: percent_format,
        }),
        Box::new(CellTypeImpl {
            name: "純収益利回り",
            get_field_value: |financial: &j_reit_mizuho_financials::Model| {
                financial.net_cash_flow_cap_rate.map(|rate| rate / 100.0)
            },
            format: percent_format,
        }),
    ];

    let mut worksheet = Worksheet::new();
    worksheet.set_name(validate_sheet_name(
        building.j_reit_building.name.clone(),
        names_map,
    ))?;

    if financials.is_empty() {
        worksheet.write(0, 0, "決算データなし")?;
        return Ok(worksheet);
    }

    // header
    worksheet.set_column_width(0, 40)?;
    for (index, cell) in cell_types.iter().enumerate() {
        cell.write_name(&mut worksheet, index as u32, 0)?
    }

    // body
    for (col, financial) in (1u16..).zip(financials) {
        worksheet.set_column_width(col, 10)?;
        for (row, cell) in cell_types.iter().enumerate() {
            cell.write_data(&mut worksheet, financial, row as u32, col)?
        }
    }

    Ok(worksheet)
}

fn calc_total_income(financial: &j_reit_mizuho_financials::Model) -> Option<f64> {
    let values = [
        financial.rental_income,
        financial.common_area_charge,
        financial.parking_income,
        financial.other_rental_income,
        financial.other_income,
    ]
    .iter()
    .filter_map(|&value| value)
    .collect::<Vec<i64>>();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<i64>() as f64 / MILLION)
    }
}

fn calc_total_expense(financial: &j_reit_mizuho_financials::Model) -> Option<f64> {
    let values = [
        financial.property_management_fee,
        financial.maintenance_fee,
        financial.utility_cost,
        financial.security_fee,
        financial.repair_cost,
        financial.cleaning_fee,
        financial.insurance_cost,
        financial.real_estate_tax,
        financial.common_area_expense,
        financial.other_operating_expense,
    ]
    .iter()
    .filter_map(|&value| value)
    .collect::<Vec<i64>>();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<i64>() as f64 / MILLION)
    }
}

fn calc_price_per_area(price: Option<i64>, area: Option<f64>) -> Option<f64> {
    match (price, area) {
        (Some(appraisal_price), Some(leasable_area)) if leasable_area != 0.0 => {
            Some(appraisal_price as f64 / leasable_area)
        }
        _ => None,
    }
}

fn validate_sheet_name(name: String, names_map: &mut HashMap<String, u64>) -> String {
    let maybe_re = Regex::new(r"[*?:\\/\[\]]");
    let validated_name = match maybe_re {
        Ok(re) => re.replace_all(&name, "").to_string(),
        _ => name,
    };
    if let Some(count) = names_map.get(&validated_name) {
        let num = *count + 1;
        names_map.insert(validated_name.clone(), num);
        format!("{}_{}", validated_name, num)
    } else {
        names_map.insert(validated_name.clone(), 1);
        validated_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sheet_name() {
        let mut names_map = HashMap::new();
        assert_eq!(
            validate_sheet_name("estie building".into(), &mut names_map),
            "estie building"
        );
        assert_eq!(
            validate_sheet_name("エ?ス:テ[ィ] ビ\\ル/デ*ィング".into(), &mut names_map),
            "エスティ ビルディング"
        );
        assert_eq!(
            validate_sheet_name("estie building".into(), &mut names_map),
            "estie building_2"
        );
        assert_eq!(
            validate_sheet_name("estie building".into(), &mut names_map),
            "estie building_3"
        )
    }
}
