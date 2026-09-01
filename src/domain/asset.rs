use anyhow::{bail, Result};
use std::net::IpAddr;

/// Classifies a normalized asset value for storage and display.
pub fn asset_type(value: &str) -> &'static str {
    if value.starts_with("http://") || value.starts_with("https://") {
        "url"
    } else if value.parse::<IpAddr>().is_ok() {
        "ip"
    } else {
        "domain"
    }
}

/// Normalizes a supported domain, IP address, or HTTP(S) URL.
pub fn normalize(raw: &str) -> Result<String> {
    let value = raw.trim().trim_end_matches('.').to_ascii_lowercase();
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        bail!("empty or invalid value");
    }
    if value.starts_with("http://") || value.starts_with("https://") {
        return Ok(value);
    }
    if value.parse::<IpAddr>().is_ok()
        || value.split('.').all(|label| {
            !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
    {
        return Ok(value);
    }
    bail!("invalid asset");
}

/// Normalizes an exact or `*.` wildcard scope rule.
pub fn normalize_pattern(raw: &str) -> Result<String> {
    let pattern = raw.trim().trim_end_matches('.').to_ascii_lowercase();
    if let Some(suffix) = pattern.strip_prefix("*.") {
        normalize(suffix)?;
    } else {
        normalize(&pattern)?;
    }
    Ok(pattern)
}

/// Reports whether a normalized value matches an exact or subdomain wildcard rule.
pub fn matches_pattern(pattern: &str, value: &str) -> bool {
    pattern == value
        || pattern
            .strip_prefix("*.")
            .is_some_and(|suffix| value.ends_with(suffix) && value.len() > suffix.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_domains_and_rejects_whitespace() {
        assert_eq!(normalize(" Example.COM. ").unwrap(), "example.com");
        assert!(normalize("example .com").is_err());
    }

    #[test]
    fn wildcard_only_matches_subdomains() {
        assert!(matches_pattern("*.example.com", "api.example.com"));
        assert!(!matches_pattern("*.example.com", "example.com"));
    }
}
