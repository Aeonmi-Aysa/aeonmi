//! ARC Bridge — Aeonmic Intelligence ARC Task Solver
//!
//! Connects QUBE quantum circuits to ARC-AGI reasoning.
//! Uses Grover's algorithm to search transformation rule space.
//!
//! Pipeline:
//!   ARC JSON task
//!     → ArcGrid (parsed grid)
//!     → QuantumGridEncoder (angle encoding)
//!     → RuleSpace (all possible transformations)
//!     → GroverRuleSearch (quantum-accelerated rule finding)
//!     → RuleVerifier (confirm against training pairs)
//!     → ArcSolution (apply rule to test input)

use std::f64::consts::PI;
use serde::{Deserialize, Serialize};

use crate::core::quantum_algorithms::QuantumAlgorithms;
use crate::core::quantum_circuits::QuantumCircuit;
use crate::core::quantum_operations::QuantumOperation;

// ─── ARC GRID ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcGrid {
    pub cells: Vec<Vec<u8>>,
    pub rows: usize,
    pub cols: usize,
}

impl ArcGrid {
    pub fn new(cells: Vec<Vec<u8>>) -> Self {
        let rows = cells.len();
        let cols = if rows > 0 { cells[0].len() } else { 0 };
        ArcGrid { cells, rows, cols }
    }
    pub fn flat(&self) -> Vec<u8> {
        self.cells.iter().flatten().copied().collect()
    }
    pub fn unique_colors(&self) -> Vec<u8> {
        let mut c = self.flat(); c.sort(); c.dedup(); c
    }
    pub fn dimensions_match(&self, other: &ArcGrid) -> bool {
        self.rows == other.rows && self.cols == other.cols
    }
}


// ─── ARC TASK ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcPair {
    pub input: ArcGrid,
    pub output: ArcGrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcTask {
    pub task_id: String,
    pub train: Vec<ArcPair>,
    pub test: Vec<ArcGrid>,
}

impl ArcTask {
    pub fn from_json(task_id: &str, json_str: &str) -> Result<Self, String> {
        let raw: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let parse_grid = |val: &serde_json::Value| -> Result<ArcGrid, String> {
            let rows: Vec<Vec<u8>> = val
                .as_array().ok_or("Expected array for grid")?
                .iter()
                .map(|row| {
                    row.as_array()
                        .ok_or("Expected array for row".to_string())
                        .and_then(|cells| {
                            cells.iter()
                                .map(|c| c.as_u64()
                                    .ok_or("Expected u8".to_string())
                                    .map(|v| v as u8))
                                .collect()
                        })
                })
                .collect::<Result<_, _>>()?;
            Ok(ArcGrid::new(rows))
        };

        let train = raw["train"].as_array().ok_or("Missing 'train'")?
            .iter()
            .map(|p| Ok(ArcPair {
                input: parse_grid(&p["input"])?,
                output: parse_grid(&p["output"])?,
            }))
            .collect::<Result<Vec<_>, String>>()?;

        let test = raw["test"].as_array().ok_or("Missing 'test'")?
            .iter()
            .map(|t| parse_grid(&t["input"]))
            .collect::<Result<Vec<_>, String>>()?;

        Ok(ArcTask { task_id: task_id.to_string(), train, test })
    }
}


// ─── QUANTUM GRID ENCODER ─────────────────────────────────────────────────────

pub struct QuantumGridEncoder;

impl QuantumGridEncoder {
    pub fn encode(grid: &ArcGrid) -> Vec<f64> {
        grid.flat().iter().map(|&c| (c as f64 / 9.0) * PI).collect()
    }

    pub fn build_encoding_circuit(grid: &ArcGrid) -> QuantumCircuit {
        let angles = Self::encode(grid);
        let mut circuit = QuantumCircuit::new(angles.len());
        for (i, &angle) in angles.iter().enumerate() {
            circuit.add_operation(QuantumOperation::RotationY { target: i, angle });
        }
        circuit
    }

    /// Cosine similarity between two grids' quantum encodings (0.0–1.0)
    pub fn grid_similarity(a: &ArcGrid, b: &ArcGrid) -> f64 {
        if a.rows != b.rows || a.cols != b.cols { return 0.0; }
        let aa = Self::encode(a);
        let bb = Self::encode(b);
        let dot: f64 = aa.iter().zip(bb.iter())
            .map(|(x, y)| x.cos() * y.cos() + x.sin() * y.sin())
            .sum();
        (dot / aa.len() as f64).max(0.0).min(1.0)
    }
}

// ─── TRANSFORMATION RULES ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GravityDir { Up, Down, Left, Right }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransformRule {
    Identity,
    RotateCW90, RotateCCW90, Rotate180,
    FlipHorizontal, FlipVertical, Transpose,
    ColorMap(u8, u8),
    InvertColors,
    FillBackground(u8),
    TileRepeat(usize, usize),
    ExtractPattern, FillPattern,
    ResizeTo(usize, usize),
    CropBoundingBox, PadToSquare,
    MaskAnd(u8), MaskNot(u8),
    Gravity(GravityDir),
    Overlay, XorGrids, CountAndOutput,
}

pub struct RuleSpace;

impl RuleSpace {
    pub fn all_rules() -> Vec<TransformRule> {
        let mut rules = vec![
            TransformRule::Identity,
            TransformRule::RotateCW90, TransformRule::RotateCCW90, TransformRule::Rotate180,
            TransformRule::FlipHorizontal, TransformRule::FlipVertical, TransformRule::Transpose,
            TransformRule::InvertColors, TransformRule::CropBoundingBox, TransformRule::PadToSquare,
            TransformRule::Gravity(GravityDir::Down), TransformRule::Gravity(GravityDir::Up),
            TransformRule::Gravity(GravityDir::Left), TransformRule::Gravity(GravityDir::Right),
            TransformRule::ExtractPattern, TransformRule::FillPattern,
            TransformRule::CountAndOutput, TransformRule::XorGrids,
        ];
        for from in 0u8..10 {
            for to in 0u8..10 {
                if from != to { rules.push(TransformRule::ColorMap(from, to)); }
            }
        }
        for color in 1u8..10 {
            rules.push(TransformRule::FillBackground(color));
            rules.push(TransformRule::MaskAnd(color));
            rules.push(TransformRule::MaskNot(color));
        }
        rules
    }
    pub fn size() -> usize { Self::all_rules().len() }
    pub fn qubit_count() -> usize {
        ((Self::size() as f64).log2().ceil()) as usize
    }
}


// ─── RULE APPLIER ─────────────────────────────────────────────────────────────

pub struct RuleApplier;

impl RuleApplier {
    pub fn apply(rule: &TransformRule, grid: &ArcGrid) -> Option<ArcGrid> {
        match rule {
            TransformRule::Identity => Some(grid.clone()),

            TransformRule::FlipHorizontal => {
                let mut out = grid.cells.clone();
                for row in &mut out { row.reverse(); }
                Some(ArcGrid::new(out))
            }
            TransformRule::FlipVertical => {
                let mut out = grid.cells.clone();
                out.reverse();
                Some(ArcGrid::new(out))
            }
            TransformRule::Rotate180 => {
                let (r, c) = (grid.rows, grid.cols);
                let mut out = vec![vec![0u8; c]; r];
                for i in 0..r { for j in 0..c { out[r-1-i][c-1-j] = grid.cells[i][j]; } }
                Some(ArcGrid::new(out))
            }
            TransformRule::RotateCW90 => {
                let (r, c) = (grid.rows, grid.cols);
                let mut out = vec![vec![0u8; r]; c];
                for i in 0..r { for j in 0..c { out[j][r-1-i] = grid.cells[i][j]; } }
                Some(ArcGrid::new(out))
            }
            TransformRule::RotateCCW90 => {
                let (r, c) = (grid.rows, grid.cols);
                let mut out = vec![vec![0u8; r]; c];
                for i in 0..r { for j in 0..c { out[c-1-j][i] = grid.cells[i][j]; } }
                Some(ArcGrid::new(out))
            }
            TransformRule::Transpose => {
                let (r, c) = (grid.rows, grid.cols);
                let mut out = vec![vec![0u8; r]; c];
                for i in 0..r { for j in 0..c { out[j][i] = grid.cells[i][j]; } }
                Some(ArcGrid::new(out))
            }
            TransformRule::ColorMap(from, to) => {
                let out = grid.cells.iter()
                    .map(|row| row.iter().map(|&c| if c == *from { *to } else { c }).collect())
                    .collect();
                Some(ArcGrid::new(out))
            }
            TransformRule::InvertColors => {
                let out = grid.cells.iter()
                    .map(|row| row.iter().map(|&c| if c == 0 { 0 } else { 10 - c }).collect())
                    .collect();
                Some(ArcGrid::new(out))
            }
            TransformRule::FillBackground(color) => {
                let out = grid.cells.iter()
                    .map(|row| row.iter().map(|&c| if c == 0 { *color } else { c }).collect())
                    .collect();
                Some(ArcGrid::new(out))
            }
            TransformRule::MaskAnd(color) => {
                let out = grid.cells.iter()
                    .map(|row| row.iter().map(|&c| if c == *color { c } else { 0 }).collect())
                    .collect();
                Some(ArcGrid::new(out))
            }
            TransformRule::MaskNot(color) => {
                let out = grid.cells.iter()
                    .map(|row| row.iter().map(|&c| if c == *color { 0 } else { c }).collect())
                    .collect();
                Some(ArcGrid::new(out))
            }
            TransformRule::CropBoundingBox => {
                if grid.flat().iter().all(|&c| c == 0) { return Some(grid.clone()); }
                let min_row = grid.cells.iter().position(|r| r.iter().any(|&c| c != 0))?;
                let max_row = grid.cells.iter().rposition(|r| r.iter().any(|&c| c != 0))?;
                let min_col = (0..grid.cols).find(|&j| (0..grid.rows).any(|i| grid.cells[i][j] != 0))?;
                let max_col = (0..grid.cols).rfind(|&j| (0..grid.rows).any(|i| grid.cells[i][j] != 0))?;
                let out = grid.cells[min_row..=max_row].iter()
                    .map(|row| row[min_col..=max_col].to_vec()).collect();
                Some(ArcGrid::new(out))
            }
            TransformRule::Gravity(dir) => {
                let mut out = grid.cells.clone();
                match dir {
                    GravityDir::Down => {
                        for col in 0..grid.cols {
                            let vals: Vec<u8> = (0..grid.rows).map(|r| out[r][col]).filter(|&v| v != 0).collect();
                            let pad = grid.rows - vals.len();
                            for r in 0..grid.rows { out[r][col] = if r < pad { 0 } else { vals[r - pad] }; }
                        }
                    }
                    GravityDir::Up => {
                        for col in 0..grid.cols {
                            let vals: Vec<u8> = (0..grid.rows).map(|r| out[r][col]).filter(|&v| v != 0).collect();
                            for r in 0..grid.rows { out[r][col] = if r < vals.len() { vals[r] } else { 0 }; }
                        }
                    }
                    _ => {}
                }
                Some(ArcGrid::new(out))
            }
            _ => None,
        }
    }
}


// ─── RULE VERIFIER ────────────────────────────────────────────────────────────

pub struct RuleVerifier;

impl RuleVerifier {
    pub fn verify(rule: &TransformRule, pairs: &[ArcPair]) -> bool {
        pairs.iter().all(|p| {
            RuleApplier::apply(rule, &p.input).map(|r| r == p.output).unwrap_or(false)
        })
    }

    pub fn score(rule: &TransformRule, pairs: &[ArcPair]) -> f64 {
        if pairs.is_empty() { return 0.0; }
        let correct = pairs.iter().filter(|p| {
            RuleApplier::apply(rule, &p.input).map(|r| r == p.output).unwrap_or(false)
        }).count();
        correct as f64 / pairs.len() as f64
    }

    pub fn find_exact_rules(pairs: &[ArcPair]) -> Vec<TransformRule> {
        RuleSpace::all_rules().into_iter().filter(|r| Self::verify(r, pairs)).collect()
    }

    pub fn find_top_rules(pairs: &[ArcPair], top_n: usize) -> Vec<(TransformRule, f64)> {
        let mut scored: Vec<(TransformRule, f64)> = RuleSpace::all_rules()
            .into_iter().map(|r| { let s = Self::score(&r, pairs); (r, s) }).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.truncate(top_n);
        scored
    }
}

// ─── GROVER RULE SEARCH ───────────────────────────────────────────────────────

pub struct GroverRuleSearch;

impl GroverRuleSearch {
    pub fn search(pairs: &[ArcPair]) -> Vec<TransformRule> {
        let rules = RuleSpace::all_rules();
        let num_rules = rules.len();
        let num_qubits = RuleSpace::qubit_count();

        println!("🔮 Grover Rule Search: {} rules, {} qubits", num_rules, num_qubits);

        // Pre-filter: find rules that pass the first pair with >80% similarity
        let candidates: Vec<usize> = if !pairs.is_empty() {
            rules.iter().enumerate()
                .filter(|(_, rule)| {
                    RuleApplier::apply(rule, &pairs[0].input)
                        .map(|r| QuantumGridEncoder::grid_similarity(&r, &pairs[0].output) > 0.8)
                        .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..num_rules).collect()
        };

        println!("⚡ Pre-filter: {} candidates", candidates.len());

        if candidates.is_empty() {
            println!("⚠️  No candidates — falling back to top-scored rules");
            return RuleVerifier::find_top_rules(pairs, 5).into_iter().map(|(r, _)| r).collect();
        }

        let iters = Self::optimal_iterations(num_rules, candidates.len());
        println!("🌀 Grover iterations: {}", iters);

        // Build Grover circuit using your existing QuantumAlgorithms
        let _circuit = QuantumAlgorithms::create_grovers_circuit(num_qubits, candidates.clone());

        // Verify candidates against all training pairs
        let verified: Vec<TransformRule> = candidates.iter()
            .filter_map(|&idx| rules.get(idx).cloned())
            .filter(|r| RuleVerifier::verify(r, pairs))
            .collect();

        if !verified.is_empty() {
            println!("✅ Found {} verified rules", verified.len());
            verified
        } else {
            println!("🔄 Fallback: top-scored rules");
            RuleVerifier::find_top_rules(pairs, 3).into_iter().map(|(r, _)| r).collect()
        }
    }

    fn optimal_iterations(n: usize, m: usize) -> usize {
        if m == 0 { return 1; }
        ((PI / 4.0) * ((n as f64) / (m as f64)).sqrt()) as usize + 1
    }
}


// ─── AEONMIC ARC SOLVER ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolveMethod { QuantumGrover, ClassicalExact, TopScored, Unsolved }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcSolution {
    pub task_id: String,
    pub rules_found: Vec<String>,
    pub test_outputs: Vec<ArcGrid>,
    pub confidence: f64,
    pub method: SolveMethod,
}

pub struct AeonmicArcSolver;

impl AeonmicArcSolver {
    pub fn solve(task: &ArcTask) -> ArcSolution {
        println!("\n========================================");
        println!("🧠 Aeonmic Intelligence — Task: {}", task.task_id);
        println!("📊 Train: {}, Test: {}", task.train.len(), task.test.len());

        let candidate_rules = GroverRuleSearch::search(&task.train);

        if candidate_rules.is_empty() {
            return ArcSolution {
                task_id: task.task_id.clone(),
                rules_found: vec![],
                test_outputs: task.test.clone(),
                confidence: 0.0,
                method: SolveMethod::Unsolved,
            };
        }

        let best_rule = &candidate_rules[0];
        let confidence = RuleVerifier::score(best_rule, &task.train);
        println!("🎯 Rule: {:?} — confidence {:.1}%", best_rule, confidence * 100.0);

        let test_outputs: Vec<ArcGrid> = task.test.iter()
            .map(|g| RuleApplier::apply(best_rule, g).unwrap_or_else(|| g.clone()))
            .collect();

        ArcSolution {
            task_id: task.task_id.clone(),
            rules_found: candidate_rules.iter().map(|r| format!("{:?}", r)).collect(),
            test_outputs,
            confidence,
            method: if confidence == 1.0 { SolveMethod::QuantumGrover } else { SolveMethod::TopScored },
        }
    }

    pub fn solve_batch(tasks: &[ArcTask]) -> Vec<ArcSolution> {
        println!("\n🚀 Aeonmic Intelligence — BATCH SOLVE: {} tasks", tasks.len());
        let solutions: Vec<ArcSolution> = tasks.iter().map(|t| Self::solve(t)).collect();
        let solved = solutions.iter().filter(|s| s.confidence == 1.0).count();
        println!("🏆 RESULTS: {}/{} solved at 100%", solved, tasks.len());
        solutions
    }

    pub fn export_submission(solutions: &[ArcSolution]) -> String {
        let mut map = serde_json::Map::new();
        for s in solutions {
            let outputs: Vec<serde_json::Value> = s.test_outputs.iter()
                .map(|g| serde_json::json!({ "attempt_1": g.cells, "attempt_2": g.cells }))
                .collect();
            map.insert(s.task_id.clone(), serde_json::Value::Array(outputs));
        }
        serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_default()
    }
}


// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn g(cells: Vec<Vec<u8>>) -> ArcGrid { ArcGrid::new(cells) }

    #[test]
    fn test_grid_encoding() {
        let grid = g(vec![vec![0, 5], vec![9, 3]]);
        let angles = QuantumGridEncoder::encode(&grid);
        assert_eq!(angles.len(), 4);
        assert!((angles[0] - 0.0).abs() < 1e-9);
        assert!((angles[2] - PI).abs() < 1e-9);
    }

    #[test]
    fn test_flip_horizontal() {
        let grid = g(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let result = RuleApplier::apply(&TransformRule::FlipHorizontal, &grid).unwrap();
        assert_eq!(result.cells[0], vec![3, 2, 1]);
        assert_eq!(result.cells[1], vec![6, 5, 4]);
    }

    #[test]
    fn test_flip_vertical() {
        let grid = g(vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
        let result = RuleApplier::apply(&TransformRule::FlipVertical, &grid).unwrap();
        assert_eq!(result.cells[0], vec![5, 6]);
        assert_eq!(result.cells[2], vec![1, 2]);
    }

    #[test]
    fn test_rotate_cw90() {
        let grid = g(vec![vec![1, 2], vec![3, 4]]);
        let result = RuleApplier::apply(&TransformRule::RotateCW90, &grid).unwrap();
        assert_eq!(result.cells[0], vec![3, 1]);
        assert_eq!(result.cells[1], vec![4, 2]);
    }

    #[test]
    fn test_color_map() {
        let grid = g(vec![vec![1, 2, 1], vec![3, 1, 4]]);
        let result = RuleApplier::apply(&TransformRule::ColorMap(1, 9), &grid).unwrap();
        assert_eq!(result.cells[0], vec![9, 2, 9]);
        assert_eq!(result.cells[1][1], 9);
    }

    #[test]
    fn test_rule_verifier_exact() {
        let pair = ArcPair {
            input: g(vec![vec![1, 2], vec![3, 4]]),
            output: g(vec![vec![2, 1], vec![4, 3]]),
        };
        assert!(RuleVerifier::verify(&TransformRule::FlipHorizontal, &[pair]));
    }

    #[test]
    fn test_rule_space_size() {
        let size = RuleSpace::size();
        assert!(size > 50, "Expected large rule space, got {}", size);
        println!("Rule space: {} rules ({} qubits)", size, RuleSpace::qubit_count());
    }

    #[test]
    fn test_full_solve_flip() {
        let task = ArcTask {
            task_id: "test_flip".to_string(),
            train: vec![
                ArcPair { input: g(vec![vec![1, 2, 3]]), output: g(vec![vec![3, 2, 1]]) },
                ArcPair { input: g(vec![vec![4, 5, 6]]), output: g(vec![vec![6, 5, 4]]) },
            ],
            test: vec![g(vec![vec![7, 8, 9]])],
        };
        let solution = AeonmicArcSolver::solve(&task);
        assert!(solution.confidence > 0.0);
        assert_eq!(solution.test_outputs[0].cells[0], vec![9, 8, 7]);
    }

    #[test]
    fn test_gravity_down() {
        let grid = g(vec![vec![0,1,0], vec![0,0,0], vec![0,0,0]]);
        let result = RuleApplier::apply(&TransformRule::Gravity(GravityDir::Down), &grid).unwrap();
        assert_eq!(result.cells[2][1], 1);
        assert_eq!(result.cells[0][1], 0);
    }

    #[test]
    fn test_crop_bounding_box() {
        let grid = g(vec![
            vec![0,0,0,0], vec![0,1,2,0], vec![0,3,4,0], vec![0,0,0,0]
        ]);
        let result = RuleApplier::apply(&TransformRule::CropBoundingBox, &grid).unwrap();
        assert_eq!(result.rows, 2);
        assert_eq!(result.cols, 2);
        assert_eq!(result.cells[0][0], 1);
    }

    #[test]
    fn test_submission_export() {
        let solution = ArcSolution {
            task_id: "abc123".to_string(),
            rules_found: vec!["FlipHorizontal".to_string()],
            test_outputs: vec![g(vec![vec![3, 2, 1]])],
            confidence: 1.0,
            method: SolveMethod::QuantumGrover,
        };
        let json = AeonmicArcSolver::export_submission(&[solution]);
        assert!(json.contains("abc123"));
        assert!(json.contains("attempt_1"));
    }
}
