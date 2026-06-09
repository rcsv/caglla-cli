use anyhow::{Context, Result};

pub(crate) const TRIP_SUMMARY_MAX_LEN: usize = 2000;
pub(crate) const DAY_SUMMARY_MAX_LEN: usize = 1000;

/// trim し、空なら None。長さ超過時はエラー。
pub(crate) fn normalize_summary(input: Option<&str>, max_len: usize) -> Result<Option<String>> {
    let Some(raw) = input else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > max_len {
        anyhow::bail!(
            "summary exceeds maximum length ({max_len} characters). Use a Note for longer text."
        );
    }
    Ok(Some(trimmed.to_string()))
}

pub(crate) fn normalize_trip_summary(input: Option<&str>) -> Result<Option<String>> {
    normalize_summary(input, TRIP_SUMMARY_MAX_LEN)
}

pub(crate) fn normalize_day_summary(input: Option<&str>) -> Result<Option<String>> {
    normalize_summary(input, DAY_SUMMARY_MAX_LEN)
}

/// import / validate 用: 空文字は NULL、長さ超過は warning 相当で error
pub(crate) fn normalize_summary_for_import(
    input: Option<&str>,
    max_len: usize,
) -> Result<Option<String>> {
    normalize_summary(input, max_len).with_context(|| "invalid summary in export JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_empty_and_whitespace_to_none() {
        assert_eq!(normalize_summary(None, 100).unwrap(), None);
        assert_eq!(normalize_summary(Some(""), 100).unwrap(), None);
        assert_eq!(normalize_summary(Some("   \t\n"), 100).unwrap(), None);
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_summary(Some("  hello world  "), 100).unwrap(),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn normalize_rejects_over_max_length() {
        let too_long = "x".repeat(TRIP_SUMMARY_MAX_LEN + 1);
        let err = normalize_trip_summary(Some(&too_long)).unwrap_err();
        assert!(err.to_string().contains(&TRIP_SUMMARY_MAX_LEN.to_string()));
    }

    #[test]
    fn normalize_accepts_at_max_length() {
        let exact = "y".repeat(DAY_SUMMARY_MAX_LEN);
        assert_eq!(normalize_day_summary(Some(&exact)).unwrap(), Some(exact));
    }

    #[test]
    fn normalize_summary_for_import_delegates_to_normalize() {
        assert_eq!(
            normalize_summary_for_import(Some("  ok  "), 10).unwrap(),
            Some("ok".to_string())
        );
    }
}
