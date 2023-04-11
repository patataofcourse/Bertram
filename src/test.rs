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
    },
    ctru::CtruError,
};

fn main() -> anyhow::Result<()> {
    //println!("{}", CtruError::from_code(0xd8c3fbf3));
    //println!("{}", CtruError::from_code(0xc8804478));

    let mut f = File::open("test_files/crash_dump_00000002.dmp")?;
    let luma_crash = CrashLuma::from_file(&mut f)?;
    let generic_luma = luma_crash.as_generic(Some(5));

    //println!("{:#X?}", generic_luma);

    //let mut f = File::open("test_files/swcrash_00000.swd")?;
    //let swd_crash = CrashSWD::from_file(&mut f)?;
    //let generic_swd = swd_crash.as_generic();
    
    //println!("{:#X?}", generic_swd);

    /*
    let mut _3gx = File::open("../SpiceRack/Saltwater/Saltwater.3gx")?;
    let mut out = File::create(format!(
        "sym/sw._{}.csv",
        analyze::get_3gx_commit_hash(&mut _3gx)?.unwrap()
    ))?;

    Symbols::ctrplugin_symbols_to_csv(&mut _3gx, &mut out, true)?;
    drop(out);
    */

    CrashAnalysis::from(&generic_luma)?;

    Ok(())
}
