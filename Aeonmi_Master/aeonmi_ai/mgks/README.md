# MGKS — Mother Glyph Knowledge System
### Aeonmi Language Project

---

## File Structure

```
src/mgks/
├── types.ai       — All core type definitions (GlyphId, MemoryTier, QuantumGlyphState, etc.)
├── glyph.ai       — Core Glyph struct + all glyph operations (create, decay, collapse, entangle)
├── genesis.ai     — Genesis Array templates (reusable glyph schemas)
├── memory.ai      — T1/T2/T3/Archive memory tier management + MemoryManager
├── retrieval.ai   — Hybrid retrieval engine (semantic + graph + temporal fusion)
├── hive.ai        — Multi-agent hive topology + Mother AI consensus interface
└── mgks.ai        — Main entry point + QUBE syntax interface

examples/
└── mgks_demo.ai   — Full usage demo showing all five agents + Mother AI
```

---

## Integration into Aeonmi Project

1. Copy the `mgks/` folder to:
   ```
   C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\src\mgks\
   ```

2. Copy `mgks_demo.ai` to:
   ```
   C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\examples\mgks_demo.ai
   ```

3. Run the demo:
   ```powershell
   cd "C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01"
   aeonmi run examples/mgks_demo.ai --native
   ```

---

## QUBE Syntax Keywords (from mgks.ai)

| Keyword     | Purpose                                      | Example |
|-------------|----------------------------------------------|---------|
| `glyph`     | Create a new knowledge glyph                 | `glyph signal : market_signal { ... }` |
| `bind`      | Create a typed edge between two glyphs       | `bind a CAUSES b weight=0.9` |
| `recall`    | Query memory with hybrid retrieval           | `recall { intent: "...", top_k: 5 }` |
| `collapse`  | Collapse a superposition to a single truth   | `collapse signal observed_by: MotherAI` |
| `entangle`  | Quantum-correlate two glyphs                 | `entangle a <~> b correlation=0.85` |

---

## Memory Tiers

| Tier       | Storage              | Capacity     | Latency | Decay         |
|------------|----------------------|--------------|---------|---------------|
| T1 Working | In-process (Rust)    | 512 glyphs   | <1ms    | Session-clear |
| T2 Episodic| sled / RocksDB       | 100k glyphs  | <10ms   | 7-day half-life |
| T3 Semantic| PostgreSQL + pgvector| Unbounded    | <100ms  | 90-day half-life |
| Archive    | Append-only disk     | Unbounded    | N/A     | Permanent     |

---

## Agent Shards

| Agent           | Constant           | Memory Focus                   |
|-----------------|--------------------|--------------------------------|
| Oracle          | AGENT_ORACLE       | Market signals, trends         |
| Hype Machine    | AGENT_HYPE         | Audience, viral patterns       |
| Closer          | AGENT_CLOSER       | Conversion, deal context       |
| Devil's Advocate| AGENT_DEVIL        | Risks, contradictions          |
| Conductor       | AGENT_CONDUCTOR    | Task state, orchestration      |
| Mother AI       | AGENT_MOTHER       | Consensus graph (T3 only)      |

---

## Quantum State Mapping

| QUBE Concept    | MGKS Operation                                    |
|-----------------|---------------------------------------------------|
| Superposition   | Glyph holds multiple truths with amplitudes       |
| Measurement     | `collapse` — Mother AI observes a single truth    |
| Entanglement    | `entangle` — two glyphs' truths are correlated    |
| Bell State      | Both qubits entangled — used in QUBE circuit demo |

---

Built for: Aeonmi Language Project
White Paper: MGKS v1.0 — April 2026
Author: Warren Williams + Aeonmic Intelligence
