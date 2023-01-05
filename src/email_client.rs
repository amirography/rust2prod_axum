//! src/email_client.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
    authorization_token: String,
}

impl EmailClient {
    pub fn new(
        base_url: impl AsRef<str>,
        sender: SubscriberEmail,
        authorization_token: String,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url: base_url.as_ref().to_owned(),
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(&url)
            .json(&request_body)
            .header("X-Postmark-Server-token", &self.authorization_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use fake::faker::{internet, lorem};

    use fake::{Fake, Faker};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    /// generate a random email subject
    fn subject() -> String {
        lorem::en::Sentence(1..2).fake()
    }

    /// generate a random email content
    fn content() -> String {
        lorem::en::Paragraph(1..10).fake()
    }

    /// generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(internet::en::SafeEmail().fake()).unwrap()
    }

    /// get a test instance of "EmailClient"
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Faker.fake(),
            std::time::Duration::from_millis(200),
        )
    }

    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            match result {
                Ok(body) => {
                    body.get("from").is_some()
                        && body.get("to").is_some()
                        && body.get("subject").is_some()
                        && body.get("html_body").is_some()
                        && body.get("text_body").is_some()
                }

                Err(_) => false,
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(wiremock::matchers::header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert

        // Mock expectations are checked on drop
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(wiremock::matchers::any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        claims::assert_err!(outcome);

        // Mock expectations are checked on drop
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(wiremock::matchers::any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        claims::assert_err!(outcome);

        // Mock expectations are checked on drop
    }
}
