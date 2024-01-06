use std::io::{Cursor, Read};

use axum::body::Bytes;
use anyhow::{bail, Ok, Result};
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub enum SectionType {
    Body,
    Footer,
}

impl SectionType {
    pub fn from_id(id: u8) -> Result<Self> {
        match id {
            0x00 => Ok(SectionType::Body),
            0x02 => Ok(SectionType::Footer),
            _ => bail!("Invalid section type {id}"),
        }
    }
}

#[derive(Debug)]
pub struct SectionHeader {
    pub section_type: SectionType,
    pub length: u16,
}

impl SectionHeader {
    pub fn parse(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        rdr.read_u8()?;
        Ok(Self {
            section_type: SectionType::from_id(rdr.read_u8()?)?,
            length: rdr.read_u16::<BigEndian>()?,
        })
    }
}

#[derive(Debug)]
pub enum Data {
    Empty,
    U32(u32),
    U64(u64),
    String(String),
    Timestamp(u64),
    Binary(Vec<u8>),
}

impl Data {
    pub fn read(rdr: &mut Cursor<Bytes>) -> Result<Self> {
        let data_type = rdr.read_u16::<BigEndian>()?;
        let len = rdr.read_u16::<BigEndian>()?;

        match data_type {
            // empty
            0x00 => {
                if len != 0 {
                    bail!("Empty data has non-zero length???: {len}");
                }
                Ok(Self::Empty)
            }
            // u32
            0x01 => {
                if len != 4 {
                    bail!("U32 data has invalid length: {len}")
                }
                Ok(Self::U32(rdr.read_u32::<BigEndian>()?))
            }
            // u64
            0x02 => {
                if len != 8 {
                    bail!("U64 data has invalid length: {len}")
                }
                Ok(Self::U64(rdr.read_u64::<BigEndian>()?))
            }
            // string
            0x04 => {
                let mut buf = vec![0; len.into()];
                rdr.read_exact(&mut buf)?;
                Ok(Self::String(String::from_utf8(buf)?))
            }
            // timestamp
            0x07 => {
                if len != 8 {
                    bail!("Timestamp data has invalid length: {len}")
                }
                Ok(Self::Timestamp(rdr.read_u64::<BigEndian>()?))
            }
            // binary
            0x08 => {
                let mut buf = vec![0; len.into()];
                rdr.read_exact(&mut buf)?;
                Ok(Self::Binary(buf))
            }
            _ => bail!("Invalid data type {data_type}"),
        }
    }

    pub fn empty(rdr: &mut Cursor<Bytes>) -> Result<()> {
        let data = Self::read(rdr)?;
        if let Data::Empty = data {
            return Ok(());
        }
        bail!("Expected empty data, found {:?}", data);
    }

    pub fn u32(rdr: &mut Cursor<Bytes>) -> Result<u32> {
        let data = Self::read(rdr)?;
        if let Data::U32(d) = data {
            return Ok(d);
        }
        bail!("Expected u32 data, found {:?}", data);
    }

    pub fn u64(rdr: &mut Cursor<Bytes>) -> Result<u64> {
        let data = Self::read(rdr)?;
        if let Data::U64(d) = data {
            return Ok(d);
        }
        bail!("Expected u64 data, found {:?}", data);
    }

    pub fn string(rdr: &mut Cursor<Bytes>) -> Result<String> {
        let data = Self::read(rdr)?;
        if let Data::String(d) = data {
            return Ok(d.trim_end_matches('\0').to_string());
        }
        bail!("Expected string data, found {:?}", data);
    }

    pub fn timestamp(rdr: &mut Cursor<Bytes>) -> Result<u64> {
        let data = Self::read(rdr)?;
        if let Data::Timestamp(d) = data {
            return Ok(d);
        }
        bail!("Expected timestamp data, found {:?}", data);
    }

    pub fn binary(rdr: &mut Cursor<Bytes>) -> Result<Vec<u8>> {
        let data = Self::read(rdr)?;
        if let Data::Binary(d) = data {
            return Ok(d);
        }
        bail!("Expected binary data, found {:?}", data);
    }

    pub fn binary_as_str(rdr: &mut Cursor<Bytes>) -> Result<String> {
        let data = Self::read(rdr)?;
        if let Data::Binary(d) = data {
            return Ok(String::from_utf8(d)?.trim_end_matches('\0').to_string());
        }
        bail!("Expected binary data, found {:?}", data);
    }
}
