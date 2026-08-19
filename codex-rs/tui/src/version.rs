/// The current Codex CLI version as embedded at compile time.
pub const CODEX_CLI_VERSION: &str = if cfg!(test) {
    "0.0.0"
} else {
    match option_env!("CODEX_CCU_BUILD_VERSION") {
        Some(version) => version,
        None => env!("CARGO_PKG_VERSION"),
    }
};
