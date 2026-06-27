use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_release = profile == "release";

    println!("cargo:rustc-env=APP_SECURE_COOKIES={is_release}");
    let commit_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(fallback_timestamp);

    println!("cargo:rustc-env=BUILD_VERSION={commit_hash}");

    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=package.json");

    let project_root: PathBuf = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set")
        .into();

    if is_release {
        run_bun(&project_root, "prod", true);
    } else {
        run_bun(&project_root, "dev", false);
    }
}

fn run_bun(cwd: &PathBuf, script: &str, wait: bool) {
    let mut cmd = Command::new("bun");
    cmd.args(["run", script]);
    cmd.current_dir(cwd);

    if wait {
        let status = cmd
            .status()
            .unwrap_or_else(|e| panic!("failed to run `bun run {script}`: {e}"));

        if !status.success() {
            panic!("`bun run {script}` exited with code: {:?}", status.code());
        }
    } else {
        match cmd.spawn() {
            Ok(_) => {
                println!("cargo:warning=started `bun run {script}` in background");
            }
            Err(e) => {
                println!("cargo:warning=failed to start `bun run {script}`: {e}");
            }
        }
    }
}

fn fallback_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}
