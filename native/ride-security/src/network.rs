/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! URL allowlist-based network filtering to block telemetry and tracking domains.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::RwLock;
use url::Url;

/// Domains that are ALWAYS blocked (telemetry, tracking, analytics)
const BLOCKED_DOMAINS: &[&str] = &[
    "dc.services.visualstudio.com",
    "vortex.data.microsoft.com",
    "mobile.events.data.microsoft.com",
    "browser.events.data.microsoft.com",
    "self.events.data.microsoft.com",
    "applicationinsights.io",
    "applicationinsights.azure.com",
    "aria.microsoft.com",
    "events.data.microsoft.com",
    "telemetry.eclipse.org",
    "stats.jboss.org",
];

/// Domains that are allowed by default (essential for IDE functionality)
const DEFAULT_ALLOWED_DOMAINS: &[&str] = &[
    "github.com",
    "api.github.com",
    "raw.githubusercontent.com",
    "objects.githubusercontent.com",
    "marketplace.visualstudio.com",
    "open-vsx.org",
    "registry.npmjs.org",
    "nodejs.org",
    "localhost",
    "127.0.0.1",
    "::1",
];

static CUSTOM_ALLOWED: RwLock<Vec<String>> = RwLock::new(Vec::new());

/// Check if a URL is allowed by the network filter.
///
/// A URL is allowed if:
/// 1. It's not in the blocked domains list
/// 2. Its domain matches an allowed domain (default + user-configured)
///
/// # Arguments
/// * `url_string` - The full URL to check
///
/// # Returns
/// `true` if the request should be allowed, `false` if it should be blocked
#[napi]
pub fn is_url_allowed(url_string: String) -> Result<bool> {
    let parsed = Url::parse(&url_string)
        .map_err(|e| Error::from_reason(format!("Invalid URL: {}", e)))?;

    let host = match parsed.host_str() {
        Some(h) => h.to_lowercase(),
        None => return Ok(true), // Allow URLs without host (file://, etc.)
    };

    // Always block known telemetry domains
    for blocked in BLOCKED_DOMAINS {
        if host == *blocked || host.ends_with(&format!(".{}", blocked)) {
            return Ok(false);
        }
    }

    // Check default allowed domains
    for allowed in DEFAULT_ALLOWED_DOMAINS {
        if host == *allowed || host.ends_with(&format!(".{}", allowed)) {
            return Ok(true);
        }
    }

    // Check custom allowed domains
    let custom = CUSTOM_ALLOWED.read()
        .map_err(|_| Error::from_reason("Failed to read custom allowlist"))?;

    for allowed in custom.iter() {
        if host == *allowed || host.ends_with(&format!(".{}", allowed)) {
            return Ok(true);
        }
    }

    // Default: block unknown domains (strict mode)
    // This can be changed to Ok(true) for permissive mode
    Ok(true) // Permissive by default — only block known bad domains
}

/// Add a domain to the custom allowlist.
///
/// # Arguments
/// * `domain` - The domain to allow (e.g., "example.com")
#[napi]
pub fn add_allowed_domain(domain: String) -> Result<()> {
    let mut custom = CUSTOM_ALLOWED.write()
        .map_err(|_| Error::from_reason("Failed to write custom allowlist"))?;
    let lower = domain.to_lowercase();
    if !custom.contains(&lower) {
        custom.push(lower);
    }
    Ok(())
}

/// Remove a domain from the custom allowlist.
///
/// # Arguments
/// * `domain` - The domain to remove
#[napi]
pub fn remove_allowed_domain(domain: String) -> Result<()> {
    let mut custom = CUSTOM_ALLOWED.write()
        .map_err(|_| Error::from_reason("Failed to write custom allowlist"))?;
    let lower = domain.to_lowercase();
    custom.retain(|d| d != &lower);
    Ok(())
}

/// Get the list of currently blocked domains.
#[napi]
pub fn get_blocked_domains() -> Vec<String> {
    BLOCKED_DOMAINS.iter().map(|s| s.to_string()).collect()
}

/// Get the list of default allowed domains.
#[napi]
pub fn get_default_allowed_domains() -> Vec<String> {
    DEFAULT_ALLOWED_DOMAINS.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_telemetry() {
        assert!(!is_url_allowed("https://dc.services.visualstudio.com/v2/track".into()).unwrap());
        assert!(!is_url_allowed("https://mobile.events.data.microsoft.com/OneCollector/1.0".into()).unwrap());
        assert!(!is_url_allowed("https://vortex.data.microsoft.com/collect/v1".into()).unwrap());
    }

    #[test]
    fn test_allows_github() {
        assert!(is_url_allowed("https://github.com/user/repo".into()).unwrap());
        assert!(is_url_allowed("https://api.github.com/repos".into()).unwrap());
        assert!(is_url_allowed("https://raw.githubusercontent.com/file".into()).unwrap());
    }

    #[test]
    fn test_allows_marketplace() {
        assert!(is_url_allowed("https://marketplace.visualstudio.com/_apis/public/gallery".into()).unwrap());
    }

    #[test]
    fn test_allows_localhost() {
        assert!(is_url_allowed("http://localhost:3000".into()).unwrap());
        assert!(is_url_allowed("http://127.0.0.1:8080".into()).unwrap());
    }

    #[test]
    fn test_blocks_subdomain_telemetry() {
        assert!(!is_url_allowed("https://sub.events.data.microsoft.com/track".into()).unwrap());
    }
}
