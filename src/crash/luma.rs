use std::{
    fmt::Display,
    io::{Read, Seek, SeekFrom},
};

use anyhow::anyhow;
use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};

use crate::crash::{CrashInfo, ExcType};

#[derive(Debug, Clone)]
pub struct LumaVersion {
    pub major: u16,
    pub minor: u8,
    pub micro: u8,
}

impl LumaVersion {
    pub const fn new(major: u16, minor: u8, micro: u8) -> Self {
        Self {
            major,
            minor,
            micro,
        }
    }

    pub const MINIMUM_VERSION: Self = Self::new(1, 0, 2);
}

impl From<u32> for LumaVersion {
    fn from(value: u32) -> Self {
        Self {
            major: (value >> 16) as u16,
            minor: (value >> 8) as u8,
            micro: value as u8,
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum LumaProcessor {
    Arm9 = 9,
    Arm11(u16) = 11,
}

impl Display for LumaProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Arm9 => "arm9".to_string(),
                Self::Arm11(c) => format!("arm11 (core {c})"),
            }
        )
    }
}

impl TryFrom<u32> for LumaProcessor {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let processor = value as u16;
        let core = (value >> 16) as u16;
        match processor {
            9 => Ok(Self::Arm9),
            11 => Ok(Self::Arm11(core)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrashLuma {
    pub version: LumaVersion,
    pub processor: LumaProcessor,
    pub exception_type: ExcType,
    pub registers: Vec<u32>,
    pub code: Vec<u8>,
    pub stack: Vec<u8>,
    pub extra: Vec<u8>,
}
/*
    ### Register order: ###
    r0    r1    r2    r3    r4    r5     r6      r7
    r8    r9    r10   r11   r12   sp     lr      pc
    cpsr  dfsr  ifsr  far   fpexc fpinst fpinst2
*/

impl CrashLuma {
    pub fn from_file(f: &mut (impl Read + Seek)) -> anyhow::Result<Self> {
        let magic_a = u32::read_from(f, LE)?;
        let magic_b = u32::read_from(f, LE)?;
        if (magic_a, magic_b) != (0xdeadc0de, 0xdeadcafe) {
            return Err(anyhow!("Not a Luma3DS crash dump"));
        }

        let version = LumaVersion::from(u32::read_from(f, LE)?);
        let Ok(processor) = LumaProcessor::try_from(u32::read_from(f, LE)?) else {
            return Err(anyhow!("Invalid processor number (should be 9 or 11)"));
        };
        let Ok(exception_type) = ExcType::try_from(u32::read_from(f, LE)?) else {
            return Err(anyhow!("Invalid exception type (should be 0-3)"));
        };

        f.seek(SeekFrom::Current(4))?;

        let num_registers = u32::read_from(f, LE)? / 4;
        let code_size = u32::read_from(f, LE)?;
        let stack_size = u32::read_from(f, LE)?;
        let extra_size = u32::read_from(f, LE)?;

        let mut registers = vec![];
        for _ in 0..num_registers {
            registers.push(u32::read_from(f, LE)?);
        }

        let mut code = vec![0u8; code_size as usize];
        let mut stack = vec![0u8; stack_size as usize];
        let mut extra = vec![0u8; extra_size as usize];

        f.read_exact(&mut code)?;
        f.read_exact(&mut stack)?;
        f.read_exact(&mut extra)?;

        Ok(Self {
            version,
            processor,
            exception_type,
            registers,
            code,
            stack,
            extra,
        })
    }

    pub fn as_generic(self, call_stack_size: Option<usize>) -> CrashInfo {
        let r = Some(self.registers[0..13].try_into().unwrap());
        CrashInfo {
            call_stack: match call_stack_size {
                None | Some(0) => None,
                Some(c) => Some(self.get_call_stack(c)),
            },
            engine: super::ModdingEngine::RHMPatch,
            r,
            sp: Some(self.registers[13]),
            lr: self.registers[14],
            pc: self.registers[15],
            cpsr: self.registers[16],
            dfsr: self.registers.get(17).copied(),
            ifsr: self.registers.get(18).copied(),
            far: self.registers.get(19).copied(),
            fpexc: self.registers.get(20).copied(),
            fpinst: self.registers.get(21).copied(),
            fpinst2: self.registers.get(22).copied(),
            stack: Some(self.stack),
        }
    }

    pub fn get_call_stack(&self, size: usize) -> Vec<u32> {
        let mut call_stack = vec![];
        let mut i = 0;
        while i < self.stack.len() && call_stack.len() < size {
            let val = u32::from_le_bytes(self.stack[i..i + 4].try_into().unwrap());
            // TODO: get start and end of sections
            if (0x00100000..0x04000000).contains(&val) || (0x07000100..0x08000000).contains(&val) {
                call_stack.push(val);
            }
            i += 4;
        }
        call_stack
    }

    pub fn get_title_info(&self) -> Option<(String, u64)> {
        if let LumaProcessor::Arm9 = self.processor {
            return None;
        } else if self.extra.len() < 16 {
            return None;
        }

        let process_raw = &self.extra[0..8];
        let tid = u64::read_from(&mut &self.extra[8..16], LE).unwrap();

        let mut process = String::new();
        for c in process_raw {
            if *c == 0 {
                break;
            }
            process += &<char as Into<String>>::into(char::from(*c));
        }

        Some((process, tid))
    }
}
