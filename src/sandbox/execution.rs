use std::time::Duration;

/// Execution limits and constraints
#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    /// Maximum execution time (wall clock)
    pub timeout: Duration,
    /// Maximum number of concurrent processes
    pub max_processes: usize,
    /// Allowed environment variables
    pub allowed_env_vars: Vec<String>,
    /// Blocked environment variables
    pub blocked_env_vars: Vec<String>,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_processes: 4,
            allowed_env_vars: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "TEMP".to_string(),
            ],
            blocked_env_vars: vec![
                "AWS_ACCESS_KEY_ID".to_string(),
                "AWS_SECRET_ACCESS_KEY".to_string(),
                "GITHUB_TOKEN".to_string(),
            ],
        }
    }
}














