pub(super) fn text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

pub(super) fn text_with_arg(
    key: &str,
    arg_name: &str,
    arg_value: impl Into<String>,
    english: impl FnOnce() -> String,
) -> String {
    crate::i18n::global().text_with_string_arg(key, arg_name, arg_value, english)
}
