use std::fs::File;

use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub digest_key: String,
    pub listen_addr: String,
    pub listen_port: u16,
    pub external_url: String,
    pub base_path: String,
    pub db_conn: String,
    pub redis_conn: String,
    pub log_level: String,

    pub server_desc: String,
    pub banner_image_url: Option<Url>,

    pub eula: String,
    pub announcement: String,

    pub resource_dir: String,
    pub resource_size_limit: u32,
    pub slot_limit: u32,

    pub create_user_on_connect: bool,
    pub rename_users_automatically: bool,

    pub verify_client_digest: bool,
    pub verify_npticket_signature: bool,
    pub verify_npticket_expiry: bool,
}

impl Config {
    pub fn parse_from_file(path: &str) -> Self {
        let file = File::open(path).expect("Couldn't open config file");
        let config: Self = serde_yaml::from_reader(file).expect("Couldn't parse config");
        config
    }
}
