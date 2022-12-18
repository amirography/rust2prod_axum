use std::sync::Arc;

use axum::{
    extract::State,
    http::{self, response::Response},
    Form,
};
use sqlx::{
    types::{chrono::Utc, Uuid},
    PgPool,
};

#[tracing::instrument(
     name = "adding a new subscriber",
     skip(form , connection),
     fields (
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
 )]
pub async fn subscribe(
    State(connection): State<Arc<PgPool>>,
    Form(form): Form<FormData>,
) -> Response<String> {
    let request_id = uuid::Uuid::new_v4();
    let _request_span = tracing::info_span!(
        "Adding checking health",
        %request_id
    );

    let _query_span = tracing::info_span!("Saving new subscriber details in the database");
    tracing::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );

    tracing::info!("Saving new subscriber details in the database");

    match insert_subscriber(&connection, &form).await {
        Ok(_) => Response::new(String::from("Success")),
        Err(e) => http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to execute query: {}", e))
            .unwrap(),
    }
}

#[tracing::instrument(
    name = "saving new subscriber details in the database",
    skip(form, connection)
)]
pub async fn insert_subscriber(connection: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}
