use super::*;
use pretty_assertions::assert_eq;

#[test]
fn parses_manager_owned_update_cache() {
    let update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.1.4".to_string(),
        "0.148.0-ccu.i18n.1".to_string(),
        PathBuf::from(r"C:\ccu\update-dismissals"),
        r#"{"latestCcuVersion":"0.1.5","packageReady":true,"releaseUrl":"https://github.com/Cec1c/codex-cli-ultra/releases/tag/v0.1.5","bundledForkVersion":"0.146.0-ccu.i18n.1"}"#,
    );

    assert_eq!(
        update,
        Some(ManagedUpdate {
            manager_path: PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
            current_version: "0.1.4".to_string(),
            latest_version: "0.1.5".to_string(),
            release_url: "https://github.com/Cec1c/codex-cli-ultra/releases/tag/v0.1.5".to_string(),
            bundled_fork_version: Some("0.146.0-ccu.i18n.1".to_string()),
            current_fork_version: "0.148.0-ccu.i18n.1".to_string(),
            dismissals_directory: PathBuf::from(r"C:\ccu\update-dismissals"),
        })
    );
}

#[tokio::test]
async fn dismissal_marker_suppresses_only_that_version() {
    let directory = tempfile::tempdir().expect("tempdir");
    dismiss_at(directory.path(), "0.1.5")
        .await
        .expect("dismiss version");
    let update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.1.4".to_string(),
        "0.148.0-ccu.i18n.1".to_string(),
        directory.path().to_path_buf(),
        r#"{"latestCcuVersion":"0.1.5","packageReady":true}"#,
    )
    .expect("managed update");
    let next_update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.1.4".to_string(),
        "0.148.0-ccu.i18n.1".to_string(),
        directory.path().to_path_buf(),
        r#"{"latestCcuVersion":"0.1.6","packageReady":true}"#,
    )
    .expect("next managed update");

    assert!(!update.should_prompt());
    assert!(next_update.should_prompt());
}

#[test]
fn incomplete_release_cache_is_ignored() {
    assert_eq!(
        parse_update(
            PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
            "0.1.4".to_string(),
            "0.148.0-ccu.i18n.1".to_string(),
            PathBuf::from(r"C:\ccu\update-dismissals"),
            r#"{"latestCcuVersion":"0.1.5","packageReady":false}"#,
        ),
        None
    );
}

#[test]
fn alpha_manager_prompts_when_bundled_fork_is_newer() {
    let update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.2.0-alpha.4".to_string(),
        "0.148.0-ccu.i18n.1".to_string(),
        PathBuf::from(r"C:\ccu\update-dismissals"),
        r#"{"latestCcuVersion":"0.1.23","packageReady":true,"bundledForkVersion":"0.153.4-ccu.i18n.1"}"#,
    )
    .expect("managed update");

    assert!(update.should_prompt());
}

#[test]
fn alpha_manager_does_not_prompt_for_older_fork_or_manager() {
    let update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.2.0-alpha.4".to_string(),
        "0.153.4-ccu.i18n.1".to_string(),
        PathBuf::from(r"C:\ccu\update-dismissals"),
        r#"{"latestCcuVersion":"0.1.23","packageReady":true,"bundledForkVersion":"0.153.4-ccu.i18n.1"}"#,
    )
    .expect("managed update");

    assert!(!update.should_prompt());
}
