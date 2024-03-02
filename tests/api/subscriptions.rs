use crate::helpers::spawn_app;

#[tokio::test]
async fn subscriber_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = String::from("name=fastbyte%20bit&email=fast@byte.bit");
    let resp = test_app.post_subscriptions(body).await;

    assert!(resp.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch one record from DB");

    assert_eq!(saved.email, "fast@byte.bit");
    assert_eq!(saved.name, "fastbyte bit");
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
        dbg!(resp.status());
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
