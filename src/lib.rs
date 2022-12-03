//! src/lib.rs

pub mod configuration;
pub mod routes;
pub mod startup;

use axum::{self, http, routing, Router, Server};
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    // tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscribe));

    Server::from_tcp(listener.into_std().expect("problem converting"))
        .expect("shit")
        .serve(app.into_make_service())
        .await
        .expect("shit happened");

    Ok(())
}
