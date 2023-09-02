use anyhow::bail;

#[derive(Debug, Clone)]
pub enum Platform {
    Psn,
    Rpcn,
}

impl Platform {
    pub fn from_key_id(key_id: &[u8]) -> anyhow::Result<Self> {
        match key_id {
            b"\x71\x9f\x1d\x4a" => Ok(Self::Psn),
            b"RPCN" => Ok(Self::Rpcn),
            _ => bail!("Unknown signature key ID {:?}", key_id),
        }
    }
}

// https://stackoverflow.com/a/57578431
impl TryFrom<u8> for Platform {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Platform::Psn as u8 => Ok(Platform::Psn),
            x if x == Platform::Rpcn as u8 => Ok(Platform::Rpcn),
            _ => Err(()),
        }
    }
}
