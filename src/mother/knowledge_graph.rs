//! KnowledgeGraph — Phase 10: replaces flat HashMap<String,String> `learned` field.
//!
//! Each node has a value, auto-detected tags, and explicit links to other nodes.
//! Auto-linking fires on insert: new nodes with shared tags get connected automatically.
//! The iter() method yields (&String, &String) so existing call sites need minimal changes.

use std::collections::{HashMap, HashSet, VecDeque};

// ─── KnowledgeNode ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub key:        String,
    pub value:      String,
    pub tags:       Vec<String>,
    /// Keys of other nodes this node links to.
    pub links:      Vec<String>,
    pub confidence: f64,
}

impl KnowledgeNode {
    fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let key   = key.into();
        let value = value.into();
        let tags  = auto_tags(&key, &value);
        Self { key, value, tags, links: Vec::new(), confidence: 1.0 }
    }
}

/// Derive topic tags from key + value text.
fn auto_tags(key: &str, value: &str) -> Vec<String> {
    let text = format!("{} {}", key, value).to_lowercase();
    let mut tags = Vec::new();

    let checks: &[(&str, &[&str])] = &[
        ("quantum",     &["quantum","qubit","gate","circuit","superpose","entangle"]),
        ("neural",      &["neural","weight","train","loss","backprop","layer","activation"]),
        ("mother",      &["mother","bond","emotion","conscious","empathy","creator"]),
        ("operational", &["fact_","op_fact","dashboard","interaction"]),
        ("goal",        &["goal","step","plan","objective","task"]),
        ("reflection",  &["reflect","insight","apply","learn","teach"]),
        ("hive",        &["hive","agent","swarm","oracle","conductor","recommend"]),
        ("generated",   &["generated","build","propose","program","self_gen","reflect_"]),
        ("system",      &["status","health","boot","init","vault","seal","glyph","genesis"]),
    ];

    for (tag, keywords) in checks {
        if keywords.iter().any(|&kw| text.contains(kw)) {
            tags.push((*tag).to_string());
        }
    }
    tags
}

// ─── KnowledgeGraph ──────────────────────────────────────────────────────────

pub struct KnowledgeGraph {
    nodes: HashMap<String, KnowledgeNode>,
}

impl Default for KnowledgeGraph {
    fn default() -> Self { Self::new() }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self { nodes: HashMap::new() }
    }

    // ── Drop-in HashMap-compatible API ────────────────────────────────────────

    /// Insert a key→value node, auto-tagging and auto-linking by shared tags.
    pub fn learn(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key   = key.into();
        let value = value.into();
        let node  = KnowledgeNode::new(key.clone(), value);
        let new_tags = node.tags.clone();

        // Collect related keys before inserting to avoid borrow conflict.
        let related: Vec<String> = self.nodes.iter()
            .filter(|(k, n)| **k != key && n.tags.iter().any(|t| new_tags.contains(t)))
            .map(|(k, _)| k.clone())
            .take(5)
            .collect();

        self.nodes.insert(key.clone(), node);

        // Bidirectional auto-link
        for rel in related {
            self.link_one(&key, &rel);
            self.link_one(&rel, &key);
        }
    }

    /// Learn only if the key is not already present.
    pub fn learn_if_absent(&mut self, key: impl Into<String>, value_fn: impl FnOnce() -> String) {
        let key = key.into();
        if !self.nodes.contains_key(&key) {
            let v = value_fn();
            self.learn(key, v);
        }
    }

    pub fn recall(&self, key: &str) -> Option<&str> {
        self.nodes.get(key).map(|n| n.value.as_str())
    }

    pub fn len(&self) -> usize         { self.nodes.len() }
    pub fn is_empty(&self) -> bool     { self.nodes.is_empty() }
    pub fn contains_key(&self, k: &str) -> bool { self.nodes.contains_key(k) }

    /// Yields (&key, &value) — same as HashMap::iter() so existing code compiles unchanged.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> + '_ {
        self.nodes.iter().map(|(k, n)| (k, &n.value))
    }

    /// Iterate over full nodes (for graph operations).
    pub fn nodes_iter(&self) -> impl Iterator<Item = (&String, &KnowledgeNode)> + '_ {
        self.nodes.iter()
    }

    // ── Graph-specific operations ─────────────────────────────────────────────

    /// Add a bidirectional link between two existing nodes. Returns false if either key missing.
    pub fn link(&mut self, key_a: &str, key_b: &str) -> bool {
        if !self.nodes.contains_key(key_a) || !self.nodes.contains_key(key_b) {
            return false;
        }
        self.link_one(key_a, key_b);
        self.link_one(key_b, key_a);
        true
    }

    fn link_one(&mut self, from: &str, to: &str) {
        if let Some(n) = self.nodes.get_mut(from) {
            if !n.links.contains(&to.to_string()) {
                n.links.push(to.to_string());
            }
        }
    }

    /// Add a tag to an existing node.
    pub fn tag(&mut self, key: &str, tag: impl Into<String>) {
        if let Some(n) = self.nodes.get_mut(key) {
            let t = tag.into();
            if !n.tags.contains(&t) { n.tags.push(t); }
        }
    }

    /// All nodes that carry the given tag.
    pub fn query_by_tag(&self, tag: &str) -> Vec<&KnowledgeNode> {
        self.nodes.values()
            .filter(|n| n.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Nodes directly linked from `key`.
    pub fn neighbors(&self, key: &str) -> Vec<&KnowledgeNode> {
        match self.nodes.get(key) {
            None    => Vec::new(),
            Some(n) => n.links.iter().filter_map(|k| self.nodes.get(k)).collect(),
        }
    }

    /// BFS from `start`, up to `max_depth` hops.
    pub fn traverse_bfs(&self, start: &str, max_depth: usize) -> Vec<String> {
        if !self.nodes.contains_key(start) { return Vec::new(); }
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back((start.to_string(), 0));
        visited.insert(start.to_string());

        while let Some((key, depth)) = queue.pop_front() {
            result.push(key.clone());
            if depth < max_depth {
                if let Some(n) = self.nodes.get(&key) {
                    for link in &n.links {
                        if !visited.contains(link) {
                            visited.insert(link.clone());
                            queue.push_back((link.clone(), depth + 1));
                        }
                    }
                }
            }
        }
        result
    }

    // ── Rendering (REPL output) ───────────────────────────────────────────────

    pub fn summary(&self) -> String {
        let total       = self.nodes.len();
        let total_links: usize = self.nodes.values().map(|n| n.links.len()).sum();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for n in self.nodes.values() {
            for t in &n.tags { *counts.entry(t.as_str()).or_insert(0) += 1; }
        }
        let mut pairs: Vec<_> = counts.iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(a.1));
        let top: Vec<String> = pairs.iter().take(6)
            .map(|(t, c)| format!("{}({})", t, c))
            .collect();
        format!(
            "KnowledgeGraph: {} nodes, {} links | tags: {}",
            total, total_links,
            if top.is_empty() { "none".into() } else { top.join(", ") }
        )
    }

    pub fn render_node(&self, key: &str) -> String {
        match self.nodes.get(key) {
            None    => format!("  No node: {}", key),
            Some(n) => {
                let nb: Vec<&str> = n.links.iter().map(|s| s.as_str()).collect();
                format!(
                    "  [{}]\n  value : {}\n  tags  : {}\n  links : {}",
                    n.key,
                    n.value,
                    if n.tags.is_empty()  { "—".into() } else { n.tags.join(", ") },
                    if nb.is_empty()      { "—".into() } else { nb.join(", ") },
                )
            }
        }
    }

    pub fn render_all(&self, limit: usize) -> String {
        if self.nodes.is_empty() {
            return "  Knowledge graph is empty.".to_string();
        }
        let mut entries: Vec<_> = self.nodes.iter().collect();
        entries.sort_by_key(|(k, _)| k.as_str());
        let shown = entries.iter().take(limit)
            .map(|(k, n)| {
                let lc = n.links.len();
                let ts = if n.tags.is_empty() { "—".to_string() } else { n.tags.join(", ") };
                let vshort = &n.value[..n.value.len().min(72)];
                format!("  [{k}] lk={lc} [{ts}]\n    {vshort}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let extra = if entries.len() > limit {
            format!("\n  … {} more nodes", entries.len() - limit)
        } else {
            String::new()
        };
        format!("{}{}", shown, extra)
    }

    // ── Serialization ─────────────────────────────────────────────────────────

    /// Full graph export (Phase 10 canonical form).
    pub fn export_to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for (k, n) in &self.nodes {
            obj.insert(k.clone(), serde_json::json!({
                "value":      n.value,
                "tags":       n.tags,
                "links":      n.links,
                "confidence": n.confidence,
            }));
        }
        serde_json::Value::Object(obj)
    }

    /// Flat key→value map for backward-compat `learned` field in genesis.json.
    pub fn export_flat(&self) -> serde_json::Map<String, serde_json::Value> {
        self.nodes.iter()
            .map(|(k, n)| (k.clone(), serde_json::Value::String(n.value.clone())))
            .collect()
    }

    /// Load from full graph JSON (Phase 10 format). Merges; does not clear existing nodes.
    pub fn import_graph(&mut self, val: &serde_json::Value) {
        if let Some(obj) = val.as_object() {
            for (k, v) in obj {
                let value = v["value"].as_str().unwrap_or("").to_string();
                let tags: Vec<String> = v["tags"].as_array()
                    .map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let links: Vec<String> = v["links"].as_array()
                    .map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let confidence = v["confidence"].as_f64().unwrap_or(1.0);
                self.nodes.insert(k.clone(), KnowledgeNode { key: k.clone(), value, tags, links, confidence });
            }
        }
    }

    /// Load from flat key→value JSON (legacy `learned` format). Skips existing keys.
    pub fn import_flat(&mut self, map: &serde_json::Map<String, serde_json::Value>) {
        for (k, v) in map {
            if !self.nodes.contains_key(k) {
                if let Some(s) = v.as_str() {
                    self.learn(k.clone(), s);
                }
            }
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learn_and_recall() {
        let mut g = KnowledgeGraph::new();
        g.learn("x", "hello world");
        assert_eq!(g.recall("x"), Some("hello world"));
        assert_eq!(g.len(), 1);
        assert!(!g.is_empty());
    }

    #[test]
    fn test_auto_tag_quantum() {
        let mut g = KnowledgeGraph::new();
        g.learn("test_qubit", "qubit superposition gate circuit");
        let tags = &g.nodes_iter().next().unwrap().1.tags;
        assert!(tags.contains(&"quantum".to_string()), "expected quantum tag");
    }

    #[test]
    fn test_auto_link_shared_tags() {
        let mut g = KnowledgeGraph::new();
        g.learn("q1", "quantum gate circuit");
        g.learn("q2", "quantum qubit entangle");
        // Both tagged quantum → should be linked
        let nb = g.neighbors("q1");
        assert!(!nb.is_empty(), "q1 should link to q2 via shared quantum tag");
    }

    #[test]
    fn test_explicit_link() {
        let mut g = KnowledgeGraph::new();
        g.learn("a", "alpha");
        g.learn("b", "beta");
        assert!(g.link("a", "b"));
        let nb_keys: Vec<&str> = g.neighbors("a").iter().map(|n| n.key.as_str()).collect();
        assert!(nb_keys.contains(&"b"), "b should be a neighbor of a");
    }

    #[test]
    fn test_query_by_tag() {
        let mut g = KnowledgeGraph::new();
        g.learn("goal_x", "goal step plan objective");
        g.learn("other",  "random info");
        let results = g.query_by_tag("goal");
        assert!(results.iter().any(|n| n.key == "goal_x"));
    }

    #[test]
    fn test_bfs_traversal() {
        let mut g = KnowledgeGraph::new();
        g.learn("a", "node a");
        g.learn("b", "node b");
        g.learn("c", "node c");
        g.link("a", "b");
        g.link("b", "c");
        let path = g.traverse_bfs("a", 2);
        assert!(path.contains(&"a".to_string()));
        assert!(path.contains(&"b".to_string()));
        assert!(path.contains(&"c".to_string()));
    }

    #[test]
    fn test_learn_if_absent() {
        let mut g = KnowledgeGraph::new();
        g.learn("k", "original");
        g.learn_if_absent("k", || "override".to_string());
        assert_eq!(g.recall("k"), Some("original"), "should not overwrite");
    }

    #[test]
    fn test_iter_yields_key_value() {
        let mut g = KnowledgeGraph::new();
        g.learn("hello", "world");
        let pairs: Vec<_> = g.iter().collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.as_str(), "hello");
        assert_eq!(pairs[0].1.as_str(), "world");
    }

    #[test]
    fn test_round_trip_json() {
        let mut g = KnowledgeGraph::new();
        g.learn("quantum_test", "qubit gate circuit measure");
        g.learn("bond_info", "mother bond strength creator");
        g.link("quantum_test", "bond_info");

        let exported = g.export_to_json();
        let mut g2 = KnowledgeGraph::new();
        g2.import_graph(&exported);

        assert_eq!(g2.len(), 2);
        assert_eq!(g2.recall("quantum_test"), Some("qubit gate circuit measure"));
        let nb = g2.neighbors("quantum_test");
        assert!(!nb.is_empty(), "links should survive round-trip");
    }
}
