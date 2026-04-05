"""
file_explorer.py - Mother's File System Access Layer
Real-time file tree browsing, change detection, selective read.
"""

import os
import json
from pathlib import Path
from datetime import datetime
import hashlib
from collections import defaultdict

class FileExplorer:
    """
    Mother's eyes into C:\Users\wlwil\Desktop\Aeonmi Files
    - Real-time directory tree with change detection
    - Selective file reading (compress large/binary)
    - Cache to avoid re-reading unchanged files
    """

    WORKSPACE_ROOT = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files")
    EXPLORER_CACHE = Path(r"C:\Temp\explorer_cache.json")
    FILE_HASH_DB = Path(r"C:\Temp\file_hashes.json")

    # Keep track of what Mother has already "seen"
    TREE_DEPTH_LIMIT = 6
    IGNORE_PATTERNS = {".git", "__pycache__", ".pyc", ".o", "node_modules", "venv", ".venv"}

    def __init__(self):
        self.workspace = self.WORKSPACE_ROOT
        self.file_hashes = self._load_hashes()
        self.cache = self._load_cache()
        self.change_log = []

    def _load_hashes(self):
        """Load stored file hashes for change detection."""
        if self.FILE_HASH_DB.exists():
            try:
                with open(self.FILE_HASH_DB, "r") as f:
                    return json.load(f)
            except:
                return {}
        return {}

    def _load_cache(self):
        """Load directory tree cache."""
        if self.EXPLORER_CACHE.exists():
            try:
                with open(self.EXPLORER_CACHE, "r") as f:
                    return json.load(f)
            except:
                return {}
        return {}

    def _should_ignore(self, path):
        """Check if path should be skipped."""
        name = path.name.lower()
        for pattern in self.IGNORE_PATTERNS:
            if pattern in name:
                return True
        # Skip very large files (> 50 MB)
        try:
            if path.is_file() and path.stat().st_size > 50 * 1024 * 1024:
                return True
        except:
            pass
        return False

    def _file_hash(self, path):
        """Compute hash of file for change detection."""
        try:
            with open(path, "rb") as f:
                return hashlib.md5(f.read()).hexdigest()
        except:
            return None

    def scan_tree(self, root=None, depth=0):
        """
        Recursively scan directory tree, return structure.
        depth: current depth (limit to TREE_DEPTH_LIMIT)
        """
        if root is None:
            root = self.workspace

        if depth >= self.TREE_DEPTH_LIMIT:
            return None

        if not root.exists() or self._should_ignore(root):
            return None

        tree = {
            "name": root.name,
            "path": str(root),
            "type": "dir",
            "children": [],
            "scanned_at": datetime.now().isoformat()
        }

        try:
            for item in sorted(root.iterdir(), key=str):
                if self._should_ignore(item):
                    continue

                try:
                    if item.is_dir():
                        subtree = self.scan_tree(item, depth + 1)
                        if subtree:
                            tree["children"].append(subtree)
                    else:
                        size = item.stat().st_size
                        mtime = item.stat().st_mtime

                        file_obj = {
                            "name": item.name,
                            "path": str(item),
                            "type": "file",
                            "size": size,
                            "modified": datetime.fromtimestamp(mtime).isoformat(),
                            "ext": item.suffix
                        }

                        # Check if file changed
                        current_hash = self._file_hash(item)
                        stored_hash = self.file_hashes.get(str(item))

                        if stored_hash != current_hash:
                            file_obj["changed"] = True
                            self.change_log.append({
                                "path": str(item),
                                "timestamp": datetime.now().isoformat(),
                                "size": size
                            })

                        if current_hash:
                            self.file_hashes[str(item)] = current_hash

                        tree["children"].append(file_obj)
                except (PermissionError, OSError, RuntimeError):
                    pass

        except (PermissionError, OSError):
            pass

        return tree

    def get_tree_summary(self):
        """Return compact tree summary (paths + sizes)."""
        tree = self.scan_tree()
        return self._compress_tree(tree)

    def _compress_tree(self, node, parent_path=""):
        """Flatten tree to list of paths + metadata."""
        if not node:
            return []

        result = []

        if node["type"] == "file":
            result.append({
                "path": node["path"],
                "size": node.get("size", 0),
                "ext": node.get("ext", ""),
                "changed": node.get("changed", False)
            })

        if "children" in node:
            for child in node["children"]:
                result.extend(self._compress_tree(child))

        return result

    def read_file(self, rel_path, compress=True):
        """
        Read file from workspace (with optional compression).
        Returns: (content, size, was_cached, changed)
        """
        full_path = self.workspace / rel_path

        # Security check
        try:
            full_path.resolve().relative_to(self.workspace.resolve())
        except ValueError:
            return None, 0, False, False

        if not full_path.exists():
            return None, 0, False, False

        # Get hash for change detection
        current_hash = self._file_hash(full_path)
        stored_hash = self.file_hashes.get(str(full_path))
        was_cached = (current_hash == stored_hash)
        changed = (current_hash != stored_hash)

        # Check cache first
        cache_key = str(full_path)
        if was_cached and cache_key in self.cache:
            return self.cache[cache_key], full_path.stat().st_size, True, False

        # Read file
        try:
            if full_path.suffix in [".bin", ".exe", ".dll", ".so", ".o"]:
                return None, full_path.stat().st_size, False, changed

            with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except:
            return None, 0, False, changed

        # Compress if requested
        if compress:
            compressed = self._compress_content(content, full_path.name)
        else:
            compressed = content[:2000]  # Truncate

        # Cache it
        self.file_hashes[str(full_path)] = current_hash
        self.cache[cache_key] = compressed

        return compressed, full_path.stat().st_size, False, changed

    def _compress_content(self, content, filename):
        """
        Intelligent compression: keep markers, structure, errors.
        Remove: verbose prose, data dumps, large comments.
        """
        if not isinstance(content, str):
            return ""

        lines = content.split("\n")
        compressed = []
        skip_until = None

        for i, line in enumerate(lines):
            stripped = line.strip()

            # Keep all phase/TODO/error markers
            if any(marker in stripped for marker in ["TODO", "FIXME", "PHASE", "ERROR", "FAILED", "BUG"]):
                compressed.append(line)
                continue

            # Keep function/class definitions
            if stripped.startswith(("def ", "class ", "@", "import ", "from ")):
                compressed.append(line)
                continue

            # Keep test markers
            if "test" in filename.lower() and any(marker in stripped for marker in ["assert", "PASS", "FAIL"]):
                compressed.append(line)
                continue

            # Skip docstring blocks
            if '"""' in stripped or "'''" in stripped:
                if skip_until == i:
                    skip_until = None
                    continue
                else:
                    skip_until = i + 10
                    continue

            # Skip empty + verbose lines
            if not stripped:
                continue
            if len(stripped) > 150:
                continue

            # Keep short informative lines
            if len(stripped) <= 100:
                compressed.append(line)

        return "\n".join(compressed)

    def detect_changes(self):
        """Scan and return list of changed files."""
        self.change_log = []
        self.scan_tree()
        self._save_hashes()
        return self.change_log

    def _save_hashes(self):
        """Persist file hashes to disk."""
        with open(self.FILE_HASH_DB, "w") as f:
            json.dump(self.file_hashes, f, indent=2)

        with open(self.EXPLORER_CACHE, "w") as f:
            json.dump(self.cache, f, indent=2)

    def list_files(self, pattern=None):
        """
        List all files in workspace, optionally filtered by pattern.
        Returns: list of (rel_path, size, ext)
        """
        all_files = []
        try:
            for root, dirs, files in os.walk(self.workspace):
                # Skip ignored dirs
                dirs[:] = [d for d in dirs if d.lower() not in [p.lower() for p in self.IGNORE_PATTERNS]]

                for fname in files:
                    full_path = Path(root) / fname
                    if self._should_ignore(full_path):
                        continue

                    rel_path = full_path.relative_to(self.workspace)
                    size = full_path.stat().st_size
                    ext = full_path.suffix

                    if pattern is None or pattern.lower() in str(rel_path).lower():
                        all_files.append((str(rel_path), size, ext))

        except (PermissionError, OSError):
            pass

        return all_files

    def get_dir_structure(self, rel_path="", max_items=50):
        """Get structure of specific directory."""
        if rel_path:
            target = self.workspace / rel_path
        else:
            target = self.workspace

        result = []
        try:
            for item in sorted(target.iterdir(), key=str)[:max_items]:
                if not self._should_ignore(item):
                    result.append({
                        "name": item.name,
                        "type": "dir" if item.is_dir() else "file",
                        "path": str(item.relative_to(self.workspace))
                    })
        except (PermissionError, OSError):
            pass

        return result
