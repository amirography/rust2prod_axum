use std::sync::Arc;

use axum::{self, routing, Router, Server};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use tokio::net::TcpListener;
use tower_http::trace;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes,
};

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<(), std::io::Error> {
    // making a new dbpool to have concurrent access to the making databases
    let shared_state = Arc::new(db_pool);
    let email_client = Arc::new(email_client);

    // making a enw router
    let app = Router::new()
        .route("/health_check", routing::get(routes::health_check))
        .route(
            "/subscriptions",
            routing::post(routes::subscribe)
                .with_state(shared_state)
                .with_state(email_client),
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

// pub async fn build(configuration: &Settings) -> Result<(), std::io::Error> {
//     let connection_pool = get_connection_pool(&configuration.database);

//     let sender_email = configuration
//         .email_client
//         .sender()
//         .expect("invalid sender email address");

//     let timeout = configuration.email_client.timeout();

//     let email_client = EmailClient::new(
//         &configuration.email_client.base_url,
//         sender_email,
//         configuration.email_client.authorization_token.to_owned(),
//         timeout,
//     );

//     let address = format!(
//         "{}:{}",
//         configuration.application.host, configuration.application.port
//     );
//     let listener = TcpListener::bind(address).await?;
//     run(listener, connection_pool, email_client).await
// }

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&configuration.connection_string_with_db())
        .expect("Connection to database failed")
}

pub struct Application {
    port: u16,
    listener: TcpListener,
    connection_pool: Pool<Postgres>,
    email_client: EmailClient,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url.to_owned(),
            sender_email,
            configuration.email_client.authorization_token.to_owned(),
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(&address).await?;

        let port = listener.local_addr().unwrap().port();

        Ok(Self {
            port,
            listener,
            connection_pool,
            email_client,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.connection_pool
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        run(self.listener, self.connection_pool, self.email_client).await
    }
}
