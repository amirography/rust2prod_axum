//! tests/health_check.rs
use once_cell::sync::Lazy;
use rust2prod_amir::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::test]
async fn health_check_works() {
    let app = tokio::spawn(spawn_app());

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/health_check",
            app.await.expect("failed to make an app").address
        ))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("problem creating a client to test");
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = {
        client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    };

    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT email , name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400, bad request when payload was {}.",
            error_message
        );
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_filter_level = String::from("info");
    let subscriber_name = String::from("test");
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_filter_level,
            std::io::stdout,
        );
        init_subscriber(subscriber);
    } else {
        let subscriber =
            get_subscriber(subscriber_name, default_filter_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let listener = match TcpListener::bind(format!("127.0.0.1:0")).await {
        Ok(o) => o,
        Err(e) => {
            tracing::error!("could not bind to address: {:?} ", e);
            panic!("{e} : {}", &configuration.application_port);
        }
    };

    let address = format!(
        "http://127.0.0.1:{}",
        listener
            .local_addr()
            .expect("Could not get the local address")
            .port()
    );
    let connection_pool = configure_database(&configuration.database).await;
    let server = run(listener, connection_pool.clone());
    let _server = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}"; "#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
