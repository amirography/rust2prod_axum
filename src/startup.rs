use std::sync::Arc;

use axum::{self, routing, Router, Server};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace;

use crate::routes;

pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<(), std::io::Error> {
    // making a new dbpool to have concurrent access to the making databases
    let shared_state = Arc::new(db_pool);

    // making a enw router
    let app = Router::new()
        .route("/health_check", routing::get(routes::health_check))
        .route(
            "/subscriptions",
            routing::post(routes::subscribe).with_state(shared_state),
        )
        .layer(tower::ServiceBuilder::new().layer(trace::TraceLayer::new_for_http()));

    // serving that router
    Server::from_tcp(listener.into_std().expect("problem converting"))
        .expect("shit")
        .serve(app.into_make_service())
        .await
        .expect("shit happened");

    Ok(())
}
