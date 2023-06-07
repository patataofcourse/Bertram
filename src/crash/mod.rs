use std::fmt::Display;

pub mod analyze;
pub mod luma;
pub mod saltwater;
pub mod solve;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub const fn from_errf_code(errf: u8) -> Option<Self> {
        match errf {
            0 => Some(Self::PrefetchAbort),
            1 => Some(Self::DataAbort),
            2 => Some(Self::UndefinedInst),
            3 => Some(Self::FloatingPoint),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModdingEngine {
    RHMPatch,
    SpiceRack(saltwater::SWDType, saltwater::SWDVersion, saltwater::Region),
}

impl ModdingEngine {
    pub fn region(&self) -> saltwater::Region {
        match self {
            Self::RHMPatch => saltwater::Region::US,
            Self::SpiceRack(_, _, region) => *region,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrashInfo {
    pub engine: ModdingEngine,

    pub r: Option<[u32; 13]>,
    pub sp: Option<u32>,
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

impl CrashInfo {
    pub fn region(&self) -> saltwater::Region {
        self.engine.region()
    }
}

pub const FAULT_STATUS_SOURCES: &[(u32, &str)] = &[
    (0b1, "Alignment"),
    (0b100, "Instruction cache maintenance operation fault"),
    (0b1100, "External Abort on translation - First-level"),
    (0b1110, "External Abort on translation - Second-level"),
    (0b101, "Translation - Section"),
    (0b111, "Translation - Page"),
    (0b11, "Access bit - Section"),
    (0b110, "Access bit - Page"),
    (0b1001, "Domain - Section"),
    (0b1011, "Domain - Page"),
    (0b1101, "Permission - Section"),
    (0b1111, "Permission - Page"),
    (0b1000, "Precise External Abort"),
    (0b10110, "Imprecise External Abort"),
    (0b10, "Debug event"),
];
