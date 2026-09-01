use anyhow::{bail, Result};
use std::fmt;
use std::net::{IpAddr, SocketAddr};

/// Supported asset types for classification, storage, filtering, and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Domain,
    Subdomain,
    Ip,
    IpPort,
    Url,
    Endpoint,
    Unknown,
}

impl AssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Domain => "domain",
            AssetType::Subdomain => "subdomain",
            AssetType::Ip => "ip",
            AssetType::IpPort => "ip_port",
            AssetType::Url => "url",
            AssetType::Endpoint => "endpoint",
            AssetType::Unknown => "unknown",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            AssetType::Domain => "DOMAIN",
            AssetType::Subdomain => "SUBDOMAIN",
            AssetType::Ip => "IP",
            AssetType::IpPort => "IP_PORT",
            AssetType::Url => "URL",
            AssetType::Endpoint => "ENDPOINT",
            AssetType::Unknown => "UNKNOWN",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "domain" => Some(AssetType::Domain),
            "subdomain" => Some(AssetType::Subdomain),
            "ip" => Some(AssetType::Ip),
            "ip_port" | "ipport" | "ip:port" => Some(AssetType::IpPort),
            "url" => Some(AssetType::Url),
            "endpoint" => Some(AssetType::Endpoint),
            "unknown" => Some(AssetType::Unknown),
            _ => None,
        }
    }
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Classifies a normalized asset value for storage and display (backwards-compatible helper).
pub fn asset_type(value: &str) -> &'static str {
    match classify_and_normalize(value) {
        Ok((kind, _)) => kind.as_str(),
        Err(_) => "domain",
    }
}

/// Classifies and deterministically canonicalizes an asset.
pub fn classify_and_normalize(raw: &str) -> Result<(AssetType, String)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty asset value");
    }

    // 1. URL parsing (HTTP / HTTPS)
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let canonical_url = normalize_url(trimmed)?;
        return Ok((AssetType::Url, canonical_url));
    }

    // 2. Relative API Endpoint (starts with /)
    if trimmed.starts_with('/') {
        let endpoint = normalize_endpoint(trimmed)?;
        return Ok((AssetType::Endpoint, endpoint));
    }

    // 3. Raw IP Address
    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        return Ok((AssetType::Ip, ip.to_string().to_ascii_lowercase()));
    }

    // 4. IP:Port Address
    if let Some((ip, port)) = parse_ip_port(trimmed) {
        return Ok((AssetType::IpPort, format!("{ip}:{port}")));
    }

    // 5. Hostname:Port or Domain / Subdomain
    let cleaned = trimmed.trim_end_matches('.').to_ascii_lowercase();
    if cleaned.is_empty() || cleaned.chars().any(char::is_whitespace) {
        bail!("invalid asset containing whitespace: '{trimmed}'");
    }

    // Check if valid domain / subdomain
    let labels: Vec<&str> = cleaned.split('.').collect();
    let valid_labels = !labels.is_empty()
        && labels.iter().all(|label| {
            !label.is_empty()
                && label
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        });

    if valid_labels {
        if labels.len() > 2 {
            Ok((AssetType::Subdomain, cleaned))
        } else {
            Ok((AssetType::Domain, cleaned))
        }
    } else {
        bail!("invalid asset syntax: '{trimmed}'");
    }
}

/// Normalizes a supported domain, IP address, or HTTP(S) URL (backwards-compatible helper).
pub fn normalize(raw: &str) -> Result<String> {
    classify_and_normalize(raw).map(|(_, val)| val)
}

/// Normalizes an exact or `*.` wildcard scope rule.
pub fn normalize_pattern(raw: &str) -> Result<String> {
    let pattern = raw.trim().trim_end_matches('.').to_ascii_lowercase();
    if pattern.is_empty() {
        bail!("scope rule pattern cannot be empty");
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        normalize(suffix)?;
    } else {
        normalize(&pattern)?;
    }
    Ok(pattern)
}

/// Extracts a host/domain/IP from any asset type for scope checking.
pub fn extract_matchable_host(asset_type: AssetType, value: &str) -> Option<String> {
    match asset_type {
        AssetType::Domain | AssetType::Subdomain | AssetType::Ip => {
            Some(value.to_ascii_lowercase())
        }
        AssetType::IpPort => {
            if let Some((ip, _)) = parse_ip_port(value) {
                Some(ip.to_string().to_ascii_lowercase())
            } else {
                Some(value.to_ascii_lowercase())
            }
        }
        AssetType::Url => extract_host_from_url(value),
        AssetType::Endpoint | AssetType::Unknown => None,
    }
}

/// Reports whether a normalized value (or its host component) matches an exact or subdomain wildcard rule.
pub fn matches_pattern(pattern: &str, value: &str) -> bool {
    let pattern = pattern.trim().trim_end_matches('.').to_ascii_lowercase();
    let value_clean = value.trim().trim_end_matches('.').to_ascii_lowercase();

    // Check direct match
    if match_single_host(&pattern, &value_clean) {
        return true;
    }

    // If value is a URL, extract host and test against pattern
    if let Some(host) = extract_host_from_url(&value_clean) {
        if match_single_host(&pattern, &host) {
            return true;
        }
    }

    // If value is an IP:Port, extract IP and test against pattern
    if let Some((ip, _)) = parse_ip_port(&value_clean) {
        if match_single_host(&pattern, &ip.to_string()) {
            return true;
        }
    }

    false
}

fn match_single_host(pattern: &str, host: &str) -> bool {
    if pattern == host {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        let expected_suffix = format!(".{suffix}");
        if host.ends_with(&expected_suffix) && host.len() > expected_suffix.len() {
            return true;
        }
    }
    false
}

fn normalize_url(raw: &str) -> Result<String> {
    let (scheme, rest) = if let Some(stripped) = raw.strip_prefix("https://") {
        ("https", stripped)
    } else if let Some(stripped) = raw.strip_prefix("http://") {
        ("http", stripped)
    } else {
        bail!("URL must start with http:// or https://");
    };

    let rest = rest.trim();
    if rest.is_empty() {
        bail!("URL missing host");
    }

    // Split authority from path / query / fragment
    let (authority, path_query) = match rest.find(['/', '?', '#']) {
        Some(pos) => (&rest[..pos], &rest[pos..]),
        None => (rest, ""),
    };

    // Parse authority: host and optional port
    let (host, port) = if authority.starts_with('[') {
        // IPv6 bracket notation
        if let Some(end_bracket) = authority.find(']') {
            let host_part = &authority[1..end_bracket];
            let port_part = authority[end_bracket + 1..].strip_prefix(':');
            (host_part, port_part)
        } else {
            bail!("invalid IPv6 URL host");
        }
    } else if let Some(colon) = authority.rfind(':') {
        let host_part = &authority[..colon];
        let port_part = Some(&authority[colon + 1..]);
        (host_part, port_part)
    } else {
        (authority, None)
    };

    let host_norm = host.trim_end_matches('.').to_ascii_lowercase();
    if host_norm.is_empty() {
        bail!("empty URL host");
    }

    // Strip default port
    let port_str = match port {
        Some(p) => {
            let port_num: u16 = p.parse().map_err(|_| anyhow::anyhow!("invalid URL port"))?;
            if (scheme == "http" && port_num == 80) || (scheme == "https" && port_num == 443) {
                String::new()
            } else {
                format!(":{port_num}")
            }
        }
        None => String::new(),
    };

    // Clean path and query, strip fragment
    let path_clean = match path_query.find('#') {
        Some(pos) => &path_query[..pos],
        None => path_query,
    };

    let final_path = if path_clean == "/" || path_clean.is_empty() {
        ""
    } else {
        path_clean
    };

    Ok(format!("{scheme}://{host_norm}{port_str}{final_path}"))
}

fn normalize_endpoint(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('/') {
        bail!("endpoint must start with /");
    }
    // Strip fragment
    let clean = match trimmed.find('#') {
        Some(pos) => &trimmed[..pos],
        None => trimmed,
    };
    Ok(clean.to_string())
}

fn parse_ip_port(raw: &str) -> Option<(IpAddr, u16)> {
    if let Ok(socket) = raw.parse::<SocketAddr>() {
        return Some((socket.ip(), socket.port()));
    }
    // Handle manual IPv4:port
    if let Some((ip_str, port_str)) = raw.split_once(':') {
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            if let Ok(port) = port_str.parse::<u16>() {
                return Some((ip, port));
            }
        }
    }
    None
}

fn extract_host_from_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = match rest.find(['/', '?', '#']) {
        Some(pos) => &rest[..pos],
        None => rest,
    };
    if authority.starts_with('[') {
        let end = authority.find(']')?;
        Some(authority[1..end].to_ascii_lowercase())
    } else if let Some((host, _)) = authority.split_once(':') {
        Some(host.trim_end_matches('.').to_ascii_lowercase())
    } else {
        Some(authority.trim_end_matches('.').to_ascii_lowercase())
    }
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
        assert!(matches_pattern("*.example.com", "sub.api.example.com"));
        assert!(!matches_pattern("*.example.com", "example.com"));
        assert!(!matches_pattern("*.example.com", "fakexample.com"));
    }

    #[test]
    fn classifies_mixed_asset_types() {
        assert_eq!(
            classify_and_normalize("api.target.com").unwrap(),
            (AssetType::Subdomain, "api.target.com".into())
        );
        assert_eq!(
            classify_and_normalize("target.com").unwrap(),
            (AssetType::Domain, "target.com".into())
        );
        assert_eq!(
            classify_and_normalize("192.168.1.10").unwrap(),
            (AssetType::Ip, "192.168.1.10".into())
        );
        assert_eq!(
            classify_and_normalize("192.168.1.20:8080").unwrap(),
            (AssetType::IpPort, "192.168.1.20:8080".into())
        );
        assert_eq!(
            classify_and_normalize("https://target.com/login").unwrap(),
            (AssetType::Url, "https://target.com/login".into())
        );
        assert_eq!(
            classify_and_normalize("http://target.com:80/").unwrap(),
            (AssetType::Url, "http://target.com".into())
        );
        assert_eq!(
            classify_and_normalize("/api/v1/users").unwrap(),
            (AssetType::Endpoint, "/api/v1/users".into())
        );
    }

    #[test]
    fn url_matches_scope_rules() {
        assert!(matches_pattern(
            "*.target.com",
            "https://api.target.com/login"
        ));
        assert!(matches_pattern("target.com", "https://target.com/about"));
        assert!(!matches_pattern("*.target.com", "https://target.com/about"));
    }
}
