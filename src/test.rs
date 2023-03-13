#![allow(unused)]

use std::fs::File;

use bertram::{crash::luma::CrashLuma, ctru::CtruError};

fn main() -> anyhow::Result<()> {
    //println!("{}", CtruError::from_code(0xd8c3fbf3));
    //println!("{}", CtruError::from_code(0xc8804478));

    let mut f = File::open("test_files/crash_dump_00000001.dmp")?;
    let luma_crash = CrashLuma::from_file(&mut f)?;
    let generic_luma = luma_crash.clone().as_generic(Some(5));

    //println!("{:#X?}", generic_luma);

    Ok(())
}
