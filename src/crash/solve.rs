// Bertram crash solver
// The way this works is: 1. get crash 2. detect specific addresses in the PC/LR/call stack 3. profit

use super::{saltwater::Region, CrashInfo};

use anyhow::anyhow;

#[derive(Clone, Debug)]
pub enum SolveDiagnosis {
    InvalidTickflowAddress(Option<u32>),
    NoEffectMemory,
    SceneLoadingError(SceneLoadDiagnosis),
    NonExecRegion(u32),
    NullRead,
}

#[derive(Clone, Debug)]
pub enum SceneLoadDiagnosis {
    Generic,
    LowSlotLayout,
}

impl SolveDiagnosis {
    pub const fn invalid_tickflow_address_pc(region: Region) -> u32 {
        match region {
            Region::US => 0x0011e764,
            Region::UNK => unreachable!(),
            _ => todo!(),
        }
    }

    pub const fn no_effect_memory_pc(region: Region) -> u32 {
        match region {
            Region::US => 0x001392c4,
            Region::UNK => unreachable!(),
            _ => todo!(),
        }
    }

    pub const fn scene_loading_lr(region: Region) -> u32 {
        match region {
            Region::US => 0x002471dc,
            Region::UNK => unreachable!(),
            _ => todo!(),
        }
    }

    pub const fn forbidden_layout_pc(region: Region) -> u32 {
        match region {
            Region::US => 0x0020b494,
            Region::UNK => unreachable!(),
            _ => todo!(),
        }
    }

    pub fn find_matches(crash: &CrashInfo) -> anyhow::Result<Vec<Self>> {
        let region = crash.region();
        let mut out = vec![];

        let Some(bounds) = super::analyze::get_megamix_bounds()?
            .iter()
            .find(|c| region.matches(&c.version))
            .cloned()
        else {
            Err(anyhow!("Bounds file doesn't include {:?} region", region))?
        };

        if region == Region::UNK {
            panic!("Cannot solve for an UNK-region crash")
        }

        if crash.pc == Self::invalid_tickflow_address_pc(region) {
            out.push(Self::InvalidTickflowAddress(crash.far))
        } else if crash.pc == Self::no_effect_memory_pc(region) {
            out.push(Self::NoEffectMemory)
        } else if crash.pc >= bounds.rodata {
            out.push(Self::NonExecRegion(crash.pc))
        }

        /*if crash
            .call_stack
            .as_ref()
            .is_some_and(|c| c.contains(&Self::scene_loading_lr(region)))
        {*/
        if crash.pc == Self::forbidden_layout_pc(region) {
            out.push(Self::SceneLoadingError(SceneLoadDiagnosis::LowSlotLayout))
            //    } else {
            //        out.push(Self::SceneLoadingError(SceneLoadDiagnosis::Generic))
        }
        //}

        //keep this in last
        if out.is_empty() && crash.far.map_or(false, |c| c < 0x00100000) {
            out.push(Self::NullRead)
        }

        Ok(out)
    }
}
