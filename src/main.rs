#[allow(unused_imports)]
use bertram::{crash::luma::CrashLuma, ctru::CtruError};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    //println!("{}", CtruError::from_code(0xd8c3fbf3));
    //println!("{}", CtruError::from_code(0xc8804478));

    let mut f = File::open("test_files/crash_dump_00000001.dmp")?;
    println!("{:#X?}", CrashLuma::from_file(&mut f)?);

    Ok(())
}
