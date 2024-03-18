use std::{env, fs::{self, File}, path::Path, process::Command};

#[cfg(windows)]
pub const NPX: &str = r"C:\Program Files\nodejs\npx.cmd";

#[cfg(not(windows))]
pub const NPX: &str = "/usr/local/bin/npx";

fn main() {
    // ensure that dist is created, since it's not tracked in git
    fs::create_dir("./page/dist").ok();

    let mut cmd = Command::new(NPX);
    cmd.arg("webpack")
        .current_dir("./page")
        .spawn()
        .expect("failed to execute process");
}