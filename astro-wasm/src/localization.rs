use crate::locales::{en, te};

/// Look up `key` for the given `lang` locale.
///
/// Fallback chain:
/// 1. Key found in requested locale → return it.
/// 2. Key missing in requested locale (or `lang` unknown) → try English.
/// 3. Key missing in English → return `"[missing]"`.
pub fn get_string(key: &str, lang: &str) -> &'static str {
    let strings: &[(&str, &str)] = match lang {
        "te" => te::STRINGS,
        _    => en::STRINGS, // unknown lang falls back to English
    };

    // Try the requested locale first.
    if let Some(val) = strings.iter().find(|(k, _)| *k == key).map(|(_, v)| *v) {
        return val;
    }

    // Fall back to English if target locale had no entry for this key.
    if lang != "en" {
        if let Some(val) = en::STRINGS.iter().find(|(k, _)| *k == key).map(|(_, v)| *v) {
            return val;
        }
    }

    "[missing]"
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_en_sun() {
        assert_eq!(get_string("planet.sun", "en"), "Sun");
    }

    #[test]
    fn test_te_sun() {
        assert_eq!(get_string("planet.sun", "te"), "సూర్యుడు");
    }

    #[test]
    fn test_unknown_lang_falls_back_to_en() {
        assert_eq!(get_string("planet.sun", "xx"), "Sun");
    }

    #[test]
    fn test_missing_key_returns_missing() {
        assert_eq!(get_string("planet.mars", "en"), "[missing]");
        assert_eq!(get_string("planet.mars", "te"), "[missing]");
    }
}
