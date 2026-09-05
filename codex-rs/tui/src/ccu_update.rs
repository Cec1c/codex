use serde::Deserialize;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use crate::version::CODEX_CLI_VERSION;

#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
const MANAGED_ENV: &str = "CODEX_CCU_MANAGED";
#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
const MANAGER_PATH_ENV: &str = "CODEX_CCU_MANAGER_PATH";
#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
const MANAGER_VERSION_ENV: &str = "CODEX_CCU_MANAGER_VERSION";
#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
const UPDATE_CACHE_PATH_ENV: &str = "CODEX_CCU_UPDATE_CACHE_PATH";
#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
const UPDATE_DISMISSALS_DIR_ENV: &str = "CODEX_CCU_UPDATE_DISMISSALS_DIR";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedUpdate {
    pub(crate) manager_path: PathBuf,
    pub(crate) current_version: String,
    pub(crate) latest_version: String,
    pub(crate) release_url: String,
    bundled_fork_version: Option<String>,
    current_fork_version: String,
    dismissals_directory: PathBuf,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCache {
    latest_ccu_version: String,
    package_ready: bool,
    release_url: Option<String>,
    #[serde(default)]
    bundled_fork_version: Option<String>,
}

#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
pub(crate) fn is_managed_environment() -> bool {
    std::env::var(MANAGED_ENV).as_deref() == Ok("1")
}

#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
pub(crate) fn current_update() -> Option<ManagedUpdate> {
    if !is_managed_environment() {
        return None;
    }
    let manager_path = PathBuf::from(std::env::var_os(MANAGER_PATH_ENV)?);
    if !manager_path.is_file() {
        return None;
    }
    let current_version = std::env::var(MANAGER_VERSION_ENV).ok()?;
    let cache_path = PathBuf::from(std::env::var_os(UPDATE_CACHE_PATH_ENV)?);
    let dismissals_directory = PathBuf::from(std::env::var_os(UPDATE_DISMISSALS_DIR_ENV)?);
    let cache_source = std::fs::read_to_string(cache_path).ok()?;
    parse_update(
        manager_path,
        current_version,
        CODEX_CLI_VERSION.to_string(),
        dismissals_directory,
        &cache_source,
    )
}

fn parse_update(
    manager_path: PathBuf,
    current_version: String,
    current_fork_version: String,
    dismissals_directory: PathBuf,
    cache_source: &str,
) -> Option<ManagedUpdate> {
    let cache: UpdateCache = serde_json::from_str(cache_source).ok()?;
    if !cache.package_ready
        || !is_ccu_version(&current_version)
        || !is_ccu_version(&cache.latest_ccu_version)
    {
        return None;
    }
    let release_url = cache
        .release_url
        .filter(|url| url.starts_with("https://github.com/Cec1c/codex-cli-ultra/releases/"))
        .unwrap_or_else(|| {
            format!(
                "https://github.com/Cec1c/codex-cli-ultra/releases/tag/v{}",
                cache.latest_ccu_version
            )
        });
    Some(ManagedUpdate {
        manager_path,
        current_version,
        latest_version: cache.latest_ccu_version,
        release_url,
        bundled_fork_version: cache.bundled_fork_version,
        current_fork_version,
        dismissals_directory,
    })
}

impl ManagedUpdate {
    pub(crate) fn should_prompt(&self) -> bool {
        (is_newer_ccu_version(&self.latest_version, &self.current_version)
            || self
                .bundled_fork_version
                .as_deref()
                .and_then(|latest| compare_fork_versions(latest, &self.current_fork_version))
                .is_some_and(|comparison| comparison == std::cmp::Ordering::Greater))
            && !self.dismissal_path().is_file()
    }

    fn dismissal_path(&self) -> PathBuf {
        self.dismissals_directory
            .join(format!("{}.dismissed", self.latest_version))
    }
}

#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
pub(crate) async fn dismiss_version(version: &str) -> anyhow::Result<()> {
    if !is_ccu_version(version) {
        anyhow::bail!("CCU update version must use x.y.z or x.y.z-alpha.N format");
    }
    let directory = PathBuf::from(
        std::env::var_os(UPDATE_DISMISSALS_DIR_ENV)
            .ok_or_else(|| anyhow::anyhow!("CCU update dismissal directory is missing"))?,
    );
    dismiss_at(&directory, version).await
}

async fn dismiss_at(directory: &Path, version: &str) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(directory).await?;
    let path = directory.join(format!("{version}.dismissed"));
    match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
    {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ParsedCcuVersion {
    major: u64,
    minor: u64,
    patch: u64,
    alpha: Option<u64>,
}

fn parse_ccu_version(version: &str) -> Option<ParsedCcuVersion> {
    let (core, alpha) = version
        .split_once("-alpha.")
        .map_or((version, None), |(core, value)| {
            (core, value.parse::<u64>().ok().filter(|value| *value > 0))
        });
    let mut components = core.split('.');
    let parsed = ParsedCcuVersion {
        major: components.next()?.parse().ok()?,
        minor: components.next()?.parse().ok()?,
        patch: components.next()?.parse().ok()?,
        alpha,
    };
    (components.next().is_none() && (alpha.is_some() || !version.contains("-"))).then_some(parsed)
}

fn is_ccu_version(version: &str) -> bool {
    parse_ccu_version(version).is_some()
}

fn is_newer_ccu_version(latest: &str, current: &str) -> bool {
    match (parse_ccu_version(latest), parse_ccu_version(current)) {
        (Some(latest), Some(current)) => {
            compare_ccu_versions(latest, current) == std::cmp::Ordering::Greater
        }
        _ => false,
    }
}

fn compare_ccu_versions(left: ParsedCcuVersion, right: ParsedCcuVersion) -> std::cmp::Ordering {
    let core = (left.major, left.minor, left.patch).cmp(&(right.major, right.minor, right.patch));
    if core != std::cmp::Ordering::Equal {
        return core;
    }
    match (left.alpha, right.alpha) {
        (None, None) => std::cmp::Ordering::Equal,
        (Some(left), Some(right)) => left.cmp(&right),
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
    }
}

fn parse_fork_version(version: &str) -> Option<(u64, u64, u64)> {
    let core = version.split_once('-').map_or(version, |(core, _)| core);
    let mut components = core.split('.');
    let parsed = (
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
    );
    components.next().is_none().then_some(parsed)
}

fn compare_fork_versions(latest: &str, current: &str) -> Option<std::cmp::Ordering> {
    Some(parse_fork_version(latest)?.cmp(&parse_fork_version(current)?))
}

#[cfg(test)]
#[path = "ccu_update_tests.rs"]
mod tests;
