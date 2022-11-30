use axum::{self, http, routing, Router, Server};
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    // tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health_check", routing::get(health_check))
        .route("/subscriptions", routing::post(subscribe));

    Server::from_tcp(listener.into_std().expect("problem converting"))
        .expect("shit")
        .serve(app.into_make_service())
        .await
        .expect("shit happened");

    Ok(())
}

async fn health_check() -> http::response::Response<String> {
    http::response::Response::new(String::from(""))
}
async fn subscribe(_form: axum::Form<FormData>) -> http::response::Response<String> {
    http::response::Response::new(String::from(""))
}

#[allow(unused)]
#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}
