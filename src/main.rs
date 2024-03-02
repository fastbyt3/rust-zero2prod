use zero2prod::{
    configuration::get_config,
    startup::Application,
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

    let config = get_config().expect("Failed to read config");
    let application = Application::build(config)
        .await
        .expect("Failed to build Application");
    application.run_until_stopped().await?;
    Ok(())
}
