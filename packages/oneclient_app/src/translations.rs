use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use freya::prelude::State;
use tokio::sync::watch;

use crate::i18n;

/// Translations shipped inside the binary. They are always offered, so a
/// language bundled here works with no network and does not depend on any
/// upstream CDN. RemixClient ships English (the source) and Russian.
const BUILTIN: &[(&str, &str)] = &[("ru_RU", include_str!("../lang/ru_RU.json"))];

const NAMES: &[(&str, &str, &str)] = &[("ru_RU", "Русский", "Russian")];

static WANTED: RwLock<Option<String>> = RwLock::new(None);
static CHANGED: LazyLock<watch::Sender<u64>> = LazyLock::new(|| watch::channel(0).0);

pub fn bind_revision(revision: State<u32>) {
    i18n::bind_revision(revision);
}

pub fn subscribe() -> watch::Receiver<u64> {
    CHANGED.subscribe()
}

pub fn names(code: Option<&str>) -> (String, String) {
    let Some(code) = code else {
        return ("English".to_string(), "English (US)".to_string());
    };
    NAMES
        .iter()
        .find(|(known, _, _)| known.eq_ignore_ascii_case(code))
        .map_or((code.to_string(), String::new()), |(_, native, english)| {
            (native.to_string(), english.to_string())
        })
}

/// The picker offers exactly what ships with the launcher: English (the
/// untranslated source) and every bundled language, currently Russian.
pub fn choices() -> Vec<Option<String>> {
    std::iter::once(None)
        .chain(BUILTIN.iter().map(|(code, _)| Some((*code).to_string())))
        .collect()
}

pub fn init(language: Option<String>) {
    select(language);
}

pub fn select(language: Option<String>) {
    let language = language.filter(|code| {
        !code.is_empty()
            && code.len() <= 16
            && code
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            && BUILTIN
                .iter()
                .any(|(known, _)| known.eq_ignore_ascii_case(code))
    });
    {
        let mut wanted = WANTED.write().unwrap_or_else(|e| e.into_inner());
        if *wanted == language {
            return;
        }
        wanted.clone_from(&language);
    }
    install_cached(language.as_deref());
}

fn notify_changed() {
    CHANGED.send_modify(|revision| *revision = revision.wrapping_add(1));
}

fn install_cached(code: Option<&str>) {
    let table = code.and_then(builtin);
    i18n::install(table);
    notify_changed();
}

fn builtin(code: &str) -> Option<HashMap<String, String>> {
    BUILTIN
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(code))
        .and_then(|(_, bytes)| parse_translations(bytes.as_bytes()))
}

fn parse_translations(bytes: &[u8]) -> Option<HashMap<String, String>> {
    let parsed: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(bytes).ok()?;
    let source = i18n::source();
    let table: HashMap<String, String> = parsed
        .into_iter()
        .filter_map(|(key, value)| {
            let value = value.as_str()?.to_string();
            let english = source.get(&key)?;
            (i18n::arity(english) == i18n::arity(&value)).then_some((key, value))
        })
        .collect();
    (!table.is_empty()).then_some(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translations_drop_unknown_keys_and_wrong_placeholders() {
        let table = parse_translations(
            br#"{
                "oneclient.common.cancel":"Anuluj",
                "oneclient.components.package_row.copies":"kopie",
                "oneclient.not.a.key":"x",
                "oneclient.cluster.settings.migrate.from":"Z %s"
            }"#,
        )
        .unwrap();
        assert_eq!(table.get("oneclient.common.cancel").unwrap(), "Anuluj");
        assert_eq!(
            table
                .get("oneclient.cluster.settings.migrate.from")
                .unwrap(),
            "Z %s"
        );
        assert!(!table.contains_key("oneclient.components.package_row.copies"));
        assert!(!table.contains_key("oneclient.not.a.key"));
    }

    #[test]
    fn picker_offers_english_and_bundled_languages_only() {
        let choices = choices();
        assert_eq!(choices.len(), BUILTIN.len() + 1);
        assert!(choices.contains(&None));
        assert!(choices.contains(&Some("ru_RU".to_string())));
    }

    #[test]
    fn select_ignores_languages_that_are_not_bundled() {
        // Pl_None is English: anything else must come from BUILTIN, so an unknown
        // code can never put the UI into a half-installed state.
        let before = WANTED.read().unwrap_or_else(|e| e.into_inner()).clone();
        select(Some("xx_XX".to_string()));
        assert_eq!(
            WANTED.read().unwrap_or_else(|e| e.into_inner()).clone(),
            before
        );
    }

    #[test]
    fn bundled_russian_has_only_known_keys_with_matching_placeholders() {
        let source = i18n::source();
        let parsed: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(BUILTIN[0].1).expect("bundled russian is valid JSON");

        for (key, value) in &parsed {
            let english = source
                .get(key)
                .unwrap_or_else(|| panic!("bundled key not in English: {key}"));
            let value = value.as_str().expect("a translation is a string");
            assert_eq!(
                i18n::arity(english),
                i18n::arity(value),
                "placeholder mismatch for {key}"
            );
        }

        let table = builtin("ru_RU").expect("bundled russian parses");
        assert_eq!(table.len(), parsed.len(), "no bundled key was dropped");
        assert_eq!(table.get("oneclient.common.yes").unwrap(), "Да");
    }
}
