use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_reporter")]
    pub reporter: String,
    #[serde(default = "default_publish_url")]
    pub publish_url: String,
    #[serde(default = "default_network_interface")]
    pub network_interface: String,
}

fn default_reporter() -> String { "linux-sensor-agent".to_string() }
fn default_publish_url() -> String { "http://localhost:8080/publish".to_string() }
fn default_network_interface() -> String { "eth0".to_string() }

pub fn read_config(filename: &str) -> Config {
    let config_str = fs::read_to_string(filename)
        .expect("Could not read config file");

    return toml::from_str(&config_str.to_string())
        .expect("Failed to parse config file")
}