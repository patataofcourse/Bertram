use crate::crash::ExcType;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum SWDType {
    Extended,
    Short,
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

#[derive(Debug, Clone)]
pub enum SWDVersion {
    Debug { commit_hash: String },
    Release { major: u8, minor: u8, micro: u8 },
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
}

impl CrashSWD {
    const CALL_STACK_SIZE: usize = 5;
}

pub struct ExtendedSWD {
    pub short: CrashSWD,
    pub registers: [u32; 14],
    pub stack: Vec<u8>,
}
