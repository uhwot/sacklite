use std::path::PathBuf;

use anyhow::{Result, Context};

pub fn str_to_hash(str: &str) -> Result<[u8; 20]> {
    hex::decode(str)
        .context("Couldn't parse hash")?
        .try_into()
        .map_err(|_| anyhow::Error::msg("Invalid hash size"))
}

pub fn get_hash_path(resource_dir: &str, hash: [u8; 20]) -> PathBuf {
    let mut path = PathBuf::from(resource_dir);
    path.push(hex::encode(hash));
    path
}
