use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde_json::json;

#[get("/api/rest/health")]
pub async fn health_check(db: web::Data<DatabaseConnection>) -> impl Responder {
    let db_ref = db.get_ref();

    let result = db_ref
        .execute(Statement::from_string(
            DatabaseBackend::MySql,
            "SELECT 1".to_string(),
        ))
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "ok",
            "message": "Service is healthy",
            "database": "healthy"
        })),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(json!({
                "status": "error",
                "message": "Service is unhealthy",
                "database": "unhealthy"
            }))
        }
    }
}
