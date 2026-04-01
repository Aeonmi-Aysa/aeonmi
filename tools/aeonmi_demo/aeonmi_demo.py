"""
Aeonmi Language Demo v2.0  —  AI-native language showcase
Press ESC to exit.  Left/Right arrows to navigate scenes.
"""

import tkinter as tk
import random, sys

# ── Palette ───────────────────────────────────────────────────────────────────
BG        = "#050a0e"
PANEL_BG  = "#080f14"
GREEN     = "#00ff88"
CYAN      = "#00d4ff"
PURPLE    = "#bf00ff"
AMBER     = "#ffbb00"
DIM_G     = "#003a1a"
DIM_C     = "#002233"
GRAY      = "#3a5a4a"
WHITE     = "#d8f8e8"

# ── Demo Scenes ───────────────────────────────────────────────────────────────
SCENES = [
    {
        "title": "HELLO  WORLD",
        "sub":   "function definition  \xb7  string concat  \xb7  print",
        "code": [
            "\u2395 Aeonmi \u2014 AI-native language",
            "",
            "\u25ef greet\u27e8name\u27e9 {",
            '    let msg = "Hello, " + name;',
            "    return msg;",
            "}",
            "",
            'print(greet("Quantum World"));',
        ],
        "out": [
            "AEONMI-VM  \u25b6  Lexer    \u2192 12 tokens",
            "AEONMI-VM  \u25b6  Parser   \u2192 AST OK",
            "AEONMI-VM  \u25b6  IR       \u2192 complete",
            "AEONMI-VM  \u25b6  Execute  \u2192",
            "",
            "Hello, Quantum World",
        ],
        "accent": GREEN,
    },
    {
        "title": "MATH  STDLIB",
        "sub":   "PI  \xb7  E  \xb7  TAU  \xb7  trig  \xb7  clamp  \xb7  sqrt",
        "code": [
            "\u2395 80+ math builtins \u2014 zero imports",
            "",
            "print(sin(PI / 2));",
            "print(sqrt(144));",
            "print(clamp(1.7, 0.0, 1.0));",
            "print(round(pow(2, 10)));",
        ],
        "out": [
            "AEONMI-VM  \u25b6  sin(\u03c0/2)  \u2192 1",
            "AEONMI-VM  \u25b6  \u221a144     \u2192 12",
            "AEONMI-VM  \u25b6  clamp    \u2192 1",
            "AEONMI-VM  \u25b6  2\u00b910      \u2192 1024",
            "",
            "1  /  12  /  1  /  1024",
        ],
        "accent": CYAN,
    },
    {
        "title": "MAP  \xb7  FILTER  \xb7  REDUCE",
        "sub":   "higher-order functions  \xb7  functional pipelines",
        "code": [
            "\u25ef dbl\u27e8x\u27e9   { return x * 2; }",
            "\u25ef big\u27e8x\u27e9   { return x > 4; }",
            "\u25ef add\u27e8a,b\u27e9 { return a + b; }",
            "",
            "let r = map([1,2,3,4,5], dbl);",
            "let f = filter(r, big);",
            "let s = reduce(r, add);",
        ],
        "out": [
            "AEONMI-VM  \u25b6  map    \u2192 [2,4,6,8,10]",
            "AEONMI-VM  \u25b6  filter \u2192 [6,8,10]",
            "AEONMI-VM  \u25b6  reduce \u2192 30",
            "",
            "[2, 4, 6, 8, 10]",
            "[6, 8, 10]",
            "30",
        ],
        "accent": PURPLE,
    },
    {
        "title": "OBJECTS  \xb7  JSON",
        "sub":   "dynamic objects  \xb7  to_json  \xb7  parse_json",
        "code": [
            "\u2395 Object CRUD + JSON roundtrip",
            "",
            "let o = object();",
            'o = set_key(o, "lang",   "Aeonmi");',
            'o = set_key(o, "ver",    2);',
            'o = set_key(o, "status", "LIVE");',
            "",
            "print(to_json(o));",
        ],
        "out": [
            "AEONMI-VM  \u25b6  object()  \u2192 {}",
            "AEONMI-VM  \u25b6  3 keys inserted",
            "AEONMI-VM  \u25b6  to_json   \u2192 serialized",
            "",
            '{"lang":"Aeonmi",',
            ' "ver":2,"status":"LIVE"}',
        ],
        "accent": AMBER,
    },
    {
        "title": "SWARM  AGENTS",
        "sub":   "multi-agent colony  \xb7  parallel execution  \xb7  coordination",
        "code": [
            "\u2395 AI swarm \u2014 3-agent colony",
            "",
            "\u25ef spawn\u27e8id\u27e9 {",
            '    let tag = "AGENT-" + toString(id);',
            '    return tag + " :: ONLINE";',
            "}",
            "",
            "map(range(3), spawn);",
        ],
        "out": [
            "AEONMI-VM  \u25b6  Spawning swarm...",
            "AEONMI-VM  \u25b6  AGENT-0 :: ONLINE",
            "AEONMI-VM  \u25b6  AGENT-1 :: ONLINE",
            "AEONMI-VM  \u25b6  AGENT-2 :: ONLINE",
            "AEONMI-VM  \u25b6  3/3 agents active",
        ],
        "accent": GREEN,
    },
]

TICKER = (
    "  \u25c8  AEONMI v2.0   \u25c8  80+ NATIVE BUILTINS   \u25c8  AI-NATIVE LANGUAGE   "
    "\u25c8  GENESIS DENSITY OPERATORS  \u25ef\u27e8\u27e9  \u25c8  ZERO IMPORTS   "
    "\u25c8  LEXER \u2192 PARSER \u2192 AST \u2192 IR \u2192 VM   \u25c8  MAP FILTER REDUCE SORT UNIQUE   "
    "\u25c8  OBJECTS + JSON ROUNDTRIP   \u25c8  MULTI-AGENT SWARM READY   \u25c8  SHARD-V2 ECOSYSTEM   "
)


# ── Matrix Rain ───────────────────────────────────────────────────────────────
class MatrixRain:
    CHARS = "01\u30A2\u30A4\u30A6\u30A8\u30AABCDEF!@#$<>{}"

    def __init__(self, canvas, w, h):
        self.canvas = canvas
        self.w, self.h = w, h
        self.cols = w // 16
        self.drops = [random.randint(-50, 0) for _ in range(self.cols)]

    def step(self):
        c = self.canvas
        for i in range(self.cols):
            y_px = self.drops[i] * 16
            x_px = i * 16 + 8
            ch = random.choice(self.CHARS)
            col = "#00ff88" if random.random() > 0.85 else "#003311"
            tag = f"r{i}"
            c.delete(tag)
            if -5 < y_px < self.h + 10:
                c.create_text(x_px, y_px, text=ch, fill=col,
                              font=("Courier", 8), tags=tag)
            self.drops[i] += 1
            if self.drops[i] * 16 > self.h + 60 and random.random() > 0.97:
                self.drops[i] = random.randint(-40, -5)


# ── Main App ──────────────────────────────────────────────────────────────────
class AeonmiDemo(tk.Tk):
    W, H = 1100, 660
    TYPE_MS  = 38    # ms per character
    OUT_MS   = 110   # ms per output line
    HOLD_MS  = 4500  # ms to hold completed scene
    RAIN_MS  = 90    # ms per rain frame
    TICK_MS  = 55    # ms per ticker step

    def __init__(self):
        super().__init__()
        self.title("Aeonmi Language Demo  v2.0")
        self.configure(bg=BG)
        self.resizable(False, False)
        sw = self.winfo_screenwidth()
        sh = self.winfo_screenheight()
        self.geometry(f"{self.W}x{self.H}+{(sw-self.W)//2}+{(sh-self.H)//2}")

        self._scene  = 0
        self._jobs   = []
        self._tick_i = 0
        self._gdir   = 1
        self._gval   = 0.0

        self._build_ui()
        self.bind("<Escape>", lambda _: self.destroy())
        self.bind("<Right>",  lambda _: self._jump(1))
        self.bind("<Left>",   lambda _: self._jump(-1))
        self.protocol("WM_DELETE_WINDOW", self.destroy)

        self._sched(200, self._rain_init)
        self._sched(300, self._start_scene)
        self._sched(80,  self._glow_tick)
        self._sched(60,  self._ticker_tick)

    # ── Helpers ───────────────────────────────────────────────────────────────
    def _sched(self, ms, fn):
        j = self.after(ms, fn)
        self._jobs.append(j)
        return j

    def _cancel_all(self):
        for j in self._jobs:
            try: self.after_cancel(j)
            except Exception: pass
        self._jobs.clear()

    def _jump(self, d):
        self._cancel_all()
        self._scene = (self._scene + d) % len(SCENES)
        self._sched(0, self._start_scene)
        self._sched(80, self._rain_init)
        self._sched(60, self._ticker_tick)
        self._sched(80, self._glow_tick)

    # ── UI Build ──────────────────────────────────────────────────────────────
    def _build_ui(self):
        W, H = self.W, self.H

        # Rain canvas (background)
        self.rain_cv = tk.Canvas(self, width=W, height=H, bg=BG,
                                  highlightthickness=0)
        self.rain_cv.place(x=0, y=0)

        # Header
        self.hdr_cv = tk.Canvas(self, width=W, height=72, bg=BG,
                                 highlightthickness=0)
        self.hdr_cv.place(x=0, y=0)
        self._draw_header()

        # Title bar
        self.tb = tk.Frame(self, bg=PANEL_BG)
        self.tb.place(x=20, y=80, width=W-40, height=36)
        self.lbl_title = tk.Label(self.tb, text="", bg=PANEL_BG,
                                   fg=GREEN, font=("Courier", 13, "bold"), anchor="w")
        self.lbl_title.place(x=10, y=7)
        self.lbl_sub = tk.Label(self.tb, text="", bg=PANEL_BG,
                                 fg=GRAY, font=("Courier", 8))
        self.lbl_sub.place(relx=1.0, x=-10, y=12, anchor="e")
        self.lbl_num = tk.Label(self.tb, text="", bg=PANEL_BG,
                                 fg=AMBER, font=("Courier", 10))
        self.lbl_num.place(relx=1.0, x=-130, y=9, anchor="e")

        # Code panel (left)
        self.code_frame = tk.Frame(self, bg=DIM_G, bd=1)
        self.code_frame.place(x=20, y=126, width=530, height=420)
        tk.Label(self.code_frame, text=" CODE ", bg=DIM_G, fg=GREEN,
                 font=("Courier", 8, "bold")).place(x=6, y=0)
        self.code_box = tk.Text(
            self.code_frame, bg=PANEL_BG, fg=GREEN,
            font=("Courier", 11), bd=0, highlightthickness=0,
            insertbackground=GREEN, state="disabled", wrap="none",
            selectbackground=DIM_G)
        self.code_box.place(x=1, y=16, width=528, height=402)
        self.code_box.tag_config("cmt",  foreground="#2a5a2a")
        self.code_box.tag_config("kw",   foreground=CYAN)
        self.code_box.tag_config("str",  foreground=AMBER)
        self.code_box.tag_config("sym",  foreground=PURPLE)

        # Output panel (right)
        self.out_frame = tk.Frame(self, bg=DIM_C, bd=1)
        self.out_frame.place(x=560, y=126, width=520, height=420)
        tk.Label(self.out_frame, text=" VM OUTPUT ", bg=DIM_C, fg=CYAN,
                 font=("Courier", 8, "bold")).place(x=6, y=0)
        self.out_box = tk.Text(
            self.out_frame, bg=PANEL_BG, fg=CYAN,
            font=("Courier", 11), bd=0, highlightthickness=0,
            state="disabled", wrap="none",
            selectbackground=DIM_C)
        self.out_box.place(x=1, y=16, width=518, height=402)
        self.out_box.tag_config("vm",  foreground="#005566")
        self.out_box.tag_config("res", foreground=WHITE)

        # Dots
        self.dot_cv = tk.Canvas(self, width=W-40, height=18, bg=BG,
                                 highlightthickness=0)
        self.dot_cv.place(x=20, y=553)

        # Ticker
        tk.Frame(self, bg=GREEN, height=1).place(x=0, y=578, width=W)
        self.tick_lbl = tk.Label(self, text="", bg="#000a04", fg="#006633",
                                  font=("Courier", 9), anchor="w")
        self.tick_lbl.place(x=0, y=579, width=W, height=22)

        # Footer
        tk.Frame(self, bg="#001a08", height=1).place(x=0, y=601, width=W)
        tk.Label(self, text=(
            "\u25c8  ESC to exit   \u2190 \u2192 to navigate   "
            "AEONMI v2.0  \u25c8  AI-NATIVE LANGUAGE  \u25c8  SHARD-V2  \u25c8"),
            bg=BG, fg=GRAY, font=("Courier", 8)).place(x=0, y=606, width=W)

    def _draw_header(self):
        c = self.hdr_cv
        c.delete("all")
        W = self.W
        for i in range(W):
            t = i / W
            g = int(200 * max(0, 1 - abs(t - 0.5) * 2.2))
            c.create_line(i, 68, i, 72, fill=f"#00{g:02x}44")
        c.create_text(W//2, 34, text="  A E O N M I  ",
                      fill=GREEN, font=("Courier", 28, "bold"), anchor="center", tags="glow")
        c.create_text(W//2, 58, text="A I - N A T I V E   P R O G R A M M I N G   L A N G U A G E",
                      fill="#2a4a2a", font=("Courier", 9), anchor="center")
        c.create_text(28, 35, text="[\u25c9]", fill=PURPLE, font=("Courier", 14))
        c.create_text(W-28, 35, text="[\u25c9]", fill=PURPLE, font=("Courier", 14))
        c.create_text(88, 18, text="v2.0", fill=AMBER, font=("Courier", 9))
        c.create_text(W-88, 18, text="SHARD-V2", fill=AMBER, font=("Courier", 9))

    def _draw_dots(self):
        c = self.dot_cv
        c.delete("all")
        n = len(SCENES)
        sp = 22
        ox = (self.W - 40 - n * sp) // 2
        for i in range(n):
            x = ox + i * sp + 11
            col = SCENES[i]["accent"] if i == self._scene else GRAY
            r = 6 if i == self._scene else 4
            c.create_oval(x-r, 9-r, x+r, 9+r, fill=col, outline=col)

    # ── Animations ────────────────────────────────────────────────────────────
    def _rain_init(self):
        self.rain = MatrixRain(self.rain_cv, self.W, self.H)
        self._rain_step()

    def _rain_step(self):
        self.rain.step()
        self._sched(self.RAIN_MS, self._rain_step)

    def _glow_tick(self):
        self._gval += 0.05 * self._gdir
        if self._gval >= 1.0: self._gval = 1.0; self._gdir = -1
        elif self._gval <= 0.0: self._gval = 0.0; self._gdir = 1
        v = int(160 + self._gval * 95)
        try:
            self.hdr_cv.itemconfig("glow", fill=f"#00{v:02x}66")
        except Exception:
            pass
        self._sched(55, self._glow_tick)

    def _ticker_tick(self):
        full = TICKER * 3
        vis = 115
        chunk = full[self._tick_i: self._tick_i + vis]
        self.tick_lbl.config(text=chunk)
        self._tick_i = (self._tick_i + 1) % len(TICKER)
        self._sched(self.TICK_MS, self._ticker_tick)

    # ── Scene runner ──────────────────────────────────────────────────────────
    def _start_scene(self):
        s = SCENES[self._scene]
        acc = s["accent"]

        self.lbl_title.config(text=f"  {s['title']}", fg=acc)
        self.lbl_sub.config(text=s["sub"])
        self.lbl_num.config(text=f"{self._scene+1}/{len(SCENES)}")
        self._draw_dots()

        # Clear both panels
        for box in (self.code_box, self.out_box):
            box.config(state="normal")
            box.delete("1.0", "end")
            box.config(state="disabled")

        # Start typing code lines
        self._type_lines(s["code"], 0, lambda: self._emit_output(s["out"], 0))

    def _type_lines(self, lines, idx, done):
        """Type lines[idx] char by char, then recurse."""
        if idx >= len(lines):
            done()
            return
        self._type_chars(lines[idx], 0,
                         lambda: self._type_lines(lines, idx + 1, done))

    def _type_chars(self, line, pos, done):
        """Type one character of `line` at position `pos`."""
        box = self.code_box
        box.config(state="normal")
        # Rewrite the current (last) line
        last = box.index("end-1c linestart")
        box.delete(last, "end")
        self._colorize(box, line[:pos])
        if pos < len(line):
            # blinking cursor stub
            box.insert("end", "\u2588", "sym")
        box.see("end")
        box.config(state="disabled")

        if pos <= len(line):
            self._sched(self.TYPE_MS,
                        lambda: self._type_chars(line, pos + 1, done))
        else:
            # Finalize line (remove cursor, add newline)
            box.config(state="normal")
            last = box.index("end-1c linestart")
            box.delete(last, "end")
            self._colorize(box, line)
            box.insert("end", "\n")
            box.see("end")
            box.config(state="disabled")
            done()

    def _colorize(self, widget, text):
        """Insert `text` into widget with syntax coloring."""
        KW = {"let", "return", "print", "if", "else"}
        i = 0
        while i < len(text):
            ch = text[i]
            if ch == '"':
                end = text.find('"', i + 1)
                if end == -1: end = len(text) - 1
                widget.insert("end", text[i:end+1], "str")
                i = end + 1
            elif ch == "\u2395":
                widget.insert("end", text[i:], "cmt")
                break
            elif ch in "\u25ef\u27e8\u27e9\u2192\u25b6\u25c8\u2190\u221a\u00b9\u00b0":
                widget.insert("end", ch, "sym")
                i += 1
            else:
                matched = False
                for kw in KW:
                    end_i = i + len(kw)
                    if text[i:end_i] == kw and (end_i >= len(text) or not text[end_i].isalnum()):
                        widget.insert("end", kw, "kw")
                        i = end_i
                        matched = True
                        break
                if not matched:
                    widget.insert("end", ch)
                    i += 1

    def _emit_output(self, lines, idx):
        """Emit output lines one by one."""
        if idx >= len(lines):
            self._sched(self.HOLD_MS, self._advance_scene)
            return
        line = lines[idx]
        box = self.out_box
        box.config(state="normal")
        tag = "vm" if line.startswith("AEONMI-VM") else ("res" if line else "")
        box.insert("end", line + "\n", tag)
        box.see("end")
        box.config(state="disabled")
        self._sched(self.OUT_MS, lambda: self._emit_output(lines, idx + 1))

    def _advance_scene(self):
        self._scene = (self._scene + 1) % len(SCENES)
        self._start_scene()


def main():
    app = AeonmiDemo()
    app.mainloop()


if __name__ == "__main__":
    main()
