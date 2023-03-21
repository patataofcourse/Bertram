use std::{
    fmt::Display,
    io::{Read, Seek, SeekFrom},
};

use anyhow::anyhow;
use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};

use crate::crash::ExcType;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SWDType {
    Extended,
    Short,
}

impl TryFrom<u8> for SWDType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SWDType::Extended),
            1 => Ok(SWDType::Short),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Region {
    JP,
    US,
    EU,
    KR,
    UNK,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JP => write!(f, "Japan"),
            Self::US => write!(f, "North America"),
            Self::EU => write!(f, "Europe"),
            Self::KR => write!(f, "Korea"),
            Self::UNK => write!(f, "Unknown title ID!"),
        }
    }
}

impl From<u8> for Region {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::JP,
            1 => Self::US,
            2 => Self::EU,
            3 => Self::KR,
            _ => Self::UNK,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SWDVersion {
    Debug { commit_hash: String },
    Release { major: u8, minor: u8, patch: u8 },
}

impl Display for SWDVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug { commit_hash } => write!(f, "commit {}", commit_hash),
            Self::Release {
                major,
                minor,
                patch,
            } => write!(
                f,
                "{}.{}{}",
                major,
                minor,
                if *patch != 0 {
                    format!(".{}", patch)
                } else {
                    "".to_string()
                }
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrashSWD {
    pub crash_type: SWDType,
    pub region: Region,
    pub exception_type: ExcType,
    pub version: SWDVersion,

    pub pc: u32,
    pub lr: u32,
    pub cpsr: u32,
    pub status_a: u32,
    pub status_b: u32,

    pub call_stack: [u32; Self::CALL_STACK_SIZE],

    pub registers: Option<[u32; 14]>,
    pub stack: Option<Vec<u8>>,
}

impl CrashSWD {
    const CALL_STACK_SIZE: usize = 5;

    pub fn from_file(f: &mut (impl Read + Seek)) -> anyhow::Result<Self> {
        let mut magic = [0u8; 8];
        f.read_exact(&mut magic)?;
        if &magic != b"SELCRAH\0" {
            return Err(anyhow!("Not a Saltwater crash dump"));
        }

        let crash_type = SWDType::try_from(u8::read_from(f, LE)?)
            .ok()
            .ok_or(anyhow!("Invalid Saltwater crash type"))?;
        let region = Region::from(u8::read_from(f, LE)?);
        let exception_type = ExcType::try_from(u8::read_from(f, LE)?)
            .ok()
            .ok_or(anyhow!("Invalid exception type in Saltwater crash"))?;
        let release = bool::read_from(f, LE)?;

        let version = if release {
            let out = SWDVersion::Release {
                major: u8::read_from(f, LE)?,
                minor: u8::read_from(f, LE)?,
                patch: u8::read_from(f, LE)?,
            };
            f.seek(SeekFrom::Current(1))?;
            out
        } else {
            SWDVersion::Debug {
                commit_hash: format!("{:x}", u32::read_from(f, LE)?),
            }
        };

        let pc = u32::read_from(f, LE)?;
        let lr = u32::read_from(f, LE)?;
        let cpsr = u32::read_from(f, LE)?;
        let status_a = u32::read_from(f, LE)?;
        let status_b = u32::read_from(f, LE)?;

        let mut call_stack = [0; Self::CALL_STACK_SIZE];
        for call in &mut call_stack {
            *call = u32::read_from(f, LE)?;
        }

        if crash_type == SWDType::Short {
            return Ok(Self {
                crash_type,
                region,
                exception_type,
                version,
                pc,
                lr,
                cpsr,
                status_a,
                status_b,
                call_stack,
                registers: None,
                stack: None,
            });
        }

        let mut registers = [0; 14];
        for reg in &mut registers {
            *reg = u32::read_from(f, LE)?;
        }

        let stack_length = {
            let c = u32::read_from(f, LE)?;
            if c > 0x100 {
                0x100
            } else {
                c
            }
        };
        let mut stack = vec![0u8; stack_length as usize];
        f.read_exact(&mut stack)?;

        Ok(Self {
            crash_type,
            region,
            exception_type,
            version,
            pc,
            lr,
            cpsr,
            status_a,
            status_b,
            call_stack,
            registers: Some(registers),
            stack: Some(stack),
        })
    }
}

pub struct ExtendedSWD {
    pub short: CrashSWD,
    pub registers: [u32; 14],
    pub stack: Vec<u8>,
}
