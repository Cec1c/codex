use super::*;
use pretty_assertions::assert_eq;

#[test]
fn parses_manager_owned_update_cache() {
    let update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.1.4".to_string(),
        PathBuf::from(r"C:\ccu\update-dismissals"),
        r#"{"latestCcuVersion":"0.1.5","packageReady":true,"releaseUrl":"https://github.com/Cec1c/codex-cli-ultra/releases/tag/v0.1.5"}"#,
    );

    assert_eq!(
        update,
        Some(ManagedUpdate {
            manager_path: PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
            current_version: "0.1.4".to_string(),
            latest_version: "0.1.5".to_string(),
            release_url: "https://github.com/Cec1c/codex-cli-ultra/releases/tag/v0.1.5".to_string(),
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
        directory.path().to_path_buf(),
        r#"{"latestCcuVersion":"0.1.5","packageReady":true}"#,
    )
    .expect("managed update");
    let next_update = parse_update(
        PathBuf::from(r"C:\ccu\bin\ccu-manager.exe"),
        "0.1.4".to_string(),
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
            PathBuf::from(r"C:\ccu\update-dismissals"),
            r#"{"latestCcuVersion":"0.1.5","packageReady":false}"#,
        ),
        None
    );
}
