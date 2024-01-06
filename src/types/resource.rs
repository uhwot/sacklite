use std::{str::FromStr, fmt, io::{Cursor, Read}};

use actix_web::web::Bytes;
use anyhow::{Context, Result};
use byteorder::{ReadBytesExt, BigEndian};
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

#[derive(Debug)]
pub struct ResourceInfo {
    res_type: ResourceType,
    revision: Option<u32>,
    // TODO: implement dependency table parsing
    dependency_table: Option<u32>,
    branch: Option<ResourceBranch>,
    compression_flags: Option<u8>,
    is_compressed: Option<bool>,
}

impl ResourceInfo {
    pub fn parse_from_res(data: Bytes) -> Self {
        let mut rdr = Cursor::new(data);
        
        let res_type = ResourceType::from_magic(&mut rdr);

        let mut revision = None;
        let mut dependency_table = None;
        let mut branch = None;
        let mut compression_flags = None;
        let mut is_compressed = None;

        match res_type {
            ResourceType::Texture => {},
            ResourceType::GtfTexture => {},
            ResourceType::Jpeg => {},
            ResourceType::Png => {},
            ResourceType::Unknown => {},
            _ => {
                let rev = rdr.read_u32::<BigEndian>().unwrap();
                revision = Some(rev);

                if rev >= 0x109 {
                    dependency_table = Some(rdr.read_u32::<BigEndian>().unwrap());
                    if rev >= 0x189 {
                        match res_type {
                            ResourceType::Mesh => {},
                            _ => {
                                if rev >= 0x271 {
                                    branch = Some(ResourceBranch {
                                        id: rdr.read_u16::<BigEndian>().unwrap(),
                                        revision: rdr.read_u16::<BigEndian>().unwrap(),
                                    });
                                }
                                let branch = branch.as_ref().unwrap();
                                if rev >= 0x297
                                || /* leerdammer */ (rev == 0x272 && branch.id == 0x4c44 && branch.revision >= 0x2) {
                                    compression_flags = Some(rdr.read_u8().unwrap());
                                }
                                is_compressed = Some(rdr.read_u8().unwrap() != 0);
                            }
                        }
                    }
                }
            }
        };

        Self {
            res_type,
            revision,
            dependency_table,
            branch,
            compression_flags,
            is_compressed,
        }
    }
}

#[derive(Debug)]
pub enum ResourceType {
    Texture,            // TEX
    GtfTexture,         // GTF
    Level,              // LVL
    Plan,               // PLN
    FishScript,         // FSH
    Mesh,               // MSH
    GfxMaterial,        // GMT
    Voice,              // VOP
    Painting,           // PTG
    CrossLevel,         // PRF
    MoveRecording,      // REC
    Quest,              // QST
    AdventureCreate,    // ADC
    AdventureShared,    // ADS
    StreamingChunk,     // CHK
    Jpeg,
    Png,
    Unknown,
}

impl ResourceType {
    pub fn from_magic(rdr: &mut Cursor<Bytes>) -> Self {
        let mut magic = [0u8; 4];
        rdr.read_exact(&mut magic).unwrap();
        match &magic {
            b"TEX " => Self::Texture,
            b"GTF " => Self::GtfTexture,
            b"LVLb" => Self::Level,
            b"PLNb" => Self::Plan,
            b"FSHb" => Self::FishScript,
            b"MSHb" => Self::Mesh,
            b"GMTb" => Self::GfxMaterial,
            b"VOPb" => Self::Voice,
            b"PTGb" => Self::Painting,
            b"PRFb" => Self::CrossLevel,
            b"RECb" => Self::MoveRecording,
            b"QSTb" => Self::Quest,
            b"ADCb" => Self::AdventureCreate,
            b"ADSb" => Self::AdventureShared,
            b"CHKb" => Self::StreamingChunk,
            [0xFF, 0xD8, 0xFF, 0xE0] => Self::Jpeg,
            _ => {
                rdr.set_position(0);
                let mut magic = [0u8; 8];
                rdr.read_exact(&mut magic).unwrap();
                if magic == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
                    return Self::Png
                }
                Self::Unknown
            }
        }
    }
}

#[derive(Debug)]
pub struct ResourceBranch {
    id: u16,
    revision: u16,
}

/*#[derive(Debug)]
pub enum SerializationType {
    Binary,             // e
    CompressedTexture,  // (space)
    GtfSwizzled,        // s
    GxtSwizzled,        // S
}*/