use std::{fs, process::Command};

cfg_select! {
    windows => {
        pub const NPM: &str = r"C:\Program Files\nodejs\npm.cmd";
        pub const NPX: &str = r"C:\Program Files\nodejs\npx.cmd";
    }
    macos => {
        // On macOS, prefer homebrew path; falls back to PATH lookup if not found
        pub const NPM: &str = "/opt/homebrew/bin/npm";
        pub const NPX: &str = "/opt/homebrew/bin/npx";
    }
    _ => {
        // Linux: rely on PATH (Alpine installs npm at /usr/bin/npm)
        pub const NPM: &str = "npm";
        pub const NPX: &str = "npx";
    }
}

fn main() {
    // ensure that dist is created, since it's not tracked in git
    fs::create_dir_all("./page/dist").ok();

    // Install node dependencies
    let npm_status = Command::new(NPM)
        .arg("install")
        .current_dir("./page")
        .status()
        .expect("failed to execute npm install");

    if !npm_status.success() {
        panic!("npm install failed with status: {}", npm_status);
    }

    // Build the frontend with webpack
    let status = Command::new(NPX)
        .arg("webpack")
        .current_dir("./page")
        .status()
        .expect("failed to execute webpack");

    if !status.success() {
        panic!("webpack build failed with status: {}", status);
    }

    // Tell Cargo to re-run build script if page source changes
    println!("cargo:rerun-if-changed=page/src");
    println!("cargo:rerun-if-changed=page/package.json");
}