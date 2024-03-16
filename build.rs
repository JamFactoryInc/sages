use std::{env, path::Path, process::Command};

#[cfg(windows)]
pub const NPX: &str = r"C:\Program Files\nodejs\npx.cmd";

#[cfg(not(windows))]
pub const NPX: &str = "/usr/local/bin/npm/npx";

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    let mut cmd = Command::new(NPX);
    cmd.arg("webpack")
        .current_dir("./page")
        .spawn()
        .expect("failed to execute process");
}