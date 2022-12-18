use axum::http::response::Response;

pub async fn health_check() -> Response<String> {
    let request_id = uuid::Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding checking health",
        %request_id
    );
    let _request_span_guard = request_span.enter();

    tracing::info!("Health check was checked.");
    Response::new(String::from(""))
}
