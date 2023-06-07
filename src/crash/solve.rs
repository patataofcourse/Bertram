// Bertram crash solver
// The way this works is: 1. get crash 2. detect specific addresses in the PC/LR/call stack 3. profit

use super::{saltwater::Region, CrashInfo};

#[derive(Clone, Debug)]
pub enum SolveDiagnosis {
    InvalidTickflowAddress(Option<u32>),
    NoEffectMemory,
    SceneLoadingError(SceneLoadDiagnosis),
    NonExecRegion(u32),
}

#[derive(Clone, Debug)]
pub enum SceneLoadDiagnosis {
    Generic(u32),
    LowSlotLayout(i32),
}

impl SolveDiagnosis {
    pub const fn invalid_tickflow_address_pc(region: &Region) -> u32 {
        match region {
            Region::US => 0x0011e764,
            Region::UNK => unreachable!(),
            _ => todo!(),
        }
    }

    pub fn find_matches(crash: &CrashInfo) -> Vec<Self> {
        let region = crash.region();
        let mut out = vec![];

        if region == Region::UNK {
            panic!("Cannot solve for an UNK-region crash")
        }

        if crash.pc == Self::invalid_tickflow_address_pc(&region) {
            out.push(Self::InvalidTickflowAddress(crash.far))
        }

        //TODO: everything else

        out
    }
}
