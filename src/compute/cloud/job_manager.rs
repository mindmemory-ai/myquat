//! Cloud Backend Job Management
//! Author: gA4ss
//!
//! Provides job queue management, caching, and retry logic for cloud quantum backends.

use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Job cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedJob {
    /// Job ID
    pub job_id: String,
    /// Job result (if completed)
    pub result: Option<HashMap<String, usize>>,
    /// Cached timestamp
    pub cached_at: SystemTime,
    /// Cache expiration
    pub expires_at: SystemTime,
    /// Job metadata
    pub metadata: HashMap<String, String>,
}

/// Job manager with caching and retry logic
pub struct JobManager {
    /// In-memory job cache
    cache: HashMap<String, CachedJob>,
    /// Maximum retry attempts
    max_retries: usize,
    /// Retry delay in seconds
    retry_delay: u64,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl JobManager {
    /// Create a new job manager
    pub fn new() -> Self {
        JobManager {
            cache: HashMap::new(),
            max_retries: 3,
            retry_delay: 5,
            cache_ttl: 3600, // 1 hour
        }
    }

    /// Create with custom configuration
    pub fn with_config(max_retries: usize, retry_delay: u64, cache_ttl: u64) -> Self {
        JobManager {
            cache: HashMap::new(),
            max_retries,
            retry_delay,
            cache_ttl,
        }
    }

    /// Cache a job result
    pub fn cache_job(
        &mut self,
        job_id: String,
        result: HashMap<String, usize>,
        metadata: HashMap<String, String>,
    ) {
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(self.cache_ttl);

        let cached = CachedJob {
            job_id: job_id.clone(),
            result: Some(result),
            cached_at: now,
            expires_at,
            metadata,
        };

        self.cache.insert(job_id, cached);
    }

    /// Get cached job result
    pub fn get_cached(&mut self, job_id: &str) -> Option<HashMap<String, usize>> {
        if let Some(cached) = self.cache.get(job_id) {
            let now = SystemTime::now();

            // Check if cache is still valid
            if now < cached.expires_at {
                return cached.result.clone();
            } else {
                // Remove expired cache
                self.cache.remove(job_id);
            }
        }
        None
    }

    /// Check if job is cached
    pub fn is_cached(&self, job_id: &str) -> bool {
        if let Some(cached) = self.cache.get(job_id) {
            let now = SystemTime::now();
            now < cached.expires_at
        } else {
            false
        }
    }

    /// Clear expired cache entries
    pub fn clear_expired(&mut self) {
        let now = SystemTime::now();
        self.cache.retain(|_, v| now < v.expires_at);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let now = SystemTime::now();
        let valid_entries = self.cache.values().filter(|v| now < v.expires_at).count();

        CacheStats {
            total_entries: self.cache.len(),
            valid_entries,
            expired_entries: self.cache.len() - valid_entries,
        }
    }

    /// Execute with retry logic
    pub fn execute_with_retry<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);

                    if attempts < self.max_retries {
                        std::thread::sleep(Duration::from_secs(self.retry_delay));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| MyQuatError::circuit_error("All retry attempts failed")))
    }

    /// Get retry configuration
    pub fn retry_config(&self) -> RetryConfig {
        RetryConfig {
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
            cache_ttl: self.cache_ttl,
        }
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cache Statistics:")?;
        writeln!(f, "  Total entries: {}", self.total_entries)?;
        writeln!(f, "  Valid entries: {}", self.valid_entries)?;
        writeln!(f, "  Expired entries: {}", self.expired_entries)?;
        Ok(())
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub retry_delay: u64,
    pub cache_ttl: u64,
}

/// Job queue for managing multiple pending jobs
pub struct JobQueue {
    /// Pending jobs
    pending: Vec<String>,
    /// Running jobs
    running: Vec<String>,
    /// Completed jobs
    completed: Vec<String>,
}

impl JobQueue {
    /// Create a new job queue
    pub fn new() -> Self {
        JobQueue {
            pending: Vec::new(),
            running: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Add a job to the queue
    pub fn add_job(&mut self, job_id: String) {
        self.pending.push(job_id);
    }

    /// Mark job as running
    pub fn mark_running(&mut self, job_id: &str) -> Result<()> {
        if let Some(pos) = self.pending.iter().position(|id| id == job_id) {
            let job = self.pending.remove(pos);
            self.running.push(job);
            Ok(())
        } else {
            Err(MyQuatError::circuit_error("Job not found in pending queue"))
        }
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self, job_id: &str) -> Result<()> {
        if let Some(pos) = self.running.iter().position(|id| id == job_id) {
            let job = self.running.remove(pos);
            self.completed.push(job);
            Ok(())
        } else {
            Err(MyQuatError::circuit_error("Job not found in running queue"))
        }
    }

    /// Get queue status
    pub fn status(&self) -> QueueStatus {
        QueueStatus {
            pending_count: self.pending.len(),
            running_count: self.running.len(),
            completed_count: self.completed.len(),
        }
    }

    /// Get next pending job
    pub fn next_pending(&self) -> Option<&String> {
        self.pending.first()
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue status
#[derive(Debug, Clone)]
pub struct QueueStatus {
    pub pending_count: usize,
    pub running_count: usize,
    pub completed_count: usize,
}

impl std::fmt::Display for QueueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Queue Status:")?;
        writeln!(f, "  Pending: {}", self.pending_count)?;
        writeln!(f, "  Running: {}", self.running_count)?;
        writeln!(f, "  Completed: {}", self.completed_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_manager_cache() {
        let mut manager = JobManager::new();

        let job_id = "test_job_123".to_string();
        let mut result = HashMap::new();
        result.insert("00".to_string(), 500);
        result.insert("11".to_string(), 524);

        let metadata = HashMap::new();
        manager.cache_job(job_id.clone(), result.clone(), metadata);

        assert!(manager.is_cached(&job_id));
        let cached = manager.get_cached(&job_id).unwrap();
        assert_eq!(cached.get("00"), Some(&500));
    }

    #[test]
    fn test_job_queue() {
        let mut queue = JobQueue::new();

        queue.add_job("job1".to_string());
        queue.add_job("job2".to_string());
        queue.add_job("job3".to_string());

        let status = queue.status();
        assert_eq!(status.pending_count, 3);
        assert_eq!(status.running_count, 0);

        queue.mark_running("job1").unwrap();
        let status = queue.status();
        assert_eq!(status.pending_count, 2);
        assert_eq!(status.running_count, 1);

        queue.mark_completed("job1").unwrap();
        let status = queue.status();
        assert_eq!(status.running_count, 0);
        assert_eq!(status.completed_count, 1);
    }

    #[test]
    fn test_retry_logic() {
        let manager = JobManager::with_config(3, 0, 3600);

        let mut attempts = 0;
        let result = manager.execute_with_retry(|| {
            attempts += 1;
            if attempts < 3 {
                Err(MyQuatError::circuit_error("Simulated failure"))
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }
}
