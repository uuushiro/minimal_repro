use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/hello")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello internal-api")
}

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    actix_web::rt::System::new().block_on(async move {
        HttpServer::new(move || App::new().service(hello))
            .bind(("0.0.0.0", 8081))?
            .run()
            .await
    })?;

    Ok(())
}
