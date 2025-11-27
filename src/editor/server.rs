use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::commands::enhanced::CompilerOptions;
use crate::config::Config;
use crate::core::runtime_engine::RuntimeEngine;

/// Application state shared across HTTP handlers
#[derive(Clone)]
pub struct AppState {
    /// Current working directory (project root)
    pub workspace_dir: PathBuf,
    /// Compiler and runtime engine
    pub engine: Arc<RwLock<RuntimeEngine>>,
    /// Active file manager
    pub file_manager: Arc<RwLock<FileManager>>,
    /// WebSocket broadcast for real-time updates
    pub broadcast_tx: broadcast::Sender<EditorEvent>,
}

/// File management state
#[derive(Default)]
pub struct FileManager {
    /// Currently open files in memory
    pub open_files: HashMap<String, String>,
    /// File modification tracking
    pub modified_files: HashMap<String, bool>,
}

/// Events sent over WebSocket for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditorEvent {
    CompileStart { file: String },
    CompileSuccess { file: String, output: String },
    CompileError { file: String, errors: Vec<String> },
    RunStart { file: String },
    RunOutput { file: String, output: String },
    RunError { file: String, error: String },
    RunComplete { file: String },
    FileChange { file: String, content: String },
    StatusUpdate { message: String },
}

/// HTTP request/response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FileOpenRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FileOpenResponse {
    pub content: String,
    pub language: String,
}

#[derive(Debug, Deserialize)]
pub struct FileSaveRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CompileRequest {
    pub file: String,
    #[allow(dead_code)]
    pub options: Option<CompilerOptions>,
}

#[derive(Debug, Serialize)]
pub struct CompileResponse {
    pub success: bool,
    pub output: String,
    pub errors: Vec<String>,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub file: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub extension: Option<String>,
}

/// Start the embedded web server
pub async fn start_editor_server(port: u16, workspace_dir: PathBuf) -> Result<()> {
    println!("🚀 Starting Aeonmi Editor Server...");

    // Initialize state
    let _config = Config::new();
    let engine = Arc::new(RwLock::new(RuntimeEngine::new()));
    let file_manager = Arc::new(RwLock::new(FileManager::default()));
    let (broadcast_tx, _) = broadcast::channel(100);

    let state = AppState {
        workspace_dir: workspace_dir.clone(),
        engine,
        file_manager,
        broadcast_tx,
    };

    // Build router with all endpoints
    let app = Router::new()
        // Static files (embedded in binary)
        .route("/", get(serve_index))
        .route("/editor.js", get(serve_editor_js))
        .route("/editor.css", get(serve_editor_css))
        .route("/monaco.js", get(serve_monaco))
        // API endpoints
        .route("/api/files", get(list_files).post(create_file))
        .route(
            "/api/files/*path",
            get(get_file).put(save_file).delete(delete_file),
        )
        .route("/api/compile", post(compile_file))
        .route("/api/run", post(run_file))
        .route("/api/stop", post(stop_execution))
        .route("/api/workspace", get(get_workspace_info))
        // WebSocket for real-time updates
        .route("/ws", get(websocket_handler))
        // File serving from workspace
        .nest_service("/workspace", ServeDir::new(&workspace_dir))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    let addr = listener.local_addr()?;

    println!("📡 Editor available at: http://{}", addr);
    println!("📁 Workspace directory: {}", workspace_dir.display());
    println!("⚡ Press Ctrl+C to stop the server");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Serve the main editor HTML page
async fn serve_index() -> Html<&'static str> {
    Html(include_str!("integrated_editor.html"))
}

/// Serve the editor JavaScript
async fn serve_editor_js() -> impl IntoResponse {
    let content = include_str!("../../gui/static/main.js");
    ([(header::CONTENT_TYPE, "application/javascript")], content)
}

/// Serve the editor CSS
async fn serve_editor_css() -> impl IntoResponse {
    // Extract CSS from the HTML file or create separate CSS
    let content = r#"
        /* CSS extracted from quantum_ide.html */
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            background: linear-gradient(135deg, #0f0f1e 0%, #1a1a2e 50%, #16213e 100%);
            color: #e6e6fa;
            height: 100vh;
            overflow: hidden;
        }
        /* Add more extracted CSS here */
    "#;
    ([(header::CONTENT_TYPE, "text/css")], content)
}

/// Serve Monaco Editor files (simplified - in production, bundle these)
async fn serve_monaco() -> impl IntoResponse {
    let content = include_str!("../../gui/static/monaco_ai_language.js");
    ([(header::CONTENT_TYPE, "application/javascript")], content)
}

/// WebSocket handler for real-time communication
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // Spawn task to handle incoming messages
    let state_clone = state.clone();
    let msg_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                // Handle incoming WebSocket messages (commands from UI)
                if let Ok(event) = serde_json::from_str::<EditorEvent>(&text) {
                    // Process the event
                    let _ = state_clone.broadcast_tx.send(event);
                }
            }
        }
    });

    // Spawn task to send broadcast messages
    let broadcast_task = tokio::spawn(async move {
        while let Ok(event) = broadcast_rx.recv().await {
            if let Ok(text) = serde_json::to_string(&event) {
                if sender.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = msg_task => {},
        _ = broadcast_task => {},
    }
}

/// List files in the workspace
async fn list_files(State(state): State<AppState>) -> Result<Json<FileListResponse>, StatusCode> {
    let mut files = Vec::new();

    fn scan_directory(
        dir: &std::path::Path,
        files: &mut Vec<FileEntry>,
    ) -> std::result::Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Skip hidden files and common ignore patterns
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            let is_directory = path.is_dir();
            let extension = if !is_directory {
                path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            };

            files.push(FileEntry {
                name,
                path: path.to_string_lossy().to_string(),
                is_directory,
                extension,
            });

            if is_directory {
                let _ = scan_directory(&path, files);
            }
        }
        Ok(())
    }

    if let Err(_) = scan_directory(&state.workspace_dir, &mut files) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(FileListResponse { files }))
}

/// Get file content
async fn get_file(
    Path(file_path): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<FileOpenResponse>, StatusCode> {
    let full_path = state.workspace_dir.join(&file_path);

    if !full_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let language = detect_language(&file_path);

    Ok(Json(FileOpenResponse { content, language }))
}

/// Save file content
async fn save_file(
    Path(file_path): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<FileSaveRequest>,
) -> Result<StatusCode, StatusCode> {
    let full_path = state.workspace_dir.join(&file_path);

    // Create parent directories if needed
    if let Some(parent) = full_path.parent() {
        if let Err(_) = std::fs::create_dir_all(parent) {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Err(_) = std::fs::write(&full_path, &request.content) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Update file manager
    {
        let mut file_manager = state.file_manager.write().await;
        file_manager
            .open_files
            .insert(file_path.clone(), request.content.clone());
        file_manager.modified_files.insert(file_path.clone(), false);
    }

    // Broadcast file change
    let _ = state.broadcast_tx.send(EditorEvent::FileChange {
        file: file_path,
        content: request.content,
    });

    Ok(StatusCode::OK)
}

/// Create new file
async fn create_file(
    State(state): State<AppState>,
    Json(request): Json<FileSaveRequest>,
) -> Result<StatusCode, StatusCode> {
    let full_path = state.workspace_dir.join(&request.path);

    if full_path.exists() {
        return Err(StatusCode::CONFLICT);
    }

    if let Some(parent) = full_path.parent() {
        if let Err(_) = std::fs::create_dir_all(parent) {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Err(_) = std::fs::write(&full_path, &request.content) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(StatusCode::CREATED)
}

/// Delete file
async fn delete_file(
    Path(file_path): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let full_path = state.workspace_dir.join(&file_path);

    if !full_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    if let Err(_) = std::fs::remove_file(&full_path) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(StatusCode::OK)
}

/// Compile file
async fn compile_file(
    State(state): State<AppState>,
    Json(request): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, StatusCode> {
    let file_path = state.workspace_dir.join(&request.file);

    // Broadcast compile start
    let _ = state.broadcast_tx.send(EditorEvent::CompileStart {
        file: request.file.clone(),
    });

    // Read file content
    let content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Compile using the runtime engine
    let engine = state.engine.read().await;
    match engine.compile_source(&content, &request.file) {
        Ok(result) => {
            let _ = state.broadcast_tx.send(EditorEvent::CompileSuccess {
                file: request.file.clone(),
                output: result.to_string(),
            });

            Ok(Json(CompileResponse {
                success: true,
                output: "Compilation successful".to_string(),
                errors: vec![],
                artifacts: vec![], // TODO: List actual artifacts
            }))
        }
        Err(err) => {
            let error_msg = err.to_string();
            let _ = state.broadcast_tx.send(EditorEvent::CompileError {
                file: request.file.clone(),
                errors: vec![error_msg.clone()],
            });

            Ok(Json(CompileResponse {
                success: false,
                output: "".to_string(),
                errors: vec![error_msg],
                artifacts: vec![],
            }))
        }
    }
}

/// Run file
async fn run_file(
    State(state): State<AppState>,
    Json(request): Json<RunRequest>,
) -> Result<Json<CompileResponse>, StatusCode> {
    let file_path = state.workspace_dir.join(&request.file);

    // Broadcast run start
    let _ = state.broadcast_tx.send(EditorEvent::RunStart {
        file: request.file.clone(),
    });

    // Read file content
    let content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Execute using the runtime engine
    let mut engine = state.engine.write().await;
    match engine.execute_source(&content, &request.file) {
        Ok(result) => {
            let output = format!("{:?}", result); // TODO: Better formatting

            let _ = state.broadcast_tx.send(EditorEvent::RunOutput {
                file: request.file.clone(),
                output: output.clone(),
            });

            let _ = state.broadcast_tx.send(EditorEvent::RunComplete {
                file: request.file.clone(),
            });

            Ok(Json(CompileResponse {
                success: true,
                output,
                errors: vec![],
                artifacts: vec![],
            }))
        }
        Err(err) => {
            let error_msg = err.to_string();
            let _ = state.broadcast_tx.send(EditorEvent::RunError {
                file: request.file.clone(),
                error: error_msg.clone(),
            });

            Ok(Json(CompileResponse {
                success: false,
                output: "".to_string(),
                errors: vec![error_msg],
                artifacts: vec![],
            }))
        }
    }
}

/// Stop execution (placeholder - requires execution context tracking)
async fn stop_execution(State(_state): State<AppState>) -> StatusCode {
    // TODO: Implement execution cancellation
    StatusCode::OK
}

/// Get workspace information
async fn get_workspace_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "workspace": state.workspace_dir.to_string_lossy(),
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME"),
    }))
}

/// Detect programming language from file extension
fn detect_language(file_path: &str) -> String {
    let path = std::path::Path::new(file_path);
    match path.extension().and_then(|s| s.to_str()) {
        Some("aeon") | Some("aeonmi") => "aeonmi".to_string(),
        Some("rs") => "rust".to_string(),
        Some("js") => "javascript".to_string(),
        Some("ts") => "typescript".to_string(),
        Some("py") => "python".to_string(),
        Some("md") => "markdown".to_string(),
        Some("toml") => "toml".to_string(),
        Some("json") => "json".to_string(),
        Some("html") => "html".to_string(),
        Some("css") => "css".to_string(),
        Some("qasm") => "qasm".to_string(),
        _ => "plaintext".to_string(),
    }
}
