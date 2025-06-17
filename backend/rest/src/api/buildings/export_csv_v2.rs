use std::collections::HashMap;

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
use csv::Writer;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::{
    all, Alias, Asterisk, CommonTableExpression, OverStatement, Query, WindowStatement, WithClause,
};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, FromQueryResult, JoinType, Order};
use serde::Deserialize;
use sql_entities::{
    j_reit_appraisals, j_reit_buildings, j_reit_corporations, j_reit_mizuho_financials,
    j_reit_mizuho_id_mappings, j_reit_transactions,
};

#[derive(Deserialize)]
struct ExportCsvRequest {
    ids: Vec<JReitBuildingIdAndCorporationId>,
}

pub async fn fetch_buildings_for_csv(
    db: &DatabaseConnection,
    ids: &[JReitBuildingIdAndCorporationId],
) -> Result<Vec<JReitBuildingForCsv>, sea_orm::DbErr> {
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
                                j_reit_transactions::Column::TransactionDate,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::TransactionPrice,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::JReitAppraisalId,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::LeasableArea,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::TransactionPartner,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::Trustee,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::PropertyManager,
                            ),
                            (
                                Alias::new("filtered_transactions"),
                                j_reit_transactions::Column::PmlAssessmentCompany,
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
                                j_reit_transactions::Column::TransactionPrice,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::TransactionPartner,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::Trustee,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::PropertyManager,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::PmlAssessmentCompany,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::LeasableUnits,
                            ),
                            (
                                Alias::new("ranked_latest_transactions"),
                                j_reit_transactions::Column::BuildingOwnershipRatio,
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
                                j_reit_mizuho_financials::Column::TerminalCapRate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::DiscountRate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::FiscalPeriodStartDate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::FiscalPeriodEndDate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::NetOperatingIncome,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::AppraisalPrice,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::OccupancyRate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::NumberOfTenants,
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
        // second_term_before_financials
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
                                j_reit_mizuho_financials::Column::TerminalCapRate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::DiscountRate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::FiscalPeriodStartDate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::FiscalPeriodEndDate,
                            ),
                            (
                                Alias::new("ranked_latest_financials"),
                                j_reit_mizuho_financials::Column::NetOperatingIncome,
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
                        .and_where(Expr::col(Alias::new("rn_latest_financials")).eq(3))
                        .to_owned(),
                )
                .table_name(Alias::new("second_term_before_financials"))
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
            Alias::new("second_term_before_financials"),
            all![
                Expr::col((j_reit_buildings::Entity, j_reit_buildings::Column::Id)).equals((
                    Alias::new("second_term_before_financials"),
                    Alias::new("j_reit_building_id"),
                )),
                Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)).equals((
                    Alias::new("second_term_before_financials"),
                    Alias::new("j_reit_corporation_id")
                )),
            ],
        )
        // initial_appraisals
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::AppraisalPrice,
            )),
            Alias::new("j_reit_appraisal_appraisal_price"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::CapRate,
            )),
            Alias::new("j_reit_appraisal_cap_rate"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::TerminalCapRate,
            )),
            Alias::new("j_reit_appraisal_terminal_cap_rate"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::DiscountRate,
            )),
            Alias::new("j_reit_appraisal_discount_rate"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::AppraisalCompany,
            )),
            Alias::new("j_reit_appraisal_appraisal_company"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_appraisals"),
                j_reit_appraisals::Column::NetOperatingIncome,
            )),
            Alias::new("j_reit_appraisal_net_operating_income"),
        )
        // j_reit_corporations
        .columns([j_reit_corporations::Column::IsDelisted])
        // latest_transactions
        .expr(Expr::col((Alias::new("latest_transactions"), Asterisk)))
        // corporations
        .expr_as(
            Expr::col((
                j_reit_corporations::Entity,
                j_reit_corporations::Column::Name,
            )),
            Alias::new("corporation_name"),
        )
        .expr_as(
            Expr::col((j_reit_corporations::Entity, j_reit_corporations::Column::Id)),
            Alias::new("corporation_id"),
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
                j_reit_transactions::Column::TransactionPartner,
            )),
            Alias::new("initial_acquisition_transaction_partner"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::Trustee,
            )),
            Alias::new("initial_acquisition_trustee"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::PropertyManager,
            )),
            Alias::new("initial_acquisition_property_manager"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::PmlAssessmentCompany,
            )),
            Alias::new("initial_acquisition_pml_assessment_company"),
        )
        .expr_as(
            Expr::col((
                Alias::new("initial_acquisitions"),
                j_reit_transactions::Column::LeasableArea,
            )),
            Alias::new("initial_acquisition_leasable_area"),
        )
        // latest_financials
        .expr(Expr::col((Alias::new("latest_financials"), Asterisk)))
        // second_term_before_financials
        .expr_as(
            Expr::col((
                Alias::new("second_term_before_financials"),
                j_reit_mizuho_financials::Column::NetOperatingIncome,
            )),
            Alias::new("second_term_before_financial_net_operating_income"),
        )
        // j_reit_buildings
        .expr(Expr::col((j_reit_buildings::Entity, Asterisk)))
        .to_owned()
        .with(with_clause)
        .to_owned();

    let j_reit_buildings =
        JReitBuildingForCsv::find_by_statement(db.get_database_backend().build(&query))
            .all(db)
            .await?;

    Ok(reorder_buildings(j_reit_buildings, ids))
}

#[post("/api/rest/buildings/csv/v2")]
pub async fn csv_route(
    db: web::Data<DatabaseConnection>,
    request: web::Json<ExportCsvRequest>,
    auth: BearerAuth,
    validator: web::Data<Auth0JwtValidator>,
) -> actix_web::Result<HttpResponse> {
    let roles = get_roles(auth, validator).await?;
    if !roles.data.j_reit_premium {
        return Err(actix_web::error::ErrorForbidden("Permission denied"));
    }

    let db = db.get_ref();

    let j_reit_buildings = fetch_buildings_for_csv(db, &request.ids)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let csv = generate_csv(j_reit_buildings);
    match csv {
        Ok(csv) => Ok(HttpResponse::Ok()
            .content_type("text/csv")
            .append_header(("Content-Disposition", "attachment; filename=\"J-REIT.csv\""))
            .body(csv)),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

fn reorder_buildings(
    j_reit_buildings: Vec<JReitBuildingForCsv>,
    ids: &[JReitBuildingIdAndCorporationId],
) -> Vec<JReitBuildingForCsv> {
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
    pub initial_acquisition_transaction_partner: Option<String>,
    pub initial_acquisition_trustee: Option<String>,
    pub initial_acquisition_property_manager: Option<String>,
    pub initial_acquisition_pml_assessment_company: Option<String>,
    pub initial_acquisition_leasable_area: Option<f64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct LatestTransaction {
    pub total_leasable_area: Option<f64>,
    pub transaction_category: Option<i8>,
    pub transaction_date: Option<NaiveDate>,
    pub transaction_price: Option<i64>,
    pub transaction_partner: Option<String>,
    pub leasable_units: Option<i64>,
    pub building_ownership_ratio: Option<f64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct LatestFinancial {
    pub cap_rate: Option<f64>,
    pub terminal_cap_rate: Option<f64>,
    pub discount_rate: Option<f64>,
    pub fiscal_period_start_date: NaiveDate,
    pub fiscal_period_end_date: NaiveDate,
    pub net_operating_income: Option<i64>,
    pub appraisal_price: Option<i64>,
    pub occupancy_rate: Option<f64>,
    pub number_of_tenants: Option<i64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct SecondTermBeforeFinancial {
    pub second_term_before_financial_net_operating_income: Option<i64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct JReitAppraisal {
    pub j_reit_appraisal_appraisal_price: Option<i64>,
    pub j_reit_appraisal_cap_rate: Option<f64>,
    pub j_reit_appraisal_terminal_cap_rate: Option<f64>,
    pub j_reit_appraisal_discount_rate: Option<f64>,
    pub j_reit_appraisal_appraisal_company: Option<String>,
    pub j_reit_appraisal_net_operating_income: Option<i64>,
}

#[derive(FromQueryResult, Debug, PartialEq)]
pub struct JReitBuildingForCsv {
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
    #[sea_orm(nested)]
    pub second_term_before_financial: Option<SecondTermBeforeFinancial>,
    pub is_delisted: i8,
    pub corporation_id: String,
    pub corporation_name: String,
}

#[derive(serde::Serialize)]
struct CsvRow {
    #[serde(rename = "建物名")]
    name: String,

    #[serde(rename = "住所")]
    address: String,

    #[serde(rename = "最寄駅")]
    nearest_station: String,

    #[serde(rename = "投資法人")]
    j_reit_corporation_name: String,

    #[serde(rename = "アセットタイプ")]
    asset_type: String,

    #[serde(rename = "初回取得日")]
    acquisition_date: String,

    #[serde(rename = "取引価格[円]")]
    acquisition_price: String,

    #[serde(rename = "専有坪単価（取引価格÷取得時賃貸可能面積）［円／坪］")]
    acquisition_price_per_leasable_area: String,

    #[serde(rename = "取得時鑑定評価額［円］")]
    initial_appraised_price: String,

    #[serde(rename = "専有坪単価（取得時鑑定評価額÷取得時賃貸可能面積）［円／坪］")]
    initial_appraised_price_per_leasable_area: String,

    #[serde(rename = "取得時キャップレート（直接還元法）［%］")]
    initial_cap_rate: String,

    #[serde(rename = "取得時最終還元利回り(DCF）［%］")]
    initial_terminal_cap_rate: String,

    #[serde(rename = "取得時割引率（DCF）［%］")]
    initial_discounted_rate: String,

    #[serde(rename = "鑑定会社")]
    appraisal_company: String,

    #[serde(rename = "最新期末鑑定日")]
    appraisal_date: String,

    #[serde(rename = "最新期末鑑定評価額［円］")]
    appraisal_price: String,

    #[serde(rename = "最新期末鑑定評価額賃貸可能坪単価［円／坪］")]
    appraisal_price_per_leasable_area: String,

    #[serde(rename = "最新期末キャップレート［%］")]
    cap_rate: String,

    #[serde(rename = "最新期末最終還元利回り（DCF）［%］")]
    terminal_cap_rate: String,

    #[serde(rename = "最新期末割引率（DCF）［%］")]
    discount_rate: String,

    #[serde(rename = "最新期末稼働率［%］")]
    occupancy_rate: String,

    #[serde(rename = "最新期末賃貸先数")]
    number_of_tenants: String,

    #[serde(rename = "取引時鑑定NOI")]
    noi: String,

    #[serde(rename = "NOI年成長率［%］")]
    noi_trend: String,

    #[serde(rename = "NOIの上昇／下降の期間(from)")]
    fiscal_period_start_date: String,

    #[serde(rename = "NOIの上昇／下降の期間(to)")]
    fiscal_period_end_date: String,

    #[serde(rename = "取引種類")]
    transaction_type: String,

    #[serde(rename = "譲渡日")]
    transfer_date: String,

    #[serde(rename = "譲渡金額")]
    transfer_price: String,

    #[serde(rename = "譲渡先")]
    transferee_company: String,

    #[serde(rename = "売主")]
    initial_acquisition_seller: String,

    #[serde(rename = "受託行")]
    trustee: String,

    #[serde(rename = "PM会社")]
    property_manager: String,

    #[serde(rename = "調査会社")]
    inspection_company: String,

    #[serde(rename = "竣工年")]
    completed_year_month: String,

    #[serde(rename = "敷地面積［㎡］")]
    land_m2: String,

    #[serde(rename = "延床面積［㎡］")]
    gross_floor_area_m2: String,

    #[serde(rename = "賃貸可能面積［㎡］")]
    total_leasable_area_m2: String,

    #[serde(rename = "賃貸可能面積（建物全体）［㎡］")]
    building_exclusive_area_total_m2: String,

    #[serde(rename = "敷地面積［坪］")]
    land: String,

    #[serde(rename = "延床面積［坪］")]
    gross_floor_area: String,

    #[serde(rename = "賃貸可能面積［坪］")]
    total_leasable_area: String,

    #[serde(rename = "賃貸可能面積（建物全体）［坪］")]
    building_exclusive_area_total: String,

    #[serde(rename = "構造")]
    structure: String,

    #[serde(rename = "地上階")]
    groundfloor: String,

    #[serde(rename = "地下階")]
    basement: String,

    #[serde(rename = "戸数・客室数")]
    leasable_units: String,

    #[serde(rename = "間取")]
    floor_plan: String,

    #[serde(rename = "保有状況")]
    ownership_status: String,
}

const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

fn make_csv_row(record: &JReitBuildingForCsv) -> CsvRow {
    let j_reit_building = &record.j_reit_building;
    let initial_acquisition = &record.initial_acquisition;
    let j_reit_appraisal = &record.j_reit_appraisal;
    let latest_transaction = &record.latest_transaction;
    let latest_transfer = latest_transaction.as_ref().filter(|transaction| {
        transaction.transaction_category == Some(TransactionCategory::FullTransfer as i8)
    });
    let latest_financial = &record.latest_financial;
    let second_term_before_financial = &record.second_term_before_financial;

    let noi_trend = calc_noi_trend(
        latest_financial
            .as_ref()
            .and_then(|a| a.net_operating_income),
        second_term_before_financial
            .as_ref()
            .and_then(|a| a.second_term_before_financial_net_operating_income),
    );
    let building_exclusive_area_total = latest_transaction
        .as_ref()
        .map(|a| calc_area_per_ratio(a.total_leasable_area, a.building_ownership_ratio))
        .unwrap_or_default();

    CsvRow {
        name: j_reit_building.name.clone(),
        address: j_reit_building.address.clone().unwrap_or_default(),
        nearest_station: j_reit_building.nearest_station.clone().unwrap_or_default(),
        j_reit_corporation_name: format_j_reit_corporation_name(
            &record.corporation_name,
            record.is_delisted == 1,
        ),
        asset_type: AssetType::from(j_reit_building).format(),
        acquisition_date: initial_acquisition
            .as_ref()
            .and_then(|a| a.initial_acquisition_transaction_date)
            .map(|date| format_native_date(&date))
            .unwrap_or_default(),
        acquisition_price: initial_acquisition
            .as_ref()
            .and_then(|a| a.initial_acquisition_transaction_price)
            .map(format_integer)
            .unwrap_or_default(),
        acquisition_price_per_leasable_area: if let Some(initial_acquisition) = initial_acquisition
        {
            calc_price_per_area(
                initial_acquisition.initial_acquisition_transaction_price,
                initial_acquisition.initial_acquisition_leasable_area,
            )
        } else {
            "".to_string()
        },
        initial_appraised_price: j_reit_appraisal
            .as_ref()
            .and_then(|a| a.j_reit_appraisal_appraisal_price)
            .map(format_integer)
            .unwrap_or_default(),
        initial_appraised_price_per_leasable_area: if let (
            Some(j_reit_appraisal),
            Some(initial_acquisition),
        ) = (j_reit_appraisal, initial_acquisition)
        {
            calc_price_per_area(
                j_reit_appraisal.j_reit_appraisal_appraisal_price,
                initial_acquisition.initial_acquisition_leasable_area,
            )
        } else {
            "".to_string()
        },
        initial_cap_rate: j_reit_appraisal
            .as_ref()
            .and_then(|a| a.j_reit_appraisal_cap_rate)
            .map(format_float)
            .unwrap_or_default(),
        initial_terminal_cap_rate: j_reit_appraisal
            .as_ref()
            .and_then(|a| a.j_reit_appraisal_terminal_cap_rate)
            .map(format_float)
            .unwrap_or_default(),
        initial_discounted_rate: j_reit_appraisal
            .as_ref()
            .and_then(|a| a.j_reit_appraisal_discount_rate)
            .map(format_float)
            .unwrap_or_default(),
        appraisal_company: j_reit_appraisal
            .as_ref()
            .and_then(|a| a.j_reit_appraisal_appraisal_company.clone())
            .unwrap_or_default(),
        appraisal_date: latest_financial
            .as_ref()
            .map(|a| format_native_date(&a.fiscal_period_end_date))
            .unwrap_or_default(),
        appraisal_price: latest_financial
            .as_ref()
            .and_then(|a| a.appraisal_price)
            .map(format_integer)
            .unwrap_or_default(),
        appraisal_price_per_leasable_area: if let Some(latest_financial) = latest_financial {
            calc_price_per_area(
                latest_financial.appraisal_price,
                latest_transaction
                    .as_ref()
                    .and_then(|a| a.total_leasable_area),
            )
        } else {
            "".to_string()
        },
        cap_rate: latest_financial
            .as_ref()
            .and_then(|c| c.cap_rate)
            .map(format_float)
            .unwrap_or_default(),
        terminal_cap_rate: latest_financial
            .as_ref()
            .and_then(|f| f.terminal_cap_rate.map(format_float))
            .unwrap_or_default(),
        discount_rate: latest_financial
            .as_ref()
            .and_then(|f| f.discount_rate.map(format_float))
            .unwrap_or_default(),
        occupancy_rate: latest_financial
            .as_ref()
            .and_then(|f| f.occupancy_rate.map(format_float))
            .unwrap_or_default(),
        number_of_tenants: latest_financial
            .as_ref()
            .and_then(|f| f.number_of_tenants.map(format_integer))
            .unwrap_or_default(),
        noi: j_reit_appraisal
            .as_ref()
            .and_then(|a| a.j_reit_appraisal_net_operating_income)
            .map(format_integer)
            .unwrap_or_default(),
        noi_trend,
        fiscal_period_start_date: latest_financial
            .as_ref()
            .map(|f| format_native_date(&f.fiscal_period_start_date))
            .unwrap_or_default(),
        fiscal_period_end_date: latest_financial
            .as_ref()
            .map(|f| format_native_date(&f.fiscal_period_end_date))
            .unwrap_or_default(),
        // NOTE: 保有状況と情報量は同じだが、個社対応で追加している (JREIT-35)
        transaction_type: if latest_transfer.and_then(|t| t.transaction_date).is_some() {
            "売却".to_string()
        } else {
            "取得".to_string()
        },
        transfer_date: latest_transfer
            .and_then(|t| t.transaction_date)
            .map(|date| format_native_date(&date))
            .unwrap_or_default(),
        transfer_price: latest_transfer
            .and_then(|t| t.transaction_price)
            .map(format_integer)
            .unwrap_or_default(),
        transferee_company: latest_transfer
            .and_then(|t| t.transaction_partner.clone())
            .unwrap_or_default(),
        initial_acquisition_seller: initial_acquisition
            .as_ref()
            .and_then(|a| a.initial_acquisition_transaction_partner.clone())
            .unwrap_or_default(),
        trustee: initial_acquisition
            .as_ref()
            .and_then(|a| a.initial_acquisition_trustee.clone())
            .unwrap_or_default(),
        property_manager: initial_acquisition
            .as_ref()
            .and_then(|a| a.initial_acquisition_property_manager.clone())
            .unwrap_or_default(),
        inspection_company: initial_acquisition
            .as_ref()
            .and_then(|a| a.initial_acquisition_pml_assessment_company.clone())
            .unwrap_or_default(),
        completed_year_month: format_completed_year_month(
            &j_reit_building.completed_year,
            &j_reit_building.completed_month,
        ),
        land_m2: j_reit_building
            .land
            .map(format_area_f64_in_square_meter)
            .unwrap_or_default(),
        gross_floor_area_m2: j_reit_building
            .gross_floor_area
            .map(format_area_f64_in_square_meter)
            .unwrap_or_default(),
        total_leasable_area_m2: latest_transaction
            .as_ref()
            .and_then(|a| a.total_leasable_area)
            .map(format_area_f64_in_square_meter)
            .unwrap_or_default(),
        building_exclusive_area_total_m2: building_exclusive_area_total
            .map(format_area_f64_in_square_meter)
            .unwrap_or_default(),
        land: j_reit_building.land.map(format_float).unwrap_or_default(),
        gross_floor_area: j_reit_building
            .gross_floor_area
            .map(format_float)
            .unwrap_or_default(),
        total_leasable_area: latest_transaction
            .as_ref()
            .and_then(|a| a.total_leasable_area)
            .map(format_float)
            .unwrap_or_default(),
        building_exclusive_area_total: building_exclusive_area_total
            .map(format_float)
            .unwrap_or_default(),
        structure: j_reit_building.structure.clone().unwrap_or_default(),
        groundfloor: j_reit_building
            .groundfloor
            .map(format_integer)
            .unwrap_or_default(),
        basement: j_reit_building
            .basement
            .map(format_integer)
            .unwrap_or_default(),
        leasable_units: latest_transaction
            .as_ref()
            .and_then(|a| a.leasable_units)
            .map(format_integer)
            .unwrap_or_default(),
        floor_plan: j_reit_building.floor_plan.clone().unwrap_or_default(),
        ownership_status: if latest_transfer
            .as_ref()
            .and_then(|t| t.transaction_date)
            .is_some()
        {
            "譲渡済み".to_string()
        } else {
            "保有中".to_string()
        },
    }
}

fn generate_csv(data: Vec<JReitBuildingForCsv>) -> anyhow::Result<String> {
    let mut wtr = Writer::from_writer(BOM.into_iter().collect::<Vec<_>>());
    for record in data {
        let csv_row = make_csv_row(&record);
        wtr.serialize(csv_row)?;
    }

    let result = String::from_utf8(wtr.into_inner()?)?;
    Ok(result)
}

fn calc_price_per_area(price: Option<i64>, area: Option<f64>) -> String {
    match (price, area) {
        (Some(appraisal_price), Some(leasable_area)) if leasable_area != 0.0 => {
            let result = appraisal_price as f64 / leasable_area;
            format_float(result)
        }
        _ => "".to_string(),
    }
}

fn calc_area_per_ratio(area: Option<f64>, ratio: Option<f64>) -> Option<f64> {
    match (area, ratio) {
        (Some(area), Some(ratio)) if ratio != 0.0 => {
            let result = area / ratio * 100.0;
            Some(result)
        }
        _ => None,
    }
}

fn calc_noi_trend(noi1: Option<i64>, noi2: Option<i64>) -> String {
    match (noi1, noi2) {
        (Some(noi1), Some(noi2)) if noi2 != 0 => {
            format_float((noi1 as f64 / noi2 as f64 - 1.0) * 100.0)
        }
        _ => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sql_entities::j_reit_buildings;

    fn create_test_j_reit_building_model() -> j_reit_buildings::Model {
        j_reit_buildings::Model {
            id: "test_building_id".to_string(),
            name: "テストビル".to_string(),
            address: Some("東京都千代田区".to_string()),
            nearest_station: Some("東京駅".to_string()),
            is_office: 1,
            is_residential: 0,
            is_retail: 0,
            is_logistic: 0,
            is_hotel: 0,
            is_health_care: 0,
            is_other: 0,
            groundfloor: Some(7),
            basement: Some(1),
            office_building_id: None,
            residential_building_id: None,
            city_id: 1,
            latitude: 35.0,
            longitude: 135.0,
            completed_year: Some(2023),
            completed_month: Some(5),
            gross_floor_area: Some(1000.0),
            structure: Some("鉄筋コンクリート".to_string()),
            floor_plan: Some("1LDK".to_string()),
            land: Some(200.0),
            building_coverage_ratio: Some(60.0),
            floor_area_ratio: Some(200.0),
            snowflake_deleted: 0,
        }
    }

    fn create_test_j_reit_appraisal_model() -> JReitAppraisal {
        JReitAppraisal {
            j_reit_appraisal_appraisal_price: Some(1000000000),
            j_reit_appraisal_cap_rate: Some(5.0),
            j_reit_appraisal_terminal_cap_rate: Some(5.5),
            j_reit_appraisal_discount_rate: Some(6.0),
            j_reit_appraisal_appraisal_company: Some("テスト鑑定会社".to_string()),
            j_reit_appraisal_net_operating_income: Some(8000000),
        }
    }

    fn create_test_latest_acquisition_model() -> LatestTransaction {
        LatestTransaction {
            total_leasable_area: Some(2000.0),
            transaction_category: Some(TransactionCategory::FullTransfer as i8),
            transaction_date: Some(NaiveDate::from_ymd_opt(2023, 3, 1).expect("Invalid date")),
            transaction_price: Some(1234567890),
            transaction_partner: Some("テスト譲渡先".to_string()),
            leasable_units: Some(100),
            building_ownership_ratio: Some(50.0),
        }
    }

    fn create_test_initial_acquisition_model() -> InitialAcquisition {
        InitialAcquisition {
            initial_acquisition_transaction_date: Some(
                NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            ),
            initial_acquisition_transaction_price: Some(1000000000),
            initial_acquisition_transaction_partner: Some("テスト売主".to_string()),
            initial_acquisition_trustee: Some("テスト信託銀行".to_string()),
            initial_acquisition_property_manager: Some("テストPM会社".to_string()),
            initial_acquisition_pml_assessment_company: Some("テストPM調査会社".to_string()),
            initial_acquisition_leasable_area: Some(2000.0),
        }
    }

    fn create_test_latest_financial_model() -> LatestFinancial {
        LatestFinancial {
            cap_rate: Some(5.0),
            terminal_cap_rate: Some(5.3),
            discount_rate: Some(6.0),
            fiscal_period_start_date: NaiveDate::from_ymd_opt(2025, 1, 1).expect("Invalid date"),
            fiscal_period_end_date: NaiveDate::from_ymd_opt(2025, 12, 31).expect("Invalid date"),
            net_operating_income: Some(10000000),
            appraisal_price: Some(500000000),
            occupancy_rate: Some(95.5),
            number_of_tenants: Some(42),
        }
    }

    fn create_test_second_term_before_financial_model() -> SecondTermBeforeFinancial {
        SecondTermBeforeFinancial {
            second_term_before_financial_net_operating_income: Some(8900000),
        }
    }

    #[test]
    fn test_make_csv_row() {
        let test_building = create_test_j_reit_building_model();
        let test_initial_acquisition = create_test_initial_acquisition_model();
        let test_appraisal = create_test_j_reit_appraisal_model();
        let test_latest_acquisition = create_test_latest_acquisition_model();
        let test_latest_financial = create_test_latest_financial_model();
        let test_second_term_before_financial = create_test_second_term_before_financial_model();

        let record = JReitBuildingForCsv {
            j_reit_building: test_building,
            initial_acquisition: Some(test_initial_acquisition),
            j_reit_appraisal: Some(test_appraisal),
            latest_transaction: Some(test_latest_acquisition),
            latest_financial: Some(test_latest_financial),
            second_term_before_financial: Some(test_second_term_before_financial),
            is_delisted: 0,
            corporation_id: "test_corporation_id".to_string(),
            corporation_name: "テスト投資法人".to_string(),
        };

        let csv_row = make_csv_row(&record);

        assert_eq!(csv_row.name, "テストビル");
        assert_eq!(csv_row.address, "東京都千代田区");
        assert_eq!(csv_row.nearest_station, "東京駅");
        assert_eq!(csv_row.j_reit_corporation_name, "テスト投資法人");
        assert_eq!(csv_row.asset_type, "オフィス");
        assert_eq!(csv_row.acquisition_date, "2023/01/01");
        assert_eq!(csv_row.acquisition_price, "1000000000");
        assert_eq!(csv_row.acquisition_price_per_leasable_area, "500000.00");
        assert_eq!(csv_row.initial_appraised_price, "1000000000");
        assert_eq!(
            csv_row.initial_appraised_price_per_leasable_area,
            "500000.00"
        );
        assert_eq!(csv_row.initial_cap_rate, "5.00");
        assert_eq!(csv_row.initial_terminal_cap_rate, "5.50");
        assert_eq!(csv_row.initial_discounted_rate, "6.00");
        assert_eq!(csv_row.appraisal_company, "テスト鑑定会社");
        assert_eq!(csv_row.appraisal_date, "2025/12/31");
        assert_eq!(csv_row.appraisal_price, "500000000");
        assert_eq!(csv_row.appraisal_price_per_leasable_area, "250000.00");
        assert_eq!(csv_row.cap_rate, "5.00");
        assert_eq!(csv_row.terminal_cap_rate, "5.30");
        assert_eq!(csv_row.discount_rate, "6.00");
        assert_eq!(csv_row.occupancy_rate, "95.50");
        assert_eq!(csv_row.number_of_tenants, "42");
        assert_eq!(csv_row.noi, "8000000");
        assert_eq!(csv_row.noi_trend, "12.36");
        assert_eq!(csv_row.fiscal_period_start_date, "2025/01/01");
        assert_eq!(csv_row.fiscal_period_end_date, "2025/12/31");
        assert_eq!(csv_row.transaction_type, "売却");
        assert_eq!(csv_row.transfer_date, "2023/03/01");
        assert_eq!(csv_row.transfer_price, "1234567890");
        assert_eq!(csv_row.transferee_company, "テスト譲渡先");
        assert_eq!(csv_row.initial_acquisition_seller, "テスト売主");
        assert_eq!(csv_row.trustee, "テスト信託銀行");
        assert_eq!(csv_row.property_manager, "テストPM会社");
        assert_eq!(csv_row.inspection_company, "テストPM調査会社");
        assert_eq!(csv_row.completed_year_month, "2023/05/01");
        assert_eq!(csv_row.land_m2, "661.16");
        assert_eq!(csv_row.gross_floor_area_m2, "3305.79");
        assert_eq!(csv_row.total_leasable_area_m2, "6611.57");
        assert_eq!(csv_row.building_exclusive_area_total_m2, "13223.14");
        assert_eq!(csv_row.land, "200.00");
        assert_eq!(csv_row.gross_floor_area, "1000.00");
        assert_eq!(csv_row.total_leasable_area, "2000.00");
        assert_eq!(csv_row.building_exclusive_area_total, "4000.00");
        assert_eq!(csv_row.structure, "鉄筋コンクリート");
        assert_eq!(csv_row.groundfloor, "7");
        assert_eq!(csv_row.basement, "1");
        assert_eq!(csv_row.leasable_units, "100");
        assert_eq!(csv_row.floor_plan, "1LDK");
        assert_eq!(csv_row.ownership_status, "譲渡済み");
    }

    #[test]
    fn test_calc_price_per_area() {
        assert_eq!(calc_price_per_area(Some(100), Some(10.0)), "10.00");
        assert_eq!(calc_price_per_area(Some(100), Some(15.0)), "6.67");
        assert_eq!(calc_price_per_area(Some(100), Some(0.0)), "");
        assert_eq!(calc_price_per_area(Some(100), None), "");
        assert_eq!(calc_price_per_area(None, Some(10.0)), "");
        assert_eq!(calc_price_per_area(None, None), "");
    }
}
