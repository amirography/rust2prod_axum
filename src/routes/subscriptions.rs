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

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
pub async fn subscribe(
    State(connection): State<Arc<PgPool>>,
    Form(form): Form<FormData>,
) -> Response<String> {
    let request_id = uuid::Uuid::new_v4();
    let _request_span = tracing::info_span!(
        "Adding checking health",
        %request_id
    );

    let new_subscriber = match form.try_into() {
        Ok(email) => email,
        Err(e) => {
            return http::Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body(format!("invalid : {}", e))
                .unwrap();
        }
    };

    let _query_span = tracing::info_span!("Saving new subscriber details in the database");
    // tracing::info!(
    //     "Adding '{}' '{}' as a new subscriber.",
    //     form.email,
    //     form.name
    // );

    tracing::info!("Saving new subscriber details in the database");

    match insert_subscriber(&connection, &new_subscriber).await {
        Ok(_) => Response::new(String::from("Success")),
        Err(e) => http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Failed to execute query: {}", e))
            .unwrap(),
    }
}

#[tracing::instrument(
    name = "saving new subscriber details in the database",
    skip(new_subscriber, connection)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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

// pub fn parse_subscriber(form: &FormData) -> Result<NewSubscriber, String> {}
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(&value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(NewSubscriber { email, name })
    }
}
