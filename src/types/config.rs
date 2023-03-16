use std::fs::File;

use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub digest_key: String,
    pub listen_addr: String,
    pub listen_port: u16,
    pub base_path: String,
    pub autodiscover_url: String,
    pub log_level: String,
    pub eula: String,
    pub announcement: String,
    pub verify_client_digest: bool,
}

impl Config {
    pub fn parse_from_file(path: &str) -> Self {
        let file = File::open(path).expect("Couldn't open config file");
        let config: Self = serde_yaml::from_reader(file).expect("Couldn't parse config");
        config
    }
}

