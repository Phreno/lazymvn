use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=LAZYMVN_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=LAZYMVN_BUILD_TAG");
    println!("cargo:rerun-if-env-changed=LAZYMVN_BUILD_CHANNEL");
    println!("cargo:rerun-if-env-changed=LAZYMVN_COMMIT_SHA");
    // Backwards compatibility with earlier workflow variables
    println!("cargo:rerun-if-env-changed=NIGHTLY_VERSION");
    println!("cargo:rerun-if-env-changed=NIGHTLY_TAG");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    let version = env::var("LAZYMVN_BUILD_VERSION")
        .or_else(|_| env::var("NIGHTLY_VERSION"))
        .unwrap_or_else(|_| {
            let mut version = pkg_version.clone();
            if let Ok(output) = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                && output.status.success()
            {
                let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !sha.is_empty() {
                    version = format!("{version}+{sha}");
                }
            }
            version
        });

    let commit = env::var("LAZYMVN_COMMIT_SHA")
        .ok()
        .or_else(|| env::var("GITHUB_SHA").ok())
        .or_else(|| {
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

    let tag = env::var("LAZYMVN_BUILD_TAG")
        .ok()
        .or_else(|| env::var("NIGHTLY_TAG").ok());

    if let Some(tag) = tag {
        println!("cargo:rustc-env=LAZYMVN_BUILD_TAG={tag}");
    }

    if let Ok(channel) = env::var("LAZYMVN_BUILD_CHANNEL") {
        println!("cargo:rustc-env=LAZYMVN_BUILD_CHANNEL={channel}");
    }

    if let Some(commit_sha) = commit {
        println!("cargo:rustc-env=LAZYMVN_COMMIT_SHA={commit_sha}");
    }

    println!("cargo:rustc-env=LAZYMVN_VERSION={version}");
}
