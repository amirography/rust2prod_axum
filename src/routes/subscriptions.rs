use axum::http::response::Response;

pub async fn subscribe(_form: axum::Form<FormData>) -> Response<String> {
    Response::new(String::from(""))
}

#[allow(unused)]
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
