//! tests/health_check.rs
use rust2prod_amir::run;
use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let a = tokio::task::spawn(spawn_app()).await.expect("fuck me");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", a))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("problem creating a client to test");
    let body = "name=le%20guin&email=urusla_le_guin%40gmail.com";

    let response = {
        client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    };

    assert!(response.status().is_success());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=urusla_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
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

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("shiiiiiiiiiiiiiit");

    let b = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().expect("good god!").port()
    );
    tokio::spawn(run(listener));
    b
}
