use std::path::PathBuf;

/// A single review finding for a line (or file-level).
#[derive(Debug, Clone)]
pub struct Finding {
    /// 1-based line number, or 0 for file-level.
    pub line: usize,
    /// Short category label (e.g. "style", "warning", "error").
    pub category: &'static str,
    /// Human-readable description.
    pub message: String,
    /// Optional suggested replacement/fix text.
    pub suggestion: Option<String>,
}

/// Review an `.ai` source file.
///
/// Returns a (possibly empty) list of findings.  The checks are intentionally
/// lightweight: they mirror the kinds of feedback a Copilot code review produces
/// (unused variables, trailing whitespace, missing semicolons, etc.) without
/// requiring a full semantic pass.
pub fn review_file(path: &PathBuf, suggest: bool) -> anyhow::Result<Vec<Finding>> {
    use anyhow::Context;
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    Ok(review_source(&source, suggest))
}

/// Run the review checks over an in-memory source string.
pub fn review_source(source: &str, suggest: bool) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    // ── file-level: empty file ─────────────────────────────────────────────
    if source.trim().is_empty() {
        findings.push(Finding {
            line: 0,
            category: "warning",
            message: "file is empty".to_string(),
            suggestion: None,
        });
        return findings;
    }

    for (idx, raw_line) in source.lines().enumerate() {
        let lineno = idx + 1;

        // ── trailing whitespace ─────────────────────────────────────────────
        if raw_line != raw_line.trim_end() {
            findings.push(Finding {
                line: lineno,
                category: "style",
                message: "trailing whitespace".to_string(),
                suggestion: if suggest {
                    Some(raw_line.trim_end().to_string())
                } else {
                    None
                },
            });
        }

        let trimmed = raw_line.trim();

        // ── `let` statements missing a semicolon ───────────────────────────
        if trimmed.starts_with("let ")
            && !trimmed.ends_with(';')
            && !trimmed.ends_with('{')
            && !trimmed.ends_with(',')
        {
            findings.push(Finding {
                line: lineno,
                category: "style",
                message: "let binding is missing a terminating semicolon".to_string(),
                suggestion: if suggest {
                    Some(format!("{};", trimmed))
                } else {
                    None
                },
            });
        }

        // ── TODO / FIXME / HACK annotations ───────────────────────────────
        let upper = trimmed.to_uppercase();
        for marker in &["TODO", "FIXME", "HACK", "XXX"] {
            if upper.contains(marker) {
                findings.push(Finding {
                    line: lineno,
                    category: "warning",
                    message: format!("{} comment left in source", marker),
                    suggestion: None,
                });
                break;
            }
        }

        // ── use of `log` without parentheses (common mistake) ─────────────
        if trimmed == "log" || trimmed == "log;" {
            findings.push(Finding {
                line: lineno,
                category: "error",
                message: "`log` used without arguments or parentheses".to_string(),
                suggestion: if suggest {
                    Some("log(value);".to_string())
                } else {
                    None
                },
            });
        }
    }

    findings
}

/// Print findings to stdout in human-readable form.
pub fn print_findings(path: &PathBuf, findings: &[Finding]) {
    if findings.is_empty() {
        println!("review: {} — no issues found ✓", path.display());
        return;
    }
    println!("review: {} — {} finding(s)", path.display(), findings.len());
    for f in findings {
        if f.line == 0 {
            println!("  [{}] {}", f.category, f.message);
        } else {
            println!("  line {:>4} [{}] {}", f.line, f.category, f.message);
        }
        if let Some(ref fix) = f.suggestion {
            println!("         suggestion: {}", fix);
        }
    }
}

/// Emit findings as a JSON array.
pub fn print_findings_json(path: &PathBuf, findings: &[Finding]) {
    let path_str = path
        .display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    print!("{{\"file\":\"{}\",\"findings\":[", path_str);
    for (i, f) in findings.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let suggestion_field = match &f.suggestion {
            Some(s) => format!(
                ",\"suggestion\":\"{}\"",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            ),
            None => String::new(),
        };
        print!(
            "{{\"line\":{},\"category\":\"{}\",\"message\":\"{}\"{}}}",
            f.line,
            f.category,
            f.message.replace('\\', "\\\\").replace('"', "\\\""),
            suggestion_field
        );
    }
    println!("]}}");
}
