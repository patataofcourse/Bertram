use std::fmt::Display;

pub mod analyze;
pub mod luma;
pub mod saltwater;

#[derive(Debug, Clone)]
pub enum ExcType {
    FloatingPoint,
    UndefinedInst,
    PrefetchAbort,
    DataAbort,
}

impl Display for ExcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::FloatingPoint => "floating point exception",
                Self::UndefinedInst => "undefined instruction",
                Self::PrefetchAbort => "prefetch abort",
                Self::DataAbort => "data abort",
            }
        )
    }
}

impl TryFrom<u32> for ExcType {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::FloatingPoint),
            1 => Ok(Self::UndefinedInst),
            2 => Ok(Self::PrefetchAbort),
            3 => Ok(Self::DataAbort),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for ExcType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        Self::try_from(value as u32)
    }
}

impl ExcType {
    /// Names for the Saltwater status registers
    pub const fn status_reg_names(&self) -> [Option<&'static str>; 2] {
        match self {
            Self::UndefinedInst => [None, None],
            Self::FloatingPoint => [Some("fpexc"), Some("fpinst")],
            Self::PrefetchAbort => [Some("ifsr"), None],
            Self::DataAbort => [Some("dfsr"), Some("far")],
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModdingEngine {
    RHMPatch,
    SpiceRack(saltwater::SWDType, saltwater::SWDVersion, saltwater::Region),
}

#[derive(Debug, Clone)]
pub struct CrashInfo {
    pub engine: ModdingEngine,

    pub r: [u32; 13],
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
    pub cpsr: u32,

    pub dfsr: Option<u32>,
    pub ifsr: Option<u32>,
    pub far: Option<u32>,
    pub fpexc: Option<u32>,
    pub fpinst: Option<u32>,
    pub fpinst2: Option<u32>,

    pub stack: Option<Vec<u8>>,
    pub call_stack: Option<Vec<u32>>,
}
