use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::SpecDbError;

/// Validated spec identifier: `spec::{segment}::{segment}` where segment = `[a-z0-9-]+`.
///
/// Canonical key shared by Tantivy, Fjall, and DeepCausality — validated once at construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpecId(String);

impl SpecId {
    pub fn try_new(raw: impl Into<String>) -> Result<Self, SpecDbError> {
        let raw = raw.into();
        if Self::is_valid(&raw) {
            Ok(Self(raw))
        } else {
            Err(SpecDbError::IngestError(format!(
                "invalid SpecId '{raw}': expected format spec::{{segment}}::{{segment}} \
                 where segment matches [a-z0-9-]+"
            )))
        }
    }

    fn is_valid(raw: &str) -> bool {
        let mut parts = raw.splitn(3, "::");
        let prefix = match parts.next() {
            Some(p) => p,
            None => return false,
        };
        if prefix != "spec" {
            return false;
        }
        let seg1 = match parts.next() {
            Some(s) => s,
            None => return false,
        };
        let seg2 = match parts.next() {
            Some(s) => s,
            None => return false,
        };
        Self::is_valid_segment(seg1) && Self::is_valid_segment(seg2)
    }

    fn is_valid_segment(s: &str) -> bool {
        !s.is_empty()
            && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }
}

impl AsRef<str> for SpecId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SpecId {
    type Err = SpecDbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

/// Trust score for a causal edge, clamped to `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrustLevel(f64);

impl TrustLevel {
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub fn value(self) -> f64 {
        self.0
    }

    pub fn human() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDoc {
    pub id: SpecId,
    pub title: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub depends_on: Vec<SpecId>,
    pub owner: Option<String>,
    pub created: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecNode {
    pub id: SpecId,
    pub title: String,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeOrigin {
    Human,
    Ai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub source: SpecId,
    pub target: SpecId,
    pub trust: TrustLevel,
    pub origin: EdgeOrigin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_spec_id_simple() {
        let id = SpecId::try_new("spec::auth::login").unwrap();
        assert_eq!(id.as_ref(), "spec::auth::login");
        assert_eq!(id.to_string(), "spec::auth::login");
    }

    #[test]
    fn valid_spec_id_with_hyphens() {
        let id = SpecId::try_new("spec::user-service::password-reset").unwrap();
        assert_eq!(id.as_ref(), "spec::user-service::password-reset");
    }

    #[test]
    fn valid_spec_id_with_digits() {
        let id = SpecId::try_new("spec::api-v2::endpoint-3").unwrap();
        assert!(id.as_ref().starts_with("spec::"));
    }

    #[test]
    fn valid_spec_id_from_str() {
        let id: SpecId = "spec::domain::name".parse().unwrap();
        assert_eq!(id.as_ref(), "spec::domain::name");
    }

    #[test]
    fn invalid_spec_id_missing_prefix() {
        assert!(SpecId::try_new("auth::login").is_err());
    }

    #[test]
    fn invalid_spec_id_single_segment() {
        assert!(SpecId::try_new("spec::onlyone").is_err());
    }

    #[test]
    fn invalid_spec_id_empty_segment() {
        assert!(SpecId::try_new("spec::::name").is_err());
    }

    #[test]
    fn invalid_spec_id_uppercase() {
        assert!(SpecId::try_new("spec::Auth::Login").is_err());
    }

    #[test]
    fn invalid_spec_id_spaces() {
        assert!(SpecId::try_new("spec::my domain::name").is_err());
    }

    #[test]
    fn invalid_spec_id_empty_string() {
        assert!(SpecId::try_new("").is_err());
    }

    #[test]
    fn invalid_spec_id_underscores() {
        assert!(SpecId::try_new("spec::my_domain::name").is_err());
    }

    #[test]
    fn trust_level_clamps_high() {
        let t = TrustLevel::new(1.5);
        assert!((t.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trust_level_clamps_low() {
        let t = TrustLevel::new(-0.1);
        assert!((t.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn trust_level_human() {
        assert!((TrustLevel::human().value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spec_id_error_is_descriptive() {
        let err = SpecId::try_new("bad").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid SpecId"));
        assert!(msg.contains("bad"));
    }
}
