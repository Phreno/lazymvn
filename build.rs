use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=NIGHTLY_VERSION");
    println!("cargo:rerun-if-env-changed=NIGHTLY_TAG");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    let version = env::var("NIGHTLY_VERSION").unwrap_or_else(|_| {
        let mut version = pkg_version.clone();
        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
        {
            if output.status.success() {
                let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !sha.is_empty() {
                    version = format!("{version}+{sha}");
                }
            }
        }
        version
    });

    let commit = env::var("GITHUB_SHA").ok().or_else(|| {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                } else {
                    None
                }
            })
    });

    if let Ok(tag) = env::var("NIGHTLY_TAG") {
        println!("cargo:rustc-env=LAZYMVN_BUILD_TAG={tag}");
    }

    if let Some(commit_sha) = commit {
        println!("cargo:rustc-env=LAZYMVN_COMMIT_SHA={commit_sha}");
    }

    println!("cargo:rustc-env=LAZYMVN_VERSION={version}");
}
