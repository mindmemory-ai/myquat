//! IBM Quantum Backend Implementation
//! Author: gA4ss
//!
//! Cloud backend for IBM Quantum Platform with job management

use crate::compute::backend_trait::ComputeBackend;
use crate::compute::cloud_config::IbmQuantumConfig;
use crate::compute::types::{
    CloudBackendVariant, ComputeBackendType, ExecutionResult, PerformanceProfile,
};
use crate::error::{MyQuatError, Result};
use crate::QuantumCircuit;
use std::collections::HashMap;
use std::time::Duration;

/// IBM Quantum cloud backend
pub struct IbmQuantumBackend {
    config: IbmQuantumConfig,
    client: IbmQuantumClient,
    job_cache: HashMap<String, CachedJob>,
}

/// IBM Quantum API client
struct IbmQuantumClient {
    api_url: String,
    api_token: String,
    http_client: reqwest::blocking::Client,
}

/// Cached job information
struct CachedJob {
    job_id: String,
    status: JobStatus,
    result: Option<ExecutionResult>,
    created_at: std::time::Instant,
}

/// Job status on IBM Quantum
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl IbmQuantumBackend {
    /// Create new IBM Quantum backend from configuration
    pub fn new(config: IbmQuantumConfig) -> Result<Self> {
        if !config.enabled {
            return Err(MyQuatError::circuit_error(
                "IBM Quantum backend is not enabled in configuration",
            ));
        }

        if config.api_token.is_empty() {
            return Err(MyQuatError::circuit_error("IBM Quantum API token is empty"));
        }

        let client = IbmQuantumClient::new(config.api_url.clone(), config.api_token.clone())?;

        Ok(IbmQuantumBackend {
            config,
            client,
            job_cache: HashMap::new(),
        })
    }

    /// Submit job to IBM Quantum
    pub fn submit_job(
        &mut self,
        circuit: &QuantumCircuit,
        shots: usize,
        backend_name: Option<&str>,
    ) -> Result<String> {
        let backend = backend_name.unwrap_or(&self.config.default_backend);

        // Convert circuit to QASM
        let qasm = self.circuit_to_qasm(circuit)?;

        // Submit job via API
        let job_id = self.client.submit_job(&qasm, shots, backend)?;

        // Cache job info
        self.job_cache.insert(
            job_id.clone(),
            CachedJob {
                job_id: job_id.clone(),
                status: JobStatus::Queued,
                result: None,
                created_at: std::time::Instant::now(),
            },
        );

        Ok(job_id)
    }

    /// Poll job status
    pub fn get_job_status(&mut self, job_id: &str) -> Result<JobStatus> {
        // Check cache first
        if let Some(cached) = self.job_cache.get(job_id) {
            if cached.status == JobStatus::Completed || cached.status == JobStatus::Failed {
                return Ok(cached.status.clone());
            }
        }

        // Query API
        let status = self.client.get_job_status(job_id)?;

        // Update cache
        if let Some(cached) = self.job_cache.get_mut(job_id) {
            cached.status = status.clone();
        }

        Ok(status)
    }

    /// Wait for job completion
    pub fn wait_for_job(
        &mut self,
        job_id: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();

        loop {
            let status = self.get_job_status(job_id)?;

            match status {
                JobStatus::Completed => {
                    return self.get_job_result(job_id);
                }
                JobStatus::Failed => {
                    return Err(MyQuatError::circuit_error(format!(
                        "Job {} failed on IBM Quantum",
                        job_id
                    )));
                }
                JobStatus::Cancelled => {
                    return Err(MyQuatError::circuit_error(format!(
                        "Job {} was cancelled",
                        job_id
                    )));
                }
                _ => {
                    if start.elapsed() > timeout {
                        return Err(MyQuatError::circuit_error(format!(
                            "Job {} timed out after {:?}",
                            job_id, timeout
                        )));
                    }
                    std::thread::sleep(poll_interval);
                }
            }
        }
    }

    /// Get job result
    pub fn get_job_result(&mut self, job_id: &str) -> Result<ExecutionResult> {
        // Check cache
        if let Some(cached) = self.job_cache.get(job_id) {
            if let Some(ref result) = cached.result {
                return Ok(result.clone());
            }
        }

        // Fetch from API
        let result = self.client.get_job_result(job_id)?;

        // Update cache
        if let Some(cached) = self.job_cache.get_mut(job_id) {
            cached.result = Some(result.clone());
            cached.status = JobStatus::Completed;
        }

        Ok(result)
    }

    /// List available IBM backends
    pub fn list_backends(&self) -> Result<Vec<String>> {
        self.client.list_backends()
    }

    /// Get backend information
    pub fn get_backend_info(&self, backend_name: &str) -> Result<BackendInfo> {
        self.client.get_backend_info(backend_name)
    }

    /// Convert circuit to QASM format
    fn circuit_to_qasm(&self, circuit: &QuantumCircuit) -> Result<String> {
        // Use existing QASM export functionality
        use crate::qasm::QasmExporter;

        let exporter = QasmExporter::new();
        exporter.export(circuit)
    }

    /// Create from environment variable (backward compatibility)
    pub fn from_env() -> Result<Self> {
        let api_token = std::env::var("IBM_QUANTUM_TOKEN").map_err(|_| {
            MyQuatError::circuit_error("IBM_QUANTUM_TOKEN environment variable not set")
        })?;

        let config = IbmQuantumConfig {
            enabled: true,
            api_token,
            api_url: "https://auth.quantum-computing.ibm.com/api".to_string(),
            hub: None,
            group: None,
            project: None,
            default_backend: "ibmq_qasm_simulator".to_string(),
            prefer_simulator: true,
            max_qubits: 127,
        };

        Self::new(config)
    }

    /// Create from apikey.json file (backward compatibility)
    pub fn from_apikey_file(path: &str) -> Result<Self> {
        use std::fs;

        let contents = fs::read_to_string(path)
            .map_err(|e| MyQuatError::io_error(format!("Failed to read apikey.json: {}", e)))?;

        let api_data: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| MyQuatError::io_error(format!("Failed to parse apikey.json: {}", e)))?;

        let api_token = api_data
            .get("apikey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MyQuatError::circuit_error("API key not found in JSON file"))?;

        let config = IbmQuantumConfig {
            enabled: true,
            api_token: api_token.to_string(),
            api_url: "https://auth.quantum-computing.ibm.com/api".to_string(),
            hub: None,
            group: None,
            project: None,
            default_backend: "ibmq_qasm_simulator".to_string(),
            prefer_simulator: true,
            max_qubits: 127,
        };

        Self::new(config)
    }

    /// Create from default apikey.json location (backward compatibility)
    pub fn from_default_apikey() -> Result<Self> {
        Self::from_apikey_file("apikey.json")
    }
}

impl ComputeBackend for IbmQuantumBackend {
    fn name(&self) -> &str {
        "ibm_quantum"
    }

    fn backend_type(&self) -> ComputeBackendType {
        ComputeBackendType::Cloud(CloudBackendVariant::IBMQuantum)
    }

    fn is_available(&self) -> bool {
        // Check if we can connect to IBM Quantum API
        self.client.test_connection().unwrap_or(false)
    }

    fn performance_profile(&self) -> PerformanceProfile {
        PerformanceProfile {
            max_qubits: self.config.max_qubits,
            ops_per_second: 100.0, // Cloud backends are slower due to network
            startup_overhead_ms: 5000.0, // Network latency + queue time
            memory_per_qubit_bytes: 0, // Cloud execution, no local memory
            supports_batching: true,
            optimal_batch_size: Some(100),
        }
    }

    fn execute(&self, _circuit: &QuantumCircuit, _shots: usize) -> Result<ExecutionResult> {
        // IBM Quantum execution requires mutable self for job management
        // This is a limitation of the current trait design
        // For now, we'll use a synchronous blocking approach

        // In a real implementation, this would:
        // 1. Submit job asynchronously
        // 2. Return a job ID
        // 3. Allow polling or waiting for completion

        Err(MyQuatError::circuit_error(
            "IBM Quantum execution requires async job management. Use submit_job() instead.",
        ))
    }

    fn apply_single_qubit_gate(
        &self,
        _state: &mut ndarray::Array1<num_complex::Complex64>,
        _gate_matrix: &ndarray::Array2<num_complex::Complex64>,
        _qubit: usize,
        _num_qubits: usize,
    ) -> Result<()> {
        Err(MyQuatError::circuit_error(
            "Cloud backends do not support direct gate application",
        ))
    }

    fn apply_two_qubit_gate(
        &self,
        _state: &mut ndarray::Array1<num_complex::Complex64>,
        _gate_matrix: &ndarray::Array2<num_complex::Complex64>,
        _control: usize,
        _target: usize,
        _num_qubits: usize,
    ) -> Result<()> {
        Err(MyQuatError::circuit_error(
            "Cloud backends do not support direct gate application",
        ))
    }
}

impl IbmQuantumClient {
    fn new(api_url: String, api_token: String) -> Result<Self> {
        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| MyQuatError::io_error(format!("Failed to create HTTP client: {}", e)))?;

        Ok(IbmQuantumClient {
            api_url,
            api_token,
            http_client,
        })
    }

    fn test_connection(&self) -> Result<bool> {
        // Simple connectivity test
        // In a real implementation, this would make an API call
        Ok(!self.api_token.is_empty() && !self.api_url.is_empty())
    }

    fn submit_job(&self, _qasm: &str, _shots: usize, _backend: &str) -> Result<String> {
        // Placeholder: In real implementation, this would:
        // 1. Make POST request to IBM Quantum API
        // 2. Return job ID

        // For now, return a mock job ID
        Ok(format!("ibm_job_{}", chrono::Utc::now().timestamp()))
    }

    fn get_job_status(&self, _job_id: &str) -> Result<JobStatus> {
        // Placeholder: Query job status from API
        Ok(JobStatus::Completed)
    }

    fn get_job_result(&self, _job_id: &str) -> Result<ExecutionResult> {
        // Placeholder: Fetch result from API
        let mut counts = HashMap::new();
        counts.insert("00".to_string(), 500);
        counts.insert("11".to_string(), 500);

        Ok(ExecutionResult {
            counts,
            shots: 1000,
            backend_used: "ibmq_qasm_simulator".to_string(),
            execution_time_ms: 1000.0,
            metadata: HashMap::new(),
        })
    }

    fn list_backends(&self) -> Result<Vec<String>> {
        // Placeholder: List available backends
        Ok(vec![
            "ibmq_qasm_simulator".to_string(),
            "ibmq_lima".to_string(),
            "ibmq_belem".to_string(),
            "ibmq_quito".to_string(),
        ])
    }

    fn get_backend_info(&self, backend_name: &str) -> Result<BackendInfo> {
        // Placeholder: Get backend information with detailed device data
        let is_simulator = backend_name.contains("simulator");

        let (num_qubits, coupling_map, gate_errors, readout_errors, t1_times, t2_times) =
            if is_simulator {
                // Simulator has full connectivity
                (
                    32,
                    vec![],
                    HashMap::new(),
                    vec![0.0; 32],
                    vec![100.0; 32],
                    vec![50.0; 32],
                )
            } else {
                // Example QPU device (ibm_nairobi-like)
                let num_qubits = 7;
                let coupling_map = vec![
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (3, 4),
                    (4, 5),
                    (5, 6),
                    (1, 0),
                    (2, 1),
                    (3, 2),
                    (4, 3),
                    (5, 4),
                    (6, 5),
                ];
                let mut gate_errors = HashMap::new();
                gate_errors.insert("u3_0".to_string(), 0.001);
                gate_errors.insert("cx_0_1".to_string(), 0.01);

                let readout_errors = vec![0.02, 0.025, 0.03, 0.028, 0.032, 0.029, 0.031];
                let t1_times = vec![85.2, 92.1, 78.5, 88.9, 91.3, 86.7, 89.4];
                let t2_times = vec![42.1, 45.8, 39.2, 44.3, 46.1, 43.5, 44.8];

                (
                    num_qubits,
                    coupling_map,
                    gate_errors,
                    readout_errors,
                    t1_times,
                    t2_times,
                )
            };

        Ok(BackendInfo {
            name: backend_name.to_string(),
            version: "1.0".to_string(),
            num_qubits,
            is_simulator,
            status: "online".to_string(),
            queue_length: if is_simulator { Some(0) } else { Some(5) },
            coupling_map,
            gate_errors,
            readout_errors,
            t1_times,
            t2_times,
        })
    }
}

/// Backend information from IBM Quantum
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub name: String,
    pub version: String,
    pub num_qubits: usize,
    pub is_simulator: bool,
    pub status: String,
    /// Queue length for QPU devices
    pub queue_length: Option<usize>,
    /// Coupling map (qubit connectivity)
    pub coupling_map: Vec<(usize, usize)>,
    /// Gate error rates by gate name
    pub gate_errors: HashMap<String, f64>,
    /// Readout error rates per qubit
    pub readout_errors: Vec<f64>,
    /// T1 coherence times in microseconds
    pub t1_times: Vec<f64>,
    /// T2 coherence times in microseconds
    pub t2_times: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::cloud_config::IbmQuantumConfig;

    fn test_config() -> IbmQuantumConfig {
        IbmQuantumConfig {
            enabled: true,
            api_token: "test_token".to_string(),
            api_url: "https://test.api.com".to_string(),
            hub: None,
            group: None,
            project: None,
            default_backend: "ibmq_qasm_simulator".to_string(),
            prefer_simulator: true,
            max_qubits: 127,
        }
    }

    #[test]
    fn test_backend_creation() {
        let config = test_config();
        let backend = IbmQuantumBackend::new(config);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_backend_properties() {
        let config = test_config();
        let backend = IbmQuantumBackend::new(config).unwrap();

        assert_eq!(backend.name(), "ibm_quantum");
        assert_eq!(
            backend.backend_type(),
            ComputeBackendType::Cloud(CloudBackendVariant::IBMQuantum)
        );
    }

    #[test]
    fn test_performance_profile() {
        let config = test_config();
        let backend = IbmQuantumBackend::new(config).unwrap();

        let profile = backend.performance_profile();
        assert_eq!(profile.max_qubits, 127);
        assert!(profile.supports_batching);
    }
}
