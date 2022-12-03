//! src/lib.rs

pub mod configuration;
pub mod routes;
pub mod startup;

use axum::{self, http, routing, Router, Server};
use sqlx::PgConnection;
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener, connection: PgConnection) -> Result<(), std::io::Error> {
    // tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscribe))
        .with_state(connection);

    Server::from_tcp(listener.into_std().expect("problem converting"))
        .expect("shit")
        .serve(app.into_make_service())
        .await
        .expect("shit happened");

    Ok(())
}
