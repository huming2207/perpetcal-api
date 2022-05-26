mod ical_json;
mod ical_endpoint;
mod error_type;
use actix_web::{HttpServer, App, web};
use ical_endpoint::ical_handler;

#[tokio::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/ical", web::post().to(ical_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}