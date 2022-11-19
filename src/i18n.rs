use std::collections::HashMap;

static LOCALE_ENGLISH: (&str, &str) = ("en", "English");
static LOCALE_FRANCE: (&str, &str) = ("fr", "Français");
static LOCALE_SPANISH: (&str, &str) = ("es", "Español");
static LOCALE_JAPANESE: (&str, &str) = ("ja", "日本語");
static LOCALE_SIMPLIFIED_CHINESE: (&str, &str) = ("zh_CN", "简体中文");
static LOCALE_TRADITIONAL_CHINESE: (&str, &str) = ("zh_TW", "繁體中文");

pub fn get_locale_name(locale: &str) -> Option<&'static str> {
    HashMap::from([
        LOCALE_ENGLISH,
        LOCALE_FRANCE,
        LOCALE_SPANISH,
        LOCALE_JAPANESE,
        LOCALE_SIMPLIFIED_CHINESE,
        LOCALE_TRADITIONAL_CHINESE,
    ])
    .get(locale)
    .copied()
}
