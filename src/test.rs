#![allow(unused)]

use std::{
    ffi::{CStr, CString},
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use bertram::{
    crash::{
        analyze::{self, CrashAnalysis, Symbols},
        luma::CrashLuma,
        saltwater::CrashSWD,
        solve::SolveDiagnosis,
    },
    ctru::CtruError,
};

fn main() -> anyhow::Result<()> {
    //println!("{}", CtruError::from_code(0xd8c3fbf3));
    //println!("{}", CtruError::from_code(0xc8804478));

    let mut f = File::open("test_files/crash_dump_00000006.dmp")?;
    let luma_crash = CrashLuma::from_file(&mut f)?;
    let generic_luma = luma_crash.as_generic(Some(5))?;

    //println!("{:#X?}", generic_luma);

    //let mut f = File::open("test_files/swcrash_00008.swd")?;
    //let swd_crash = CrashSWD::from_file(&mut f)?;
    //let generic_swd = swd_crash.as_generic();

    //println!("{:#X?}", generic_swd);

    println!("{:?}", SolveDiagnosis::find_matches(&generic_luma));

    Ok(())
}
