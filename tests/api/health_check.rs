use crate::helpers::spawn_app;

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
