use std::{env, fs::{self, File}, path::Path, process::Command};

cfg_select! {
    windows => {
        pub const NPX: &str = r"C:\Program Files\nodejs\npx.cmd";
    }
    target_arch = "aarch64" => {
        pub const NPX: &str = "/opt/homebrew/bin/npx";
    }
    _ => {
        pub const NPX: &str = "/usr/local/bin/npx";
    }
}

fn main() {
    // ensure that dist is created, since it's not tracked in git
    fs::create_dir("./page/dist").ok();

    let mut cmd = Command::new(NPX);
    cmd.arg("webpack")
        .current_dir("./page")
        .spawn()
        .expect("failed to execute process");
}