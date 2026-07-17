use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use fluent_bundle::FluentResource;
use serde::Deserialize;
use sha2::Digest;
use sha2::Sha256;
use unic_langid::LanguageIdentifier;

pub(crate) const I18N_API_VERSION: u32 = 1;
pub(crate) const LANGUAGE_PACK_ROOT_ENV: &str = "CODEX_CCU_LANGUAGE_PACK_ROOT";

const LANGUAGE_PACK_RELATIVE_ROOT: &str = "ccu/languages";
const LANGUAGE_PACK_MANIFEST: &str = "manifest.json";
const LANGUAGE_PACK_RESOURCE: &str = "messages.ftl";
const LANGUAGE_PACK_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub(crate) struct LanguagePackCandidate {
    pub(crate) directory_name: String,
    pub(crate) id: Option<String>,
    pub(crate) locale: String,
    pub(crate) display_name: String,
    pub(crate) source: Option<String>,
    pub(crate) disabled_reason: Option<String>,
}

impl LanguagePackCandidate {
    pub(crate) fn is_available(&self) -> bool {
        self.source.is_some() && self.disabled_reason.is_none()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LanguagePackManifest {
    schema_version: u32,
    #[serde(rename = "type")]
    package_type: String,
    id: String,
    locale: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    native_name: Option<String>,
    i18n_api: I18nApiRange,
    resources: Vec<LanguagePackResource>,
}

#[derive(Debug, Deserialize)]
struct I18nApiRange {
    min: u32,
    max: u32,
}

#[derive(Debug, Deserialize)]
struct LanguagePackResource {
    path: String,
    sha256: String,
}

pub(crate) fn language_pack_root(codex_home: &Path) -> PathBuf {
    std::env::var_os(LANGUAGE_PACK_ROOT_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| codex_home.join(LANGUAGE_PACK_RELATIVE_ROOT))
}

pub(crate) fn discover_language_packs(root: &Path) -> Result<Vec<LanguagePackCandidate>, String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "Could not read the language pack directory: {error}"
            ));
        }
    };

    let mut candidates = entries
        .filter_map(Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => Some(inspect_language_pack(&entry.path())),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut locale_counts = HashMap::new();
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.is_available())
    {
        *locale_counts
            .entry(candidate.locale.to_ascii_lowercase())
            .or_insert(0usize) += 1;
    }
    for candidate in &mut candidates {
        if candidate.is_available()
            && locale_counts
                .get(&candidate.locale.to_ascii_lowercase())
                .copied()
                .unwrap_or_default()
                > 1
        {
            candidate.source = None;
            candidate.disabled_reason = Some(format!(
                "More than one installed pack declares locale {}.",
                candidate.locale
            ));
        }
    }

    candidates.sort_by(|left, right| {
        left.display_name
            .to_ascii_lowercase()
            .cmp(&right.display_name.to_ascii_lowercase())
            .then_with(|| left.directory_name.cmp(&right.directory_name))
    });
    Ok(candidates)
}

pub(crate) fn normalized_requested_locale(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let alias = match trimmed.to_ascii_lowercase().as_str() {
        "english" => "en",
        "chinese" => "zh",
        _ => trimmed,
    };
    alias
        .parse::<LanguageIdentifier>()
        .ok()
        .map(|locale| locale.to_string())
}

pub(crate) fn is_english_locale(input: &str) -> bool {
    normalized_requested_locale(input)
        .and_then(|locale| locale.split('-').next().map(str::to_string))
        .is_some_and(|language| language.eq_ignore_ascii_case("en"))
}

pub(crate) fn resolve_language_pack<'a>(
    input: &str,
    candidates: &'a [LanguagePackCandidate],
) -> Option<&'a LanguagePackCandidate> {
    let normalized = normalized_requested_locale(input)?;
    let exact = candidates.iter().find(|candidate| {
        candidate.is_available() && candidate.locale.eq_ignore_ascii_case(&normalized)
    });
    if exact.is_some() || normalized.contains('-') {
        return exact;
    }

    let mut matching_language = candidates.iter().filter(|candidate| {
        candidate.is_available()
            && candidate
                .locale
                .split('-')
                .next()
                .is_some_and(|language| language.eq_ignore_ascii_case(&normalized))
    });
    let candidate = matching_language.next()?;
    matching_language.next().is_none().then_some(candidate)
}

fn inspect_language_pack(directory: &Path) -> LanguagePackCandidate {
    let directory_name = directory
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());
    let manifest_path = directory.join(LANGUAGE_PACK_MANIFEST);
    let manifest_source = match fs::read_to_string(&manifest_path) {
        Ok(source) => source,
        Err(error) => {
            return disabled_candidate(
                directory_name,
                /*manifest*/ None,
                format!("Could not read manifest.json: {error}"),
            );
        }
    };
    let manifest = match serde_json::from_str::<LanguagePackManifest>(&manifest_source) {
        Ok(manifest) => manifest,
        Err(error) => {
            return disabled_candidate(
                directory_name,
                /*manifest*/ None,
                format!("Invalid manifest.json: {error}"),
            );
        }
    };

    if manifest.schema_version != LANGUAGE_PACK_SCHEMA_VERSION {
        let schema_version = manifest.schema_version;
        return disabled_candidate(
            directory_name,
            Some(manifest),
            format!(
                "Unsupported language pack schema {schema_version}; expected {LANGUAGE_PACK_SCHEMA_VERSION}."
            ),
        );
    }
    if manifest.package_type != "language" {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "manifest.json type must be language.".to_string(),
        );
    }
    if manifest.i18n_api.min > I18N_API_VERSION || manifest.i18n_api.max < I18N_API_VERSION {
        let min = manifest.i18n_api.min;
        let max = manifest.i18n_api.max;
        return disabled_candidate(
            directory_name,
            Some(manifest),
            format!(
                "Requires i18n API {min}..={max}; this Codex build provides {I18N_API_VERSION}."
            ),
        );
    }

    let locale = match manifest.locale.parse::<LanguageIdentifier>() {
        Ok(locale) => locale,
        Err(_) => {
            return disabled_candidate(
                directory_name,
                Some(manifest),
                "manifest.json contains an invalid locale.".to_string(),
            );
        }
    };
    if locale.language.as_str() == "en" {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "English is built into Codex and cannot be replaced by an external pack.".to_string(),
        );
    }

    let Some(resource) = manifest
        .resources
        .iter()
        .find(|resource| resource.path == LANGUAGE_PACK_RESOURCE)
    else {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "manifest.json must declare messages.ftl.".to_string(),
        );
    };
    let Some(expected_hash) = resource.sha256.strip_prefix("sha256:") else {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "messages.ftl hash must use the sha256:<hex> form.".to_string(),
        );
    };
    if expected_hash.len() != 64
        || !expected_hash
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "messages.ftl hash is not a valid SHA256 value.".to_string(),
        );
    }

    let resource_bytes = match fs::read(directory.join(LANGUAGE_PACK_RESOURCE)) {
        Ok(bytes) => bytes,
        Err(error) => {
            return disabled_candidate(
                directory_name,
                Some(manifest),
                format!("Could not read messages.ftl: {error}"),
            );
        }
    };
    let actual_hash = format!("{:x}", Sha256::digest(&resource_bytes));
    if !actual_hash.eq_ignore_ascii_case(expected_hash) {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "messages.ftl SHA256 does not match manifest.json.".to_string(),
        );
    }
    let source = match String::from_utf8(resource_bytes) {
        Ok(source) => source,
        Err(_) => {
            return disabled_candidate(
                directory_name,
                Some(manifest),
                "messages.ftl must be UTF-8.".to_string(),
            );
        }
    };
    if FluentResource::try_new(source.clone()).is_err() {
        return disabled_candidate(
            directory_name,
            Some(manifest),
            "messages.ftl could not be parsed as Fluent.".to_string(),
        );
    }

    let locale = locale.to_string();
    let display_name = manifest_display_name(&manifest, &locale);
    LanguagePackCandidate {
        directory_name,
        id: Some(manifest.id),
        locale,
        display_name,
        source: Some(source),
        disabled_reason: None,
    }
}

fn disabled_candidate(
    directory_name: String,
    manifest: Option<LanguagePackManifest>,
    reason: String,
) -> LanguagePackCandidate {
    let locale = manifest
        .as_ref()
        .map(|manifest| manifest.locale.clone())
        .filter(|locale| !locale.trim().is_empty())
        .unwrap_or_else(|| directory_name.clone());
    let display_name = manifest
        .as_ref()
        .map(|manifest| manifest_display_name(manifest, &locale))
        .unwrap_or_else(|| directory_name.clone());
    LanguagePackCandidate {
        directory_name,
        id: manifest.map(|manifest| manifest.id),
        locale,
        display_name,
        source: None,
        disabled_reason: Some(reason),
    }
}

fn manifest_display_name(manifest: &LanguagePackManifest, locale: &str) -> String {
    let label = manifest
        .native_name
        .as_deref()
        .or(manifest.display_name.as_deref())
        .map(str::trim)
        .filter(|label| !label.is_empty());
    match label {
        Some(label) if !label.eq_ignore_ascii_case(locale) => format!("{label} ({locale})"),
        _ => locale.to_string(),
    }
}
