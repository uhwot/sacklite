use std::{str::FromStr, fmt};

use anyhow::{Context, Result};
use serde::{de, Deserialize, Deserializer};

use crate::utils::resource::{get_hash_path, str_to_hash};

#[derive(Debug, Clone)]
pub enum ResourceRef {
    Guid(u32),
    Hash([u8; 20]),
}

impl ResourceRef {
    pub fn exists(&self, resource_dir: &str) -> bool {
        match self {
            Self::Guid(_) => true,
            Self::Hash(h) => get_hash_path(resource_dir, *h).exists()
        }
    }
}

impl FromStr for ResourceRef {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(guid) = s.strip_prefix('g') {
            Ok(Self::Guid(
                guid.parse::<u32>().context("Couldn't parse guid")?,
            ))
        } else {
            Ok(Self::Hash(
                str_to_hash(s)?
            ))
        }
    }
}

impl<'de> Deserialize<'de> for ResourceRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guid(g) => write!(f, "g{g}"),
            Self::Hash(h) => write!(f, "{}", hex::encode(h)),
        }
    }
}
