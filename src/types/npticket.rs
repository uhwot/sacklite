use std::io::{Cursor, Seek, SeekFrom, Read};

use actix_web::web::Bytes;
use anyhow::{Result, Ok};
use byteorder::{ReadBytesExt, BigEndian};

// https://www.psdevwiki.com/ps3/X-I-5-Ticket
// https://github.com/RipleyTom/rpcn/blob/master/src/server/client/ticket.rs

#[derive(Debug)]
enum SectionType {
    TicketData,
    Signature,
}

impl SectionType {
    fn from_id(id: u8) -> Self {
        match id {
            0x00 => SectionType::TicketData,
            0x02 => SectionType::Signature,
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
    pub data: TicketDataSection,
    pub signature: SignatureSection,
    pub data_to_verify: Vec<u8>,
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

        let data_start = rdr.stream_position()? as usize;
        let data = TicketDataSection::parse(&mut rdr)?;
        let sig_start = rdr.stream_position()? as usize;
        let signature = SignatureSection::parse(&mut rdr)?;

        let data_to_verify = match signature.signature_id.as_slice() {
            // PSN
            b"\x71\x9f\x1d\x4a" => rdr.into_inner()[..sig_start].to_vec(),
            // RPCN
            b"RPCN" => rdr.into_inner()[data_start..sig_start].to_vec(),
            _ => todo!("Unknown signature ID {:?}", signature.signature_id)
        };

        Ok(Self {
            version,
            data,
            signature,
            data_to_verify,
        })
    }
}

#[derive(Debug)]
pub struct TicketDataSection {
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

impl TicketDataSection {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let header = SectionHeader::parse(rdr)?;
        match header.section_type {
            SectionType::TicketData => {},
            _ => todo!("Expected ticket data section, got {:?}", header.section_type),
        }

        let ticket_data = Self {
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

        Ok(ticket_data)
    }
}

#[derive(Debug)]
pub struct SignatureSection {
    pub signature_id: Vec<u8>,
    pub signature: Vec<u8>,
}

impl SignatureSection {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let header = SectionHeader::parse(rdr)?;
        match header.section_type {
            SectionType::Signature => {},
            _ => todo!("Expected signature section, got {:?}", header.section_type),
        }

        let signature = Self {
            signature_id: Data::binary(rdr)?,
            signature: Data::binary(rdr)?,
        };

        Ok(signature)
    }
}