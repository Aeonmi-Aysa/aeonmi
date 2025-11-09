use std::fs;
use anyhow::Result;
use tempfile;

// Import the Project module for library-level tests
use aeonmi_project::project::{Project, BuildProfile};

fn create_test_project(tmp: &tempfile::TempDir, main_src: &str) -> Result<Project> {
    let root = tmp.path();
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/main.ai"), main_src)?;
    fs::write(
        root.join("Aeonmi.toml"),
        "[package]\nname=\"testproj\"\nversion=\"0.1.0\"\n[aeonmi]\nentry=\"src/main.ai\"\n",
    )?;
    Project::load(Some(root.join("Aeonmi.toml")))
}

#[test]
fn project_requires_main_on_check() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let project = create_test_project(&tmp, "fn helper:\n    print \"not main\"\n")?;
    
    let err = project.check().unwrap_err();
    
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("fn main:") || err_msg.contains("entry point"),
        "Error should mention main requirement, got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("helper"),
        "Error should list available function 'helper', got: {}",
        err_msg
    );
    
    Ok(())
}

#[test]
fn project_requires_main_on_build() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let project = create_test_project(&tmp, "")?;  // Empty file
    
    let err = project.build(BuildProfile::Debug).unwrap_err();
    
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("fn main:") || err_msg.contains("entry point"),
        "Build error should mention main, got: {}",
        err_msg
    );
    
    Ok(())
}

#[test]
fn project_run_executes_main() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let project = create_test_project(
        &tmp,
        "fn main:\n    print \"Hello from main!\"\n    let x = 2 + 2\n    assert x == 4\n",
    )?;
    
    // Check should pass
    project.check()?;
    
    // Run should execute main successfully
    project.run(false)?;
    
    Ok(())
}

#[test]
fn project_with_helper_and_main() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let project = create_test_project(
        &tmp,
        "fn helper:\n    print \"I help\"\n\nfn main:\n    call helper\n    assert 1 == 1\n",
    )?;
    
    // Should pass check
    project.check()?;
    
    // Should run successfully
    project.run(false)?;
    
    Ok(())
}

#[test]
fn error_message_helpful_when_no_functions() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let project = create_test_project(&tmp, "")?;
    
    let err = project.check().unwrap_err();
    
    let err_msg = err.to_string();
    // Should suggest adding main
    assert!(
        err_msg.contains("fn main:"),
        "Error should mention fn main:, got: {}",
        err_msg
    );
    // Should include example
    assert!(
        err_msg.contains("Example:") || err_msg.contains("log"),
        "Error should include example, got: {}",
        err_msg
    );
    
    Ok(())
}

#[test]
fn error_message_lists_available_functions() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let project = create_test_project(
        &tmp,
        "fn alpha:\n    print \"a\"\n\nfn beta:\n    print \"b\"\n",
    )?;
    
    let err = project.check().unwrap_err();
    
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("alpha") && err_msg.contains("beta"),
        "Error should list available functions (alpha, beta), got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("Available functions"),
        "Error should have 'Available functions' header, got: {}",
        err_msg
    );
    
    Ok(())
}
