use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscriber_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let body = String::from("name=jakob&email=jaking@off.com");
    let resp = test_app.post_subscriptions(body).await;

    assert!(resp.status().is_success());
}

#[tokio::test]
async fn post_subscribe_persists_data_on_success() {
    let test_app = spawn_app().await;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let body = String::from("name=fastbyte%20bit&email=fast@byte.bit");
    test_app.post_subscriptions(body).await;

    let saved = sqlx::query!("SELECT name, email, status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch one record from DB");

    assert_eq!(saved.email, "fast@byte.bit");
    assert_eq!(saved.name, "fastbyte bit");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscriber_returns_400_when_fields_are_present_but_invalid() {
    let test_app = spawn_app().await;

    let scenarios = vec![
        ("name=&email=test@mail.com", "name field is empty"),
        ("name=test&email=", "email is empty"),
        (
            "name=test&email=test",
            "email is present but invalid format",
        ),
    ];

    for (payload, description) in scenarios {
        let resp = test_app.post_subscriptions(payload.to_string()).await;
        assert_eq!(
            resp.status().as_u16(),
            400,
            "API did not return 400 BAD REQUEST when payload was: {}",
            description
        );
    }
}

#[tokio::test]
async fn subscriber_returns_400_invalid_incomplete_data() {
    let test_app = spawn_app().await;

    let scenarios = vec![
        ("name=bobobbo", "only name field is passed"),
        ("email=bob@mail.com", "only email is passed"),
        ("", "nothing is passed"),
    ];

    for (payload, err_msg) in scenarios {
        let resp = test_app.post_subscriptions(payload.to_string()).await;
        assert_eq!(
            resp.status().as_u16(),
            400,
            "Did not recv resp code 400 for {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn subscribe_sends_confirmation_mail_for_valid_data() {
    let app = spawn_app().await;

    let body = String::from("name=fastbyte%20bit&email=fast@byte.bit");

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);
    assert_eq!(confirmation_links.plain_text, confirmation_links.html);
}
