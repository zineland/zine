use std::collections::HashMap;

static LOCALES: [(&str, &str); 7] = [
    ("en", "English"),
    ("fr", "Français"),
    ("es", "Español"),
    ("ja", "日本語"),
    ("zh", "中文"),
    ("zh_CN", "简体中文"),
    ("zh_TW", "繁體中文"),
];

pub fn get_locale_name(locale: &str) -> Option<&'static str> {
    HashMap::from(LOCALES).get(locale).copied()
}
