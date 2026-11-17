mod api;
mod application;
mod domain;
mod infrastructure;
mod telemetry;

use infrastructure::couch::CouchClient;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let couch = Arc::new(CouchClient::from_env());
    let metricas = Arc::new(telemetry::Metricas::default());
    let app = api::router(couch, metricas);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("API escuchando en http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}