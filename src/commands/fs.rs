use crate::sandbox::filesystem::SandboxedFileSystem;
use anyhow::Result;
use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Create a sandboxed filesystem restricted to the current project directory
fn create_project_sandbox() -> Result<SandboxedFileSystem> {
    let current_dir = std::env::current_dir()?;
    let mut sandbox = SandboxedFileSystem::with_root(&current_dir);
    sandbox.set_allow_parent_access(false); // Prevent .. access
    sandbox.set_allow_absolute_paths(false); // Prevent absolute path access
    Ok(sandbox)
}

pub fn new_file(path: Option<PathBuf>) -> Result<()> {
    let target = path.unwrap_or_else(|| PathBuf::from("untitled.ai"));

    // Create sandbox to validate the target path
    let sandbox = create_project_sandbox()?;
    let safe_target = sandbox
        .validate_path(&target)
        .map_err(|e| anyhow::anyhow!("File creation denied - {}", e))?;

    if safe_target.exists() {
        println!(
            "new: '{}' already exists (leaving unchanged)",
            safe_target.display()
        );
        return Ok(());
    }

    if let Some(parent) = safe_target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let template = format!(
        r#"// Aeonmi source created {now}
let greeting = "Hello Aeonmi";
function square(x) {{ return x * x; }}
let total = 0;
for let i = 0; i < 5; i = i + 1 {{ total = total + square(i); }}
log(greeting, total);
"#
    );
    let mut f = fs::File::create(&safe_target)?;
    f.write_all(template.as_bytes())?;
    println!("new: created '{}' (sandboxed)", safe_target.display());
    Ok(())
}

pub fn open(path: PathBuf) -> Result<()> {
    println!("open: '{}' (placeholder)", path.display());
    Ok(())
}

pub fn save(path: Option<PathBuf>) -> Result<()> {
    if let Some(p) = path {
        println!("save: '{}' (placeholder)", p.display());
    } else {
        println!("save: current buffer (placeholder)");
    }
    Ok(())
}

pub fn save_as(path: PathBuf) -> Result<()> {
    println!("saveas: '{}' (placeholder)", path.display());
    Ok(())
}

pub fn close(path: Option<PathBuf>) -> Result<()> {
    if let Some(p) = path {
        println!("close: '{}' (placeholder)", p.display());
    } else {
        println!("close: current buffer (placeholder)");
    }
    Ok(())
}

pub fn import(path: PathBuf) -> Result<()> {
    println!("import: '{}' (placeholder)", path.display());
    Ok(())
}

pub fn export(path: PathBuf, format: Option<String>) -> Result<()> {
    println!(
        "export: '{}' as '{}' (placeholder)",
        path.display(),
        format.unwrap_or_else(|| "auto".into())
    );
    Ok(())
}

pub fn upload(path: PathBuf) -> Result<()> {
    // Create sandbox restricted to project directory
    let sandbox = create_project_sandbox()?;

    // Validate the source path through sandbox
    let src = sandbox
        .validate_path(&path)
        .map_err(|e| anyhow::anyhow!("Upload denied - {}", e))?;

    // Ensure source exists and is within project
    if !sandbox.exists(&src) {
        anyhow::bail!(
            "upload failed: source '{}' does not exist in project directory",
            src.display()
        );
    }

    let repo_root = std::env::current_dir()?;
    let uploads_dir = repo_root.join("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)?;
    }

    if src.is_dir() {
        let options = fs_extra::dir::CopyOptions::new();
        let to = uploads_dir.join(
            src.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("uploaded_dir")),
        );
        fs_extra::dir::copy(&src, &to, &options)?;
        println!(
            "uploaded dir '{}' -> '{}' (sandboxed)",
            src.display(),
            to.display()
        );
        return Ok(());
    }

    let filename = src
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "uploaded.bin".into());
    let dst = uploads_dir.join(filename);
    fs::copy(&src, &dst)?;
    println!(
        "uploaded '{}' -> '{}' (sandboxed)",
        src.display(),
        dst.display()
    );
    Ok(())
}

pub fn download(path: PathBuf) -> Result<()> {
    // Create sandbox restricted to project directory
    let sandbox = create_project_sandbox()?;

    // Validate the source path through sandbox
    let src = sandbox
        .validate_path(&path)
        .map_err(|e| anyhow::anyhow!("Download denied - {}", e))?;

    // Ensure source exists within project
    if !sandbox.exists(&src) {
        anyhow::bail!(
            "download failed: source '{}' does not exist in project workspace",
            src.display()
        );
    }

    let repo_root = std::env::current_dir()?;
    let downloads_dir = repo_root.join("downloads");
    if !downloads_dir.exists() {
        fs::create_dir_all(&downloads_dir)?;
    }

    if src.is_dir() {
        let options = fs_extra::dir::CopyOptions::new();
        let to = downloads_dir.join(
            src.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("downloaded_dir")),
        );
        fs_extra::dir::copy(&src, &to, &options)?;
        println!(
            "downloaded dir '{}' -> '{}' (sandboxed)",
            src.display(),
            to.display()
        );
        return Ok(());
    }

    let filename = src
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "downloaded.bin".into());
    let dst = downloads_dir.join(filename);
    fs::copy(&src, &dst)?;
    println!(
        "downloaded '{}' -> '{}' (sandboxed)",
        src.display(),
        dst.display()
    );
    Ok(())
}
