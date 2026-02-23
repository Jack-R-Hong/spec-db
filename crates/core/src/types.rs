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

impl fmt::Display for EdgeOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Human => "human",
            Self::Ai => "ai",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeType {
    #[default]
    DependsOn,
    Constrains,
    Implements,
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::DependsOn => "depends_on",
            Self::Constrains => "constrains",
            Self::Implements => "implements",
        };
        f.write_str(value)
    }
}

impl FromStr for EdgeType {
    type Err = SpecDbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "depends_on" => Ok(Self::DependsOn),
            "constrains" => Ok(Self::Constrains),
            "implements" => Ok(Self::Implements),
            _ => Err(SpecDbError::IngestError(format!("invalid EdgeType '{s}'"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub source: SpecId,
    pub target: SpecId,
    #[serde(default)]
    pub edge_type: EdgeType,
    pub trust: TrustLevel,
    pub origin: EdgeOrigin,
    #[serde(default)]
    pub created_at: Option<String>,
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

    #[test]
    fn edge_type_display_from_str_roundtrip() {
        for edge_type in [EdgeType::DependsOn, EdgeType::Constrains, EdgeType::Implements] {
            let rendered = edge_type.to_string();
            let reparsed: EdgeType = rendered.parse().unwrap();
            assert_eq!(reparsed, edge_type);
        }
    }

    #[test]
    fn edge_type_default_is_depends_on() {
        assert_eq!(EdgeType::default(), EdgeType::DependsOn);
    }

    #[test]
    fn causal_edge_serde_includes_edge_type() {
        let edge = CausalEdge {
            source: SpecId::try_new("spec::svc::api").unwrap(),
            target: SpecId::try_new("spec::svc::auth").unwrap(),
            edge_type: EdgeType::Constrains,
            trust: TrustLevel::new(0.8),
            origin: EdgeOrigin::Ai,
            created_at: Some("2026-02-23T10:00:00Z".to_owned()),
        };

        let encoded = serde_yml::to_string(&edge).unwrap();
        assert!(encoded.contains("edge_type: Constrains"));

        let decoded: CausalEdge = serde_yml::from_str(&encoded).unwrap();
        assert_eq!(decoded.edge_type, EdgeType::Constrains);
        assert!((decoded.trust.value() - 0.8).abs() < f64::EPSILON);
        assert_eq!(decoded.origin, EdgeOrigin::Ai);
    }

    #[test]
    fn causal_edge_serde_missing_edge_type_defaults_to_depends_on() {
        let encoded = r#"
source: spec::svc::api
target: spec::svc::auth
trust: 1.0
origin: Human
"#;
        let decoded: CausalEdge = serde_yml::from_str(encoded).unwrap();
        assert_eq!(decoded.edge_type, EdgeType::DependsOn);
        assert_eq!(decoded.origin, EdgeOrigin::Human);
        assert!((decoded.trust.value() - 1.0).abs() < f64::EPSILON);
    }
}
