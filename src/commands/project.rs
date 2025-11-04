use std::path::PathBuf;

use anyhow::{bail, Result};

use crate::project::{BuildProfile, Project, TestReport};

pub fn build(manifest_path: Option<PathBuf>, release: bool) -> Result<()> {
    let project = Project::load(manifest_path)?;
    let profile = if release {
        BuildProfile::Release
    } else {
        BuildProfile::Debug
    };
    let artifact = project.build(profile)?;
    println!(
        "Built {} v{} [{}] -> {}",
        project.package_name(),
        project.package_version(),
        profile.as_str(),
        artifact.display()
    );
    Ok(())
}

pub fn check(manifest_path: Option<PathBuf>) -> Result<()> {
    let project = Project::load(manifest_path)?;
    project.check()?;
    println!(
        "Checked {} v{} successfully",
        project.package_name(),
        project.package_version()
    );
    Ok(())
}

pub fn run(manifest_path: Option<PathBuf>, release: bool) -> Result<()> {
    let project = Project::load(manifest_path)?;
    project.run(release)
}

pub fn test(manifest_path: Option<PathBuf>, release: bool, filter: Option<String>) -> Result<()> {
    let project = Project::load(manifest_path)?;
    let reports = project.run_tests(release, filter.as_deref())?;

    if reports.is_empty() {
        println!("No tests found");
        return Ok(());
    }

    let mut passed = 0usize;
    for report in &reports {
        if report.passed {
            passed += 1;
        }
        print_report(report);
    }

    if passed == reports.len() {
        println!("test result: ok. {} passed; 0 failed", reports.len());
        Ok(())
    } else {
        let failed = reports.len() - passed;
        println!("test result: FAILED. {} passed; {} failed", passed, failed);
        bail!("{} test(s) failed", failed)
    }
}

fn print_report(report: &TestReport) {
    let group = report
        .group
        .as_ref()
        .map(|g| format!("{}::", g))
        .unwrap_or_default();
    if report.passed {
        println!("    ok - {}{}", group, report.name);
    } else {
        println!("    FAILED - {}{}", group, report.name);
        if let Some(message) = &report.message {
            println!("        reason: {}", message);
        }
    }
}
