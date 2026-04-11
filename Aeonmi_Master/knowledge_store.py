"""
knowledge_store.py — Mother's persistent knowledge base.

Chunks textbooks and uploaded files into sections, indexes them, and provides
keyword-ranked retrieval so Mother's context window always includes relevant knowledge.

Storage layout:
    Aeonmi_Master/knowledge/
        index.json          — fast lookup metadata for all chunks
        chunks/             — one .txt file per knowledge chunk
"""

from __future__ import annotations
import json
import math
import os
import re
import textwrap
from pathlib import Path

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
_ROOT       = Path(__file__).parent.resolve()
_KG_DIR     = _ROOT / "knowledge"
_CHUNK_DIR  = _KG_DIR / "chunks"
_INDEX_PATH = _KG_DIR / "index.json"

_CHUNK_DIR.mkdir(parents=True, exist_ok=True)

# ---------------------------------------------------------------------------
# Index helpers
# ---------------------------------------------------------------------------

def _load_index() -> dict:
    if _INDEX_PATH.exists():
        try:
            return json.loads(_INDEX_PATH.read_text(encoding="utf-8"))
        except Exception:
            pass
    return {}   # key -> {title, tags, summary, file, word_freq}


def _save_index(idx: dict):
    _INDEX_PATH.write_text(json.dumps(idx, indent=2, ensure_ascii=False), encoding="utf-8")


def _word_freq(text: str) -> dict[str, int]:
    """Lowercase token frequency (stops excluded)."""
    STOPS = {
        "a","an","the","is","are","was","were","it","its","in","on","at",
        "to","of","for","and","or","but","not","this","that","be","by",
        "with","as","from","has","have","had","do","does","did","will",
        "can","if","then","so","all","any","each","which","when","where",
        "how","what","who","we","you","they","he","she","i","my","your",
    }
    freq: dict[str, int] = {}
    for tok in re.findall(r"[a-zA-Z_ψφαβγδεζηθλμνξπρστυχω]{2,}", text.lower()):
        if tok not in STOPS:
            freq[tok] = freq.get(tok, 0) + 1
    return freq


def _score(query_freq: dict[str, int], chunk_freq: dict[str, int]) -> float:
    """TF-IDF-like dot product between query and chunk word frequencies."""
    score = 0.0
    for word, qf in query_freq.items():
        if word in chunk_freq:
            score += qf * math.log1p(chunk_freq[word])
    return score

# ---------------------------------------------------------------------------
# Ingestion
# ---------------------------------------------------------------------------

def _textbook_sections(text: str, source_name: str) -> list[dict]:
    """
    Parse textbook format markers into sections.
    Returns list of {title, tags, body} dicts.
    """
    sections = []
    current_title = source_name
    current_tags  = [source_name.lower().replace(" ", "_")]
    current_lines: list[str] = []

    def _flush():
        body = "\n".join(current_lines).strip()
        if body:
            sections.append({
                "title": current_title,
                "tags":  list(current_tags),
                "body":  body,
            })

    for raw in text.splitlines():
        line = raw.strip()

        if line.startswith("# PART"):
            _flush()
            current_lines = []
            current_title = line.lstrip("# ").strip()
            current_tags  = ["part", source_name.lower().replace(" ", "_")]

        elif line.startswith("## CHAPTER") or line.startswith("## CHAPTER SUPPLEMENT"):
            _flush()
            current_lines = []
            current_title = line.lstrip("# ").strip()
            current_tags  = ["chapter", source_name.lower().replace(" ", "_")]
            # extract chapter number if present
            m = re.search(r"(\d+)", current_title)
            if m:
                current_tags.append(f"ch{m.group(1)}")

        elif line.startswith("### SECTION:"):
            _flush()
            current_lines = []
            current_title = line.replace("### SECTION:", "").strip()
            current_tags  = ["section", source_name.lower().replace(" ", "_")]

        elif line.startswith("## APPENDIX"):
            _flush()
            current_lines = []
            current_title = line.lstrip("# ").strip()
            current_tags  = ["appendix", source_name.lower().replace(" ", "_")]

        elif line.startswith(("BODY:", "CODE:", "NOTE:", "TIP:", "WARNING:",
                               "EXERCISE_TITLE:", "EXERCISE_BODY:", "EXERCISE_SOLUTION:",
                               "TABLE_HEADER:", "TABLE_ROW:", "EXERCISE_LEVEL:")):
            # Keep raw marker lines — useful context
            current_lines.append(raw)
        else:
            current_lines.append(raw)

    _flush()
    return sections


def _plain_sections(text: str, source_name: str, chunk_size: int = 800) -> list[dict]:
    """
    Fallback chunker for plain text files (no markers).
    Splits at paragraph boundaries roughly every chunk_size words.
    """
    paragraphs = re.split(r"\n{2,}", text.strip())
    sections   = []
    buf: list[str] = []
    word_count = 0
    idx = 0

    def _flush_plain():
        nonlocal idx, buf, word_count
        body = "\n\n".join(buf).strip()
        if body:
            idx += 1
            sections.append({
                "title": f"{source_name} §{idx}",
                "tags":  [source_name.lower().replace(" ", "_"), "plain"],
                "body":  body,
            })
        buf = []
        word_count = 0

    for para in paragraphs:
        words = len(para.split())
        if word_count + words > chunk_size and buf:
            _flush_plain()
        buf.append(para)
        word_count += words

    if buf:
        _flush_plain()

    return sections


def ingest_text(text: str, source_name: str, tags: list[str] | None = None) -> int:
    """
    Parse text into chunks, write to knowledge/chunks/, update index.
    Returns number of chunks added.
    """
    idx   = _load_index()
    added = 0

    # Choose parser
    has_markers = any(
        line.strip().startswith(("# PART", "## CHAPTER", "### SECTION:", "## APPENDIX"))
        for line in text.splitlines()
    )
    sections = _textbook_sections(text, source_name) if has_markers else _plain_sections(text, source_name)

    extra_tags = tags or []

    for i, sec in enumerate(sections):
        body  = sec["body"]
        if len(body) < 40:          # skip trivially short chunks
            continue

        title = sec["title"]
        stags = sec["tags"] + extra_tags

        # Stable key: source + section index
        safe  = re.sub(r"[^\w]", "_", source_name.lower())
        key   = f"{safe}_{i:04d}"

        # Write chunk file
        chunk_file = _CHUNK_DIR / f"{key}.txt"
        chunk_file.write_text(f"=== {title} ===\n\n{body}", encoding="utf-8")

        # Summary: first 300 chars of body
        summary = textwrap.shorten(body.replace("\n", " "), width=300, placeholder="…")

        idx[key] = {
            "title":     title,
            "tags":      stags,
            "summary":   summary,
            "file":      str(chunk_file.relative_to(_ROOT)),
            "word_freq": _word_freq(body),
        }
        added += 1

    _save_index(idx)
    return added


def ingest_file(path: str | Path) -> tuple[int, str]:
    """
    Read a file and ingest it. Returns (chunks_added, message).
    Supports: .txt, .md, .ai, .qube, .py, .rs, .docx (basic), .pdf (basic)
    """
    p = Path(path)
    if not p.exists():
        return 0, f"File not found: {path}"

    ext  = p.suffix.lower()
    name = p.stem

    # --- PDF ---
    if ext == ".pdf":
        try:
            import urllib.request as _ur
            # Try pdfplumber first, fallback to basic extraction
            try:
                import pdfplumber
                text_parts = []
                with pdfplumber.open(str(p)) as pdf:
                    for page in pdf.pages:
                        t = page.extract_text()
                        if t:
                            text_parts.append(t)
                text = "\n\n".join(text_parts)
            except ImportError:
                # Manual PDF text extraction (basic)
                raw = p.read_bytes()
                # Extract readable ASCII runs from PDF stream
                text = re.sub(rb'\(([^)]{4,})\)', lambda m: m.group(1) + b'\n', raw)
                text = text.decode("latin-1", errors="ignore")
                text = re.sub(r"[^\x20-\x7e\n]", " ", text)
                text = re.sub(r" {3,}", " ", text)
        except Exception as e:
            return 0, f"PDF read error: {e}"

    # --- DOCX ---
    elif ext in (".docx", ".doc"):
        try:
            import zipfile, xml.etree.ElementTree as ET
            with zipfile.ZipFile(str(p)) as z:
                xml_content = z.read("word/document.xml")
            root = ET.fromstring(xml_content)
            NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            paras = []
            for para in root.iter(f"{{{NS}}}p"):
                line = "".join(t.text or "" for t in para.iter(f"{{{NS}}}t")).strip()
                if line:
                    paras.append(line)
            text = "\n".join(paras)
        except Exception as e:
            return 0, f"DOCX read error: {e}"

    # --- Plain text / code ---
    else:
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except Exception as e:
            return 0, f"Read error: {e}"

    if not text.strip():
        return 0, "File appears empty"

    # Tag by extension
    ext_tags = {
        ".ai": ["aeonmi", "code"], ".qube": ["qube", "quantum", "code"],
        ".rs": ["rust", "code"],   ".py": ["python", "code"],
        ".txt": ["text"],          ".md": ["markdown"],
        ".pdf": ["pdf"],           ".docx": ["document"],
    }
    tags = ext_tags.get(ext, [])

    n = ingest_text(text, name, tags)
    return n, f"Ingested {n} chunks from '{p.name}'"

# ---------------------------------------------------------------------------
# Retrieval
# ---------------------------------------------------------------------------

def search(query: str, top_k: int = 4) -> list[dict]:
    """
    Return top_k most relevant chunks for the query.
    Each result: {key, title, tags, summary, body, score}
    """
    idx = _load_index()
    if not idx:
        return []

    qf = _word_freq(query)
    if not qf:
        return []

    scored = []
    for key, meta in idx.items():
        s = _score(qf, meta.get("word_freq", {}))
        if s > 0:
            scored.append((s, key, meta))

    scored.sort(key=lambda x: x[0], reverse=True)
    results = []
    for score, key, meta in scored[:top_k]:
        body = ""
        try:
            body = (_ROOT / meta["file"]).read_text(encoding="utf-8")
        except Exception:
            body = meta.get("summary", "")
        results.append({
            "key":     key,
            "title":   meta["title"],
            "tags":    meta["tags"],
            "summary": meta["summary"],
            "body":    body,
            "score":   round(score, 2),
        })
    return results


def get_context_for_query(query: str, max_chars: int = 4000) -> str:
    """
    Return a formatted knowledge context block for injection into Mother's system prompt.
    Limits total characters to max_chars to protect context budget.
    """
    hits = search(query, top_k=4)
    if not hits:
        return ""

    parts = ["=== RELEVANT KNOWLEDGE FROM AEONMI DOCS ==="]
    used  = len(parts[0])

    for hit in hits:
        body    = hit["body"]
        excerpt = textwrap.shorten(body.replace("\n", " "), width=800, placeholder="…")
        block   = f"\n\n[{hit['title']}]\n{excerpt}"
        if used + len(block) > max_chars:
            break
        parts.append(block)
        used += len(block)

    if len(parts) == 1:
        return ""

    parts.append("\n=== END KNOWLEDGE ===")
    return "\n".join(parts)


# ---------------------------------------------------------------------------
# Status
# ---------------------------------------------------------------------------

def status() -> dict:
    idx = _load_index()
    tags: dict[str, int] = {}
    for meta in idx.values():
        for t in meta.get("tags", []):
            tags[t] = tags.get(t, 0) + 1
    return {
        "chunk_count": len(idx),
        "tag_counts":  dict(sorted(tags.items(), key=lambda x: x[1], reverse=True)),
        "index_path":  str(_INDEX_PATH),
    }


# ---------------------------------------------------------------------------
# CLI — run directly to ingest textbooks
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import sys
    files = sys.argv[1:] or [
        str(_ROOT / "textbook_part1_2.txt"),
        str(_ROOT / "textbook_part3_4.txt"),
        str(_ROOT / "textbook_appendices.txt"),
        str(_ROOT / "textbook_source_review.txt"),
        str(_ROOT / "vscode_extension_spec.txt"),
    ]
    total = 0
    for f in files:
        n, msg = ingest_file(f)
        print(msg)
        total += n
    print(f"\nTotal chunks: {total}")
    st = status()
    print(f"Knowledge base: {st['chunk_count']} chunks")
    print("Top tags:", dict(list(st["tag_counts"].items())[:10]))
