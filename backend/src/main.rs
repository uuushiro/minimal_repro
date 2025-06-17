use actix_web::{dev::Service, route, web, App, HttpServer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use anyhow::Context;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use auth0_jwt_validator::Auth0JwtValidator;
use graphql::{metadata::get_user_metadata, schema, Schema};
use log::LevelFilter;
use opentelemetry_bootstrap::otel;
use rest::data_hub_client::DataHubClient;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

#[route("/api/graphql", method = "GET", method = "POST")]
#[otel(
    name = "graphql.execute",
    fields(
        span.kind = "server",
        operation.name = "graphql.execute",
        resource.name = get_graphql_operation_name(&request),
    )
)]
async fn graphql_route(
    schema: web::Data<Schema>,
    request: GraphQLRequest,
    auth: BearerAuth,
    validator: web::Data<Auth0JwtValidator>,
) -> actix_web::Result<GraphQLResponse> {
    let user_metadata = get_user_metadata(auth, validator).await?;
    let auth_id = user_metadata.auth0_id.0.clone();
    let request = request
        .into_inner()
        .data(user_metadata.organization_id)
        .data(user_metadata.auth0_id)
        .data(user_metadata.roles);

    let query = serde_json::Value::from(request.query.as_str());
    let variables = request.variables.clone().into_value().into_json()?;
    let email = user_metadata.email.clone();

    let response = schema.execute(request).await;

    for error in &response.errors {
        sentry::with_scope(
            |scope| {
                scope.set_user(Some(sentry::User {
                    id: Some(auth_id.clone()),
                    email: Some(email.clone()),
                    ..Default::default()
                }));
                scope.set_extra("query", query.to_owned());
                scope.set_extra("variables", variables.clone());
            },
            || sentry::capture_message(&error.to_string(), sentry::protocol::Level::Error),
        );
    }

    Ok(response.into())
}

async fn connect_db() -> DatabaseConnection {
    let user = std::env::var("MY_SQL_USER").expect("MY_SQL_USER not set");
    let password = std::env::var("MY_SQL_PASSWORD").expect("MY_SQL_PASSWORD not set");
    let host = std::env::var("MY_SQL_HOST").expect("MY_SQL_HOST not set");
    let database = std::env::var("MY_SQL_DATABASE").expect("MY_SQL_DATABASE not set");
    let url = format!("mysql://{user}:{password}@{host}/{database}");

    let mut opt = ConnectOptions::new(url);
    opt.sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    Database::connect(opt)
        .await
        .expect("Failed to connect to database")
}

fn setup_data_hub_client() -> anyhow::Result<DataHubClient> {
    let endpoint = std::env::var("DATA_HUB_GRAPHQL_ENDPOINT")
        .context("DATA_HUB_GRAPHQL_ENDPOINT is not set")?;
    Ok(DataHubClient::new(endpoint))
}

fn setup_auth0_validator() -> anyhow::Result<Auth0JwtValidator> {
    let authority = std::env::var("AUTH0_ISSUER_URL").context("AUTH0_ISSUER_URL not set")?;
    let validator = Auth0JwtValidator::new(&authority)?;
    Ok(validator)
}

fn get_graphql_operation_name(request: &GraphQLRequest) -> String {
    request
        .0
        .operation_name
        .clone()
        .unwrap_or_else(|| "anonymous_operation".to_string())
}

fn main() -> anyhow::Result<()> {
    match opentelemetry_bootstrap::init() {
        Ok(_) => tracing::info!("OpenTelemetry initialized successfully"),
        Err(err) => {
            tracing::warn!(
                error = %err,
                "Failed to initialize OpenTelemetry, continuing without distributed tracing"
            );
        }
    }

    if let Ok(env_name) = std::env::var("ENV_NAME") {
        let _guard = sentry::init((
            "https://7c29157dacc3a0248e49ca7478af56b0@o383617.ingest.us.sentry.io/4508120230002688",
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(env_name.into()),
                ..Default::default()
            },
        ));
    }
    std::env::set_var("RUST_BACKTRACE", "1");

    let client = setup_data_hub_client()?;
    let validator = setup_auth0_validator()?;
    actix_web::rt::System::new().block_on(async move {
        let db = connect_db().await;
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(schema(db.clone())))
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(client.clone()))
                .app_data(web::Data::new(validator.clone()))
                .service(graphql_route)
                .service(rest::api::buildings::export_csv_v2::csv_route)
                .service(rest::api::buildings::export_excel_v2::excel_route)
                .service(rest::api::health::health_check)
                .wrap(sentry_actix::Sentry::new())
                .wrap_fn(|req, srv| {
                    let fut = srv.call(req);
                    async move {
                        let res = fut.await?;
                        if res.status().is_server_error() {
                            let msg = format!(
                                "HTTP {} error at {} {}",
                                res.status().as_u16(),
                                res.request().method(),
                                res.request().path()
                            );
                            // 500エラーをSentryに送信
                            sentry::capture_message(&msg, sentry::protocol::Level::Error);
                        }
                        Ok(res)
                    }
                })
        })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
    })?;

    Ok(())
}
