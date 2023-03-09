pub mod analyze;
pub mod luma;
pub mod saltwater;

pub enum ExcType {
    FloatingPoint,
    UndefinedInst,
    PrefetchAbort,
    DataAbort,
}

pub enum ModdingEngine {
    RHMPatch,
    SpiceRack(saltwater::SWDVersion, saltwater::Region),
}

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

    pub stack: Option<u32>,
    pub call_stack: Option<u32>,
}