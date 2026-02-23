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

impl Default for SpecDbConfig {
    fn default() -> Self {
        Self {
            specs_dir: default_specs_dir(),
            data_dir: default_data_dir(),
            transport: TransportConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
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
    serde_yml::from_str(&content)
        .map_err(|e| SpecDbError::ConfigError(format!("invalid config YAML: {e}")))
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
    }
}
