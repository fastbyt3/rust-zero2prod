use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmation_without_token_returns_400() {
    let app = spawn_app().await;

    let resp = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed to send request to confirm subscriptions endpoint");

    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn link_returned_by_subscribe_rets_200_if_called() {
    let app = spawn_app().await;
    let body = "name=mark&email=mark@weeb.com";

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.to_string()).await;

    let email_sent = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_link = app.get_confirmation_links(&email_sent).html;
    let resp = reqwest::get(confirmation_link).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_confirmation_link_confirms_subscription() {
    let app = spawn_app().await;

    let body = "name=guido&email=guido@ferrari.com".to_string();

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT name, email, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to read record from subscriptions table");

    assert_eq!(saved.name, "guido");
    assert_eq!(saved.email, "guido@ferrari.com");
    assert_eq!(saved.status, "confirmed");
}
