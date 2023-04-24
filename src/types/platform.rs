use anyhow::{bail, Result};
use strum_macros::IntoStaticStr;

#[derive(Debug, IntoStaticStr)]
pub enum Platform {
    Psn,
    Rpcn,
}

impl Platform {
    pub fn from_key_id(key_id: &[u8]) -> Result<Self> {
        match key_id {
            b"\x71\x9f\x1d\x4a" => Ok(Self::Psn),
            b"RPCN" => Ok(Self::Rpcn),
            _ => bail!("Unknown signature key ID {:?}", key_id),
        }
    }
}
