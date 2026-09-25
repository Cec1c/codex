use codex_utils_approval_presets::ApprovalPreset;

pub(super) fn text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

pub(super) fn text_with_arg<F>(
    key: &str,
    arg_name: &str,
    arg_value: impl Into<String>,
    english: F,
) -> String
where
    F: FnOnce() -> String,
{
    crate::i18n::global().text_with_string_arg(key, arg_name, arg_value, english)
}

pub(super) fn ask_for_approval_label() -> String {
    text("permissions-ask-for-approval", "Ask for approval")
}

pub(super) fn approve_for_me_label() -> String {
    text("permissions-approve-for-me", "Approve for me")
}

pub(super) fn auto_review_description() -> String {
    text(
        "permissions-auto-review-description",
        "Only ask for actions detected as potentially unsafe",
    )
}

pub(super) fn preset_label(preset: &ApprovalPreset) -> String {
    match preset.id {
        "read-only" => text("permissions-read-only", "Read Only"),
        "auto" => text("permissions-default", "Default"),
        "full-access" => text("permissions-full-access", "Full Access"),
        _ => preset.label.to_string(),
    }
}

pub(super) fn preset_description(preset: &ApprovalPreset) -> String {
    let english = super::permissions_menu::permission_preset_description(preset);
    match preset.id {
        "read-only" => text("permissions-read-only-description", english),
        "auto" => text("permissions-default-description", english),
        "full-access" => text("permissions-full-access-description", english),
        _ => english.to_string(),
    }
}
