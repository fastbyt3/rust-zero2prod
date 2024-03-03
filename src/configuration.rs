use std::time::Duration;

use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub authorization_token: Secret<String>,
    pub base_url: String,
    pub sender_email: String,
    pub timeout_ms: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "prod" | "production" => Ok(Self::Production),
            _ => Err(format!(
                "Expected either local / production. {value} is not valid"
            )),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let config_dir = std::env::current_dir()
        .expect("Failed to determine current dir")
        .join("config");

    let environment: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| String::from("local"))
        .try_into()
        .expect("Unable to parse APP_ENV");
    let env_specific_config = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::format(
            config::File::from(config_dir.join("base.yaml")),
            config::FileFormat::Yaml,
        ))
        .add_source(config::File::format(
            config::File::from(config_dir.join(env_specific_config)),
            config::FileFormat::Yaml,
        ))
        // APP_APPLICATION__PORT=9999 -> Settings.application.port = 9999
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .host(&self.host)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        options.log_statements(tracing_log::log::LevelFilter::Trace)
    }
}
