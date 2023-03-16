use std::io::{Cursor, Seek, SeekFrom, Read};

use actix_web::web::Bytes;
use anyhow::{Result, Ok};
use byteorder::{ReadBytesExt, BigEndian};

// https://www.psdevwiki.com/ps3/X-I-5-Ticket
#[derive(Debug)]
pub struct NpTicket {

}

#[derive(Debug)]
enum SectionType {
    Body,
    Footer,
}

#[derive(Debug)]
struct SectionHeader {
    section_type: SectionType,
    length: u8
}

impl SectionHeader {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        Ok(Self {
            section_type: match rdr.read_u8()? {
                0x00 => SectionType::Body,
                0x02 => SectionType::Footer,
            },
            length: rdr.read_u8()?,
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

        let buf = vec![0; len.into()];
        rdr.read_exact(&mut buf)?;

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
                if len != 4 {
                    todo!("U64 data has invalid length: {len}")
                }
                Ok(Self::U64(rdr.read_u64::<BigEndian>()?))
            },
            // string
            0x04 => {
                let buf = vec![0; len.into()];
                rdr.read_exact(&mut buf)?;
                Ok(Self::String(String::from_utf8(buf)?))
            },
            // timestamp
            0x07 => {
                if len != 4 {
                    todo!("Timestamp data has invalid length: {len}")
                }
                Ok(Self::U64(rdr.read_u64::<BigEndian>()?))
            },
            // binary
            0x08 => {
                let buf = vec![0; len.into()];
                rdr.read_exact(&mut buf)?;
                Ok(Self::Binary(buf))
            }
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
            return Ok(d);
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
}

impl NpTicket {
    pub fn parse_from_bytes(bytes: Bytes) -> Result<Self> {
        // TODO: replace todos with proper errors
        let mut rdr = Cursor::new(bytes);

        let version = (rdr.read_u8()? >> 4, rdr.read_u8()?);

        if version != (2, 1) {
            todo!("Unsupported NpTicket version {version:?}");
        }
        
        // four null bytes, bruh
        rdr.seek(SeekFrom::Current(4));

        let ticket_len = rdr.read_u16::<BigEndian>()?;
        // ticket header isn't included in length, so we subtract 8 bytes
        if bytes.len() - 0x8 != ticket_len.into() {
            todo!("Ticket length mismatch, expected = {}, actual = {}", ticket_len, bytes.len() - 0x8);
        }

        let body = BodySection::parse(&mut rdr)?;
    }
}

#[derive(Debug)]
struct BodySection {
    issuer_id: u32,
    issued_date: u64,
    expire_date: u64,

    user_id: u64,
    username: String,
    country: String,
    domain: String,

    title_id: String,

    status: u32,
}

impl BodySection {
    fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let header = SectionHeader::parse(&mut rdr)?;
        match header.section_type {
            SectionType::Body => {},
            _ => todo!("Expected body section, got {:?}", header.section_type),
        }

        Data::string(rdr)?; // serial id
        let body = Self {
            issuer_id: Data::u32(rdr)?,
            issued_date: Data::u64(rdr)?,
            expire_date: Data::u64(rdr)?,

            user_id: Data::u64(rdr)?,
            username: Data::string(rdr)?,
            country: Data::string(rdr)?,
            domain: Data::string(rdr)?,

            title_id: Data::string(rdr)?,

            status: Data::u32(rdr)?,
        };

        // padding
        Data::empty(rdr)?;
        Data::empty(rdr)?;

        Ok(body)
    }
}