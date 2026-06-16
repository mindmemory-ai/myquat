//! Cloud Backend Configuration Management
//! Author: gA4ss
//!
//! Secure and flexible configuration system for cloud quantum backends

use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Complete cloud configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub global: GlobalConfig,
    pub ibm_quantum: Option<IbmQuantumConfig>,
    pub aws_braket: Option<AwsBraketConfig>,
    pub azure_quantum: Option<AzureQuantumConfig>,
    pub google_quantum: Option<GoogleQuantumConfig>,
    pub preferences: PreferencesConfig,
    pub job_management: JobManagementConfig,
    pub logging: LoggingConfig,
}

/// Global settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_secs: u64,
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

fn default_timeout() -> u64 {
    300
}
fn default_retries() -> u32 {
    3
}
fn default_retry_delay() -> u64 {
    5
}
fn default_true() -> bool {
    true
}
fn default_cache_dir() -> String {
    ".myquat_cache".to_string()
}

/// IBM Quantum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IbmQuantumConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub api_token: String,
    #[serde(default = "default_ibm_api_url")]
    pub api_url: String,
    #[serde(default)]
    pub hub: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default = "default_ibm_backend")]
    pub default_backend: String,
    #[serde(default = "default_true")]
    pub prefer_simulator: bool,
    #[serde(default = "default_ibm_max_qubits")]
    pub max_qubits: usize,
}

fn default_ibm_api_url() -> String {
    "https://auth.quantum-computing.ibm.com/api".to_string()
}
fn default_ibm_backend() -> String {
    "ibmq_qasm_simulator".to_string()
}
fn default_ibm_max_qubits() -> usize {
    127
}

/// AWS Braket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsBraketConfig {
    #[serde(default)]
    pub enabled: bool,
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(default)]
    pub session_token: Option<String>,
    #[serde(default = "default_aws_region")]
    pub region: String,
    #[serde(default = "default_aws_bucket")]
    pub s3_bucket: String,
    #[serde(default = "default_aws_prefix")]
    pub s3_prefix: String,
    #[serde(default = "default_aws_backend")]
    pub default_backend: String,
    #[serde(default = "default_aws_max_qubits")]
    pub max_qubits: usize,
}

fn default_aws_region() -> String {
    "us-east-1".to_string()
}
fn default_aws_bucket() -> String {
    "amazon-braket-results".to_string()
}
fn default_aws_prefix() -> String {
    "myquat-results".to_string()
}
fn default_aws_backend() -> String {
    "arn:aws:braket:::device/quantum-simulator/amazon/sv1".to_string()
}
fn default_aws_max_qubits() -> usize {
    34
}

/// Azure Quantum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureQuantumConfig {
    #[serde(default)]
    pub enabled: bool,
    pub subscription_id: String,
    pub resource_group: String,
    pub workspace_name: String,
    #[serde(default = "default_azure_location")]
    pub location: String,
    pub storage_account: String,
    #[serde(default = "default_azure_container")]
    pub storage_container: String,
    #[serde(default = "default_azure_provider")]
    pub default_provider: String,
    #[serde(default = "default_azure_target")]
    pub default_target: String,
    #[serde(default = "default_azure_max_qubits")]
    pub max_qubits: usize,
}

fn default_azure_location() -> String {
    "eastus".to_string()
}
fn default_azure_container() -> String {
    "quantum-results".to_string()
}
fn default_azure_provider() -> String {
    "ionq".to_string()
}
fn default_azure_target() -> String {
    "ionq.simulator".to_string()
}
fn default_azure_max_qubits() -> usize {
    29
}

/// Google Quantum AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleQuantumConfig {
    #[serde(default)]
    pub enabled: bool,
    pub project_id: String,
    #[serde(default)]
    pub service_account_key: Option<String>,
    #[serde(default)]
    pub service_account_key_base64: Option<String>,
    #[serde(default = "default_google_processor")]
    pub processor_id: String,
    #[serde(default = "default_google_engine")]
    pub default_engine: String,
    #[serde(default = "default_google_max_qubits")]
    pub max_qubits: usize,
}

fn default_google_processor() -> String {
    "rainbow".to_string()
}
fn default_google_engine() -> String {
    "cirq-engine".to_string()
}
fn default_google_max_qubits() -> usize {
    23
}

/// Backend selection preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesConfig {
    #[serde(default = "default_backend_priority")]
    pub backend_priority: Vec<String>,
    #[serde(default = "default_true")]
    pub auto_select: bool,
    #[serde(default = "default_true")]
    pub prefer_simulators: bool,
    #[serde(default = "default_max_cost")]
    pub max_cost_per_job: f64,
}

fn default_backend_priority() -> Vec<String> {
    vec![
        "ibm_quantum".to_string(),
        "aws_braket".to_string(),
        "azure_quantum".to_string(),
        "google_quantum".to_string(),
    ]
}
fn default_max_cost() -> f64 {
    10.0
}

/// Job management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobManagementConfig {
    #[serde(default = "default_true")]
    pub enable_result_cache: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
    #[serde(default = "default_true")]
    pub auto_poll: bool,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_max_wait")]
    pub max_wait_time_secs: u64,
}

fn default_cache_ttl() -> u64 {
    604800
} // 7 days
fn default_poll_interval() -> u64 {
    10
}
fn default_max_wait() -> u64 {
    3600
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub log_api_calls: bool,
    #[serde(default)]
    pub log_file: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for CloudConfig {
    fn default() -> Self {
        CloudConfig {
            global: GlobalConfig {
                timeout_secs: default_timeout(),
                max_retries: default_retries(),
                retry_delay_secs: default_retry_delay(),
                enable_cache: default_true(),
                cache_dir: default_cache_dir(),
            },
            ibm_quantum: None,
            aws_braket: None,
            azure_quantum: None,
            google_quantum: None,
            preferences: PreferencesConfig {
                backend_priority: default_backend_priority(),
                auto_select: default_true(),
                prefer_simulators: default_true(),
                max_cost_per_job: default_max_cost(),
            },
            job_management: JobManagementConfig {
                enable_result_cache: default_true(),
                cache_ttl_secs: default_cache_ttl(),
                auto_poll: default_true(),
                poll_interval_secs: default_poll_interval(),
                max_wait_time_secs: default_max_wait(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                log_api_calls: false,
                log_file: None,
            },
        }
    }
}

impl CloudConfig {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| MyQuatError::io_error(format!("Failed to read config file: {}", e)))?;

        Self::from_toml(&content)
    }

    /// Parse configuration from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str)
            .map_err(|e| MyQuatError::parse_error(format!("Failed to parse TOML config: {}", e)))
    }

    /// Load from default locations
    /// Searches in order: ./cloud_config.toml, ~/.myquat/cloud_config.toml, /etc/myquat/cloud_config.toml
    pub fn load_default() -> Result<Self> {
        let search_paths = vec![
            PathBuf::from("cloud_config.toml"),
            dirs::home_dir()
                .map(|h| h.join(".myquat/cloud_config.toml"))
                .unwrap_or_default(),
            PathBuf::from("/etc/myquat/cloud_config.toml"),
        ];

        for path in search_paths {
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        Err(MyQuatError::io_error(
            "No cloud_config.toml found. Please create one from cloud_config.toml.example",
        ))
    }

    /// Load from environment variables (for CI/CD)
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // IBM Quantum from env
        if let Ok(token) = std::env::var("IBM_QUANTUM_TOKEN") {
            config.ibm_quantum = Some(IbmQuantumConfig {
                enabled: true,
                api_token: token,
                api_url: std::env::var("IBM_QUANTUM_URL").unwrap_or_else(|_| default_ibm_api_url()),
                hub: std::env::var("IBM_QUANTUM_HUB").ok(),
                group: std::env::var("IBM_QUANTUM_GROUP").ok(),
                project: std::env::var("IBM_QUANTUM_PROJECT").ok(),
                default_backend: std::env::var("IBM_QUANTUM_BACKEND")
                    .unwrap_or_else(|_| default_ibm_backend()),
                prefer_simulator: true,
                max_qubits: default_ibm_max_qubits(),
            });
        }

        // AWS Braket from env
        if let (Ok(key_id), Ok(secret)) = (
            std::env::var("AWS_ACCESS_KEY_ID"),
            std::env::var("AWS_SECRET_ACCESS_KEY"),
        ) {
            config.aws_braket = Some(AwsBraketConfig {
                enabled: true,
                access_key_id: key_id,
                secret_access_key: secret,
                session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
                region: std::env::var("AWS_REGION").unwrap_or_else(|_| default_aws_region()),
                s3_bucket: std::env::var("AWS_BRAKET_BUCKET")
                    .unwrap_or_else(|_| default_aws_bucket()),
                s3_prefix: std::env::var("AWS_BRAKET_PREFIX")
                    .unwrap_or_else(|_| default_aws_prefix()),
                default_backend: std::env::var("AWS_BRAKET_BACKEND")
                    .unwrap_or_else(|_| default_aws_backend()),
                max_qubits: default_aws_max_qubits(),
            });
        }

        Ok(config)
    }

    /// Get enabled backends
    pub fn enabled_backends(&self) -> Vec<&str> {
        let mut backends = Vec::new();

        if self
            .ibm_quantum
            .as_ref()
            .map(|c| c.enabled)
            .unwrap_or(false)
        {
            backends.push("ibm_quantum");
        }
        if self.aws_braket.as_ref().map(|c| c.enabled).unwrap_or(false) {
            backends.push("aws_braket");
        }
        if self
            .azure_quantum
            .as_ref()
            .map(|c| c.enabled)
            .unwrap_or(false)
        {
            backends.push("azure_quantum");
        }
        if self
            .google_quantum
            .as_ref()
            .map(|c| c.enabled)
            .unwrap_or(false)
        {
            backends.push("google_quantum");
        }

        backends
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let enabled = self.enabled_backends();

        if enabled.is_empty() {
            return Err(MyQuatError::circuit_error("No cloud backends are enabled"));
        }

        // Validate IBM Quantum config
        if let Some(ref ibm) = self.ibm_quantum {
            if ibm.enabled && ibm.api_token.is_empty() {
                return Err(MyQuatError::circuit_error(
                    "IBM Quantum is enabled but api_token is empty",
                ));
            }
        }

        // Validate AWS Braket config
        if let Some(ref aws) = self.aws_braket {
            if aws.enabled {
                if aws.access_key_id.is_empty() {
                    return Err(MyQuatError::circuit_error(
                        "AWS Braket is enabled but access_key_id is empty",
                    ));
                }
                if aws.secret_access_key.is_empty() {
                    return Err(MyQuatError::circuit_error(
                        "AWS Braket is enabled but secret_access_key is empty",
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CloudConfig::default();
        assert_eq!(config.global.timeout_secs, 300);
        assert_eq!(config.preferences.auto_select, true);
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
[global]
timeout_secs = 300

[ibm_quantum]
enabled = true
api_token = "test_token"

[preferences]
auto_select = true

[job_management]
enable_result_cache = true
cache_ttl_secs = 3600
auto_poll = true
poll_interval_secs = 5

[logging]
log_level = "info"
log_to_file = false
        "#;

        let config = CloudConfig::from_toml(toml_str).unwrap();
        assert!(config.ibm_quantum.is_some());
        assert_eq!(config.ibm_quantum.unwrap().api_token, "test_token");
        assert_eq!(config.job_management.cache_ttl_secs, 3600);
    }

    #[test]
    fn test_enabled_backends() {
        let mut config = CloudConfig::default();
        config.ibm_quantum = Some(IbmQuantumConfig {
            enabled: true,
            api_token: "test".to_string(),
            api_url: default_ibm_api_url(),
            hub: None,
            group: None,
            project: None,
            default_backend: default_ibm_backend(),
            prefer_simulator: true,
            max_qubits: 127,
        });

        let enabled = config.enabled_backends();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0], "ibm_quantum");
    }
}
