//! AEONMI Genesis Glyph NFT Marketplace — Phase 5 IDEA 3
//!
//! CLI + engine for listing, minting, and inspecting Genesis Glyph NFTs
//! built from `.qube` quantum circuit files and `.ai` glyph programs.
//!
//! CLI: `aeonmi market list|info|mint`

use crate::qube::parser::QubeParser;
use crate::qube::executor::QubeExecutor;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Glyph NFT Metadata ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GlyphNft {
    pub name: String,
    pub file_path: String,
    pub circuit_diagram: String,
    pub qubit_count: usize,
    pub gate_count: usize,
    pub entangled_pairs: usize,
    pub quantum_complexity: f64,
    pub source_hash: String,
    pub quantum_signature: String,
}

impl GlyphNft {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
  "name": "{}",
  "file": "{}",
  "qubit_count": {},
  "gate_count": {},
  "entangled_pairs": {},
  "quantum_complexity": {:.2},
  "source_hash": "{}",
  "quantum_signature": "{}",
  "circuit_diagram": {}
}}"#,
            self.name,
            self.file_path,
            self.qubit_count,
            self.gate_count,
            self.entangled_pairs,
            self.quantum_complexity,
            self.source_hash,
            self.quantum_signature,
            serde_json::json!(self.circuit_diagram),
        )
    }
}

// ── Genesis Glyphs (G-1..G-12) ──────────────────────────────────────────────

/// The 12 genesis glyph operators with their quantum significance
pub struct GenesisGlyph {
    pub id: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

pub const GENESIS_GLYPHS: &[GenesisGlyph] = &[
    GenesisGlyph { id: "G-1",  symbol: "⊗", name: "Tensor Product",    description: "Quantum state composition" },
    GenesisGlyph { id: "G-2",  symbol: "⊕", name: "XOR / Addition",    description: "Quantum exclusive-or gate" },
    GenesisGlyph { id: "G-3",  symbol: "⟨ψ|", name: "Bra State",       description: "Quantum bra (dual state)" },
    GenesisGlyph { id: "G-4",  symbol: "|ψ⟩", name: "Ket State",       description: "Quantum ket (state vector)" },
    GenesisGlyph { id: "G-5",  symbol: "†",   name: "Adjoint",         description: "Hermitian conjugate" },
    GenesisGlyph { id: "G-6",  symbol: "∇",   name: "Gradient",        description: "Quantum gradient operator" },
    GenesisGlyph { id: "G-7",  symbol: "∞",   name: "Infinity",        description: "Infinite-dimensional space" },
    GenesisGlyph { id: "G-8",  symbol: "ℏ",   name: "Reduced Planck",  description: "Quantum action unit" },
    GenesisGlyph { id: "G-9",  symbol: "Ω",   name: "Omega",           description: "Frequency / phase space" },
    GenesisGlyph { id: "G-10", symbol: "Δ",   name: "Delta",           description: "Change / uncertainty" },
    GenesisGlyph { id: "G-11", symbol: "Σ",   name: "Sigma",           description: "Summation / superposition" },
    GenesisGlyph { id: "G-12", symbol: "Π",   name: "Pi Product",      description: "Tensor product chain" },
];

// ── Marketplace Scanner ──────────────────────────────────────────────────────

pub struct MarketScanner {
    pub workspace: PathBuf,
}

impl MarketScanner {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    /// Scan workspace for `.qube` files and analyze each as a potential NFT
    pub fn scan_qube_files(&self) -> Vec<GlyphNft> {
        let mut nfts = Vec::new();
        self.walk_dir(&self.workspace, &mut nfts);
        // Sort by quantum complexity descending
        nfts.sort_by(|a, b| b.quantum_complexity.partial_cmp(&a.quantum_complexity).unwrap_or(std::cmp::Ordering::Equal));
        nfts
    }

    fn walk_dir(&self, dir: &Path, nfts: &mut Vec<GlyphNft>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and common non-source dirs
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    self.walk_dir(&path, nfts);
                }
            } else if let Some(ext) = path.extension() {
                if ext == "qube" {
                    if let Some(nft) = self.analyze_qube_file(&path) {
                        nfts.push(nft);
                    }
                }
            }
        }
    }

    /// Analyze a single `.qube` file and produce NFT metadata
    pub fn analyze_qube_file(&self, path: &Path) -> Option<GlyphNft> {
        let source = std::fs::read_to_string(path).ok()?;
        self.analyze_qube_source(&source, path.to_str().unwrap_or("unknown.qube"))
    }

    /// Analyze `.qube` source string and produce NFT metadata
    pub fn analyze_qube_source(&self, source: &str, file_path: &str) -> Option<GlyphNft> {
        // Parse
        let mut parser = QubeParser::from_str(source);
        let prog = parser.parse().ok()?;

        // Execute to get circuit info
        let mut exec = QubeExecutor::new();
        exec.execute(&prog).ok()?;

        let diagram = exec.circuit_diagram();
        let _summary = exec.summary();

        // Count qubits and gates from executor state
        let qubit_count = exec.env.qubits.len();
        let gate_count = exec.env.log.len();

        // Estimate entangled pairs (CNOT operations create entanglement)
        let entangled_pairs = exec.env.log.iter()
            .filter(|entry| {
                let lower = entry.to_lowercase();
                lower.contains("cnot") || lower.contains("entangle") || lower.contains("bell")
            })
            .count();

        // Quantum complexity score based on circuit properties
        let quantum_complexity = compute_complexity(qubit_count, gate_count, entangled_pairs);

        // Source hash
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let source_hash = format!("{:x}", hasher.finalize());

        // Quantum signature — hash of circuit diagram + source
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(diagram.as_bytes());
        sig_hasher.update(source_hash.as_bytes());
        let quantum_signature = format!("{:x}", sig_hasher.finalize());

        // Derive name from file
        let name = Path::new(file_path)
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");

        Some(GlyphNft {
            name: format!("AEONMI Glyph: {}", name),
            file_path: file_path.to_string(),
            circuit_diagram: diagram,
            qubit_count,
            gate_count,
            entangled_pairs,
            quantum_complexity,
            source_hash,
            quantum_signature,
        })
    }

    /// Scan `.ai` files for genesis glyph usage
    pub fn scan_glyph_usage(&self) -> HashMap<String, Vec<String>> {
        let mut usage: HashMap<String, Vec<String>> = HashMap::new();
        self.scan_ai_files(&self.workspace, &mut usage);
        usage
    }

    fn scan_ai_files(&self, dir: &Path, usage: &mut HashMap<String, Vec<String>>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    self.scan_ai_files(&path, usage);
                }
            } else if let Some(ext) = path.extension() {
                if ext == "ai" {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let file_str = path.to_str().unwrap_or("unknown").to_string();
                        for glyph in GENESIS_GLYPHS {
                            if content.contains(glyph.symbol) {
                                usage.entry(glyph.id.to_string())
                                    .or_default()
                                    .push(file_str.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Complexity Scoring ───────────────────────────────────────────────────────

fn compute_complexity(qubits: usize, gates: usize, entangled: usize) -> f64 {
    // Weighted score:
    // - Each qubit adds base complexity
    // - Gates add linear complexity
    // - Entangled pairs add exponential complexity (most valuable)
    let qubit_score = qubits as f64 * 1.5;
    let gate_score = gates as f64 * 0.5;
    let entangle_score = if entangled > 0 {
        (entangled as f64).powf(1.5) * 3.0
    } else {
        0.0
    };
    qubit_score + gate_score + entangle_score
}

// ── Display Helpers ──────────────────────────────────────────────────────────

pub fn format_marketplace_listing(nfts: &[GlyphNft]) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    out.push_str("║         AEONMI Genesis Glyph NFT Marketplace               ║\n");
    out.push_str("╠══════════════════════════════════════════════════════════════╣\n");

    if nfts.is_empty() {
        out.push_str("║  No .qube circuits found in workspace.                     ║\n");
        out.push_str("║  Create a .qube file to mint your first Genesis Glyph NFT. ║\n");
    } else {
        out.push_str("║  #  Name                  Qubits  Gates  Entangled  Score  ║\n");
        out.push_str("║  ─  ────                  ──────  ─────  ─────────  ─────  ║\n");
        for (i, nft) in nfts.iter().enumerate() {
            let short_name = if nft.name.len() > 22 {
                format!("{}…", &nft.name[..21])
            } else {
                nft.name.clone()
            };
            out.push_str(&format!(
                "║  {:2} {:<22} {:>6}  {:>5}  {:>9}  {:>5.1}  ║\n",
                i + 1,
                short_name,
                nft.qubit_count,
                nft.gate_count,
                nft.entangled_pairs,
                nft.quantum_complexity,
            ));
        }
    }

    out.push_str("╠══════════════════════════════════════════════════════════════╣\n");
    out.push_str(&format!("║  Total circuits: {:<43} ║\n", nfts.len()));
    if !nfts.is_empty() {
        let total_complexity: f64 = nfts.iter().map(|n| n.quantum_complexity).sum();
        out.push_str(&format!("║  Total quantum complexity: {:<33.1} ║\n", total_complexity));
    }
    out.push_str("╚══════════════════════════════════════════════════════════════╝\n");
    out
}

pub fn format_glyph_info(nft: &GlyphNft) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    out.push_str("║         Genesis Glyph NFT — Detailed Info                  ║\n");
    out.push_str("╠══════════════════════════════════════════════════════════════╣\n");
    out.push_str(&format!("║  Name: {:<53} ║\n", &nft.name));
    out.push_str(&format!("║  File: {:<53} ║\n", truncate_str(&nft.file_path, 53)));
    out.push_str(&format!("║  Qubits: {:<51} ║\n", nft.qubit_count));
    out.push_str(&format!("║  Gates: {:<52} ║\n", nft.gate_count));
    out.push_str(&format!("║  Entangled Pairs: {:<42} ║\n", nft.entangled_pairs));
    out.push_str(&format!("║  Quantum Complexity: {:<39.2} ║\n", nft.quantum_complexity));
    out.push_str(&format!("║  Source Hash: {:<46} ║\n", truncate_str(&nft.source_hash, 46)));
    out.push_str(&format!("║  Quantum Signature: {:<40} ║\n", truncate_str(&nft.quantum_signature, 40)));
    out.push_str("╠══════════════════════════════════════════════════════════════╣\n");
    out.push_str("║  Circuit Diagram:                                          ║\n");
    for line in nft.circuit_diagram.lines() {
        out.push_str(&format!("║  {:<58} ║\n", truncate_str(line, 58)));
    }
    out.push_str("╚══════════════════════════════════════════════════════════════╝\n");
    out
}

pub fn format_genesis_glyphs() -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    out.push_str("║         Genesis Glyphs (G-1 through G-12)                  ║\n");
    out.push_str("╠══════════════════════════════════════════════════════════════╣\n");
    for g in GENESIS_GLYPHS {
        out.push_str(&format!("║  {:<5} {} {:<18} — {:<25} ║\n",
            g.id, g.symbol, g.name, truncate_str(g.description, 25)));
    }
    out.push_str("╚══════════════════════════════════════════════════════════════╝\n");
    out
}

fn truncate_str(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let truncated: String = chars[..max - 1].iter().collect();
        format!("{}…", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_glyphs_count() {
        assert_eq!(GENESIS_GLYPHS.len(), 12);
    }

    #[test]
    fn test_genesis_glyphs_display() {
        let output = format_genesis_glyphs();
        assert!(output.contains("G-1"));
        assert!(output.contains("G-12"));
        assert!(output.contains("⊗"));
    }

    #[test]
    fn test_complexity_scoring() {
        // No qubits = 0
        assert_eq!(compute_complexity(0, 0, 0), 0.0);

        // More entangled pairs = higher score
        let simple = compute_complexity(2, 3, 0);
        let entangled = compute_complexity(2, 3, 2);
        assert!(entangled > simple, "Entangled circuits should score higher");
    }

    #[test]
    fn test_analyze_qube_source() {
        let scanner = MarketScanner::new(std::env::temp_dir());
        let source = "state q = |0⟩\napply H → q\ncollapse q → result";
        let nft = scanner.analyze_qube_source(source, "test.qube");
        assert!(nft.is_some());
        let nft = nft.unwrap();
        assert_eq!(nft.qubit_count, 1);
        assert!(nft.gate_count > 0);
        assert!(!nft.source_hash.is_empty());
        assert!(!nft.quantum_signature.is_empty());
    }

    #[test]
    fn test_nft_json_output() {
        let nft = GlyphNft {
            name: "Test Glyph".to_string(),
            file_path: "test.qube".to_string(),
            circuit_diagram: "H -> q".to_string(),
            qubit_count: 1,
            gate_count: 1,
            entangled_pairs: 0,
            quantum_complexity: 2.0,
            source_hash: "abc123".to_string(),
            quantum_signature: "def456".to_string(),
        };
        let json = nft.to_json();
        assert!(json.contains("Test Glyph"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_empty_marketplace_listing() {
        let listing = format_marketplace_listing(&[]);
        assert!(listing.contains("No .qube circuits found"));
    }

    #[test]
    fn test_marketplace_listing_with_items() {
        let nfts = vec![
            GlyphNft {
                name: "Bell State".to_string(),
                file_path: "bell.qube".to_string(),
                circuit_diagram: "CNOT".to_string(),
                qubit_count: 2,
                gate_count: 3,
                entangled_pairs: 1,
                quantum_complexity: 7.5,
                source_hash: "hash1".to_string(),
                quantum_signature: "sig1".to_string(),
            },
        ];
        let listing = format_marketplace_listing(&nfts);
        assert!(listing.contains("Bell State"));
        assert!(listing.contains("Total circuits: 1"));
    }

    #[test]
    fn test_scan_empty_dir() {
        let tmp = std::env::temp_dir().join(format!("aeonmi_market_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).ok();
        let scanner = MarketScanner::new(tmp.clone());
        let nfts = scanner.scan_qube_files();
        assert!(nfts.is_empty());
        std::fs::remove_dir_all(&tmp).ok();
    }
}
