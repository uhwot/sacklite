use std::io::{Cursor, Seek, SeekFrom};

use actix_web::web::Bytes;
use anyhow::{bail, Context, Ok, Result};
use byteorder::{BigEndian, ReadBytesExt};
use openssl::{hash::MessageDigest, sign::Verifier};

use super::pub_key_store::PubKeyStore;
use crate::{types::platform::Platform, utils::ticket_read::*};

// useful links
// https://www.psdevwiki.com/ps3/X-I-5-Ticket
// https://github.com/hallofmeat/Skateboard3Server/blob/1398cb3114da3b8e7e13bf52497c3c9d7c21d4e6/docs/PS3Ticket.md
// https://github.com/LBPUnion/ProjectLighthouse/tree/b87c16ab7c51337ef386affc52a98ad697cf3295/ProjectLighthouse/Tickets
// https://github.com/RipleyTom/rpcn/blob/512df608ce59a9715a8e2bada2ff4e5b7abde165/src/server/client/ticket.rs

#[derive(Debug)]
pub struct NpTicket {
    pub version: (u8, u8),
    pub body: BodySection,
    pub footer: FooterSection,
    data_to_verify: Vec<u8>,
}

impl NpTicket {
    pub fn parse_from_bytes(bytes: Bytes) -> Result<Self> {
        if bytes.len() < 0x8 {
            bail!("NpTicket has length less than 8 bytes: {}", bytes.len());
        }

        // ticket header isn't included in length, so we subtract 8 bytes
        let real_ticket_len = bytes.len() - 0x8;

        let mut rdr = Cursor::new(bytes);

        let version = (rdr.read_u8()? >> 4, rdr.read_u8()?);

        if version != (2, 1) {
            bail!("Unsupported NpTicket version {version:?}");
        }

        // four null bytes, bruh
        rdr.seek(SeekFrom::Current(4))?;

        let ticket_len = rdr.read_u16::<BigEndian>()?;
        if real_ticket_len != ticket_len as usize {
            bail!(
                "Ticket length mismatch, expected = {}, actual = {}",
                ticket_len,
                real_ticket_len
            );
        }

        let body_start = rdr.stream_position()? as usize;
        let body = BodySection::parse(&mut rdr)?;
        let body_end = rdr.stream_position()? as usize;
        let signature = FooterSection::parse(&mut rdr)?;

        let data_to_verify = match signature.key_id {
            Platform::Psn => rdr.into_inner()[..signature.sig_data_start as usize].to_vec(),
            Platform::Rpcn => rdr.into_inner()[body_start..body_end].to_vec(),
        };

        Ok(Self {
            version,
            body,
            footer: signature,
            data_to_verify,
        })
    }

    pub fn verify_signature(&self, pub_key_store: &PubKeyStore) -> Result<bool> {
        let (digest_alg, pub_key) = match self.footer.key_id {
            Platform::Psn => (MessageDigest::sha1(), &pub_key_store.psn),
            Platform::Rpcn => (MessageDigest::sha224(), &pub_key_store.rpcn),
        };

        let mut verifier = Verifier::new(digest_alg, pub_key)?;

        let mut signature = self.footer.signature.as_slice();

        // PSN signatures are fixed-length and might have extra null bytes at the end
        // so we have to read the length from the TLV data
        // https://letsencrypt.org/docs/a-warm-welcome-to-asn1-and-der/#type-length-value
        if let Platform::Psn = self.footer.key_id {
            let sig_length = signature
                .get(1)
                .context("Couldn't read PSN signature length")?
                + 0x2;
            signature = signature
                .get(..sig_length as usize)
                .context("PSN signature length is invalid")?;
        }

        Ok(verifier.verify_oneshot(signature, &self.data_to_verify)?)
    }
}

#[derive(Debug)]
pub struct BodySection {
    pub serial: Vec<u8>,
    pub issuer_id: u32,
    pub issued_date: u64,
    pub expire_date: u64,

    pub user_id: u64,
    pub online_id: String,
    pub region: String,
    pub domain: String,

    pub service_id: String,

    pub status: u32,
}

impl BodySection {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let header = SectionHeader::parse(rdr)?;
        match header.section_type {
            SectionType::Body => {}
            _ => bail!("Expected body section, got {:?}", header.section_type),
        }

        let body = Self {
            serial: Data::binary(rdr)?,
            issuer_id: Data::u32(rdr)?,
            issued_date: Data::timestamp(rdr)?,
            expire_date: Data::timestamp(rdr)?,

            user_id: Data::u64(rdr)?,
            online_id: Data::string(rdr)?,
            region: Data::binary_as_str(rdr)?, // no, i'm not going to brazil >:(
            domain: Data::string(rdr)?,

            service_id: Data::binary_as_str(rdr)?,

            status: Data::u32(rdr)?,
        };

        // padding
        Data::empty(rdr)?;
        Data::empty(rdr)?;

        Ok(body)
    }
}

#[derive(Debug)]
pub struct FooterSection {
    pub key_id: Platform,
    sig_data_start: u64,
    pub signature: Vec<u8>,
}

impl FooterSection {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let header = SectionHeader::parse(rdr)?;
        match header.section_type {
            SectionType::Footer => {}
            _ => bail!("Expected footer section, got {:?}", header.section_type),
        }

        let footer = Self {
            key_id: Platform::from_key_id(&Data::binary(rdr)?)?,
            sig_data_start: rdr.stream_position()? + 0x4,
            signature: Data::binary(rdr)?,
        };

        Ok(footer)
    }
}
