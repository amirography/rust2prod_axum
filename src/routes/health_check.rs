use axum::http::response::Response;

pub async fn health_check() -> Response<String> {
    Response::new(String::from(""))
}
