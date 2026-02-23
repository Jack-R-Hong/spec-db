use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::SpecDbError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDbConfig {
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default)]
    pub transport: TransportConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub web: WebConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_trust")]
    pub default_trust: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,
    pub endpoint: Option<String>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(default = "default_true")]
    pub stdio: bool,
    #[serde(default)]
    pub http: Option<HttpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_web_host")]
    pub host: String,
    #[serde(default = "default_web_port")]
    pub port: u16,
    pub auth_token: Option<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self { enabled: true, host: default_web_host(), port: default_web_port(), auth_token: None }
    }
}

fn default_web_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_web_port() -> u16 {
    3000
}

fn default_specs_dir() -> String {
    "specs".to_owned()
}

fn default_data_dir() -> String {
    "data".to_owned()
}

fn default_true() -> bool {
    true
}

fn default_protocol() -> String {
    "grpc".to_owned()
}

fn default_ai_trust() -> f64 {
    0.5
}

impl Default for SpecDbConfig {
    fn default() -> Self {
        Self {
            specs_dir: default_specs_dir(),
            data_dir: default_data_dir(),
            transport: TransportConfig::default(),
            telemetry: TelemetryConfig::default(),
            ai: AiConfig::default(),
            web: WebConfig::default(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self { default_trust: default_ai_trust() }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { enabled: false, endpoint: None, protocol: default_protocol() }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self { stdio: true, http: None }
    }
}

pub fn load_config(path: &Path) -> Result<SpecDbConfig, SpecDbError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| SpecDbError::ConfigError(format!("failed to read config: {e}")))?;
    let config: SpecDbConfig = serde_yml::from_str(&content)
        .map_err(|e| SpecDbError::ConfigError(format!("invalid config YAML: {e}")))?;

    if !(0.0..=1.0).contains(&config.ai.default_trust) {
        return Err(SpecDbError::ConfigError(format!(
            "invalid ai.default_trust '{}': must be between 0.0 and 1.0",
            config.ai.default_trust
        )));
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let config = SpecDbConfig::default();
        assert_eq!(config.specs_dir, "specs");
        assert_eq!(config.data_dir, "data");
        assert!(config.transport.stdio);
        assert!(config.transport.http.is_none());
        assert!(!config.telemetry.enabled);
        assert!(config.telemetry.endpoint.is_none());
        assert_eq!(config.telemetry.protocol, "grpc");
        assert!((config.ai.default_trust - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn telemetry_defaults() {
        let telemetry = TelemetryConfig::default();
        assert!(!telemetry.enabled);
        assert!(telemetry.endpoint.is_none());
        assert_eq!(telemetry.protocol, "grpc");
    }

    #[test]
    fn load_config_from_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(
            &config_path,
            "specs_dir: custom-specs\ndata_dir: custom-data\ntransport:\n  stdio: false\n  http:\n    host: 127.0.0.1\n    port: 8080\ntelemetry:\n  enabled: true\n  endpoint: http://localhost:4317\n  protocol: http\n",
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.specs_dir, "custom-specs");
        assert_eq!(config.data_dir, "custom-data");
        assert!(!config.transport.stdio);
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.endpoint.as_deref(), Some("http://localhost:4317"));
        assert_eq!(config.telemetry.protocol, "http");

        let http = config.transport.http.unwrap();
        assert_eq!(http.host, "127.0.0.1");
        assert_eq!(http.port, 8080);
        assert!((config.ai.default_trust - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn telemetry_config_from_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(
            &config_path,
            "telemetry:\n  enabled: true\n  endpoint: http://localhost:4318\n  protocol: http\n",
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();
        assert!(config.telemetry.enabled);
        assert_eq!(config.telemetry.endpoint.as_deref(), Some("http://localhost:4318"));
        assert_eq!(config.telemetry.protocol, "http");
    }

    #[test]
    fn load_config_missing_file_returns_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("missing.yaml");

        let err = load_config(&config_path).unwrap_err();
        match err {
            SpecDbError::ConfigError(message) => {
                assert!(message.contains("failed to read config"));
            }
            _ => panic!("expected ConfigError"),
        }
    }

    #[test]
    fn load_config_with_partial_yaml_uses_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "transport:\n  stdio: false\n").unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.specs_dir, "specs");
        assert_eq!(config.data_dir, "data");
        assert!(!config.transport.stdio);
        assert!(config.transport.http.is_none());
        assert!(!config.telemetry.enabled);
        assert!(config.telemetry.endpoint.is_none());
        assert_eq!(config.telemetry.protocol, "grpc");
        assert!((config.ai.default_trust - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn ai_default_trust_from_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "ai:\n  default_trust: 0.7\n").unwrap();

        let config = load_config(&config_path).unwrap();
        assert!((config.ai.default_trust - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn ai_default_trust_out_of_range_fails_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(&config_path, "ai:\n  default_trust: 1.5\n").unwrap();

        let err = load_config(&config_path).unwrap_err();
        match err {
            SpecDbError::ConfigError(message) => {
                assert!(message.contains("invalid ai.default_trust"));
                assert!(message.contains("between 0.0 and 1.0"));
            }
            _ => panic!("expected ConfigError"),
        }
    }
}
