//! AEONMI Reactive Web Framework — Phase 5 IDEA 2
//!
//! Lightweight HTTP server and web framework written in `.ai` syntax.
//! Provides built-in functions: `http_listen`, `http_response`, `http_get`, `http_post`.
//! Backed by `std::net::TcpListener` for zero-dependency HTTP serving.
//!
//! CLI: `aeonmi serve <app.ai>`

use std::collections::HashMap;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpListener;
use std::path::Path;

// ── HTTP Request ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub query: HashMap<String, String>,
}

impl HttpRequest {
    /// Parse an HTTP request from raw bytes
    pub fn parse(raw: &str) -> Option<Self> {
        let mut lines = raw.lines();
        let request_line = lines.next()?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let method = parts[0].to_string();
        let full_path = parts[1].to_string();

        // Parse path and query string
        let (path, query) = if let Some(idx) = full_path.find('?') {
            let p = full_path[..idx].to_string();
            let q = parse_query_string(&full_path[idx + 1..]);
            (p, q)
        } else {
            (full_path, HashMap::new())
        };

        // Parse headers
        let mut headers = HashMap::new();
        let mut header_done = false;
        let mut body_lines = Vec::new();
        for line in lines {
            if line.is_empty() {
                header_done = true;
                continue;
            }
            if header_done {
                body_lines.push(line);
            } else if let Some((key, value)) = line.split_once(':') {
                headers.insert(
                    key.trim().to_lowercase(),
                    value.trim().to_string(),
                );
            }
        }

        Some(HttpRequest {
            method,
            path,
            headers,
            body: body_lines.join("\n"),
            query,
        })
    }

    /// Convert to a map representation for the VM
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("method".to_string(), self.method.clone());
        map.insert("path".to_string(), self.path.clone());
        map.insert("body".to_string(), self.body.clone());
        for (k, v) in &self.headers {
            map.insert(format!("header_{}", k), v.clone());
        }
        for (k, v) in &self.query {
            map.insert(format!("query_{}", k), v.clone());
        }
        map
    }
}

fn parse_query_string(qs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in qs.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(
                url_decode(k),
                url_decode(v),
            );
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

// ── HTTP Response ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub content_type: String,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl HttpResponse {
    pub fn new(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "text/html; charset=utf-8".to_string(),
            body,
            headers: HashMap::new(),
        }
    }

    pub fn json(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "application/json; charset=utf-8".to_string(),
            body,
            headers: HashMap::new(),
        }
    }

    pub fn status_text(&self) -> &str {
        match self.status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }

    pub fn to_http(&self) -> Vec<u8> {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nServer: Aeonmi/1.0\r\n",
            self.status,
            self.status_text(),
            self.content_type,
            self.body.len(),
        );
        for (k, v) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", k, v));
        }
        response.push_str("\r\n");
        response.push_str(&self.body);
        response.into_bytes()
    }
}

// ── Static File Server ───────────────────────────────────────────────────────

/// Serve static files from a directory with basic MIME type detection
pub fn serve_static_file(base_dir: &Path, url_path: &str) -> Option<HttpResponse> {
    // Prevent directory traversal
    let clean_path = url_path.trim_start_matches('/');
    if clean_path.contains("..") {
        return Some(HttpResponse::new(403, "Forbidden".to_string()));
    }

    let file_path = if clean_path.is_empty() || clean_path == "/" {
        base_dir.join("index.html")
    } else {
        base_dir.join(clean_path)
    };

    if file_path.is_file() {
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                let content_type = mime_type_for(&file_path);
                Some(HttpResponse {
                    status: 200,
                    content_type,
                    body: content,
                    headers: HashMap::new(),
                })
            }
            Err(_) => Some(HttpResponse::new(500, "Internal Server Error".to_string())),
        }
    } else {
        None
    }
}

fn mime_type_for(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css")  => "text/css; charset=utf-8",
        Some("js")   => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png")  => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg")  => "image/svg+xml",
        Some("txt")  => "text/plain; charset=utf-8",
        Some("ai")   => "text/plain; charset=utf-8",
        Some("qube") => "text/plain; charset=utf-8",
        _            => "application/octet-stream",
    }.to_string()
}

// ── Simple HTTP Server ───────────────────────────────────────────────────────

/// A minimal HTTP server that dispatches to a handler function.
/// Used by `aeonmi serve` to run `.ai` web applications.
pub struct AeonmiServer {
    pub port: u16,
    pub static_dir: Option<std::path::PathBuf>,
}

impl AeonmiServer {
    pub fn new(port: u16) -> Self {
        Self { port, static_dir: None }
    }

    pub fn with_static_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }

    /// Start the server, running the `.ai` script to register routes,
    /// then listen for requests. This is a blocking call.
    pub fn listen_and_serve<F>(&self, handler: F) -> Result<(), String>
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + 'static,
    {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("Cannot bind to {}: {}", addr, e))?;

        println!("  ◈  Aeonmi server listening on http://{}", addr);
        println!("  ◈  Press Ctrl+C to stop\n");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = [0u8; 8192];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n == 0 { continue; }

                    let raw = String::from_utf8_lossy(&buf[..n]).to_string();
                    let response = if let Some(req) = HttpRequest::parse(&raw) {
                        // Try static file first if configured
                        if let Some(ref dir) = self.static_dir {
                            if let Some(resp) = serve_static_file(dir, &req.path) {
                                resp
                            } else {
                                handler(&req)
                            }
                        } else {
                            handler(&req)
                        }
                    } else {
                        HttpResponse::new(400, "Bad Request".to_string())
                    };

                    let _ = stream.write_all(&response.to_http());
                    let _ = stream.flush();
                }
                Err(e) => {
                    eprintln!("  ⚠  Connection error: {}", e);
                }
            }
        }

        Ok(())
    }
}

// ── Route Table ──────────────────────────────────────────────────────────────

/// Simple route table for matching URL paths to handlers
#[derive(Debug, Clone)]
pub struct RouteTable {
    routes: Vec<Route>,
}

#[derive(Debug, Clone)]
struct Route {
    method: String,
    path: String,
    handler_name: String,
}

impl RouteTable {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: &str) {
        self.routes.push(Route {
            method: method.to_uppercase(),
            path: path.to_string(),
            handler_name: handler.to_string(),
        });
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<&str> {
        for route in &self.routes {
            if route.method == method.to_uppercase() && route.path == path {
                return Some(&route.handler_name);
            }
        }
        None
    }

    pub fn list_routes(&self) -> Vec<(String, String, String)> {
        self.routes.iter().map(|r| {
            (r.method.clone(), r.path.clone(), r.handler_name.clone())
        }).collect()
    }
}

// ── VM Built-in Helpers ──────────────────────────────────────────────────────

/// Create an HTTP response string (for VM built-in `http_response`)
pub fn make_http_response(status: u16, body: &str) -> String {
    format!("HTTP {} | {}", status, body)
}

/// Format a route table as a displayable string
pub fn format_routes(routes: &RouteTable) -> String {
    let mut out = String::new();
    out.push_str("┌─ Aeonmi Routes ────────────────────────┐\n");
    for (method, path, handler) in routes.list_routes() {
        out.push_str(&format!("│  {} {:<20} → {}\n", method, path, handler));
    }
    out.push_str("└────────────────────────────────────────┘\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let raw = "GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/hello");
        assert_eq!(req.headers.get("host"), Some(&"localhost".to_string()));
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let raw = "POST /api/data HTTP/1.1\r\nContent-Type: application/json\r\n\r\n{\"key\":\"value\"}";
        let req = HttpRequest::parse(raw).unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/api/data");
        assert!(req.body.contains("key"));
    }

    #[test]
    fn test_parse_query_string() {
        let raw = "GET /search?q=quantum&lang=ai HTTP/1.1\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();
        assert_eq!(req.path, "/search");
        assert_eq!(req.query.get("q"), Some(&"quantum".to_string()));
        assert_eq!(req.query.get("lang"), Some(&"ai".to_string()));
    }

    #[test]
    fn test_http_response_format() {
        let resp = HttpResponse::new(200, "<h1>Hello</h1>".to_string());
        let http = String::from_utf8(resp.to_http()).unwrap();
        assert!(http.starts_with("HTTP/1.1 200 OK"));
        assert!(http.contains("Content-Type: text/html"));
        assert!(http.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_json_response() {
        let resp = HttpResponse::json(200, r#"{"status":"ok"}"#.to_string());
        let http = String::from_utf8(resp.to_http()).unwrap();
        assert!(http.contains("application/json"));
    }

    #[test]
    fn test_route_table() {
        let mut routes = RouteTable::new();
        routes.add_route("GET", "/", "index_handler");
        routes.add_route("POST", "/api/data", "data_handler");

        assert_eq!(routes.match_route("GET", "/"), Some("index_handler"));
        assert_eq!(routes.match_route("POST", "/api/data"), Some("data_handler"));
        assert_eq!(routes.match_route("GET", "/not-found"), None);
    }

    #[test]
    fn test_make_http_response_helper() {
        let resp = make_http_response(200, "Hello AEONMI");
        assert!(resp.contains("200"));
        assert!(resp.contains("Hello AEONMI"));
    }

    #[test]
    fn test_directory_traversal_blocked() {
        let dir = std::env::temp_dir();
        let resp = serve_static_file(&dir, "/../../../etc/passwd");
        assert!(resp.is_some());
        assert_eq!(resp.unwrap().status, 403);
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(mime_type_for(Path::new("file.html")), "text/html; charset=utf-8");
        assert_eq!(mime_type_for(Path::new("file.json")), "application/json; charset=utf-8");
        assert_eq!(mime_type_for(Path::new("file.css")), "text/css; charset=utf-8");
        assert_eq!(mime_type_for(Path::new("circuit.qube")), "text/plain; charset=utf-8");
    }
}
