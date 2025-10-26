/// Accessors for build-time version metadata injected via `build.rs`.
pub fn current() -> &'static str {
    option_env!("LAZYMVN_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
}

/// Returns the build channel (e.g. "nightly", "main") if provided.
pub fn build_channel() -> Option<&'static str> {
    option_env!("LAZYMVN_BUILD_CHANNEL")
}

/// Returns `true` when this binary was produced from a nightly build.
pub fn is_nightly() -> bool {
    build_channel()
        .map(|channel| channel.eq_ignore_ascii_case("nightly"))
        .unwrap_or_else(|| current().contains("nightly"))
}

/// Tag associated with the build, when available.
pub fn build_tag() -> Option<&'static str> {
    option_env!("LAZYMVN_BUILD_TAG")
}

/// Commit SHA baked into the binary, when available.
pub fn commit_sha() -> Option<&'static str> {
    option_env!("LAZYMVN_COMMIT_SHA")
}
