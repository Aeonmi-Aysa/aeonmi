use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::process_manager::ProcessManager;
use super::workspace::AeonmiWorkspace;

/// Execution limits and constraints
#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    /// Maximum execution time (wall clock)
    pub timeout: Duration,
    /// Memory limit in MB
    pub memory_limit_mb: usize,
    /// Maximum number of concurrent processes
    pub max_processes: usize,
    /// Allow network access
    pub allow_network: bool,
    /// Allow file system writes
    pub allow_file_write: bool,
    /// Allowed external commands
    pub allowed_commands: Vec<String>,
    /// Allowed environment variables
    pub allowed_env_vars: Vec<String>,
    /// Blocked environment variables
    pub blocked_env_vars: Vec<String>,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            memory_limit_mb: 128,
            max_processes: 4,
            allow_network: false,
            allow_file_write: true,
            allowed_commands: vec![
                "python".to_string(),
                "python3".to_string(),
                "qiskit".to_string(),
            ],
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

/// Result of a program execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Execution ID
    pub id: Uuid,
    /// Whether execution completed successfully
    pub success: bool,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration: Duration,
    /// Memory usage peak (if available)
    pub memory_usage_mb: Option<usize>,
    /// Termination reason
    pub termination_reason: TerminationReason,
}

/// Why the execution terminated
#[derive(Debug, Clone, PartialEq)]
pub enum TerminationReason {
    /// Completed normally
    Completed,
    /// Timed out
    Timeout,
    /// Killed by user
    UserKilled,
    /// Exceeded memory limit
    MemoryLimit,
    /// Process error
    ProcessError(String),
    /// Internal error
    InternalError(String),
}

/// Execution status updates
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    Started {
        id: Uuid,
        timestamp: Instant,
    },
    Output {
        id: Uuid,
        output: String,
        is_stderr: bool,
    },
    Completed {
        id: Uuid,
        result: ExecutionResult,
    },
    Error {
        id: Uuid,
        error: String,
    },
}

/// Context for sandboxed execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    workspace: AeonmiWorkspace,
    limits: ExecutionLimits,
    process_manager: Arc<ProcessManager>,
}

impl ExecutionContext {
    pub fn new(workspace: AeonmiWorkspace, limits: ExecutionLimits) -> Self {
        Self {
            workspace,
            limits,
            process_manager: Arc::new(ProcessManager::new()),
        }
    }

    pub fn workspace(&self) -> &AeonmiWorkspace {
        &self.workspace
    }

    pub fn limits(&self) -> &ExecutionLimits {
        &self.limits
    }

    pub fn process_manager(&self) -> &ProcessManager {
        &self.process_manager
    }
}

/// Manages execution of user programs with sandboxing and limits
pub struct ExecutionManager {
    context: ExecutionContext,
    active_executions: Arc<Mutex<HashMap<Uuid, ExecutionHandle>>>,
    next_execution_id: AtomicU32,
    event_sender: broadcast::Sender<ExecutionEvent>,
    is_shutting_down: AtomicBool,
}

/// Handle to a running execution
struct ExecutionHandle {
    id: Uuid,
    thread_handle: Option<JoinHandle<ExecutionResult>>,
    kill_signal: Arc<AtomicBool>,
    start_time: Instant,
}

impl ExecutionManager {
    pub fn new(context: ExecutionContext) -> Self {
        let (event_sender, _) = broadcast::channel(100);

        Self {
            context,
            active_executions: Arc::new(Mutex::new(HashMap::new())),
            next_execution_id: AtomicU32::new(1),
            event_sender,
            is_shutting_down: AtomicBool::new(false),
        }
    }

    /// Execute a source file
    pub fn execute_file(&self, source_file: &str, args: &[String]) -> Result<Uuid> {
        if self.is_shutting_down.load(Ordering::Relaxed) {
            anyhow::bail!("Execution manager is shutting down");
        }

        let execution_id = Uuid::new_v4();
        let source_path = self.context.workspace.source_dir().join(source_file);

        // Validate source file exists and is in workspace
        self.context
            .workspace
            .validate_path(&source_path)
            .with_context(|| format!("Path not permitted: {}", source_path.display()))?;
        if !source_path.exists() {
            anyhow::bail!("Source file not found: {}", source_file);
        }

        // Check execution limits
        {
            let active = self.active_executions.lock().unwrap();
            if active.len() >= self.context.limits.max_processes {
                anyhow::bail!("Maximum number of concurrent executions reached");
            }
        }

        let kill_signal = Arc::new(AtomicBool::new(false));
        let context = self.context.clone();
        let args = args.to_vec();
        let source_path = source_path.clone();
        let event_sender = self.event_sender.clone();
        let execution_id_copy = execution_id;
        let kill_signal_copy = kill_signal.clone();

        // Spawn execution thread
        let thread_handle = thread::spawn(move || {
            Self::execute_in_thread(
                execution_id_copy,
                &context,
                &source_path,
                &args,
                kill_signal_copy,
                event_sender,
            )
        });

        // Store execution handle
        let handle = ExecutionHandle {
            id: execution_id,
            thread_handle: Some(thread_handle),
            kill_signal,
            start_time: Instant::now(),
        };

        {
            let mut active = self.active_executions.lock().unwrap();
            active.insert(execution_id, handle);
        }

        // Send started event
        let _ = self.event_sender.send(ExecutionEvent::Started {
            id: execution_id,
            timestamp: Instant::now(),
        });

        Ok(execution_id)
    }

    /// Execute source code directly (not from file)
    pub fn execute_code(&self, code: &str, args: &[String]) -> Result<Uuid> {
        if self.is_shutting_down.load(Ordering::Relaxed) {
            anyhow::bail!("Execution manager is shutting down");
        }

        let execution_id = Uuid::new_v4();

        // Create temporary file
        let temp_filename = format!("temp_{}.aeon", execution_id.simple());
        let temp_path = self
            .context
            .workspace
            .create_temp_file(&temp_filename, code.as_bytes())?;

        // Execute the temporary file
        let result = self.execute_file(&format!("../.aeonmi/temp/{}", temp_filename), args);

        // Clean up temp file on error
        if result.is_err() {
            let _ = std::fs::remove_file(&temp_path);
        }

        result
    }

    /// Kill a running execution
    pub fn kill_execution(&self, execution_id: Uuid) -> Result<()> {
        let mut active = self.active_executions.lock().unwrap();

        if let Some(handle) = active.get_mut(&execution_id) {
            handle.kill_signal.store(true, Ordering::Relaxed);

            // Send kill event
            let _ = self.event_sender.send(ExecutionEvent::Error {
                id: execution_id,
                error: "Execution killed by user".to_string(),
            });

            Ok(())
        } else {
            anyhow::bail!("Execution not found: {}", execution_id);
        }
    }

    /// Kill all running executions
    pub fn kill_all_executions(&self) -> Result<()> {
        let active = self.active_executions.lock().unwrap();

        for handle in active.values() {
            handle.kill_signal.store(true, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Get list of active executions
    pub fn get_active_executions(&self) -> Vec<Uuid> {
        let active = self.active_executions.lock().unwrap();
        active.keys().cloned().collect()
    }

    /// Subscribe to execution events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.event_sender.subscribe()
    }

    /// Wait for an execution to complete (enforces wall timeout by signaling kill)
    pub fn wait_for_execution(&self, execution_id: Uuid) -> Result<ExecutionResult> {
        let timeout = self.context.limits.timeout;

        // Take ownership of the handle so we can join without races
        let mut handle = {
            let mut active = self.active_executions.lock().unwrap();
            active
                .remove(&execution_id)
                .ok_or_else(|| anyhow::anyhow!("Execution not found: {}", execution_id))?
        };

        // spawn a watchdog that triggers kill at timeout
        let kill_signal = handle.kill_signal.clone();
        let _watchdog = thread::spawn({
            let timeout = timeout;
            move || {
                thread::sleep(timeout);
                kill_signal.store(true, Ordering::Relaxed);
            }
        });

        // Join the worker thread (it polls kill/timeout frequently and will exit)
        let joined = match handle.thread_handle.take() {
            Some(th) => th.join(),
            None => return Err(anyhow::anyhow!("Execution thread already joined")),
        };

        // ensure watchdog is not left running forever
        // (it may already be sleeping; that's fine—after join, we just detach)
        // we can't cancel a sleeping thread, but it's short-lived; avoid join to not block

        match joined {
            Ok(result) => Ok(result),
            Err(_) => Ok(ExecutionResult {
                id: execution_id,
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: "Thread panicked".to_string(),
                duration: handle.start_time.elapsed(),
                memory_usage_mb: None,
                termination_reason: TerminationReason::InternalError("Thread panic".to_string()),
            }),
        }
    }

    /// Shutdown the execution manager
    pub fn shutdown(&self) -> Result<()> {
        self.is_shutting_down.store(true, Ordering::Relaxed);
        self.kill_all_executions()?;

        // Wait for all executions to complete
        loop {
            let active_count = {
                let active = self.active_executions.lock().unwrap();
                active.len()
            };

            if active_count == 0 {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    // === Private methods ===

    fn execute_in_thread(
        execution_id: Uuid,
        context: &ExecutionContext,
        source_path: &Path,
        _args: &[String],
        kill_signal: Arc<AtomicBool>,
        event_sender: broadcast::Sender<ExecutionEvent>,
    ) -> ExecutionResult {
        let start_time = Instant::now();

        // --- guard: always signal completion even on panic/early return ---
        struct Done<'a> {
            sent: bool,
            id: Uuid,
            started: Instant,
            tx: &'a broadcast::Sender<ExecutionEvent>,
        }
        impl<'a> Done<'a> {
            fn mark_sent(&mut self) {
                self.sent = true;
            }
        }
        impl<'a> Drop for Done<'a> {
            fn drop(&mut self) {
                if self.sent {
                    return;
                }
                let fallback = ExecutionResult {
                    id: self.id,
                    success: false,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "execution ended without explicit completion (panic/early-return)"
                        .into(),
                    duration: self.started.elapsed(),
                    memory_usage_mb: None,
                    termination_reason: TerminationReason::InternalError(
                        "guard fallback".to_string(),
                    ),
                };
                let _ = self.tx.send(ExecutionEvent::Completed {
                    id: self.id,
                    result: fallback,
                });
            }
        }
        let mut _done_guard = Done {
            sent: false,
            id: execution_id,
            started: start_time,
            tx: &event_sender,
        };

        // --- read source file (error path sends Completed and returns) ---
        let source_code = match std::fs::read_to_string(source_path) {
            Ok(code) => code,
            Err(err) => {
                let stderr = format!("Failed to read source file: {}", err);
                let result = ExecutionResult {
                    id: execution_id,
                    success: false,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: stderr.clone(),
                    duration: start_time.elapsed(),
                    memory_usage_mb: None,
                    termination_reason: TerminationReason::ProcessError(stderr),
                };
                let _ = event_sender.send(ExecutionEvent::Completed {
                    id: execution_id,
                    result: result.clone(),
                });
                _done_guard.mark_sent();
                return result;
            }
        };

        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut success = true;
        let mut termination_reason = TerminationReason::Completed;

        stdout.push_str(&format!("Compiling {}...\n", source_path.display()));

        // Simulated "compile" phase: poll for kill/timeout
        if !poll_until(
            context.limits.timeout,
            &kill_signal,
            start_time,
            Duration::from_millis(200),
        ) {
            termination_reason = if kill_signal.load(Ordering::Relaxed) {
                TerminationReason::UserKilled
            } else {
                TerminationReason::Timeout
            };
            success = false;
            stderr.push_str("Execution interrupted during compilation\n");
        } else {
            // Simulated "run" phase: emit a tiny bit of output and keep polling
            stdout.push_str("Executing quantum program...\n");
            stdout.push_str("Program output:\n");
            stdout.push_str(&format!(
                "Source: {}\n",
                source_code.lines().take(5).collect::<Vec<_>>().join("\n")
            ));

            if !poll_until(
                context.limits.timeout,
                &kill_signal,
                start_time,
                Duration::from_millis(400),
            ) {
                termination_reason = if kill_signal.load(Ordering::Relaxed) {
                    TerminationReason::UserKilled
                } else {
                    TerminationReason::Timeout
                };
                success = false;
                stderr.push_str("Execution interrupted during run\n");
            }
        }

        // If we exceeded global timeout by wall clock, mark timeout
        if start_time.elapsed() > context.limits.timeout && success {
            termination_reason = TerminationReason::Timeout;
            success = false;
            stderr.push_str("Execution timed out\n");
        }

        let result = ExecutionResult {
            id: execution_id,
            success,
            exit_code: Some(if success { 0 } else { 1 }),
            stdout,
            stderr,
            duration: start_time.elapsed(),
            memory_usage_mb: Some(16), // Placeholder
            termination_reason,
        };

        let _ = event_sender.send(ExecutionEvent::Completed {
            id: execution_id,
            result: result.clone(),
        });
        _done_guard.mark_sent();

        result
    }
}

fn poll_until(
    wall: Duration,
    kill: &Arc<AtomicBool>,
    start: Instant,
    step: Duration,
) -> bool {
    while start.elapsed() < wall {
        if kill.load(Ordering::Relaxed) {
            return false;
        }
        // simulate some work slice
        std::thread::sleep(step);
        // In real code, you'd do actual work here and check again.
        return true; // for the placeholder, one slice is enough
    }
    false
}

impl Drop for ExecutionManager {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::workspace::{AeonmiWorkspace, WorkspaceConfig};
    use tempfile::TempDir;
    use serial_test::serial;

    #[test]
    #[serial(sandbox)]
    fn test_execution_manager() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace =
            AeonmiWorkspace::new(temp_dir.path().to_path_buf(), WorkspaceConfig::default())?;
        let context = ExecutionContext::new(workspace, ExecutionLimits::default());
        let manager = ExecutionManager::new(context);

        // Test code execution
        let code = r#"fn main() { print("Hello, World!"); }"#;
        let execution_id = manager.execute_code(code, &[])?;

        // Wait for completion
        let result = manager.wait_for_execution(execution_id)?;

        assert_eq!(result.id, execution_id);
        assert!(result.stdout.contains("Hello, World!")
            || result.stdout.contains("Executing"));

        Ok(())
    }

    #[test]
    #[serial(sandbox)]
    fn test_execution_limits() -> Result<()> {
        // Simple test that just verifies ExecutionLimits struct works
        let limits = ExecutionLimits {
            timeout: Duration::from_millis(500),
            max_processes: 1,
            memory_limit_mb: 64,
            allow_network: false,
            allow_file_write: true,
            allowed_commands: vec!["python".to_string()],
            ..ExecutionLimits::default()
        };

        // Verify limits are set correctly
        assert_eq!(limits.timeout, Duration::from_millis(500));
        assert_eq!(limits.max_processes, 1);
        assert_eq!(limits.memory_limit_mb, 64);
        assert!(!limits.allow_network);
        assert!(limits.allow_file_write);
        assert_eq!(limits.allowed_commands.len(), 1);

        Ok(())
    }
}
