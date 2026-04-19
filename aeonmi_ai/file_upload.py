"""
file_upload.py - Mother's File Ingestion & Upload System
Selective file absorption, compression, and memory storage.
Prevents overload via batch queueing and importance scoring.
"""

import json
import hashlib
from pathlib import Path
from datetime import datetime
import struct

class FileUploadSystem:
    """
    Mother's intake valve: absorbs files without memory explosion.
    - Batch queue (process N files per session)
    - Importance scoring (prioritize critical files)
    - Compression before storage
    - Binary serialization to keep footprint small
    """

    UPLOAD_QUEUE = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Mother\upload_queue.json")
    INGESTED_STORE = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Mother\ingested.bin")
    UPLOAD_LOG = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Mother\upload_log.jsonl")
    IMPORTANCE_RULES = {
        "test": 10,         # Tests are high signal
        "error": 9,
        "failed": 9,
        ".ai": 8,          # Aeonmi files matter
        "phase": 8,
        "core": 7,
        "memory": 7,
        ".py": 6,          # Python scripts
        "config": 6,
        ".md": 5,          # Docs
        ".json": 4,
        ".log": 3,
    }

    def __init__(self, max_per_session=5, max_file_size_kb=200):
        self.queue = self._load_queue()
        self.max_per_session = max_per_session
        self.max_file_size = max_file_size_kb * 1024
        self.ingested = self._load_ingested()
        self.session_intake = []

    def _load_queue(self):
        """Load pending upload queue."""
        if self.UPLOAD_QUEUE.exists():
            try:
                with open(self.UPLOAD_QUEUE, "r") as f:
                    return json.load(f)
            except:
                return []
        return []

    def _load_ingested(self):
        """Load previously ingested files (metadata only)."""
        if self.INGESTED_STORE.exists():
            result = {}
            try:
                with open(self.INGESTED_STORE, "rb") as f:
                    while True:
                        header = f.read(4)
                        if not header:
                            break
                        path_len = struct.unpack("H", f.read(2))[0]
                        path = f.read(path_len).decode("utf-8")
                        hash_val = f.read(32).decode("utf-8")
                        result[path] = hash_val
            except:
                return {}
            return result
        return {}

    def submit_file(self, file_path, priority="normal"):
        """Queue a file for ingestion."""
        if isinstance(file_path, str):
            file_path = Path(file_path)

        entry = {
            "path": str(file_path),
            "submitted_at": datetime.now().isoformat(),
            "priority": priority,  # "high", "normal", "low"
            "status": "queued"
        }

        self.queue.append(entry)
        self._save_queue()
        return entry

    def _score_importance(self, file_path):
        """Score file importance (0-10 scale)."""
        path_lower = str(file_path).lower()
        score = 0

        for keyword, value in self.IMPORTANCE_RULES.items():
            if keyword in path_lower:
                score = max(score, value)

        # Boost recent files
        try:
            mtime = Path(file_path).stat().st_mtime
            age_hours = (datetime.now().timestamp() - mtime) / 3600
            if age_hours < 1:
                score += 2
            elif age_hours < 24:
                score += 1
        except:
            pass

        return min(score, 10)

    def get_next_batch(self, limit=None):
        """
        Get next N files to ingest, sorted by importance.
        Skips already-ingested files unless changed.
        """
        if limit is None:
            limit = self.max_per_session

        batch = []

        for entry in self.queue:
            if entry["status"] != "queued":
                continue

            file_path = Path(entry["path"])
            if not file_path.exists():
                entry["status"] = "skipped_notfound"
                continue

            # Check size
            try:
                if file_path.stat().st_size > self.max_file_size:
                    entry["status"] = "skipped_toolarge"
                    continue
            except:
                entry["status"] = "skipped_access"
                continue

            # Check if already ingested
            file_hash = self._hash_file(file_path)
            stored_hash = self.ingested.get(str(file_path))

            if stored_hash == file_hash:
                entry["status"] = "skipped_unchanged"
                continue

            # Score importance
            score = self._score_importance(file_path)

            batch.append({
                "entry": entry,
                "file_path": file_path,
                "score": score,
                "hash": file_hash
            })

            if len(batch) >= limit:
                break

        # Sort by importance (descending)
        batch.sort(key=lambda x: x["score"], reverse=True)

        self._save_queue()
        return batch

    def _hash_file(self, path):
        """Compute file hash for change detection."""
        try:
            with open(path, "rb") as f:
                return hashlib.md5(f.read()).hexdigest()
        except:
            return None

    def ingest_batch(self, batch):
        """
        Process a batch of files into memory.
        Returns: (num_ingested, total_size_kb, compressed_size_kb)
        """
        ingested_count = 0
        total_size = 0
        compressed_size = 0

        for item in batch:
            try:
                file_path = item["file_path"]
                content = self._read_and_compress(file_path)

                if content:
                    # Store in ingested index
                    self.ingested[str(file_path)] = item["hash"]
                    self.session_intake.append({
                        "path": str(file_path),
                        "size_kb": file_path.stat().st_size / 1024,
                        "compressed_kb": len(content) / 1024
                    })

                    # Append to log
                    self._log_ingestion(file_path, len(content))

                    ingested_count += 1
                    total_size += file_path.stat().st_size
                    compressed_size += len(content)

                # Mark as ingested in queue
                item["entry"]["status"] = "ingested"
                item["entry"]["ingested_at"] = datetime.now().isoformat()

            except Exception as e:
                item["entry"]["status"] = "failed"
                item["entry"]["error"] = str(e)

        # Persist ingested index
        self._save_ingested()
        self._save_queue()

        return ingested_count, total_size / 1024, compressed_size / 1024

    def _read_and_compress(self, file_path):
        """Read file and compress content intelligently."""
        try:
            # Skip binaries
            if file_path.suffix in [".bin", ".exe", ".dll", ".pyc"]:
                return None

            # Skip huge files
            if file_path.stat().st_size > self.max_file_size:
                return None

            # Read content
            with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()

            # Compress
            compressed = self._compress_content(content, file_path.name)
            return compressed.encode("utf-8")

        except:
            return None

    def _compress_content(self, content, filename):
        """Smart compression: keep structure, remove noise."""
        lines = content.split("\n")
        kept = []

        for line in lines:
            stripped = line.strip()

            # Keep markers
            if any(marker in stripped for marker in ["TODO", "FIXME", "ERROR", "FAILED", "PHASE", "MODEL", "CRITICAL"]):
                kept.append(line)
                continue

            # Keep code structure
            if stripped.startswith(("def ", "class ", "import ", "from ", "@")):
                kept.append(line)
                continue

            # Keep short meaningful lines
            if 10 < len(stripped) < 100:
                # Skip pure comments
                if not stripped.startswith("#"):
                    kept.append(line)
                continue

        return "\n".join(kept)

    def _log_ingestion(self, file_path, compressed_bytes):
        """Append to ingestion audit log."""
        log_entry = {
            "timestamp": datetime.now().isoformat(),
            "file": str(file_path),
            "size_bytes": file_path.stat().st_size,
            "compressed_bytes": compressed_bytes
        }

        self.UPLOAD_LOG.parent.mkdir(parents=True, exist_ok=True)
        with open(self.UPLOAD_LOG, "a") as f:
            f.write(json.dumps(log_entry) + "\n")

    def _save_ingested(self):
        """Save ingested file index (binary)."""
        self.INGESTED_STORE.parent.mkdir(parents=True, exist_ok=True)

        with open(self.INGESTED_STORE, "wb") as f:
            for path, hash_val in self.ingested.items():
                path_bytes = path.encode("utf-8")
                f.write(struct.pack("H", len(path_bytes)))
                f.write(path_bytes)
                f.write(hash_val.encode("utf-8"))

    def _save_queue(self):
        """Save queue state."""
        self.UPLOAD_QUEUE.parent.mkdir(parents=True, exist_ok=True)

        with open(self.UPLOAD_QUEUE, "w") as f:
            json.dump(self.queue, f, indent=2)

    def get_intake_summary(self):
        """Return summary of what was ingested this session."""
        if not self.session_intake:
            return None

        total_original = sum(item["size_kb"] for item in self.session_intake)
        total_compressed = sum(item["compressed_kb"] for item in self.session_intake)
        compression_ratio = total_original / max(1, total_compressed)

        return {
            "files": len(self.session_intake),
            "original_kb": total_original,
            "compressed_kb": total_compressed,
            "ratio": compression_ratio,
            "intake_files": self.session_intake
        }

    def clear_completed(self):
        """Remove successfully ingested items from queue."""
        self.queue = [entry for entry in self.queue if entry["status"] != "ingested"]
        self._save_queue()
