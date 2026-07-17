use std::fs;
use std::path::Path;

use fluent_bundle::FluentArgs;
use pretty_assertions::assert_eq;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use tempfile::TempDir;

use super::Localizer;
use super::discover_language_packs;
use super::language_pack_root;
use super::normalized_language;
use super::save_language_preference;
use super::self_check_json;

const TEST_FTL: &str = r#"
status-line-configure-title = 配置状态栏
history-worked-for = 工作了 { $duration }
onboarding-sign-in-chatgpt = 登录 ChatGPT
status-card-model-label = 模型
command-popup-no-matches = 无匹配项
approval-run-command-title = 是否运行以下命令？
composer-write-file-tests = 为 @filename 编写测试
mcp-client-failed-to-start = MCP 客户端 `{ $name }` 启动失败
status-line-context-used = 上下文已用 { $percent }%
slash-unrecognized-command = 无法识别命令“/{ $name }”。输入“/”查看支持的命令列表。
slash-model-description = 选择模型和推理强度
slash-language-description = 查看或选择显示语言
language-saved = 已选择 { $locale }；重启 Codex 后生效。
language-unsupported = 语言 { $locale } 未安装或不兼容。
"#;

#[test]
fn language_aliases_are_normalized() {
    assert_eq!(normalized_language("zh-cn"), Some("zh-CN".to_string()));
    assert_eq!(normalized_language("zh-Hans"), Some("zh-Hans".to_string()));
    assert_eq!(normalized_language("English"), Some("en".to_string()));
    assert_eq!(normalized_language("en-us"), Some("en-US".to_string()));
    assert_eq!(normalized_language("not a locale"), None);
}

#[test]
fn static_message_uses_fluent_translation() {
    let localizer = Localizer::from_ftl("zh-CN", TEST_FTL);

    assert_eq!(
        localizer.text("status-line-configure-title", None, || {
            "Configure Status Line".to_string()
        }),
        "配置状态栏"
    );
}

#[test]
fn duration_argument_is_formatted() {
    let localizer = Localizer::from_ftl("zh-CN", TEST_FTL);
    let mut args = FluentArgs::new();
    args.set("duration", "7m 57s");

    assert_eq!(
        localizer.text("history-worked-for", Some(&args), || {
            "Worked for 7m 57s".to_string()
        }),
        "工作了 7m 57s"
    );
}

#[test]
fn missing_message_uses_english_closure() {
    let localizer = Localizer::from_ftl("zh-CN", TEST_FTL);

    assert_eq!(
        localizer.text("i18n-missing-key", None, || {
            "English fallback".to_string()
        }),
        "English fallback"
    );
}

#[test]
fn whitespace_only_message_uses_english_closure() {
    let localizer = Localizer::from_ftl("zh-CN", r#"probe-empty = { "   " }"#);

    assert_eq!(
        localizer.text("probe-empty", None, || "English fallback".to_string()),
        "English fallback"
    );
}

#[test]
fn malformed_resource_disables_the_whole_localizer() {
    let localizer = Localizer::from_ftl(
        "zh-CN",
        "status-line-configure-title = 配置状态栏\nbroken = {",
    );

    assert_eq!(
        localizer.text("status-line-configure-title", None, || {
            "Configure Status Line".to_string()
        }),
        "Configure Status Line"
    );
}

#[test]
fn missing_fluent_argument_uses_english_closure() {
    let localizer = Localizer::from_ftl("zh-CN", TEST_FTL);

    assert_eq!(
        localizer.text("history-worked-for", None, || {
            "Worked for 7m 57s".to_string()
        }),
        "Worked for 7m 57s"
    );
}

#[test]
fn invalid_locale_disables_the_whole_localizer() {
    let localizer = Localizer::from_ftl("not a locale", TEST_FTL);

    assert_eq!(
        localizer.text("status-line-configure-title", None, || {
            "Configure Status Line".to_string()
        }),
        "Configure Status Line"
    );
}

#[test]
fn self_check_includes_catalog_messages_and_missing_key_fallback() {
    let localizer = Localizer::from_ftl("zh-CN", TEST_FTL);
    let payload: Value = serde_json::from_str(&self_check_json(&localizer)).expect("valid JSON");

    assert_eq!(payload["schemaVersion"], 1);
    assert_eq!(payload["active"], true);
    assert_eq!(payload["locale"], "zh-CN");
    assert_eq!(
        payload["messages"]["tui.status-line.setup.configure-title"],
        "配置状态栏"
    );
    assert_eq!(
        payload["messages"]["tui.onboarding.auth.sign-in-chatgpt"],
        "登录 ChatGPT"
    );
    assert_eq!(payload["messages"]["tui.status-card.model-label"], "模型");
    assert_eq!(
        payload["messages"]["tui.command-popup.no-matches"],
        "无匹配项"
    );
    assert_eq!(
        payload["messages"]["tui.approval.run-command-title"],
        "是否运行以下命令？"
    );
    assert_eq!(
        payload["messages"]["tui.composer.placeholder.write-file-tests"],
        "为 @filename 编写测试"
    );
    assert_eq!(
        payload["messages"]["tui.mcp.client-failed-to-start"],
        "MCP 客户端 `openaiDeveloperDocs` 启动失败"
    );
    assert_eq!(
        payload["messages"]["tui.status-line.context-used"],
        "上下文已用 58%"
    );
    assert_eq!(
        payload["messages"]["tui.slash-command.unrecognized"],
        "无法识别命令“/sdsd”。输入“/”查看支持的命令列表。"
    );
    assert_eq!(
        payload["messages"]["tui.slash-command.description.model"],
        "选择模型和推理强度"
    );
    assert_eq!(
        payload["messages"]["tui.slash-command.description.language"],
        "查看或选择显示语言"
    );
    assert_eq!(
        payload["messages"]["tui.history.worked-for"],
        "工作了 7m 57s"
    );
    assert_eq!(
        payload["messages"].as_object().map(serde_json::Map::len),
        Some(131)
    );
    assert_eq!(payload["messages"]["i18n.missing-key"], "English fallback");
}

#[test]
fn english_self_check_is_inactive() {
    let payload: Value =
        serde_json::from_str(&self_check_json(&Localizer::english())).expect("valid JSON");

    assert_eq!(payload["active"], false);
    assert_eq!(payload["locale"], Value::Null);
    assert_eq!(
        payload["messages"]["tui.history.worked-for"],
        "Worked for 7m 57s"
    );
    assert_eq!(
        payload["messages"]["tui.slash-command.description.model"],
        "choose what model and reasoning effort to use"
    );
}

#[test]
fn external_language_pack_is_discovered_and_loaded() {
    let temp = TempDir::new().expect("temp dir");
    write_language_pack(
        temp.path(),
        "zh-CN",
        "zh-CN",
        /*api_min*/ 1,
        /*api_max*/ 1,
        TEST_FTL,
        /*hash_override*/ None,
    );

    let candidates = discover_language_packs(temp.path()).expect("discover packs");
    assert_eq!(candidates.len(), 1);
    assert!(candidates[0].is_available());
    assert_eq!(candidates[0].locale, "zh-CN");
    assert_eq!(candidates[0].display_name, "简体中文 (zh-CN)");

    let localizer = Localizer::from_language_pack_root("zh", temp.path());
    assert_eq!(
        localizer.text("status-line-configure-title", None, || {
            "Configure Status Line".to_string()
        }),
        "配置状态栏"
    );
}

#[test]
fn incompatible_language_pack_is_disabled_with_a_reason() {
    let temp = TempDir::new().expect("temp dir");
    write_language_pack(
        temp.path(),
        "fr-FR",
        "fr-FR",
        /*api_min*/ 2,
        /*api_max*/ 3,
        "probe = Bonjour",
        /*hash_override*/ None,
    );

    let candidates = discover_language_packs(temp.path()).expect("discover packs");
    assert_eq!(candidates.len(), 1);
    assert!(!candidates[0].is_available());
    assert_eq!(
        candidates[0].disabled_reason.as_deref(),
        Some("Requires i18n API 2..=3; this Codex build provides 1.")
    );
}

#[test]
fn hash_mismatch_disables_language_pack() {
    let temp = TempDir::new().expect("temp dir");
    let invalid_hash = "0".repeat(64);
    write_language_pack(
        temp.path(),
        "zh-CN",
        "zh-CN",
        /*api_min*/ 1,
        /*api_max*/ 1,
        TEST_FTL,
        Some(&invalid_hash),
    );

    let candidates = discover_language_packs(temp.path()).expect("discover packs");
    assert_eq!(
        candidates[0].disabled_reason.as_deref(),
        Some("messages.ftl SHA256 does not match manifest.json.")
    );
}

#[test]
fn missing_language_pack_root_is_an_empty_catalog() {
    let temp = TempDir::new().expect("temp dir");
    let candidates = discover_language_packs(&temp.path().join("missing")).expect("discover packs");
    assert!(candidates.is_empty());
}

#[test]
fn language_preference_accepts_an_installed_primary_language_alias() {
    let temp = TempDir::new().expect("temp dir");
    let root = language_pack_root(temp.path());
    write_language_pack(
        &root, "zh-CN", "zh-CN", /*api_min*/ 1, /*api_max*/ 1, TEST_FTL,
        /*hash_override*/ None,
    );

    save_language_preference(temp.path(), "zh").expect("save language preference");
    assert_eq!(
        fs::read_to_string(temp.path().join("ui-language")).expect("read preference"),
        "zh-CN\n"
    );
}

fn write_language_pack(
    root: &Path,
    directory: &str,
    locale: &str,
    api_min: u32,
    api_max: u32,
    source: &str,
    hash_override: Option<&str>,
) {
    let pack_dir = root.join(directory);
    fs::create_dir_all(&pack_dir).expect("create pack directory");
    fs::write(pack_dir.join("messages.ftl"), source).expect("write FTL");
    let hash = hash_override
        .map(str::to_string)
        .unwrap_or_else(|| format!("{:x}", Sha256::digest(source.as_bytes())));
    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "type": "language",
        "id": format!("test.{locale}"),
        "locale": locale,
        "displayName": "Simplified Chinese",
        "nativeName": "简体中文",
        "i18nApi": {
            "min": api_min,
            "max": api_max
        },
        "resources": [
            {
                "path": "messages.ftl",
                "sha256": format!("sha256:{hash}")
            }
        ]
    });
    fs::write(
        pack_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}
