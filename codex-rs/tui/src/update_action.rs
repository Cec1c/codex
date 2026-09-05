#[cfg(any(not(debug_assertions), test))]
use crate::ccu_update;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallContext;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallMethod;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::StandalonePlatform;

/// Update action the CLI should perform after the TUI exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateAction {
    /// Open CCU Manager with the selected target so the user can review network settings.
    CcuManager {
        manager_path: String,
        current_version: String,
        target_version: String,
        release_url: String,
    },
    /// Update via `npm install -g @openai/codex@latest`.
    NpmGlobalLatest,
    /// Update via `bun install -g @openai/codex@latest`.
    BunGlobalLatest,
    /// Update via `vp install -g @openai/codex@latest`.
    VitePlusGlobalLatest,
    /// Update via `pnpm add -g @openai/codex@latest`.
    PnpmGlobalLatest,
    /// Update via `brew upgrade codex`.
    BrewUpgrade,
    /// Update via `curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh`.
    StandaloneUnix,
    /// Update via `$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex`.
    StandaloneWindows,
}

impl UpdateAction {
    #[cfg(any(not(debug_assertions), test))]
    pub(crate) fn from_install_context(context: &InstallContext) -> Option<Self> {
        match &context.method {
            InstallMethod::Npm => Some(UpdateAction::NpmGlobalLatest),
            InstallMethod::Bun => Some(UpdateAction::BunGlobalLatest),
            InstallMethod::VitePlus => Some(UpdateAction::VitePlusGlobalLatest),
            InstallMethod::Pnpm => Some(UpdateAction::PnpmGlobalLatest),
            InstallMethod::Brew => Some(UpdateAction::BrewUpgrade),
            InstallMethod::Standalone { platform, .. } => Some(match platform {
                StandalonePlatform::Unix => UpdateAction::StandaloneUnix,
                StandalonePlatform::Windows => UpdateAction::StandaloneWindows,
            }),
            InstallMethod::Other => None,
        }
    }

    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(&self) -> (String, Vec<String>) {
        match self {
            UpdateAction::CcuManager {
                manager_path,
                target_version,
                ..
            } => (
                manager_path.clone(),
                vec![
                    "--upgrade".to_string(),
                    "--target".to_string(),
                    target_version.clone(),
                ],
            ),
            UpdateAction::NpmGlobalLatest => (
                "npm".to_string(),
                vec![
                    "install".to_string(),
                    "-g".to_string(),
                    "@openai/codex@latest".to_string(),
                ],
            ),
            UpdateAction::BunGlobalLatest => (
                "bun".to_string(),
                vec![
                    "install".to_string(),
                    "-g".to_string(),
                    "@openai/codex@latest".to_string(),
                ],
            ),
            UpdateAction::VitePlusGlobalLatest => (
                "vp".to_string(),
                vec![
                    "install".to_string(),
                    "-g".to_string(),
                    "@openai/codex@latest".to_string(),
                ],
            ),
            UpdateAction::PnpmGlobalLatest => (
                "pnpm".to_string(),
                vec![
                    "add".to_string(),
                    "-g".to_string(),
                    "@openai/codex@latest".to_string(),
                ],
            ),
            UpdateAction::BrewUpgrade => (
                "brew".to_string(),
                vec![
                    "upgrade".to_string(),
                    "--cask".to_string(),
                    "codex".to_string(),
                ],
            ),
            UpdateAction::StandaloneUnix => (
                "sh".to_string(),
                vec![
                    "-c".to_string(),
                    "curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh"
                        .to_string(),
                ],
            ),
            UpdateAction::StandaloneWindows => (
                "powershell".to_string(),
                vec![
                    "-ExecutionPolicy".to_string(),
                    "Bypass".to_string(),
                    "-c".to_string(),
                    "$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex"
                        .to_string(),
                ],
            ),
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(&self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command.as_str()).chain(args.iter().map(String::as_str)))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }

    pub(crate) fn ccu_prompt_details(&self) -> Option<(&str, &str)> {
        match self {
            UpdateAction::CcuManager {
                current_version,
                release_url,
                ..
            } => Some((current_version, release_url)),
            UpdateAction::NpmGlobalLatest
            | UpdateAction::BunGlobalLatest
            | UpdateAction::VitePlusGlobalLatest
            | UpdateAction::PnpmGlobalLatest
            | UpdateAction::BrewUpgrade
            | UpdateAction::StandaloneUnix
            | UpdateAction::StandaloneWindows => None,
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
#[cfg(any(not(debug_assertions), test))]
pub fn get_update_action() -> Option<UpdateAction> {
    if ccu_update::is_managed_environment() {
        return ccu_update::current_update().map(|update| UpdateAction::CcuManager {
            manager_path: update.manager_path.to_string_lossy().into_owned(),
            current_version: update.current_version,
            target_version: update.latest_version,
            release_url: update.release_url,
        });
    }
    UpdateAction::from_install_context(InstallContext::current())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;

    #[test]
    fn maps_install_context_to_update_action() {
        let native_release_dir =
            AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("native-release"))
                .expect("temp dir path should be absolute");

        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Other,
                package_layout: None,
            }),
            None
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Npm,
                package_layout: None,
            }),
            Some(UpdateAction::NpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Bun,
                package_layout: None,
            }),
            Some(UpdateAction::BunGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Pnpm,
                package_layout: None,
            }),
            Some(UpdateAction::PnpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Brew,
                package_layout: None,
            }),
            Some(UpdateAction::BrewUpgrade)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Standalone {
                    platform: StandalonePlatform::Unix,
                    release_dir: native_release_dir.clone(),
                    resources_dir: Some(native_release_dir.join("codex-resources")),
                },
                package_layout: None,
            }),
            Some(UpdateAction::StandaloneUnix)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Standalone {
                    platform: StandalonePlatform::Windows,
                    release_dir: native_release_dir.clone(),
                    resources_dir: Some(native_release_dir.join("codex-resources")),
                },
                package_layout: None,
            }),
            Some(UpdateAction::StandaloneWindows)
        );
    }

    #[test]
    fn standalone_update_commands_rerun_latest_installer() {
        assert_eq!(
            UpdateAction::StandaloneUnix.command_args(),
            (
                "sh".to_string(),
                vec![
                    "-c".to_string(),
                    "curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh"
                        .to_string()
                ],
            )
        );
        assert_eq!(
            UpdateAction::StandaloneWindows.command_args(),
            (
                "powershell".to_string(),
                vec![
                    "-ExecutionPolicy".to_string(),
                    "Bypass".to_string(),
                    "-c".to_string(),
                    "$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex"
                        .to_string()
                ],
            )
        );
    }

    #[test]
    fn ccu_manager_update_opens_the_selected_target() {
        let action = UpdateAction::CcuManager {
            manager_path: r"C:\ccu\bin\ccu-manager.exe".to_string(),
            current_version: "0.1.4".to_string(),
            target_version: "0.1.5".to_string(),
            release_url: "https://github.com/Cec1c/codex-cli-ultra/releases/tag/v0.1.5".to_string(),
        };

        assert_eq!(
            action.command_args(),
            (
                r"C:\ccu\bin\ccu-manager.exe".to_string(),
                vec![
                    "--upgrade".to_string(),
                    "--target".to_string(),
                    "0.1.5".to_string(),
                ],
            )
        );
    }
}
