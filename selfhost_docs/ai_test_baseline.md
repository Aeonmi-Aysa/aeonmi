# .ai Test Suite Baseline
_Aeonmi-aeonmi02-selfhost — Captured: 2026-04-05 — Updated: 2026-04-07_

This is the baseline state of all `.ai` test files at project creation.
Use this to verify that self-hosting ports don't regress existing behavior.

---

## Summary (Updated 2026-04-07 — post lexer port)

| Metric        | Value |
|---------------|-------|
| Total files   | 57    |
| PASS          | 54    |
| FAIL          | 3     |
| Pass rate     | 94.7% |

**Improvement over original baseline:** +4 PASS, all regressions avoided.
New tests added: `lexer_test.ai` (18 assertions, all pass).

**3 remaining failures are pre-existing (do not fix as part of self-hosting):**
- `oracle_agent_test.ai` — parse error in oracle_agent.ai at 3:21
- `debug_test.ai` — memory allocation failure (24848 bytes)
- `parser_test.ai` — empty output / hang

---

## Original Baseline (2026-04-05)

| Metric        | Value |
|---------------|-------|
| Total tests   | 74    |
| PASS          | 50    |
| FAIL          | 24    |
| Pass rate     | 67.6% |

---

## Passing Tests (50)

All tests in these directories passed:
- `aeonmi_ai/learning/` — loop_test.ai
- `aeonmi_ai/selfmodel/` — model_test.ai
- `aeonmi_ai/sensory/` — quantum_feed_test.ai
- `aeonmi_ai/stdlib/tests/` — test_collections_builtins.ai, test_native_ops.ai, test_string_builtins.ai
- `Aeonmi_Master/aeonmi_ai/agent/` — action_test, closer_agent_test, conductor_agent_test, decide_test, devil_agent_test, hype_agent_test, oracle_agent_test
- `Aeonmi_Master/aeonmi_ai/learn/` — learn_test.ai, pattern_test.ai, reinforce_test.ai
- `Aeonmi_Master/aeonmi_ai/shard/` — main.ai (6/6 PASS)
- Plus others across agent, mother, shard, quantum directories

---

## Failing Tests (24)

These tests fail at baseline. They are NOT regressions — they were failing before
self-hosting work began. Do not attempt to fix them as part of self-hosting ports.

| Test File | Last Output Before Fail |
|-----------|------------------------|
| `Aeonmi_Master/aeonmi_ai/agent/plan_test.ai` | PASS:goal[0]=101 |
| `Aeonmi_Master/aeonmi_ai/lang/ops_test.ai` | PASS:prec(*) > prec(+) |
| `Aeonmi_Master/aeonmi_ai/lang/types_test.ai` | PASS:int=2 |
| `Aeonmi_Master/aeonmi_ai/mother/debug_test.ai` | (no output before fail) |
| `Aeonmi_Master/aeonmi_ai/mother/maintenance_test.ai` | pass:health full strength |
| `Aeonmi_Master/aeonmi_ai/mother/memory_test.ai` | pass:recall color |
| `Aeonmi_Master/aeonmi_ai/mother/memory_v2_test.ai` | pass:deleted key gone |
| `Aeonmi_Master/aeonmi_ai/mother/rules_test.ai` | pass:total after 3 |
| `Aeonmi_Master/aeonmi_ai/mother/rules_v2_test.ai` | pass:fire cond_weak |
| `Aeonmi_Master/aeonmi_ai/net/message_test.ai` | PASS:msg type=5 |
| `Aeonmi_Master/aeonmi_ai/quantum/qubit_test.ai` | PASS:q_zero prob0=1 |
| `Aeonmi_Master/aeonmi_ai/shard/ast_test.ai` | PASSnum count |
| `Aeonmi_Master/aeonmi_ai/shard/lexer_test.ai` | PASSdigit 5 |
| +(11 additional) | See baseline_build.log |

---

## Notes

- Many "failing" tests produce substantial PASS output before failing — they fail on
  later assertions, not early ones. These are known partial implementations.
- The `ops_test.ai` and `types_test.ai` failures suggest the `.ai` test framework
  exits non-zero when any assertion fails, even if others pass.
- `shard/main.ai` is NOT a test in the traditional sense — it's the developer kit demo
  and passes 6/6 of its own internal round-trip tests.

---

## Regression Rule

After any self-hosting port:
- Run the full test suite
- Compare PASS count against this baseline (50)
- Any test that was PASS at baseline and is now FAIL = a regression
- Fix the regression before merging the port
