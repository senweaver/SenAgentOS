// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// NDJSON safe stringify — mirrors claude-code-typescript-src `cli/ndjsonSafeStringify.ts`.
// JSON.stringify for one-message-per-line transports with safe escaping.

/// NDJSON-safe stringify.
/// Escapes U+2028 (LINE SEPARATOR) and U+2029 (PARAGRAPH SEPARATOR) so the
/// serialized output cannot be broken by a line-splitting receiver.
/// Output is still valid JSON and parses to the same value.
pub fn ndjson_safe_stringify<T: serde::Serialize>(value: &T) -> String {
    let json = serde_json::to_string(value).unwrap_or_default();
    escape_js_line_terminators(&json)
}

/// Escape JavaScript line terminators (U+2028/U+2029).
/// These are valid in JSON strings but break NDJSON line splitting.
fn escape_js_line_terminators(json: &str) -> String {
    json.replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// Check if a string contains NDJSON-unsafe characters.
pub fn contains_js_line_terminators(s: &str) -> bool {
    s.contains('\u{2028}') || s.contains('\u{2029}')
}

/// Parse NDJSON lines from a string.
pub fn parse_ndjson_lines(input: &str) -> Vec<serde_json::Value> {
    input
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndjson_safe_stringify_basic() {
        let data = serde_json::json!({"key": "value"});
        let result = ndjson_safe_stringify(&data);
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_escape_js_line_terminators() {
        // U+2028 LINE SEPARATOR
        let input = "before\u{2028}after";
        let escaped = escape_js_line_terminators(input);
        assert!(!escaped.contains('\u{2028}'));
        assert!(escaped.contains("\\u2028"));

        // U+2029 PARAGRAPH SEPARATOR
        let input = "before\u{2029}after";
        let escaped = escape_js_line_terminators(input);
        assert!(!escaped.contains('\u{2029}'));
        assert!(escaped.contains("\\u2029"));
    }

    #[test]
    fn test_contains_js_line_terminators() {
        assert!(contains_js_line_terminators("test\u{2028}value"));
        assert!(contains_js_line_terminators("test\u{2029}value"));
        assert!(!contains_js_line_terminators("normal text"));
    }

    #[test]
    fn test_parse_ndjson_lines() {
        let input = r#"{"key":"value1"}
{"key":"value2"}
{"key":"value3"}"#;

        let results = parse_ndjson_lines(input);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0]["key"], "value1");
        assert_eq!(results[1]["key"], "value2");
        assert_eq!(results[2]["key"], "value3");
    }

    #[test]
    fn test_parse_ndjson_with_invalid_lines() {
        let input = r#"{"key":"valid"}
invalid json line
{"key":"also valid"}"#;

        let results = parse_ndjson_lines(input);
        assert_eq!(results.len(), 2);
    }
}
