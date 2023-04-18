use strum_macros::IntoStaticStr;
use anyhow::{Result, bail};

#[derive(Debug, IntoStaticStr)]
pub enum Platform {
    PSN,
    RPCN,
}

impl Platform {
    pub fn from_key_id(key_id: &[u8]) -> Result<Self> {
        match key_id {
            b"\x71\x9f\x1d\x4a" => Ok(Self::PSN),
            b"RPCN" => Ok(Self::RPCN),
            _ => bail!("Unknown signature key ID {:?}", key_id)
        }
    }
}