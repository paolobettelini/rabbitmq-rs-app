pub use serde::Deserialize;
use std::error::Error;
use std::{fs, path::Path};

pub trait Connectable {
    fn get_connection_string(&self) -> String;
}

// These two should be moved into their respective crates

#[derive(Deserialize, Debug)]
pub struct WorkerConfig {
    pub database: Option<Database>,
    pub rabbit: Option<Rabbit>,
    pub log: Option<Log>,
}

#[derive(Deserialize, Debug)]
pub struct WebserverConfig {
    pub http: Http,
    pub rabbit: Option<Rabbit>,
    pub log: Option<Log>,
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Rabbit {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub vhost: String,
}

#[derive(Deserialize, Debug)]
pub struct Http {
    pub www: String,
    pub ip: String,
    pub port: u16
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub log: String,
    #[serde(default = "default_log_style")]
    pub style: String,
}

fn default_log_style() -> String {
    "auto".to_string()
}

impl Connectable for Database {
    fn get_connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.address, self.port, self.name
        )
    }
}

impl Connectable for Rabbit {
    fn get_connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username,
            self.password,
            self.address,
            self.port,
            self.vhost.replace("/", "%2F")
        )
    }
}

pub fn parse_config<P: AsRef<Path>, ConfigType: for<'a> Deserialize<'a>>(
    config_path: P,
) -> Result<Box<ConfigType>, Box<dyn Error>> {
    let content = fs::read_to_string(config_path)?;

    let config = toml::from_str(&content)?;

    Ok(Box::new(config))
}
