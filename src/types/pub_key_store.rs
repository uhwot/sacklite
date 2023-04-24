use anyhow::{Ok, Result};
use openssl::bn::BigNum;
use openssl::ec::*;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Public};

use hex_literal::hex;

// key parameters from:
// https://github.com/LBPUnion/ProjectLighthouse/blob/b87c16ab7c51337ef386affc52a98ad697cf3295/ProjectLighthouse/Tickets/NPTicket.cs#L57

const PSN_PARAMS: KeyParams = KeyParams {
    curve: Nid::X9_62_PRIME192V1,
    x: &hex!("39c62d061d4ee35c5f3f7531de0af3cf918346526edac727"),
    y: &hex!("a5d578b55113e612bf1878d4cc939d61a41318403b5bdf86"),
};

const RPCN_PARAMS: KeyParams = KeyParams {
    curve: Nid::SECP224K1,
    x: &hex!("b07bc0f0addb97657e9f389039e8d2b9c97dc2a31d3042e7d0479b93"),
    y: &hex!("d81c42b0abdf6c42191a31e31f93342f8f033bd529c2c57fdb5a0a7d"),
};

#[derive(Debug)]
struct KeyParams<'a> {
    curve: Nid,
    x: &'a [u8],
    y: &'a [u8],
}

impl KeyParams<'_> {
    fn to_public_key(&self) -> Result<PKey<Public>> {
        let group = EcGroup::from_curve_name(self.curve)?;
        let x = BigNum::from_slice(self.x)?;
        let y = BigNum::from_slice(self.y)?;

        let ec_key = EcKey::from_public_key_affine_coordinates(&group, &x, &y)?;

        Ok(PKey::from_ec_key(ec_key)?)
    }
}

#[derive(Debug)]
pub struct PubKeyStore {
    pub psn: PKey<Public>,
    pub rpcn: PKey<Public>,
}

impl PubKeyStore {
    pub fn new() -> Result<Self> {
        Ok(Self {
            psn: PSN_PARAMS.to_public_key()?,
            rpcn: RPCN_PARAMS.to_public_key()?,
        })
    }
}
