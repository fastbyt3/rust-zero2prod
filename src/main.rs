use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_config,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        String::from("zero2prod"),
        String::from("info"),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to get config");
    let connection_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to postgres");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))
        .expect("Binding failed");
    run(listener, connection_pool)?.await
}
