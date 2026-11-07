mod api;
mod application;
mod domain;
mod infrastructure;

use infrastructure::couch::CouchClient;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let couch = Arc::new(CouchClient::from_env());
    let app = api::router(couch);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("API escuchando en http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}