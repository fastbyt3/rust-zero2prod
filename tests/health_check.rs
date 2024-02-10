use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_config, DatabaseSettings},
    startup::run,
};

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let health_endpoint = &format!("{}/health", test_app.address);
    println!("health_endpoint: {health_endpoint}");
    let res = client
        .get(health_endpoint)
        .send()
        .await
        .expect("health GET reqwest failed");

    assert!(res.status().is_success());
    assert_eq!(res.content_length(), Some(0));
}

#[tokio::test]
async fn subscriber_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=fastbyte%20bit&email=fast@byte.bit";
    let resp = reqwest::Client::new()
        .post(format!("{}/subscribe", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to POST reqwest");

    assert!(resp.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch one record from DB");

    assert_eq!(saved.email, "fast@byte.bit");
    assert_eq!(saved.name, "fastbyte bit");
}

#[tokio::test]
async fn subscriber_returns_400_invalid_incomplete_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let scenarios = vec![
        ("name=bobobbo", "only name field is passed"),
        ("email=bob@mail.com", "only email is passed"),
        ("", "nothing is passed"),
    ];

    let endpoint = &format!("{}/subscribe", test_app.address);

    for (payload, err_msg) in scenarios {
        let resp = client
            .post(endpoint)
            .header("Content-Type", "x-www-form-urlencoded")
            .body(payload)
            .send()
            .await
            .expect("Failed to send POST reqwest");
        assert_eq!(
            resp.status().as_u16(),
            400,
            "Did not recv resp code 400 for {}",
            err_msg
        );
    }
}

struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let mut config = get_config().expect("Failed to get config");
    config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_db(&config.database).await;

    let server = run(listener, db_pool.clone()).expect("Failed to get server");

    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{port}");
    TestApp { address, db_pool }
}

pub async fn configure_db(config: &DatabaseSettings) -> PgPool {
    PgConnection::connect(&config.connection_string_without_dbname())
        .await
        .expect("Failed to connect to Postgres")
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create test DB");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to test db");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migration on test DB");

    connection_pool
}
