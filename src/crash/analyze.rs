use crate::crash::{ModdingEngine, CrashInfo};

#[derive(Debug, Clone)]
pub struct CrashAnalysis {
    pub oob_pc: bool,
    pub ctype: ModdingEngine,
    pub pc: Function,
    pub lr: Function,
    pub call_stack: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub reg_pos: u32,
    pub func_pos: u32,
    pub symbol: String,
}

pub struct Symbols;

impl CrashAnalysis {
    const DISPLAY_PC_IF_OOB: bool = false;

    pub fn from(crash: &CrashInfo) -> Self {
        todo!();
    }
}