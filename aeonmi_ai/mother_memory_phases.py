"""
mother_memory_phases.py - Full 4-Phase Memory System for Mother
Phase 1: Episodic (journal) - Append-only event log
Phase 2: Candidate - Single-use learnings
Phase 3: Provisional - Proven rules (3+ uses)
Phase 4: Anchored - Consolidated principles (10+ contexts)

Consolidation cycle: [5,5,5,10,5,5,5,10] sessions
Decay, merge, strengthen, and compress rules via symbolic binding.
"""

import struct
import json
import os
from datetime import datetime
from pathlib import Path
import hashlib

class MotherMemoryPhases:
    """
    Full 4-phase memory system with active consolidation.
    Phase transitions: Episodic → Candidate → Provisional → Anchored
    Consolidation: merge duplicates, decay stale, strengthen reliable
    """

    MEMORY_ROOT = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Mother")

    # Phase storage paths
    JOURNAL_PATH = MEMORY_ROOT / "journal" / "events.log"
    CANDIDATES_PATH = MEMORY_ROOT / "candidates.bin"
    PROVISIONAL_PATH = MEMORY_ROOT / "provisional.bin"
    ANCHORED_PATH = MEMORY_ROOT / "anchored.bin"

    # Metadata
    DECAY_META_PATH = MEMORY_ROOT / "decay.meta"
    CONSOLIDATION_LOG = MEMORY_ROOT / "consolidation.log"
    SESSION_STATE = MEMORY_ROOT / "session_state.json"

    # Consolidation cycle (sessions per phase before merge)
    CONSOLIDATION_CYCLE = [5, 5, 5, 10, 5, 5, 5, 10]

    # Promotion thresholds
    CANDIDATE_TO_PROVISIONAL = {"min_uses": 3, "min_success": 2, "min_rate": 0.5}
    PROVISIONAL_TO_ANCHORED = {"min_uses": 10, "min_success": 8, "min_rate": 0.8}

    # Decay parameters
    DECAY_THRESHOLD_AGE = 50
    DECAY_THRESHOLD_RATE = 0.3  # Success rate below 30%

    def __init__(self):
        self.memory_root = self.MEMORY_ROOT
        self.memory_root.mkdir(parents=True, exist_ok=True)
        (self.memory_root / "journal").mkdir(exist_ok=True)

        # Load all phases
        self.journal = []
        self.candidates = {}
        self.provisional = {}
        self.anchored = {}
        self.decay_log = []

        # Session tracking
        self.session_num = 0
        self.cycle_position = 0
        self.consolidation_due = False

        self._load_all()
        self._determine_cycle_position()

    def _determine_cycle_position(self):
        """Determine position in consolidation cycle from session state."""
        state = self._load_session_state()
        if state:
            self.session_num = state.get("session_num", 0)
            self.cycle_position = self.session_num % len(self.CONSOLIDATION_CYCLE)
            # Consolidation due at end of phase
            phase_end = sum(self.CONSOLIDATION_CYCLE[:self.cycle_position + 1])
            self.consolidation_due = (self.session_num % phase_end == phase_end - 1)

    def _load_session_state(self):
        """Load session tracking info."""
        if self.SESSION_STATE.exists():
            try:
                with open(self.SESSION_STATE, "r") as f:
                    return json.load(f)
            except:
                return None
        return None

    def _save_session_state(self):
        """Save session tracking info."""
        state = {
            "session_num": self.session_num,
            "cycle_position": self.cycle_position,
            "consolidation_due": self.consolidation_due,
            "timestamp": datetime.now().isoformat(),
            "memory_stats": {
                "candidates": len(self.candidates),
                "provisional": len(self.provisional),
                "anchored": len(self.anchored)
            }
        }
        self.SESSION_STATE.parent.mkdir(parents=True, exist_ok=True)
        with open(self.SESSION_STATE, "w") as f:
            json.dump(state, f, indent=2)

    def begin_session(self, session_num):
        """Start a new session."""
        self.session_num = session_num
        self.cycle_position = (session_num - 1) % len(self.CONSOLIDATION_CYCLE)

        # Check if consolidation should run
        phase_sessions = sum(self.CONSOLIDATION_CYCLE[:self.cycle_position + 1])
        self.consolidation_due = (session_num % phase_sessions == 0)

    def _load_all(self):
        """Load all memory phases from disk."""
        self._load_journal()
        self._load_phase(self.CANDIDATES_PATH, self.candidates)
        self._load_phase(self.PROVISIONAL_PATH, self.provisional)
        self._load_phase(self.ANCHORED_PATH, self.anchored)
        self._load_decay_log()

    def _load_journal(self):
        """Load append-only episodic journal (Phase 1)."""
        if self.JOURNAL_PATH.exists():
            with open(self.JOURNAL_PATH, "r") as f:
                for line in f:
                    if line.strip():
                        try:
                            self.journal.append(json.loads(line))
                        except:
                            pass

    def _load_phase(self, path, phase_dict):
        """Load memory phase in binary format."""
        if not path.exists():
            return

        try:
            with open(path, "rb") as f:
                while True:
                    header = f.read(4)
                    if not header:
                        break

                    rule_len = struct.unpack("H", f.read(2))[0]
                    rule = f.read(rule_len).decode("utf-8")
                    uses, success, age = struct.unpack("HHH", f.read(6))

                    rule_id = hashlib.md5(rule.encode()).hexdigest()[:8]
                    phase_dict[rule_id] = {
                        "rule": rule,
                        "uses": uses,
                        "success": success,
                        "age": age
                    }
        except:
            pass

    def _load_decay_log(self):
        """Load decay/pruning audit log."""
        if self.DECAY_META_PATH.exists():
            with open(self.DECAY_META_PATH, "r") as f:
                for line in f:
                    if line.strip():
                        try:
                            self.decay_log.append(json.loads(line))
                        except:
                            pass

    def record_event(self, event_type, data):
        """
        Phase 1: Record immutable event to journal.
        This is the ground truth of experience.
        """
        entry = {
            "timestamp": datetime.now().isoformat(),
            "session": self.session_num,
            "type": event_type,
            "data": data
        }

        self.journal.append(entry)
        self.JOURNAL_PATH.parent.mkdir(parents=True, exist_ok=True)

        with open(self.JOURNAL_PATH, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def learn_candidate(self, rule, success=False):
        """
        Phase 2: Learn a new candidate rule (single use).
        Candidates are exploration; most won't stick.
        """
        rule_id = hashlib.md5(rule.encode()).hexdigest()[:8]

        self.candidates[rule_id] = {
            "rule": rule,
            "uses": 1,
            "success": 1 if success else 0,
            "age": 0
        }

        self.record_event("candidate_learned", {
            "id": rule_id,
            "rule": rule,
            "success": success
        })

        return rule_id

    def record_rule_use(self, rule_id, success):
        """Record that a rule was used (success or failure)."""
        for phase in [self.candidates, self.provisional, self.anchored]:
            if rule_id in phase:
                phase[rule_id]["uses"] += 1
                if success:
                    phase[rule_id]["success"] += 1

                self.record_event("rule_used", {
                    "id": rule_id,
                    "success": success,
                    "phase": self._phase_name(phase)
                })
                return True

        return False

    def _phase_name(self, phase_dict):
        """Get name of phase dict."""
        if phase_dict is self.candidates:
            return "candidate"
        elif phase_dict is self.provisional:
            return "provisional"
        elif phase_dict is self.anchored:
            return "anchored"
        return "unknown"

    def try_promote_candidate(self, rule_id):
        """
        Phase 2→3: Promote candidate to provisional if threshold met.
        3+ uses, 50%+ success rate.
        """
        if rule_id not in self.candidates:
            return False

        cand = self.candidates[rule_id]
        meets_uses = cand["uses"] >= self.CANDIDATE_TO_PROVISIONAL["min_uses"]
        meets_success = cand["success"] >= self.CANDIDATE_TO_PROVISIONAL["min_success"]
        meets_rate = (cand["success"] / max(1, cand["uses"])) >= self.CANDIDATE_TO_PROVISIONAL["min_rate"]

        if meets_uses and meets_success and meets_rate:
            self.provisional[rule_id] = cand
            del self.candidates[rule_id]

            self.record_event("promoted_to_provisional", {
                "id": rule_id,
                "rule": cand["rule"],
                "uses": cand["uses"],
                "success_rate": cand["success"] / cand["uses"]
            })

            return True

        return False

    def try_promote_provisional(self, rule_id):
        """
        Phase 3→4: Promote provisional to anchored if threshold met.
        10+ uses, 80%+ success rate.
        """
        if rule_id not in self.provisional:
            return False

        prov = self.provisional[rule_id]
        meets_uses = prov["uses"] >= self.PROVISIONAL_TO_ANCHORED["min_uses"]
        meets_success = prov["success"] >= self.PROVISIONAL_TO_ANCHORED["min_success"]
        meets_rate = (prov["success"] / max(1, prov["uses"])) >= self.PROVISIONAL_TO_ANCHORED["min_rate"]

        if meets_uses and meets_success and meets_rate:
            self.anchored[rule_id] = prov
            del self.provisional[rule_id]

            self.record_event("promoted_to_anchored", {
                "id": rule_id,
                "rule": prov["rule"],
                "uses": prov["uses"],
                "success_rate": prov["success"] / prov["uses"]
            })

            return True

        return False

    def consolidate(self):
        """
        Run full consolidation cycle:
        - Promote candidates → provisional
        - Promote provisional → anchored
        - Decay stale low-utility rules
        - Merge near-duplicate rules
        - Strengthen high-utility rules
        - Age all rules
        """
        changes = {
            "promoted": [],
            "decayed": [],
            "merged": [],
            "strengthened": []
        }

        # Phase 2→3: Promote candidates
        for rule_id in list(self.candidates.keys()):
            if self.try_promote_candidate(rule_id):
                changes["promoted"].append(rule_id)

        # Phase 3→4: Promote provisional
        for rule_id in list(self.provisional.keys()):
            if self.try_promote_provisional(rule_id):
                changes["promoted"].append(rule_id)

        # Decay stale low-utility items
        for rule_id in list(self.candidates.keys()):
            cand = self.candidates[rule_id]
            success_rate = cand["success"] / max(1, cand["uses"])

            if cand["age"] > self.DECAY_THRESHOLD_AGE or success_rate < self.DECAY_THRESHOLD_RATE:
                decay_reason = f"age={cand['age']}, rate={success_rate:.2f}"

                self.decay_log.append({
                    "timestamp": datetime.now().isoformat(),
                    "rule_id": rule_id,
                    "rule": cand["rule"],
                    "reason": decay_reason,
                    "uses": cand["uses"],
                    "success": cand["success"]
                })

                del self.candidates[rule_id]
                changes["decayed"].append(rule_id)

        # Merge near-duplicates (same rule text)
        seen = {}
        for phase in [self.candidates, self.provisional, self.anchored]:
            for rule_id in list(phase.keys()):
                rule_text = phase[rule_id]["rule"]

                if rule_text in seen:
                    # Merge uses/success into canonical version
                    canonical_id = seen[rule_text]
                    phase[canonical_id]["uses"] += phase[rule_id]["uses"]
                    phase[canonical_id]["success"] += phase[rule_id]["success"]

                    del phase[rule_id]
                    changes["merged"].append((rule_id, canonical_id))
                else:
                    seen[rule_text] = rule_id

        # Strengthen high-utility rules
        for phase in [self.candidates, self.provisional, self.anchored]:
            for rule_id in phase:
                success_rate = phase[rule_id]["success"] / max(1, phase[rule_id]["uses"])

                if success_rate >= 0.9:
                    phase[rule_id]["age"] = 0  # Reset age as reward
                    changes["strengthened"].append(rule_id)

        # Age all remaining rules by 1
        for phase in [self.candidates, self.provisional, self.anchored]:
            for rule_id in phase:
                phase[rule_id]["age"] += 1

        # Log consolidation
        self.record_event("consolidation", changes)
        self._log_consolidation(changes)

        # Persist all phases
        self._save_all()

        return changes

    def _log_consolidation(self, changes):
        """Append consolidation event to log."""
        log_entry = {
            "timestamp": datetime.now().isoformat(),
            "session": self.session_num,
            "promoted": len(changes["promoted"]),
            "decayed": len(changes["decayed"]),
            "merged": len(changes["merged"]),
            "strengthened": len(changes["strengthened"]),
            "stats": {
                "candidates": len(self.candidates),
                "provisional": len(self.provisional),
                "anchored": len(self.anchored)
            }
        }

        self.CONSOLIDATION_LOG.parent.mkdir(parents=True, exist_ok=True)
        with open(self.CONSOLIDATION_LOG, "a") as f:
            f.write(json.dumps(log_entry) + "\n")

    def _save_all(self):
        """Persist all memory phases to disk."""
        self._save_phase(self.CANDIDATES_PATH, self.candidates)
        self._save_phase(self.PROVISIONAL_PATH, self.provisional)
        self._save_phase(self.ANCHORED_PATH, self.anchored)
        self._save_decay_log()
        self._save_session_state()

    def _save_phase(self, path, phase_dict):
        """Save memory phase in binary format."""
        path.parent.mkdir(parents=True, exist_ok=True)

        with open(path, "wb") as f:
            for rule_id, data in phase_dict.items():
                rule_bytes = data["rule"].encode("utf-8")
                rule_len = len(rule_bytes)

                f.write(struct.pack("H", rule_len))
                f.write(rule_bytes)
                f.write(struct.pack("HHH", data["uses"], data["success"], data["age"]))

    def _save_decay_log(self):
        """Save decay audit log."""
        if self.decay_log:
            self.DECAY_META_PATH.parent.mkdir(parents=True, exist_ok=True)

            with open(self.DECAY_META_PATH, "a") as f:
                for entry in self.decay_log:
                    # Avoid duplicates
                    if not any(e == entry for e in self.decay_log[:-1]):
                        f.write(json.dumps(entry) + "\n")

    def get_working_context(self, limit=15):
        """
        Return active memory for current session.
        Small, fast context for inference: anchored + recent provisional + candidates.
        """
        context = {
            "anchored": list(self.anchored.values())[:limit],
            "provisional": list(self.provisional.values())[:limit//2],
            "candidates": list(self.candidates.values())[:limit//3]
        }

        return context

    def get_memory_status(self):
        """Return comprehensive memory statistics."""
        return {
            "session": self.session_num,
            "cycle_position": self.cycle_position,
            "consolidation_due": self.consolidation_due,
            "episodic_events": len(self.journal),
            "candidates": len(self.candidates),
            "provisional": len(self.provisional),
            "anchored": len(self.anchored),
            "total_rules": len(self.candidates) + len(self.provisional) + len(self.anchored),
            "decayed": len(self.decay_log)
        }
