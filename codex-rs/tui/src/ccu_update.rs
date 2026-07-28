use crate::update_versions::is_newer;
use serde::Deserialize;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

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
    dismissals_directory: PathBuf,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCache {
    latest_ccu_version: String,
    package_ready: bool,
    release_url: Option<String>,
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
        dismissals_directory,
        &cache_source,
    )
}

fn parse_update(
    manager_path: PathBuf,
    current_version: String,
    dismissals_directory: PathBuf,
    cache_source: &str,
) -> Option<ManagedUpdate> {
    let cache: UpdateCache = serde_json::from_str(cache_source).ok()?;
    if !cache.package_ready
        || !is_stable_version(&current_version)
        || !is_stable_version(&cache.latest_ccu_version)
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
        dismissals_directory,
    })
}

impl ManagedUpdate {
    pub(crate) fn should_prompt(&self) -> bool {
        is_newer(&self.latest_version, &self.current_version).unwrap_or(false)
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
    if !is_stable_version(version) {
        anyhow::bail!("CCU update version must use x.y.z format");
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

fn is_stable_version(version: &str) -> bool {
    let mut components = version.split('.');
    (0..3).all(|_| {
        components.next().is_some_and(|value| {
            !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
        })
    }) && components.next().is_none()
}

#[cfg(test)]
#[path = "ccu_update_tests.rs"]
mod tests;
