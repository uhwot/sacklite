use anyhow::{bail, Context};
use regex::Regex;
use std::{convert::TryFrom, fmt};

// title ids copied from:
// https://github.com/LBPUnion/ProjectLighthouse/blob/329ab660430820e87879f60f310840b9682eac4f/ProjectLighthouse/Types/Users/GameVersion.cs

const LBP1_TITLE_IDS: [&str; 29] = [
    "BCES00141",
    "BCAS20091",
    "BCUS98208",
    "BCAS20078",
    "BCJS70009",
    "BCES00611",
    "BCUS98148",
    "BCAS20058",
    "BCJS30018",
    "BCUS98199",
    "BCJB95003",
    "NPEA00241",
    "NPUA98208",
    "NPHA80092",
    "BCKS10059",
    "BCKS10088",
    "BCUS70030",
    "NPJA00052",
    "NPUA80472",
    // Debug, Beta and Demo
    "BCET70011",
    "NPUA70045",
    "NPEA00147",
    "BCET70002",
    "NPHA80067",
    "NPJA90074",
    // Move
    "NPEA00243",
    "NPUA80479",
    "NPHA80093",
    "NPJA00058",
];

const LBP2_TITLE_IDS: [&str; 33] = [
    "BCUS98249",
    "BCES01086",
    "BCAS20113",
    "BCJS70024",
    "BCAS20201",
    "BCUS98245",
    "BCES01345",
    "BCJS30058",
    "BCUS98372",
    "BCES00850",
    "BCES01346",
    "BCUS90260",
    "BCES01694",
    "NPUA80662",
    "NPEA00324",
    "NPEA00437",
    "BCES01693",
    "BCKS10150",
    // Debug, Beta and Demo
    "NPUA70117",
    "BCET70023",
    "BCET70035",
    "NPEA90077",
    "NPEA90098",
    "NPHA80113",
    "NPHA80125",
    "NPJA90152",
    "NPUA70127",
    "NPUA70169",
    "NPUA70174",
    // HUB
    "BCET70055",
    "NPEA00449",
    "NPHA80261",
    "NPUA80967",
];

const LBP3_TITLE_IDS: [&str; 27] = [
    // PS3
    "BCES02068",
    "BCAS20322",
    "BCJS30095",
    "BCUS98362",
    "NPUA81116",
    "NPEA00515",
    "BCUS81138",
    "NPJA00123",
    "NPHO00189",
    "NPHA80277",
    // Debug, Beta and Demo
    "NPEA90128",
    "NPUA81174",
    "BCES01663",
    // PS4
    "CUSA00693",
    "CUSA00810",
    "CUSA00738",
    "PCJS50003",
    "CUSA00063",
    "PCKS90007",
    "PCAS00012",
    "CUSA00601",
    "CUSA00762",
    "PCAS20007",
    "CUSA00473",
    // Debug, Beta and Demo
    "CUSA01072",
    "CUSA01077",
    "CUSA01304",
];

#[derive(Debug, Copy, Clone)]
pub enum GameVersion {
    Lbp1,
    Lbp2,
    Lbp3,
}

impl GameVersion {
    pub fn from_service_id(service_id: &str) -> anyhow::Result<Self> {
        let re = Regex::new(r"^[A-Z]{2}\d{4}-([A-Z]{4}\d{5})_00$").unwrap();
        let captures = re.captures(service_id).context("No title ID match found")?;
        let title_id = &captures[1];

        match title_id {
            _ if LBP1_TITLE_IDS.contains(&title_id) => anyhow::Ok(Self::Lbp1),
            _ if LBP2_TITLE_IDS.contains(&title_id) => anyhow::Ok(Self::Lbp2),
            _ if LBP3_TITLE_IDS.contains(&title_id) => anyhow::Ok(Self::Lbp3),
            _ => bail!("Invalid title ID"),
        }
    }
}

// https://stackoverflow.com/a/57578431
impl TryFrom<u8> for GameVersion {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == GameVersion::Lbp1 as u8 => Ok(GameVersion::Lbp1),
            x if x == GameVersion::Lbp2 as u8 => Ok(GameVersion::Lbp2),
            x if x == GameVersion::Lbp3 as u8 => Ok(GameVersion::Lbp3),
            _ => Err(()),
        }
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameVersion::Lbp1 => write!(f, "lbp1"),
            GameVersion::Lbp2 => write!(f, "lbp2"),
            GameVersion::Lbp3 => write!(f, "lbp3"),
        }
    }
}