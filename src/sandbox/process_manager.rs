use anyhow::Result;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::sandbox::execution::ExecutionLimits;

/// Process manager for handling sandboxed process execution
#[derive(Debug, Clone)]
pub struct ProcessManager {
    /// Active processes
    processes: Arc<Mutex<HashMap<Uuid, ProcessHandle>>>,
    /// Global execution limits
    pub(crate) global_limits: ExecutionLimits,
}

/// Handle to a managed process
#[derive(Debug)]
pub struct ProcessHandle {
    /// Unique process ID
    pub id: Uuid,
    /// Process command
    pub command: String,
    /// Working directory
    pub working_dir: std::path::PathBuf,
    /// Start time
    pub start_time: Instant,
    /// Process child handle (if still running)
    pub child: Option<Child>,
    /// Process status
    pub status: ProcessStatus,
    /// Execution limits
    pub limits: ExecutionLimits,
    /// Kill signal sender
    pub kill_sender: Option<oneshot::Sender<()>>,
}

/// Process execution status
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    /// Process is starting
    Starting,
    /// Process is running
    Running,
    /// Process completed successfully
    Completed(i32),
    /// Process was killed by timeout
    TimedOut,
    /// Process was killed by user
    Killed,
    /// Process failed to start
    Failed(String),
}

/// Process execution configuration
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: std::path::PathBuf,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Execution limits
    pub limits: ExecutionLimits,
    /// Capture stdout
    pub capture_stdout: bool,
    /// Capture stderr
    pub capture_stderr: bool,
}



/// Process manager errors
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Process not found: {0}")]
    ProcessNotFound(Uuid),
    #[error("Process limit exceeded: {0}")]
    LimitExceeded(String),
    #[error("Process execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Process timeout after {0:?}")]
    Timeout(Duration),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Safe OS-level process kill that ignores "already exited" errors
#[cfg(windows)]
fn safe_kill_by_pid(pid: u32) -> Result<()> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};
    const PROCESS_TERMINATE: u32 = 0x0001;

    unsafe {
        let h = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if h == 0 {
            // already exited or invalid pid → treat as success
            return Ok(());
        }
        let _ = TerminateProcess(h, 1);
        CloseHandle(h);
    }
    Ok(())
}

#[cfg(not(windows))]
fn safe_kill_by_pid(pid: i32) -> Result<()> {
    use nix::errno::Errno;
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    
    match kill(Pid::from_raw(pid), Signal::SIGKILL) {
        Ok(_) => Ok(()),
        Err(nix::Error::Sys(Errno::ESRCH)) => Ok(()), // Process not found - that's ok
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            global_limits: ExecutionLimits::default(),
        }
    }

    /// Create a process manager with custom global limits
    pub fn with_limits(limits: ExecutionLimits) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            global_limits: limits,
        }
    }

    /// Start a new process
    pub fn start_process(&self, config: ProcessConfig) -> Result<Uuid, ProcessError> {
        // Check global process limit
        let processes = self.processes.lock().unwrap();
        let active_count = processes
            .values()
            .filter(|p| matches!(p.status, ProcessStatus::Running | ProcessStatus::Starting))
            .count();

        if active_count >= self.global_limits.max_processes {
            return Err(ProcessError::LimitExceeded(format!(
                "Maximum processes ({}) exceeded",
                self.global_limits.max_processes
            )));
        }
        drop(processes);

        // Validate command
        self.validate_command(&config.command)?;

        // Create process ID
        let process_id = Uuid::new_v4();

        // Set up environment variables (filtered)
        let env_vars = self.filter_environment_vars(&config.env_vars, &config.limits)?;

        // Create kill signal channel
        let (kill_sender, kill_receiver) = oneshot::channel::<()>();

        // Create process handle
        let handle = ProcessHandle {
            id: process_id,
            command: config.command.clone(),
            working_dir: config.working_dir.clone(),
            start_time: Instant::now(),
            child: None,
            status: ProcessStatus::Starting,
            limits: config.limits.clone(),
            kill_sender: Some(kill_sender),
        };

        // Store handle
        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(process_id, handle);
        }

        // Start process in background thread
        let processes_clone = Arc::clone(&self.processes);
        let config_clone = config.clone();

        thread::spawn(move || {
            let result = Self::execute_process(
                process_id,
                config_clone,
                env_vars,
                kill_receiver,
                Arc::clone(&processes_clone),
            );

            // Update process status
            let mut processes = processes_clone.lock().unwrap();
            if let Some(handle) = processes.get_mut(&process_id) {
                match result {
                    Ok(exit_code) => {
                        handle.status = ProcessStatus::Completed(exit_code);
                    }
                    Err(ProcessError::Timeout(_)) => {
                        handle.status = ProcessStatus::TimedOut;
                    }
                    Err(_) => {
                        handle.status = ProcessStatus::Killed;
                    }
                }
                handle.child = None;
            }
        });

        Ok(process_id)
    }

    /// Kill a running process
    pub fn kill_process(&self, process_id: Uuid) -> Result<(), ProcessError> {
        let mut processes = self.processes.lock().unwrap();

        if let Some(handle) = processes.get_mut(&process_id) {
            match handle.status {
                ProcessStatus::Running | ProcessStatus::Starting => {
                    // Send kill signal
                    if let Some(kill_sender) = handle.kill_sender.take() {
                        let _ = kill_sender.send(());
                    }

                    // Kill child process if available
                    if let Some(ref mut child) = handle.child {
                        let _ = child.kill();
                    }

                    handle.status = ProcessStatus::Killed;
                    Ok(())
                }
                _ => Err(ProcessError::ProcessNotFound(process_id)),
            }
        } else {
            Err(ProcessError::ProcessNotFound(process_id))
        }
    }

    /// Get process status
    pub fn get_process_status(&self, process_id: Uuid) -> Option<ProcessStatus> {
        let processes = self.processes.lock().unwrap();
        processes.get(&process_id).map(|h| h.status.clone())
    }

    /// List all processes
    pub fn list_processes(&self) -> Vec<(Uuid, ProcessStatus, String, Duration)> {
        let processes = self.processes.lock().unwrap();
        processes
            .values()
            .map(|h| {
                (
                    h.id,
                    h.status.clone(),
                    h.command.clone(),
                    h.start_time.elapsed(),
                )
            })
            .collect()
    }

    /// Get active process count
    pub fn active_process_count(&self) -> usize {
        let processes = self.processes.lock().unwrap();
        processes
            .values()
            .filter(|p| matches!(p.status, ProcessStatus::Running | ProcessStatus::Starting))
            .count()
    }

    /// Kill all running processes
    pub fn kill_all_processes(&self) -> Result<(), ProcessError> {
        let process_ids: Vec<Uuid> = {
            let processes = self.processes.lock().unwrap();
            processes.keys().copied().collect()
        };

        for process_id in process_ids {
            let _ = self.kill_process(process_id);
        }

        Ok(())
    }

    /// Clean up completed processes from memory
    #[allow(dead_code)]
    pub fn cleanup_completed(&self) {
        let mut processes = self.processes.lock().unwrap();
        processes.retain(|_, handle| {
            !matches!(
                handle.status,
                ProcessStatus::Completed(_)
                    | ProcessStatus::TimedOut
                    | ProcessStatus::Killed
                    | ProcessStatus::Failed(_)
            )
        });
    }

    /// Wait for a process to complete
    pub fn wait_for_process(
        &self,
        process_id: Uuid,
        timeout: Option<Duration>,
    ) -> Result<ProcessStatus, ProcessError> {
        let start_time = Instant::now();

        loop {
            if let Some(status) = self.get_process_status(process_id) {
                match status {
                    ProcessStatus::Running | ProcessStatus::Starting => {
                        // Check timeout
                        if let Some(timeout) = timeout {
                            if start_time.elapsed() > timeout {
                                self.kill_process(process_id)?;
                                return Err(ProcessError::Timeout(timeout));
                            }
                        }

                        // Wait a bit before checking again
                        thread::sleep(Duration::from_millis(100));
                    }
                    _ => return Ok(status),
                }
            } else {
                return Err(ProcessError::ProcessNotFound(process_id));
            }
        }
    }

    // --- Private helper methods ---

    fn validate_command(&self, command: &str) -> Result<(), ProcessError> {
        // Block potentially dangerous commands
        let blocked_commands = [
            "rm", "rmdir", "del", "format", "fdisk", "sudo", "su", "chmod", "chown", "wget",
            "curl", "nc", "netcat", "python", "node", "ruby", "perl",
        ];

        let command_name = command.split_whitespace().next().unwrap_or("");

        if blocked_commands.contains(&command_name) {
            return Err(ProcessError::PermissionDenied(format!(
                "Command '{}' is not allowed",
                command_name
            )));
        }

        Ok(())
    }

    fn filter_environment_vars(
        &self,
        env_vars: &HashMap<String, String>,
        limits: &ExecutionLimits,
    ) -> Result<HashMap<String, String>, ProcessError> {
        let mut filtered = HashMap::new();

        // Add allowed system environment variables
        for var_name in &limits.allowed_env_vars {
            if let Ok(value) = std::env::var(var_name) {
                filtered.insert(var_name.clone(), value);
            }
        }

        // Add user-provided environment variables (if not blocked)
        for (key, value) in env_vars {
            if !limits.blocked_env_vars.contains(key) {
                filtered.insert(key.clone(), value.clone());
            }
        }

        Ok(filtered)
    }

    fn execute_process(
        process_id: Uuid,
        config: ProcessConfig,
        env_vars: HashMap<String, String>,
        mut kill_receiver: oneshot::Receiver<()>,
        processes: Arc<Mutex<HashMap<Uuid, ProcessHandle>>>,
    ) -> Result<i32, ProcessError> {
        // Update status to running
        {
            let mut processes = processes.lock().unwrap();
            if let Some(handle) = processes.get_mut(&process_id) {
                handle.status = ProcessStatus::Running;
            }
        }

        // Set up command
        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .current_dir(&config.working_dir)
            .envs(&env_vars);

        // Windows: isolate process group so kill is reliable
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
            command.creation_flags(CREATE_NEW_PROCESS_GROUP);
        }

        // Set up stdio
        if config.capture_stdout {
            command.stdout(Stdio::piped());
        }
        if config.capture_stderr {
            command.stderr(Stdio::piped());
        }

        // Start process
        let child = command
            .spawn()
            .map_err(|e| ProcessError::ExecutionFailed(e.to_string()))?;

        // Store child handle
        {
            let mut processes = processes.lock().unwrap();
            if let Some(handle) = processes.get_mut(&process_id) {
                handle.child = Some(child);
            }
        }

        // Timeout parameters
        let timeout_duration = config.limits.timeout;
        let start_time = Instant::now();

        // Wait for process completion or timeout/kill
        loop {
            // Kill signal?
            if kill_receiver.try_recv().is_ok() {
                let mut processes = processes.lock().unwrap();
                if let Some(handle) = processes.get_mut(&process_id) {
                    if let Some(ref mut child) = handle.child {
                        let _ = child.kill();
                        #[cfg(windows)]
                        {
                            // Best effort to reap the whole tree - use safe kill to avoid noise
                            let _ = safe_kill_by_pid(child.id());
                        }
                    }
                }
                return Err(ProcessError::ExecutionFailed("Process killed".to_string()));
            }

            // Timeout?
            if start_time.elapsed() > timeout_duration {
                let mut processes = processes.lock().unwrap();
                if let Some(handle) = processes.get_mut(&process_id) {
                    if let Some(ref mut child) = handle.child {
                        let _ = child.kill();
                        #[cfg(windows)]
                        {
                            let _ = safe_kill_by_pid(child.id());
                        }
                    }
                }
                return Err(ProcessError::Timeout(timeout_duration));
            }

            // Process finished?
            {
                let mut processes = processes.lock().unwrap();
                if let Some(handle) = processes.get_mut(&process_id) {
                    if let Some(ref mut child_ref) = handle.child {
                        match child_ref.try_wait() {
                            Ok(Some(status)) => {
                                let code = status.code().unwrap_or(-1);
                                return Ok(code);
                            }
                            Ok(None) => {
                                // Still running; fall through to sleep
                            }
                            Err(e) => {
                                return Err(ProcessError::Io(e));
                            }
                        }
                    } else {
                        // No child stored; treat as failure
                        return Err(ProcessError::ExecutionFailed(
                            "missing child handle".to_string(),
                        ));
                    }
                } else {
                    return Err(ProcessError::ProcessNotFound(process_id));
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        assert_eq!(manager.active_process_count(), 0);
    }

    #[test]
    fn test_process_limits() {
        let limits = ExecutionLimits {
            timeout: Duration::from_secs(5),
            max_processes: 2,
            allowed_env_vars: vec!["PATH".to_string()],
            blocked_env_vars: vec!["SECRET".to_string()],
        };

        let manager = ProcessManager::with_limits(limits);
        assert_eq!(manager.global_limits.max_processes, 2);
    }

    #[test]
    #[cfg(unix)]
    fn test_simple_process_execution() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let manager = ProcessManager::new();

        let config = ProcessConfig {
            command: "echo".to_string(),
            args: vec!["Hello, World!".to_string()],
            working_dir: temp_dir.path().to_path_buf(),
            env_vars: HashMap::new(),
            limits: ExecutionLimits::default(),
            capture_stdout: true,
            capture_stderr: false,
        };

        let process_id = manager.start_process(config)?;

        // Wait for completion
        let status = manager.wait_for_process(process_id, Some(Duration::from_secs(5)))?;

        match status {
            ProcessStatus::Completed(exit_code) => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Process should have completed successfully"),
        }

        Ok(())
    }

    #[test]
    fn test_process_timeout() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let manager = ProcessManager::new();

        // Cross-platform sleepy command
        #[cfg(unix)]
        let cmd = ("sleep".to_string(), vec!["10".to_string()]);
        #[cfg(windows)]
        let cmd = (
            "powershell".to_string(),
            vec!["-Command".to_string(), "Start-Sleep -Seconds 10".to_string()],
        );

        let config = ProcessConfig {
            command: cmd.0,
            args: cmd.1,
            working_dir: temp_dir.path().to_path_buf(),
            env_vars: HashMap::new(),
            limits: ExecutionLimits {
                timeout: Duration::from_secs(1),
                ..ExecutionLimits::default()
            },
            capture_stdout: false,
            capture_stderr: false,
        };

        let process_id = manager.start_process(config)?;

        // Wait for timeout
        let status = manager.wait_for_process(process_id, Some(Duration::from_secs(5)))?;
        assert_eq!(status, ProcessStatus::TimedOut);

        Ok(())
    }

    #[test]
    fn test_process_killing() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let manager = ProcessManager::new();

        #[cfg(unix)]
        let cmd = ("sleep".to_string(), vec!["10".to_string()]);
        #[cfg(windows)]
        let cmd = (
            "powershell".to_string(),
            vec!["-Command".to_string(), "Start-Sleep -Seconds 10".to_string()],
        );

        let config = ProcessConfig {
            command: cmd.0,
            args: cmd.1,
            working_dir: temp_dir.path().to_path_buf(),
            env_vars: HashMap::new(),
            limits: ExecutionLimits::default(),
            capture_stdout: false,
            capture_stderr: false,
        };

        let process_id = manager.start_process(config)?;

        // Wait a bit then kill
        thread::sleep(Duration::from_millis(150));
        manager.kill_process(process_id)?;

        let status = manager.wait_for_process(process_id, Some(Duration::from_secs(5)))?;
        assert_eq!(status, ProcessStatus::Killed);

        Ok(())
    }

    #[test]
    fn test_blocked_commands() {
        let manager = ProcessManager::new();

        // Should block dangerous commands
        assert!(manager.validate_command("rm -rf /").is_err());
        assert!(manager.validate_command("sudo rm file").is_err());
        assert!(manager.validate_command("wget malicious.com").is_err());

        // Should allow safe commands
        assert!(manager.validate_command("echo hello").is_ok());
        assert!(manager.validate_command("ls -la").is_ok());
    }
}
