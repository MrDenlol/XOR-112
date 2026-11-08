use crate::preferences::Language;
use std::{cell::Cell, collections::HashMap, sync::OnceLock};
thread_local! {static LANGUAGE:Cell<Language>=const{Cell::new(Language::Russian)};}
pub fn set_language(language: Language) {
    LANGUAGE.with(|v| v.set(language));
}
pub fn is_english() -> bool {
    LANGUAGE.with(|v| v.get() == Language::English)
}
fn catalog() -> &'static HashMap<String, String> {
    static DATA: OnceLock<HashMap<String, String>> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("translations.json"))
            .expect("built-in translation catalog")
    })
}
pub fn tr(text: &str) -> &str {
    if is_english() {
        catalog().get(text).map(String::as_str).unwrap_or(text)
    } else {
        text
    }
}
pub fn display_message(text: &str) -> String {
    for (ru, en) in catalog() {
        let target = if is_english() { en } else { ru };
        if text == ru || text == en {
            return target.clone();
        }
        // Status messages containing a path/error keep their dynamic value.
        if ru.matches('{').count() == 1 {
            for source in [ru, en] {
                if let (Some(open), Some(close)) = (source.find('{'), source.find('}')) {
                    if let Some(rest) = text
                        .strip_prefix(&source[..open])
                        .and_then(|s| s.strip_suffix(&source[close + 1..]))
                    {
                        if let (Some(a), Some(b)) = (target.find('{'), target.find('}')) {
                            return format!("{}{}{}", &target[..a], rest, &target[b + 1..]);
                        }
                    }
                }
            }
        }
    }
    text.into()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn language_switch_translates_commands_and_preserves_user_text() {
        set_language(Language::English);
        assert_eq!(tr("Настройки"), "Settings");
        assert_eq!(tr("AND"), "AND");
        assert_eq!(tr("Моя схема 123"), "Моя схема 123");
        assert_eq!(
            display_message("Сохранено: C:/test.binary"),
            "Saved: C:/test.binary"
        );
        set_language(Language::Russian);
        assert_eq!(display_message("Ready"), "Готово");
    }
}
