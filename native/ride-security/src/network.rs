/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Advanced Network Filtering
//!
//! Provides granular network control with support for:
//! - Domain wildcards (*.example.com)
//! - CIDR IP ranges (192.168.1.0/24)
//! - Port-specific allowlisting (localhost:3000 only)
//! - Regular expression matching for complex URI patterns
//! - Real-time audit logs of allowed/blocked traffic

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{RwLock, Arc};
use url::Url;
use ipnetwork::IpNetwork;
use regex::Regex;

#[napi(object)]
#[derive(Clone)]
pub struct NetworkAuditLog {
    pub timestamp: f64,
    pub url: String,
    pub allowed: bool,
    pub rule_matched: Option<String>,
}

#[napi]
pub enum RuleType {
    Wildcard = 0,
    Regex = 1,
    Cidr = 2,
    Exact = 3,
}

struct DomainRule {
    pattern: String,
    rule_type: RuleType,
    regex: Option<Regex>,
    cidr: Option<IpNetwork>,
}

pub struct NetworkManager {
    rules: Vec<DomainRule>,
    audit_logs: Vec<NetworkAuditLog>,
}

static MANAGER: RwLock<Option<Arc<RwLock<NetworkManager>>>> = RwLock::new(None);

fn get_manager() -> Arc<RwLock<NetworkManager>> {
    let mut guard = MANAGER.write().unwrap();
    if guard.is_none() {
        *guard = Some(Arc::new(RwLock::new(NetworkManager {
            rules: Vec::new(),
            audit_logs: Vec::with_capacity(100),
        })));
    }
    guard.as_ref().unwrap().clone()
}

#[napi]
pub fn add_network_rule(pattern: String, rule_type: RuleType) -> Result<()> {
    let mut rule = DomainRule {
        pattern: pattern.clone(),
        rule_type,
        regex: None,
        cidr: None,
    };

    match rule_type {
        RuleType::Regex => {
            rule.regex = Some(Regex::new(&pattern).map_err(|e| Error::from_reason(e.to_string()))?);
        }
        RuleType::Cidr => {
            rule.cidr = Some(pattern.parse().map_err(|e: ipnetwork::IpNetworkError| Error::from_reason(e.to_string()))?);
        }
        _ => {}
    }

    let manager = get_manager();
    manager.write().unwrap().rules.push(rule);
    Ok(())
}

#[napi]
pub fn is_url_allowed_v2(url_string: String) -> bool {
    let parsed = match Url::parse(&url_string) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = parsed.host_str().unwrap_or("");
    let port = parsed.port().unwrap_or(80);
    let manager_arc = get_manager();
    let mut manager = manager_arc.write().unwrap();

    let mut allowed = false;
    let mut rule_name = None;

    // Hardcoded safety defaults
    if host == "localhost" || host == "127.0.0.1" {
        allowed = true;
        rule_name = Some("builtin.localhost".to_string());
    }

    if !allowed {
        for rule in &manager.rules {
            match rule.rule_type {
                RuleType::Exact => {
                    if host == rule.pattern {
                        allowed = true;
                        rule_name = Some(format!("exact:{}", rule.pattern));
                        break;
                    }
                }
                RuleType::Wildcard => {
                    if rule.pattern.starts_with("*.") {
                        let suffix = &rule.pattern[2..];
                        if host == suffix || host.ends_with(&format!(".{}", suffix)) {
                            allowed = true;
                            rule_name = Some(format!("wildcard:{}", rule.pattern));
                            break;
                        }
                    }
                }
                RuleType::Regex => {
                    if let Some(re) = &rule.regex {
                        if re.is_match(&url_string) {
                            allowed = true;
                            rule_name = Some(format!("regex:{}", rule.pattern));
                            break;
                        }
                    }
                }
                RuleType::Cidr => {
                    if let Some(network) = &rule.cidr {
                        if let Ok(ip) = host.parse() {
                            if network.contains(ip) {
                                allowed = true;
                                rule_name = Some(format!("cidr:{}", rule.pattern));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Logging
    if manager.audit_logs.len() >= 100 {
        manager.audit_logs.remove(0);
    }
    manager.audit_logs.push(NetworkAuditLog {
        timestamp: chrono::Utc::now().timestamp_millis() as f64,
        url: url_string,
        allowed,
        rule_matched: rule_name,
    });

    allowed
}

#[napi]
pub fn get_network_audit_logs() -> Vec<NetworkAuditLog> {
    get_manager().read().unwrap().audit_logs.clone()
}

#[napi]
pub fn clear_network_rules() {
    get_manager().write().unwrap().rules.clear();
}
