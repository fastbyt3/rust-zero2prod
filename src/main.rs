use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration::get_config, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_config().expect("Failed to get config");
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to postgres");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.application_port))
        .expect("Binding failed");
    run(listener, connection_pool)?.await
}
