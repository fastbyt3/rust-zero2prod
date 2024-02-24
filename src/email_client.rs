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
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
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
        let email = SendEmailRequest {
            from: EmailUser {
                email: self.sender.as_ref().to_owned(),
                name: String::from("Mark"),
            },
            to: vec![EmailUser {
                email: recipient.as_ref().to_owned(),
                name: String::from("Subscriber"),
            }],
            subject: subject.to_owned(),
            text_part: text_content.to_owned(),
            html_part: html_content.to_owned(),
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
            .await?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequestFormat {
    #[serde(rename(serialize = "Messages"))]
    messages: Vec<SendEmailRequest>,
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    #[serde(rename(serialize = "From"))]
    from: EmailUser,
    #[serde(rename(serialize = "To"))]
    to: Vec<EmailUser>,
    #[serde(rename(serialize = "Subject"))]
    subject: String,
    #[serde(rename(serialize = "TextPart"))]
    text_part: String,
    #[serde(rename(serialize = "HTMLPart"))]
    html_part: String,
}

#[derive(serde::Serialize)]
struct EmailUser {
    #[serde(rename(serialize = "Email"))]
    email: String,
    #[serde(rename(serialize = "Name"))]
    name: String,
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{header, header_exists, method},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_fires_req_to_base_url() {
        let mock_server = MockServer::start().await;

        let sender = SubscriberEmail::parse(String::from("markantonyyystabs@gmail.com")).unwrap();
        let authorization_token = Secret::new(Faker.fake());
        let email_client = EmailClient::new(mock_server.uri(), sender, authorization_token);

        Mock::given(method("POST"))
            .and(header("Content-type", "application/json"))
            .and(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let text_content: String = Paragraph(1..5).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &text_content, &text_content)
            .await;
    }
}
