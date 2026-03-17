//! Session key parsing for scoped gateway access.
//!
//! Session keys encode a scope that limits what operations the bearer
//! can perform. Format: `zcs_<scope>_<random>` where scope is one of:
//! `main`, `sender_<id>`, `cron`, `subagent`.

use serde::{Deserialize, Serialize};

/// The scope encoded in a session key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionScope {
    /// Full access (equivalent to the main bearer token)
    Main,
    /// Scoped to a specific sender/channel
    Sender(String),
    /// Scoped to cron job execution
    Cron,
    /// Scoped to a sub-agent delegation
    SubAgent,
}

/// A parsed session key.
#[derive(Debug, Clone)]
pub struct SessionKey {
    pub scope: SessionScope,
    pub raw: String,
}

impl SessionKey {
    /// Parse a session key string.
    /// Returns None if the format is invalid.
    pub fn parse(key: &str) -> Option<Self> {
        let key = key.trim();
        if !key.starts_with("zcs_") {
            return None;
        }

        let rest = &key[4..]; // skip "zcs_"
        let scope = if rest.starts_with("main_") {
            SessionScope::Main
        } else if let Some(sender_rest) = rest.strip_prefix("sender_") {
            // Format: zcs_sender_<id>_<random>
            let parts: Vec<&str> = sender_rest.splitn(2, '_').collect();
            if parts.len() < 2 {
                return None;
            }
            SessionScope::Sender(parts[0].to_string())
        } else if rest.starts_with("cron_") {
            SessionScope::Cron
        } else if rest.starts_with("subagent_") {
            SessionScope::SubAgent
        } else {
            return None;
        };

        Some(Self {
            scope,
            raw: key.to_string(),
        })
    }

    /// Check if this key's scope allows a given operation.
    pub fn allows(&self, operation: &str) -> bool {
        match &self.scope {
            SessionScope::Main => true, // full access
            SessionScope::Sender(_) => {
                matches!(operation, "chat" | "memory_read" | "status")
            }
            SessionScope::Cron => {
                matches!(
                    operation,
                    "chat" | "tool_execute" | "memory_read" | "memory_write"
                )
            }
            SessionScope::SubAgent => {
                matches!(operation, "chat" | "tool_execute" | "memory_read")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_main() {
        let key = SessionKey::parse("zcs_main_abc123def456").unwrap();
        assert_eq!(key.scope, SessionScope::Main);
        assert!(key.allows("chat"));
        assert!(key.allows("anything"));
    }

    #[test]
    fn test_parse_sender() {
        let key = SessionKey::parse("zcs_sender_telegram123_abc456").unwrap();
        assert_eq!(key.scope, SessionScope::Sender("telegram123".to_string()));
        assert!(key.allows("chat"));
        assert!(!key.allows("tool_execute"));
    }

    #[test]
    fn test_parse_cron() {
        let key = SessionKey::parse("zcs_cron_random789").unwrap();
        assert_eq!(key.scope, SessionScope::Cron);
        assert!(key.allows("chat"));
        assert!(key.allows("tool_execute"));
        assert!(!key.allows("admin"));
    }

    #[test]
    fn test_parse_subagent() {
        let key = SessionKey::parse("zcs_subagent_xyz").unwrap();
        assert_eq!(key.scope, SessionScope::SubAgent);
    }

    #[test]
    fn test_invalid_prefix() {
        assert!(SessionKey::parse("zc_main_abc").is_none());
        assert!(SessionKey::parse("").is_none());
        assert!(SessionKey::parse("zcs_unknown_abc").is_none());
    }
}
