use std::io::{Cursor, Seek, SeekFrom, Read};

use actix_web::web::Bytes;
use anyhow::{Result, Ok};
use byteorder::{ReadBytesExt, BigEndian};
use openssl::{sign::Verifier, hash::MessageDigest};

use super::pub_key_store::PubKeyStore;

// useful links
// https://www.psdevwiki.com/ps3/X-I-5-Ticket
// https://github.com/hallofmeat/Skateboard3Server/blob/1398cb3114da3b8e7e13bf52497c3c9d7c21d4e6/docs/PS3Ticket.md
// https://github.com/LBPUnion/ProjectLighthouse/tree/b87c16ab7c51337ef386affc52a98ad697cf3295/ProjectLighthouse/Tickets
// https://github.com/RipleyTom/rpcn/blob/512df608ce59a9715a8e2bada2ff4e5b7abde165/src/server/client/ticket.rs

#[derive(Debug)]
enum SectionType {
    Body,
    Footer,
}

impl SectionType {
    fn from_id(id: u8) -> Self {
        match id {
            0x00 => SectionType::Body,
            0x02 => SectionType::Footer,
            _ => todo!("Invalid section type {id}"),
        }
    }
}

#[derive(Debug)]
struct SectionHeader {
    section_type: SectionType,
    length: u16,
}

impl SectionHeader {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        rdr.read_u8()?;
        Ok(Self {
            section_type: SectionType::from_id(rdr.read_u8()?),
            length: rdr.read_u16::<BigEndian>()?,
        })
    }
}

#[derive(Debug)]
enum Data {
    Empty,
    U32(u32),
    U64(u64),
    String(String),
    Timestamp(u64),
    Binary(Vec<u8>),
}

impl Data {
    fn read(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let data_type = rdr.read_u16::<BigEndian>()?;
        let len = rdr.read_u16::<BigEndian>()?;

        match data_type {
            // empty
            0x00 => {
                if len != 0 {
                    todo!("Empty data has non-zero length???: {len}");
                }
                Ok(Self::Empty)
            }
            // u32
            0x01 => {
                if len != 4 {
                    todo!("U32 data has invalid length: {len}")
                }
                Ok(Self::U32(rdr.read_u32::<BigEndian>()?))
            },
            // u64
            0x02 => {
                if len != 8 {
                    todo!("U64 data has invalid length: {len}")
                }
                Ok(Self::U64(rdr.read_u64::<BigEndian>()?))
            },
            // string
            0x04 => {
                let mut buf = vec![0; len.into()];
                rdr.read_exact(&mut buf)?;
                Ok(Self::String(String::from_utf8(buf)?))
            },
            // timestamp
            0x07 => {
                if len != 8 {
                    todo!("Timestamp data has invalid length: {len}")
                }
                Ok(Self::Timestamp(rdr.read_u64::<BigEndian>()?))
            },
            // binary
            0x08 => {
                let mut buf = vec![0; len.into()];
                rdr.read_exact(&mut buf)?;
                Ok(Self::Binary(buf))
            },
            _ => todo!("Invalid data type {data_type}"),
        }
    }

    fn empty(rdr: &mut Cursor<Bytes>) -> Result<()> {
        let data = Self::read(rdr)?;
        if let Data::Empty = data {
            return Ok(());
        }
        todo!("Expected empty data, found {:?}", data);
    }

    fn u32(rdr: &mut Cursor<Bytes>) -> Result<u32> {
        let data = Self::read(rdr)?;
        if let Data::U32(d) = data {
            return Ok(d);
        }
        todo!("Expected u32 data, found {:?}", data);
    }

    fn u64(rdr: &mut Cursor<Bytes>) -> Result<u64> {
        let data = Self::read(rdr)?;
        if let Data::U64(d) = data {
            return Ok(d);
        }
        todo!("Expected u64 data, found {:?}", data);
    }

    fn string(rdr: &mut Cursor<Bytes>) -> Result<String> {
        let data = Self::read(rdr)?;
        if let Data::String(d) = data {
            return Ok(d.trim_end_matches('\0').to_string());
        }
        todo!("Expected string data, found {:?}", data);
    }

    fn timestamp(rdr: &mut Cursor<Bytes>) -> Result<u64> {
        let data = Self::read(rdr)?;
        if let Data::Timestamp(d) = data {
            return Ok(d);
        }
        todo!("Expected timestamp data, found {:?}", data);
    }

    fn binary(rdr: &mut Cursor<Bytes>) -> Result<Vec<u8>> {
        let data = Self::read(rdr)?;
        if let Data::Binary(d) = data {
            return Ok(d);
        }
        todo!("Expected binary data, found {:?}", data);
    }

    fn binary_as_str(rdr: &mut Cursor<Bytes>) -> Result<String> {
        let data = Self::read(rdr)?;
        if let Data::Binary(d) = data {
            return Ok(String::from_utf8(d)?.trim_end_matches('\0').to_string());
        }
        todo!("Expected binary data, found {:?}", data);
    }
}

#[derive(Debug)]
pub struct NpTicket {
    pub version: (u8, u8),
    pub data: BodySection,
    pub footer: FooterSection,
    data_to_verify: Vec<u8>,
}

impl NpTicket {
    pub fn parse_from_bytes(bytes: Bytes) -> Result<Self> {
        // TODO: replace todos with proper errors

        // ticket header isn't included in length, so we subtract 8 bytes
        let real_ticket_len = bytes.len() - 0x8;

        let mut rdr = Cursor::new(bytes);

        let version = (rdr.read_u8()? >> 4, rdr.read_u8()?);

        if version != (2, 1) {
            todo!("Unsupported NpTicket version {version:?}");
        }

        // four null bytes, bruh
        rdr.seek(SeekFrom::Current(4))?;

        let ticket_len = rdr.read_u16::<BigEndian>()?;
        if real_ticket_len != ticket_len as usize {
            todo!("Ticket length mismatch, expected = {}, actual = {}", ticket_len, real_ticket_len);
        }

        let body_start = rdr.stream_position()? as usize;
        let body = BodySection::parse(&mut rdr)?;
        let body_end = rdr.stream_position()? as usize;
        let signature = FooterSection::parse(&mut rdr)?;

        let data_to_verify = match signature.key_id {
            KeyId::PSN => rdr.into_inner()[..signature.sig_data_start as usize].to_vec(),
            KeyId::RPCN => rdr.into_inner()[body_start..body_end].to_vec(),
        };

        Ok(Self {
            version,
            data: body,
            footer: signature,
            data_to_verify,
        })
    }

    pub fn verify_signature(&self, pub_key_store: &PubKeyStore) -> Result<bool> {
        let (digest_alg, pub_key) = match self.footer.key_id {
            KeyId::PSN => (MessageDigest::sha1(), &pub_key_store.psn),
            KeyId::RPCN => (MessageDigest::sha224(), &pub_key_store.rpcn),
        };

        let mut verifier = Verifier::new(digest_alg, pub_key)?;
        // TODO: remove null bytes in end of signature
        Ok(verifier.verify_oneshot(&self.footer.signature, &self.data_to_verify)?)
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
            SectionType::Body => {},
            _ => todo!("Expected body section, got {:?}", header.section_type),
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
pub enum KeyId {
    PSN,
    RPCN,
}

impl KeyId {
    fn from_slice(slice: &[u8]) -> Self {
        match slice {
            b"\x71\x9f\x1d\x4a" => Self::PSN,
            b"RPCN" => Self::RPCN,
            _ => todo!("Unknown signature key ID {:?}", slice)
        }
    }
}

#[derive(Debug)]
pub struct FooterSection {
    pub key_id: KeyId,
    pub sig_data_start: u64,
    pub signature: Vec<u8>,
}

impl FooterSection {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let header = SectionHeader::parse(rdr)?;
        match header.section_type {
            SectionType::Footer => {},
            _ => todo!("Expected footer section, got {:?}", header.section_type),
        }

        let footer = Self {
            key_id: KeyId::from_slice(&Data::binary(rdr)?),
            sig_data_start: rdr.stream_position()? + 4,
            signature: Data::binary(rdr)?,
        };

        Ok(footer)
    }
}