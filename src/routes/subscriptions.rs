use std::sync::Arc;

use axum::{
    extract::State,
    http::{self, response::Response},
    Form,
};
use axum_macros::debug_handler;
use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};

#[debug_handler]
pub async fn subscribe(
    State(connection): State<Arc<PgPool>>,
    Form(form): Form<FormData>,
) -> Response<String> {
    match sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.as_ref())
    .await
    {
        Ok(_) => Response::new(String::from("Success")),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Failed to execute query: {}", e))
                .unwrap()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
