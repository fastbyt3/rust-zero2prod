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
    let connection_pool = PgPool::connect_lazy_with(config.database.with_db());
    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .expect("Binding failed");
    run(listener, connection_pool)?.await
}
