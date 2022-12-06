use std::sync::Arc;

use axum::{self, routing, Router, Server};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::routes;

pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<(), std::io::Error> {
    // tracing_subscriber::fmt::init();

    let shared_state = Arc::new(db_pool);
    let app = Router::new()
        .route("/health_check", routing::get(routes::health_check))
        .route(
            "/subscriptions",
            routing::post(routes::subscribe).with_state(shared_state),
        );

    Server::from_tcp(listener.into_std().expect("problem converting"))
        .expect("shit")
        .serve(app.into_make_service())
        .await
        .expect("shit happened");

    Ok(())
}
