use std::time::Duration;

use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_part: &str,
        text_part: &str,
    ) -> Result<(), reqwest::Error> {
        let email = SendEmailRequest {
            from: EmailUser {
                email: self.sender.as_ref(),
                name: "Mark",
            },
            to: vec![EmailUser {
                email: recipient.as_ref(),
                name: "Subscriber",
            }],
            subject,
            text_part,
            html_part,
        };

        let req_body = SendEmailRequestFormat {
            messages: vec![email],
        };

        self.http_client
            .post(&self.base_url)
            .json(&req_body)
            .header(
                "Authorization",
                format!("Basic {}", self.authorization_token.expose_secret()),
            )
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestFormat<'a> {
    messages: Vec<SendEmailRequest<'a>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: EmailUser<'a>,
    to: Vec<EmailUser<'a>>,
    subject: &'a str,
    text_part: &'a str,
    #[serde(rename(serialize = "HTMLPart"))]
    html_part: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmailUser<'a> {
    email: &'a str,
    name: &'a str,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{any, header, header_exists, method},
        Mock, MockServer, Request, ResponseTemplate,
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..5).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            Duration::from_millis(500),
        )
    }

    struct SendEmailRequestFormatMatcher;

    impl wiremock::Match for SendEmailRequestFormatMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                if body.get("Messages").is_some() {
                    let messages = body
                        .get("Messages")
                        .and_then(|messages| messages.as_array())
                        .unwrap();

                    let msg = &messages[0];
                    return msg.get("From").is_some()
                        && msg.get("To").is_some()
                        && msg.get("Subject").is_some()
                        && msg.get("TextPart").is_some()
                        && msg.get("HTMLPart").is_some();
                }
            }
            false
        }
    }

    #[tokio::test]
    async fn send_email_fires_req_to_base_url() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(method("POST"))
            .and(header("Content-type", "application/json"))
            .and(header_exists("Authorization"))
            .and(SendEmailRequestFormatMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_ret_200() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_ret_500() {
        let mock_server = MockServer::start().await;

        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn send_email_times_out_if_resp_takes_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let mock_response = ResponseTemplate::new(500).set_delay(Duration::from_secs(180));

        Mock::given(any())
            .respond_with(mock_response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(result);
    }
}
