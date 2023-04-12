use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    let git_hash = match output {
        Ok(c) => String::from_utf8(c.stdout).unwrap_or(String::from("Invalid git output")),
        Err(_) => String::from("Could not invoke git"),
    };
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let output = Command::new("rustc").arg("-V").output();
    let rustc_ver = match output {
        Ok(c) => String::from_utf8(c.stdout).unwrap_or(String::from("Invalid rustc output")),
        Err(_) => String::from("Could not invoke rustc"),
    };
    println!("cargo:rustc-env=RUSTC_VER={}", rustc_ver);
}
