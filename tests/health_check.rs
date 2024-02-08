use std::net::TcpListener;

use zero2prod::run;

#[tokio::test]
async fn health_check_works() {
    let server_addr = spawn_app();
    let client = reqwest::Client::new();
    let health_endpoint = &format!("{}/health", server_addr);
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
    let addr = spawn_app();
    let body = "name=fastbyte%20bit&email=fast@byte.bit";
    let resp = reqwest::Client::new().post(format!("{addr}/subscribe"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await.expect("Failed to POST reqwest");

    assert!(resp.status().is_success());
}

#[tokio::test]
async fn subscriber_returns_400_invalid_incomplete_data() {
    let addr = spawn_app();

    let client = reqwest::Client::new();

    let scenarios = vec![
        ("name=bobobbo", "only name field is passed"),
        ("email=bob@mail.com", "only email is passed"),
        ("", "nothing is passed"),
    ];

    let endpoint = &format!("{addr}/subscribe");
    
    for (payload, err_msg) in scenarios {
        let resp = client.post(endpoint)
            .header("Content-Type", "x-www-form-urlencoded")
            .body(payload)
            .send()
            .await
            .expect("Failed to send POST reqwest");
        assert_eq!(resp.status().as_u16(), 400, "Did not recv resp code 400 for {}", err_msg);
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to get server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{port}")
}
