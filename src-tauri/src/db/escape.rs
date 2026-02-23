use crate::error::AppError;

/// Comprehensive escape for SQL string literals.
/// Handles single quotes, backslashes, null bytes, and control characters
/// that could be used for SQL injection in HTTP/REST-based drivers.
pub fn escape_sql_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\'' => out.push_str("''"),
            '\\' => out.push_str("\\\\"),
            '\0' => out.push_str("\\0"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\x08' => out.push_str("\\b"),
            '\x1a' => out.push_str("\\Z"),
            _ => out.push(ch),
        }
    }
    out
}

/// Validate that an identifier (schema, table, column name) is safe.
/// Rejects empty strings and strings containing null bytes.
pub fn validate_identifier(name: &str) -> Result<&str, AppError> {
    if name.is_empty() {
        return Err(AppError::InvalidConfig(
            "Identifier must not be empty".to_string(),
        ));
    }
    if name.contains('\0') {
        return Err(AppError::InvalidConfig(
            "Identifier must not contain null bytes".to_string(),
        ));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_basic() {
        assert_eq!(escape_sql_literal("hello"), "hello");
        assert_eq!(escape_sql_literal("it's"), "it''s");
        assert_eq!(escape_sql_literal("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_control_chars() {
        assert_eq!(escape_sql_literal("a\0b"), "a\\0b");
        assert_eq!(escape_sql_literal("a\nb"), "a\\nb");
        assert_eq!(escape_sql_literal("a\rb"), "a\\rb");
        assert_eq!(escape_sql_literal("a\x08b"), "a\\bb");
        assert_eq!(escape_sql_literal("a\x1ab"), "a\\Zb");
    }

    #[test]
    fn test_escape_combined() {
        assert_eq!(escape_sql_literal("'; DROP TABLE--"), "''; DROP TABLE--");
    }

    #[test]
    fn test_validate_identifier_ok() {
        assert!(validate_identifier("users").is_ok());
        assert!(validate_identifier("my_table").is_ok());
    }

    #[test]
    fn test_validate_identifier_empty() {
        assert!(validate_identifier("").is_err());
    }

    #[test]
    fn test_validate_identifier_null_byte() {
        assert!(validate_identifier("foo\0bar").is_err());
    }
}
