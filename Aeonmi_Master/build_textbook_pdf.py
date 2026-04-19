#!/usr/bin/env python3
"""
AEONMI Textbook PDF Compiler
Reads textbook_part1_2.txt, textbook_part3_4.txt, and textbook_appendices.txt
and compiles them into AEONMI_Complete_Guide.pdf using ReportLab.

Run from Aeonmi_Master/:
    py -3 build_textbook_pdf.py

Dependencies:
    pip install reportlab
"""

import os
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Dependency check
# ---------------------------------------------------------------------------
try:
    from reportlab.lib.pagesizes import letter
    from reportlab.lib.units import inch
    from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
    from reportlab.lib.colors import HexColor, white, black
    from reportlab.lib.enums import TA_LEFT, TA_CENTER, TA_RIGHT, TA_JUSTIFY
    from reportlab.platypus import (
        SimpleDocTemplate, Paragraph, Spacer, PageBreak,
        Table, TableStyle, KeepTogether, HRFlowable,
        Flowable
    )
    from reportlab.lib import colors
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.ttfonts import TTFont
    from reportlab.platypus.flowables import Flowable
except ImportError:
    print("ERROR: reportlab is not installed.")
    print("Install it with:  pip install reportlab")
    sys.exit(1)

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).parent.resolve()
OUTPUT_PDF = SCRIPT_DIR / "AEONMI_Complete_Guide.pdf"

INPUT_FILES = [
    SCRIPT_DIR / "textbook_part1_2.txt",
    SCRIPT_DIR / "textbook_part3_4.txt",
    SCRIPT_DIR / "textbook_appendices.txt",
]

# ---------------------------------------------------------------------------
# Color Palette
# ---------------------------------------------------------------------------
C_PURPLE       = HexColor("#7c3aed")
C_PURPLE_LIGHT = HexColor("#ede9fe")
C_PURPLE_MED   = HexColor("#c4b5fd")
C_CYAN         = HexColor("#06b6d4")
C_CYAN_LIGHT   = HexColor("#e0f7fa")
C_DARK         = HexColor("#1e1b4b")
C_DARK_HEADER  = HexColor("#0f0a2e")
C_BODY         = HexColor("#1a1a2e")
C_NOTE_BG      = HexColor("#dbeafe")
C_NOTE_BORDER  = HexColor("#3b82f6")
C_TIP_BG       = HexColor("#dcfce7")
C_TIP_BORDER   = HexColor("#16a34a")
C_WARN_BG      = HexColor("#ffedd5")
C_WARN_BORDER  = HexColor("#ea580c")
C_EX_BG        = HexColor("#f5f3ff")
C_EX_BORDER    = HexColor("#7c3aed")
C_TABLE_HEADER = HexColor("#4c1d95")
C_TABLE_ROW_A  = HexColor("#faf5ff")
C_TABLE_ROW_B  = HexColor("#ffffff")
C_HR           = HexColor("#d1d5db")
C_CODE_BG      = HexColor("#f8f8ff")
C_CODE_BORDER  = HexColor("#06b6d4")

# ---------------------------------------------------------------------------
# Page dimensions
# ---------------------------------------------------------------------------
PAGE_W, PAGE_H = letter
MARGIN_LEFT   = 0.85 * inch
MARGIN_RIGHT  = 0.85 * inch
MARGIN_TOP    = 1.1  * inch
MARGIN_BOTTOM = 0.9  * inch
TEXT_W = PAGE_W - MARGIN_LEFT - MARGIN_RIGHT

# ---------------------------------------------------------------------------
# Style definitions
# ---------------------------------------------------------------------------
def build_styles():
    base = getSampleStyleSheet()

    def s(name, **kwargs):
        kwargs.setdefault("fontName", "Helvetica")
        kwargs.setdefault("fontSize", 10)
        kwargs.setdefault("textColor", C_BODY)
        kwargs.setdefault("spaceAfter", 6)
        kwargs.setdefault("spaceBefore", 0)
        kwargs.setdefault("leading", 14)
        return ParagraphStyle(name, **kwargs)

    styles = {}

    styles["body"] = s(
        "body",
        fontSize=10.5,
        leading=15,
        spaceAfter=8,
        alignment=TA_JUSTIFY,
    )

    styles["chapter"] = s(
        "chapter",
        fontName="Helvetica-Bold",
        fontSize=22,
        textColor=white,
        spaceAfter=0,
        spaceBefore=0,
        leading=28,
        alignment=TA_LEFT,
    )

    styles["part"] = s(
        "part",
        fontName="Helvetica-Bold",
        fontSize=30,
        textColor=white,
        spaceAfter=0,
        spaceBefore=0,
        leading=38,
        alignment=TA_CENTER,
    )

    styles["section"] = s(
        "section",
        fontName="Helvetica-Bold",
        fontSize=13,
        textColor=C_PURPLE,
        spaceAfter=6,
        spaceBefore=14,
        leading=18,
    )

    styles["subsection"] = s(
        "subsection",
        fontName="Helvetica-Bold",
        fontSize=11,
        textColor=C_DARK,
        spaceAfter=4,
        spaceBefore=10,
        leading=15,
    )

    styles["code"] = s(
        "code",
        fontName="Courier",
        fontSize=8.5,
        textColor=C_DARK,
        leading=12,
        spaceAfter=2,
        spaceBefore=2,
        alignment=TA_LEFT,
        leftIndent=8,
        rightIndent=8,
    )

    styles["note"] = s(
        "note",
        fontSize=9.5,
        textColor=HexColor("#1e3a5f"),
        leading=14,
        leftIndent=10,
    )

    styles["tip"] = s(
        "tip",
        fontSize=9.5,
        textColor=HexColor("#14532d"),
        leading=14,
        leftIndent=10,
    )

    styles["warning"] = s(
        "warning",
        fontSize=9.5,
        textColor=HexColor("#7c2d12"),
        leading=14,
        leftIndent=10,
    )

    styles["exercise_title"] = s(
        "exercise_title",
        fontName="Helvetica-Bold",
        fontSize=11,
        textColor=C_PURPLE,
        leading=15,
        leftIndent=8,
    )

    styles["exercise_body"] = s(
        "exercise_body",
        fontSize=10,
        textColor=C_DARK,
        leading=14,
        leftIndent=8,
    )

    styles["solution_label"] = s(
        "solution_label",
        fontName="Helvetica-Bold",
        fontSize=9,
        textColor=C_PURPLE,
        leading=13,
        leftIndent=8,
    )

    styles["toc_chapter"] = s(
        "toc_chapter",
        fontName="Helvetica-Bold",
        fontSize=11,
        textColor=C_DARK,
        leading=16,
        spaceAfter=2,
    )

    styles["toc_section"] = s(
        "toc_section",
        fontSize=9.5,
        textColor=HexColor("#4b5563"),
        leading=14,
        spaceAfter=1,
        leftIndent=16,
    )

    styles["cover_title"] = s(
        "cover_title",
        fontName="Helvetica-Bold",
        fontSize=42,
        textColor=white,
        leading=52,
        alignment=TA_CENTER,
    )

    styles["cover_subtitle"] = s(
        "cover_subtitle",
        fontName="Helvetica",
        fontSize=16,
        textColor=C_PURPLE_MED,
        leading=22,
        alignment=TA_CENTER,
    )

    styles["cover_meta"] = s(
        "cover_meta",
        fontSize=11,
        textColor=C_PURPLE_MED,
        leading=16,
        alignment=TA_CENTER,
    )

    styles["table_header"] = s(
        "table_header",
        fontName="Helvetica-Bold",
        fontSize=9,
        textColor=white,
        leading=13,
        alignment=TA_LEFT,
    )

    styles["table_cell"] = s(
        "table_cell",
        fontSize=8.5,
        textColor=C_BODY,
        leading=12,
        alignment=TA_LEFT,
    )

    return styles

# ---------------------------------------------------------------------------
# Custom Flowables
# ---------------------------------------------------------------------------

class ColorBar(Flowable):
    """A full-width colored bar (used for chapter headings)."""
    def __init__(self, height, color, radius=4):
        super().__init__()
        self.bar_height = height
        self.color = color
        self.radius = radius
        self.width = TEXT_W

    def wrap(self, *args):
        return self.width, self.bar_height

    def draw(self):
        self.canv.setFillColor(self.color)
        self.canv.roundRect(0, 0, self.width, self.bar_height,
                            self.radius, fill=1, stroke=0)


class SideBarBox(Flowable):
    """A box with a colored left border — for NOTE, TIP, WARNING."""
    def __init__(self, paragraphs, bg_color, border_color, label, label_color,
                 width=None):
        super().__init__()
        self.paragraphs = paragraphs
        self.bg_color = bg_color
        self.border_color = border_color
        self.label = label
        self.label_color = label_color
        self._width = width or TEXT_W
        self._height = None

    def wrap(self, avail_w, avail_h):
        self._width = min(self._width, avail_w)
        inner_w = self._width - 20
        total_h = 12  # top padding
        for p in self.paragraphs:
            w, h = p.wrap(inner_w, 10000)
            total_h += h + 4
        total_h += 8  # bottom padding
        self._height = total_h
        self.width = self._width
        return self._width, self._height

    def draw(self):
        c = self.canv
        h = self._height
        w = self._width
        # background
        c.setFillColor(self.bg_color)
        c.roundRect(0, 0, w, h, 4, fill=1, stroke=0)
        # left border bar
        c.setFillColor(self.border_color)
        c.rect(0, 0, 5, h, fill=1, stroke=0)
        # label
        c.setFillColor(self.label_color)
        c.setFont("Helvetica-Bold", 8)
        c.drawString(14, h - 12, self.label)
        # paragraphs
        y = h - 12
        for p in self.paragraphs:
            pw, ph = p.wrap(w - 20, 10000)
            y -= ph + 4
            p.drawOn(c, 14, y)


class ExerciseBox(Flowable):
    """A box for exercises with light purple background."""
    def __init__(self, title, level, body_paragraphs, solution_paragraphs,
                 styles, width=None):
        super().__init__()
        self.title = title
        self.level = level
        self.body_paras = body_paragraphs
        self.sol_paras = solution_paragraphs
        self.styles = styles
        self._width = width or TEXT_W
        self._height = None

    def wrap(self, avail_w, avail_h):
        self._width = min(self._width, avail_w)
        inner_w = self._width - 24
        total_h = 14  # top pad
        if self.title:
            p = Paragraph(
                f"Exercise: {self.title}" + (f"  <font color='#7c3aed'>[{self.level}]</font>" if self.level else ""),
                self.styles["exercise_title"]
            )
            _, h = p.wrap(inner_w, 10000)
            total_h += h + 6
        for p in self.body_paras:
            _, h = p.wrap(inner_w, 10000)
            total_h += h + 4
        if self.sol_paras:
            lbl = Paragraph("Solution:", self.styles["solution_label"])
            _, h = lbl.wrap(inner_w, 10000)
            total_h += h + 4
            for p in self.sol_paras:
                _, h = p.wrap(inner_w, 10000)
                total_h += h + 4
        total_h += 10  # bottom pad
        self._height = total_h
        self.width = self._width
        return self._width, self._height

    def draw(self):
        c = self.canv
        h = self._height
        w = self._width
        # background
        c.setFillColor(C_EX_BG)
        c.roundRect(0, 0, w, h, 6, fill=1, stroke=0)
        # top border
        c.setFillColor(C_EX_BORDER)
        c.roundRect(0, h - 5, w, 5, 3, fill=1, stroke=0)
        c.rect(0, h - 5, w, 2.5, fill=1, stroke=0)

        inner_w = w - 24
        y = h - 14

        # title
        if self.title:
            level_html = f"  <font color='#7c3aed'>[{self.level}]</font>" if self.level else ""
            p = Paragraph(f"Exercise: {self.title}{level_html}", self.styles["exercise_title"])
            pw, ph = p.wrap(inner_w, 10000)
            y -= ph
            p.drawOn(c, 12, y)
            y -= 6

        for p in self.body_paras:
            pw, ph = p.wrap(inner_w, 10000)
            y -= ph
            p.drawOn(c, 12, y)
            y -= 4

        if self.sol_paras:
            lbl = Paragraph("Solution:", self.styles["solution_label"])
            pw, ph = lbl.wrap(inner_w, 10000)
            y -= ph
            lbl.drawOn(c, 12, y)
            y -= 4
            for p in self.sol_paras:
                pw, ph = p.wrap(inner_w, 10000)
                y -= ph
                p.drawOn(c, 12, y)
                y -= 4


class CodeBlock(Flowable):
    """Monospace code block with cyan border — splittable across pages."""
    def __init__(self, lines, styles, width=None):
        super().__init__()
        self.lines = lines
        self.styles = styles
        self._width = width or TEXT_W
        self._height = None
        self._avail_w = None

    def _line_heights(self, inner_w):
        heights = []
        for line in self.lines:
            safe = line.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
            p = Paragraph(safe if safe.strip() else "&nbsp;", self.styles["code"])
            _, h = p.wrap(inner_w, 10000)
            heights.append(h)
        return heights

    def wrap(self, avail_w, avail_h):
        # Use avail_w so we never exceed the frame
        self._width = min(self._width, avail_w)
        self._avail_w = avail_w
        inner_w = self._width - 24
        total_h = 20  # top + bottom pad
        for h in self._line_heights(inner_w):
            total_h += h
        self._height = total_h
        self.width = self._width
        return self._width, self._height

    def split(self, avail_w, avail_h):
        """Split code block across pages if too tall."""
        self._width = min(self._width, avail_w)
        inner_w = self._width - 24
        heights = self._line_heights(inner_w)
        budget = avail_h - 20  # subtract padding
        # Find how many lines fit in available height
        used = 0
        cut = 0
        for i, h in enumerate(heights):
            if used + h > budget:
                break
            used += h
            cut = i + 1
        if cut == 0:
            # Nothing fits — consume remaining space and push block to next page
            return [Spacer(0, avail_h), self]
        if cut >= len(self.lines):
            # Everything fits — no split needed
            return [self]
        first = CodeBlock(self.lines[:cut], self.styles, self._width)
        rest  = CodeBlock(self.lines[cut:], self.styles, self._width)
        return [first, rest]

    def draw(self):
        c = self.canv
        h = self._height
        w = self._width
        # background
        c.setFillColor(C_CODE_BG)
        c.roundRect(0, 0, w, h, 5, fill=1, stroke=0)
        # border
        c.setStrokeColor(C_CODE_BORDER)
        c.setLineWidth(1.5)
        c.roundRect(0, 0, w, h, 5, fill=0, stroke=1)

        inner_w = w - 24
        y = h - 10
        for line in self.lines:
            safe = line.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
            p = Paragraph(safe if safe.strip() else "&nbsp;", self.styles["code"])
            pw, ph = p.wrap(inner_w, 10000)
            y -= ph
            p.drawOn(c, 12, y)

# ---------------------------------------------------------------------------
# Page template callbacks
# ---------------------------------------------------------------------------

_current_chapter = [""]

def _set_chapter(name):
    _current_chapter[0] = name

def _page_template(canvas, doc):
    canvas.saveState()
    page_num = doc.page

    # Header line
    canvas.setStrokeColor(C_HR)
    canvas.setLineWidth(0.5)
    canvas.line(MARGIN_LEFT, PAGE_H - 0.75 * inch,
                PAGE_W - MARGIN_RIGHT, PAGE_H - 0.75 * inch)

    # Left header: chapter name
    canvas.setFont("Helvetica", 8)
    canvas.setFillColor(HexColor("#6b7280"))
    chapter_text = _current_chapter[0]
    if chapter_text:
        canvas.drawString(MARGIN_LEFT, PAGE_H - 0.65 * inch, chapter_text)

    # Right header: book title
    title_text = "AEONMI \u2014 The Complete Guide"
    canvas.drawRightString(PAGE_W - MARGIN_RIGHT, PAGE_H - 0.65 * inch, title_text)

    # Footer line
    canvas.line(MARGIN_LEFT, 0.7 * inch,
                PAGE_W - MARGIN_RIGHT, 0.7 * inch)

    # Page number centered
    canvas.drawCentredString(PAGE_W / 2, 0.5 * inch, str(page_num))

    # Footer branding
    canvas.setFont("Helvetica", 7)
    canvas.setFillColor(HexColor("#9ca3af"))
    canvas.drawString(MARGIN_LEFT, 0.5 * inch, "AEONMI INC \u2014 EIN 41-4625361")
    canvas.drawRightString(PAGE_W - MARGIN_RIGHT, 0.5 * inch, "\u00a9 2026 AEONMI INC")

    canvas.restoreState()


def _cover_template(canvas, doc):
    # Full dark background
    canvas.setFillColor(C_DARK_HEADER)
    canvas.rect(0, 0, PAGE_W, PAGE_H, fill=1, stroke=0)
    # Purple accent bar at top
    canvas.setFillColor(C_PURPLE)
    canvas.rect(0, PAGE_H - 0.5 * inch, PAGE_W, 0.5 * inch, fill=1, stroke=0)
    # Cyan accent bar at bottom
    canvas.setFillColor(C_CYAN)
    canvas.rect(0, 0, PAGE_W, 0.25 * inch, fill=1, stroke=0)


def _toc_template(canvas, doc):
    _page_template(canvas, doc)

# ---------------------------------------------------------------------------
# Cover page builder
# ---------------------------------------------------------------------------

def build_cover(styles):
    elements = []
    elements.append(Spacer(1, 2.2 * inch))
    elements.append(Paragraph("AEONMI", styles["cover_title"]))
    elements.append(Spacer(1, 0.15 * inch))
    elements.append(Paragraph(
        "The Complete Guide",
        ParagraphStyle("cover_title2",
                       fontName="Helvetica",
                       fontSize=26,
                       textColor=C_PURPLE_MED,
                       leading=32,
                       alignment=TA_CENTER)
    ))
    elements.append(Spacer(1, 0.35 * inch))
    elements.append(HRFlowable(width=TEXT_W, thickness=1, color=C_PURPLE_MED,
                               spaceAfter=0.3 * inch))
    elements.append(Paragraph(
        "From First Program to Aeonmic Intelligence",
        styles["cover_subtitle"]
    ))
    elements.append(Spacer(1, 2.5 * inch))
    elements.append(Paragraph("AEONMI INC", styles["cover_meta"]))
    elements.append(Paragraph("EIN 41-4625361", styles["cover_meta"]))
    elements.append(Paragraph("April 2026", styles["cover_meta"]))
    elements.append(PageBreak())
    return elements

# ---------------------------------------------------------------------------
# TOC builder
# ---------------------------------------------------------------------------

def build_toc(toc_entries, styles):
    elements = []
    # TOC heading
    bar = ColorBar(38, C_DARK_HEADER)
    elements.append(bar)

    heading_style = ParagraphStyle(
        "toc_heading",
        fontName="Helvetica-Bold",
        fontSize=18,
        textColor=white,
        leading=24,
        alignment=TA_LEFT,
    )
    # We draw on top of the bar — use a KeepTogether with the bar's text overlay
    # Since we can't overlay, use a styled paragraph after
    elements.append(Spacer(1, 0.12 * inch))
    elements.append(Paragraph(
        '<font color="#7c3aed">TABLE OF</font> <font color="white">CONTENTS</font>',
        ParagraphStyle("toc_head2",
                       fontName="Helvetica-Bold",
                       fontSize=20,
                       textColor=C_PURPLE,
                       leading=26,
                       spaceBefore=4,
                       spaceAfter=16)
    ))

    for entry in toc_entries:
        level = entry["level"]
        text  = entry["text"]
        page  = entry.get("page", "")

        if level == "part":
            elements.append(Spacer(1, 0.1 * inch))
            elements.append(HRFlowable(width=TEXT_W, thickness=1, color=C_PURPLE,
                                       spaceAfter=4))
            elements.append(Paragraph(
                f'<b><font color="#4c1d95">{text}</font></b>',
                styles["toc_chapter"]
            ))
        elif level == "chapter":
            elements.append(Paragraph(
                f'<b>{text}</b>',
                styles["toc_chapter"]
            ))
        elif level == "section":
            elements.append(Paragraph(
                f'\u2022 {text}',
                styles["toc_section"]
            ))

    elements.append(PageBreak())
    return elements

# ---------------------------------------------------------------------------
# Text file parser
# ---------------------------------------------------------------------------

def parse_file(filepath):
    """
    Parse a textbook text file and return a list of token dicts.
    Each token has a 'type' and 'content' (and optionally 'lines' for code).
    """
    tokens = []
    if not filepath.exists():
        tokens.append({"type": "missing", "content": str(filepath)})
        return tokens

    with open(filepath, "r", encoding="utf-8") as f:
        raw = f.read()

    lines = raw.splitlines()
    i = 0
    n = len(lines)

    while i < n:
        line = lines[i]

        # --- PART title ---
        if line.startswith("# PART "):
            tokens.append({"type": "part", "content": line[2:].strip()})
            i += 1

        # --- CHAPTER heading ---
        elif line.startswith("## CHAPTER "):
            tokens.append({"type": "chapter", "content": line[3:].strip()})
            i += 1

        # --- APPENDIX heading (treated as chapter) ---
        elif re.match(r"^# APPENDIX [A-Z]:", line):
            tokens.append({"type": "chapter", "content": line[2:].strip()})
            i += 1

        # --- SECTION heading ---
        elif line.startswith("### SECTION:"):
            content = line[len("### SECTION:"):].strip()
            tokens.append({"type": "section", "content": content})
            i += 1

        # --- BODY ---
        elif line.startswith("BODY:"):
            content = line[5:].strip()
            tokens.append({"type": "body", "content": content})
            i += 1

        # --- CODE block ---
        elif line.startswith("CODE:"):
            i += 1
            # expect a ``` fence
            code_lines = []
            if i < n and lines[i].strip().startswith("```"):
                i += 1  # skip opening fence
                while i < n and not lines[i].strip().startswith("```"):
                    code_lines.append(lines[i])
                    i += 1
                if i < n:
                    i += 1  # skip closing fence
            tokens.append({"type": "code", "lines": code_lines})

        # --- TABLE_HEADER ---
        elif line.startswith("TABLE_HEADER:"):
            content = line[len("TABLE_HEADER:"):].strip()
            cols = [c.strip() for c in content.split("|")]
            tokens.append({"type": "table_header", "cols": cols})
            i += 1

        # --- TABLE_ROW ---
        elif line.startswith("TABLE_ROW:"):
            content = line[len("TABLE_ROW:"):].strip()
            cols = [c.strip() for c in content.split("|")]
            tokens.append({"type": "table_row", "cols": cols})
            i += 1

        # --- NOTE ---
        elif line.startswith("NOTE:"):
            content = line[5:].strip()
            tokens.append({"type": "note", "content": content})
            i += 1

        # --- TIP ---
        elif line.startswith("TIP:"):
            content = line[4:].strip()
            tokens.append({"type": "tip", "content": content})
            i += 1

        # --- WARNING ---
        elif line.startswith("WARNING:"):
            content = line[8:].strip()
            tokens.append({"type": "warning", "content": content})
            i += 1

        # --- EXERCISE_TITLE ---
        elif line.startswith("EXERCISE_TITLE:"):
            ex_title = line[len("EXERCISE_TITLE:"):].strip()
            ex_level = ""
            ex_body_lines = []
            ex_sol_lines = []
            i += 1
            while i < n:
                l2 = lines[i]
                if l2.startswith("EXERCISE_LEVEL:"):
                    ex_level = l2[len("EXERCISE_LEVEL:"):].strip()
                    i += 1
                elif l2.startswith("EXERCISE_BODY:"):
                    ex_body_lines.append(l2[len("EXERCISE_BODY:"):].strip())
                    i += 1
                elif l2.startswith("EXERCISE_SOLUTION:"):
                    i += 1
                    if i < n and lines[i].strip().startswith("```"):
                        i += 1
                        while i < n and not lines[i].strip().startswith("```"):
                            ex_sol_lines.append(lines[i])
                            i += 1
                        if i < n:
                            i += 1
                else:
                    break
            tokens.append({
                "type": "exercise",
                "title": ex_title,
                "level": ex_level,
                "body": ex_body_lines,
                "solution": ex_sol_lines,
            })

        # --- Blank lines: skip ---
        elif line.strip() == "" or line.strip() == "---":
            i += 1

        # --- Unknown: treat as body text ---
        else:
            stripped = line.strip()
            if stripped:
                tokens.append({"type": "body", "content": stripped})
            i += 1

    return tokens

# ---------------------------------------------------------------------------
# Flush accumulated table rows
# ---------------------------------------------------------------------------

def flush_table(header_cols, row_data, styles):
    """Convert accumulated table header + rows into a ReportLab Table."""
    if not header_cols:
        return []

    col_count = len(header_cols)
    # Compute column widths: equal distribution
    col_w = TEXT_W / col_count

    table_data = []

    # Header row
    header_cells = [
        Paragraph(c, styles["table_header"]) for c in header_cols
    ]
    table_data.append(header_cells)

    # Data rows
    for row in row_data:
        # Pad or trim to col_count
        padded = (row + [""] * col_count)[:col_count]
        cells = [Paragraph(c, styles["table_cell"]) for c in padded]
        table_data.append(cells)

    tbl = Table(table_data, colWidths=[col_w] * col_count, repeatRows=1)

    row_colors = []
    for idx in range(1, len(table_data)):
        bg = C_TABLE_ROW_A if idx % 2 == 1 else C_TABLE_ROW_B
        row_colors.append(("BACKGROUND", (0, idx), (-1, idx), bg))

    tbl.setStyle(TableStyle([
        ("BACKGROUND",  (0, 0), (-1, 0), C_TABLE_HEADER),
        ("TEXTCOLOR",   (0, 0), (-1, 0), white),
        ("FONTNAME",    (0, 0), (-1, 0), "Helvetica-Bold"),
        ("FONTSIZE",    (0, 0), (-1, 0), 8.5),
        ("ROWBACKGROUND", (0, 1), (-1, -1), [C_TABLE_ROW_A, C_TABLE_ROW_B]),
        ("GRID",        (0, 0), (-1, -1), 0.4, C_HR),
        ("TOPPADDING",  (0, 0), (-1, -1), 5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
        ("LEFTPADDING", (0, 0), (-1, -1), 6),
        ("RIGHTPADDING", (0, 0), (-1, -1), 6),
        ("VALIGN",      (0, 0), (-1, -1), "TOP"),
        ("ROWBACKGROUND", (0, 1), (-1, -1), [C_TABLE_ROW_A, C_TABLE_ROW_B]),
        *row_colors,
    ]))

    return [Spacer(1, 6), tbl, Spacer(1, 10)]

# ---------------------------------------------------------------------------
# Build chapter heading block
# ---------------------------------------------------------------------------

def build_chapter_block(text, styles, is_appendix=False):
    """Return flowables for a chapter heading."""
    elements = []
    elements.append(PageBreak())

    # Dark bar
    bar_h = 56
    elements.append(ColorBar(bar_h, C_DARK_HEADER, radius=0))

    # Chapter text overlaid — we use a Paragraph with dark background
    prefix = "APPENDIX" if is_appendix else "CHAPTER"
    # We can't truly overlay text on the bar with platypus flowables,
    # so we use a table to achieve the effect
    heading_para = Paragraph(
        f'<font color="white">{text}</font>',
        styles["chapter"]
    )
    # Purple accent strip below the bar
    accent = ColorBar(6, C_PURPLE, radius=0)

    # Use a single-cell table that acts as the heading box
    heading_tbl = Table(
        [[heading_para]],
        colWidths=[TEXT_W],
    )
    heading_tbl.setStyle(TableStyle([
        ("BACKGROUND",    (0, 0), (-1, -1), C_DARK_HEADER),
        ("LEFTPADDING",   (0, 0), (-1, -1), 14),
        ("RIGHTPADDING",  (0, 0), (-1, -1), 14),
        ("TOPPADDING",    (0, 0), (-1, -1), 14),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 14),
        ("TEXTCOLOR",     (0, 0), (-1, -1), white),
    ]))

    # Remove the plain ColorBar and replace with the table
    elements.pop()  # remove the ColorBar we added
    elements.append(heading_tbl)
    elements.append(accent)
    elements.append(Spacer(1, 0.2 * inch))
    return elements


def build_part_block(text, styles):
    """Return flowables for a part title page."""
    elements = []
    elements.append(PageBreak())
    # Full-page-ish centered block
    elements.append(Spacer(1, 1.8 * inch))

    part_tbl = Table(
        [[Paragraph(f'<font color="white">{text}</font>', styles["part"])]],
        colWidths=[TEXT_W],
    )
    part_tbl.setStyle(TableStyle([
        ("BACKGROUND",    (0, 0), (-1, -1), C_DARK_HEADER),
        ("LEFTPADDING",   (0, 0), (-1, -1), 30),
        ("RIGHTPADDING",  (0, 0), (-1, -1), 30),
        ("TOPPADDING",    (0, 0), (-1, -1), 40),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 40),
        ("TEXTCOLOR",     (0, 0), (-1, -1), white),
        ("ALIGN",         (0, 0), (-1, -1), "CENTER"),
    ]))
    elements.append(part_tbl)
    elements.append(Spacer(1, 0.3 * inch))
    elements.append(HRFlowable(width=TEXT_W, thickness=2, color=C_PURPLE, spaceAfter=10))
    elements.append(PageBreak())
    return elements

# ---------------------------------------------------------------------------
# Escape helpers
# ---------------------------------------------------------------------------

SAFE_CHARS = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
}

def xml_escape(text):
    for ch, rep in SAFE_CHARS.items():
        text = text.replace(ch, rep)
    return text

def safe_body(text, styles):
    """Return a body Paragraph, handling markdown-ish bold and code spans."""
    # Bold: **text** -> <b>text</b>
    text = xml_escape(text)
    text = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", text)
    # Inline code: `text` -> monospace
    text = re.sub(r"`([^`]+)`",
                  r'<font name="Courier" size="9">\1</font>', text)
    return Paragraph(text, styles["body"])

# ---------------------------------------------------------------------------
# Main token → flowable conversion
# ---------------------------------------------------------------------------

def tokens_to_flowables(tokens, styles, toc_entries):
    """Convert parsed tokens into ReportLab flowables, collecting TOC entries."""
    elements = []

    # Table accumulation
    pending_header = None
    pending_rows = []

    def flush_pending_table():
        nonlocal pending_header, pending_rows
        if pending_header is not None:
            elements.extend(flush_table(pending_header, pending_rows, styles))
            pending_header = None
            pending_rows = []

    for tok in tokens:
        t = tok["type"]

        if t == "missing":
            elements.append(Paragraph(
                f'<font color="red">FILE NOT FOUND: {xml_escape(tok["content"])}</font>',
                styles["body"]
            ))
            continue

        # --- PART ---
        if t == "part":
            flush_pending_table()
            text = tok["content"]
            toc_entries.append({"level": "part", "text": text})
            elements.extend(build_part_block(text, styles))
            _set_chapter("")

        # --- CHAPTER ---
        elif t == "chapter":
            flush_pending_table()
            text = tok["content"]
            toc_entries.append({"level": "chapter", "text": text})
            is_app = text.upper().startswith("APPENDIX")
            elements.extend(build_chapter_block(text, styles, is_appendix=is_app))
            _set_chapter(text)

        # --- SECTION ---
        elif t == "section":
            flush_pending_table()
            text = tok["content"]
            toc_entries.append({"level": "section", "text": text})
            elements.append(Spacer(1, 4))
            elements.append(Paragraph(xml_escape(text), styles["section"]))
            elements.append(HRFlowable(width=TEXT_W * 0.4, thickness=1,
                                       color=C_PURPLE_MED, spaceAfter=4))

        # --- BODY ---
        elif t == "body":
            flush_pending_table()
            elements.append(safe_body(tok["content"], styles))

        # --- CODE ---
        elif t == "code":
            flush_pending_table()
            code_lines = tok["lines"]
            block = CodeBlock(code_lines, styles)
            elements.append(Spacer(1, 4))
            elements.append(block)
            elements.append(Spacer(1, 8))

        # --- TABLE_HEADER ---
        elif t == "table_header":
            flush_pending_table()
            pending_header = tok["cols"]
            pending_rows = []

        # --- TABLE_ROW ---
        elif t == "table_row":
            if pending_header is not None:
                pending_rows.append(tok["cols"])
            else:
                # Orphan row — skip
                pass

        # --- NOTE ---
        elif t == "note":
            flush_pending_table()
            p = Paragraph(f"<b>NOTE:</b> {xml_escape(tok['content'])}", styles["note"])
            box = SideBarBox([p], C_NOTE_BG, C_NOTE_BORDER, "NOTE", C_NOTE_BORDER)
            elements.append(Spacer(1, 4))
            elements.append(box)
            elements.append(Spacer(1, 8))

        # --- TIP ---
        elif t == "tip":
            flush_pending_table()
            p = Paragraph(f"<b>TIP:</b> {xml_escape(tok['content'])}", styles["tip"])
            box = SideBarBox([p], C_TIP_BG, C_TIP_BORDER, "TIP", C_TIP_BORDER)
            elements.append(Spacer(1, 4))
            elements.append(box)
            elements.append(Spacer(1, 8))

        # --- WARNING ---
        elif t == "warning":
            flush_pending_table()
            p = Paragraph(f"<b>WARNING:</b> {xml_escape(tok['content'])}", styles["warning"])
            box = SideBarBox([p], C_WARN_BG, C_WARN_BORDER, "WARNING", C_WARN_BORDER)
            elements.append(Spacer(1, 4))
            elements.append(box)
            elements.append(Spacer(1, 8))

        # --- EXERCISE ---
        elif t == "exercise":
            flush_pending_table()
            body_paras = [
                Paragraph(xml_escape(bl), styles["exercise_body"])
                for bl in tok["body"]
            ]
            sol_paras = []
            if tok["solution"]:
                for sl in tok["solution"]:
                    safe = sl.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
                    sol_paras.append(Paragraph(
                        safe if safe.strip() else "&nbsp;",
                        styles["code"]
                    ))
            box = ExerciseBox(
                tok["title"], tok["level"],
                body_paras, sol_paras, styles
            )
            elements.append(Spacer(1, 6))
            elements.append(box)
            elements.append(Spacer(1, 10))

    flush_pending_table()
    return elements

# ---------------------------------------------------------------------------
# Document builder
# ---------------------------------------------------------------------------

def build_pdf():
    print("AEONMI Textbook PDF Compiler")
    print("=" * 45)

    # Check input files
    missing = [f for f in INPUT_FILES if not f.exists()]
    if missing:
        print("\nWARNING: The following input files were not found:")
        for m in missing:
            print(f"  MISSING: {m.name}")
        print("  These sections will show a placeholder in the PDF.")
        print()

    styles = build_styles()
    toc_entries = []
    all_tokens = []

    print("Parsing input files...")
    for fpath in INPUT_FILES:
        if fpath.exists():
            print(f"  [OK]  {fpath.name}")
        else:
            print(f"  [--]  {fpath.name}  (not found, will show placeholder)")
        tokens = parse_file(fpath)
        all_tokens.extend(tokens)

    print(f"  Total tokens parsed: {len(all_tokens)}")
    print()

    # First pass: collect TOC (we do this as part of token_to_flowables)
    print("Building flowables...")
    content_elements = tokens_to_flowables(all_tokens, styles, toc_entries)

    print(f"  TOC entries: {len(toc_entries)}")
    print(f"  Content flowables: {len(content_elements)}")
    print()

    # Assemble final document
    doc = SimpleDocTemplate(
        str(OUTPUT_PDF),
        pagesize=letter,
        leftMargin=MARGIN_LEFT,
        rightMargin=MARGIN_RIGHT,
        topMargin=MARGIN_TOP,
        bottomMargin=MARGIN_BOTTOM,
        title="AEONMI — The Complete Guide",
        author="Mother AI & Warren Williams — AEONMI INC",
        subject="Aeonmi Language Reference and Tutorial",
        creator="AEONMI Textbook Compiler v1.0",
    )

    all_elements = []

    # Cover page (uses cover template callback)
    all_elements.extend(build_cover(styles))

    # TOC page
    all_elements.extend(build_toc(toc_entries, styles))

    # Main content
    all_elements.extend(content_elements)

    print(f"Writing PDF to: {OUTPUT_PDF}")

    # Build with page templates
    # We use onFirstPage for cover (no header/footer), onLaterPages for rest
    doc.build(
        all_elements,
        onFirstPage=_cover_template,
        onLaterPages=_page_template,
    )

    size_mb = OUTPUT_PDF.stat().st_size / (1024 * 1024)
    print(f"Done! PDF size: {size_mb:.2f} MB")
    print(f"Output: {OUTPUT_PDF}")

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    build_pdf()
