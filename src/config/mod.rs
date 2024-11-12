use config::{Config as RustConfig, ConfigError, Environment, File};
use dotenv::dotenv;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sms::Sms;
use smtp::Smtp;
use std::env;

// Import the `sms` module from a separate file
pub mod sms;
pub mod smtp;

// Struct definitions for configuration

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transactions {
    pub url: String,
    pub connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Database {
    pub url: String,
    pub connections: u32,
    pub transactions: Transactions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwt {
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub env: String,
    pub host: String,
    pub domain: String,
    pub port: u16,
    pub database: Database,
    pub jwt: Jwt,
    pub cors: String,
    pub sms: Sms,
    pub smtp: Smtp,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();

        let env = env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        let s = RustConfig::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()?;

        s.try_deserialize()
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new().unwrap();
}
