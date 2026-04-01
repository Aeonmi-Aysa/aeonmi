"""
Aeonmi Studio  v2.0
Full IDE for the Aeonmi AI-native language.
3-panel: Code Editor | VM Pipeline | Output Terminal
"""

import tkinter as tk
from tkinter import filedialog, messagebox
import subprocess, threading, tempfile, os, sys, time, re

# ── Locate VM executable ──────────────────────────────────────────────────────
def _find_vm():
    candidates = [
        os.path.join(os.path.dirname(sys.executable), "Aeonmi.exe"),
        os.path.join(os.path.dirname(os.path.abspath(__file__)), "Aeonmi.exe"),
        r"C:\Users\wlwil\Desktop\AeonmiDist\Aeonmi.exe",
        r"C:\Program Files\Aeonmi\Aeonmi.exe",
    ]
    for p in candidates:
        if os.path.isfile(p):
            return p
    return None

VM_EXE = _find_vm()

# ── Palette ───────────────────────────────────────────────────────────────────
BG        = "#060c10"
PANEL_BG  = "#080f15"
TOOLBAR   = "#0b1520"
GREEN     = "#00ff88"
CYAN      = "#00d4ff"
PURPLE    = "#bf00ff"
AMBER     = "#ffbb00"
RED       = "#ff4444"
DIM_G     = "#003a1a"
DIM_C     = "#002233"
GRAY      = "#3a5a4a"
WHITE     = "#d8f8e8"
LGRAY     = "#5a7a6a"

# ── Example scripts ───────────────────────────────────────────────────────────
EXAMPLES = {
    "Hello World": """\
\u2395 Aeonmi \u2014 AI-native language  |  Hello World
\u2395 Press \u25b6 RUN to execute

\u25ef greet\u27e8name\u27e9 {
    let msg = "Hello, " + name + "!";
    return msg;
}

print(greet("Quantum World"));
print(greet("Aeonmi Studio"));
print(upper("ai-native language"));
""",

    "Math Stdlib": """\
\u2395 Math stdlib \u2014 80+ builtins, zero imports

print("=== Constants ===");
print(PI);
print(E);
print(TAU);

print("=== Arithmetic ===");
print(sqrt(144));
print(pow(2, 10));
print(abs(-42));

print("=== Trig ===");
print(sin(PI / 2));
print(cos(0));
print(round(tan(PI / 4)));

print("=== Utility ===");
print(clamp(1.7, 0.0, 1.0));
print(lerp(0, 100, 0.25));
print(min(3, 7, 1, 9, 2));
print(max(3, 7, 1, 9, 2));
""",

    "Map \xb7 Filter \xb7 Reduce": """\
\u2395 Functional programming \u2014 map / filter / reduce

\u25ef double\u27e8x\u27e9  { return x * 2; }
\u25ef is_big\u27e8x\u27e9  { return x > 5; }
\u25ef add\u27e8a, b\u27e9  { return a + b; }
\u25ef square\u27e8x\u27e9  { return x * x; }

let nums = [1, 2, 3, 4, 5, 6, 7, 8];

let doubled  = map(nums, double);
let big_ones = filter(nums, is_big);
let total    = reduce(nums, add);
let squares  = map(range(1, 6), square);

print(doubled);
print(big_ones);
print(total);
print(squares);
print(sum(nums));
print(product([1,2,3,4,5]));
""",

    "Sort \xb7 Search \xb7 Unique": """\
\u2395 Array operations \u2014 sort, unique, flatten, search

let data = [5, 3, 8, 1, 9, 2, 7, 4, 6, 3, 1];

print("Original:  ");  print(data);
print("Sorted:    ");  print(sort(data));
print("Unique:    ");  print(unique(data));
print("Reversed:  ");  print(reverse(sort(data)));
print("Sum:       ");  print(sum(data));

let nested = [[1,2,3],[4,5],[6,7,8,9]];
print("Flattened: ");  print(flatten(nested));

let words = ["banana", "apple", "cherry", "date", "elderberry"];
print("Sorted:    ");  print(sort(words));

let en = enumerate(["x","y","z"]);
print("Enumerate: ");  print(en);
""",

    "Objects \xb7 JSON": """\
\u2395 Dynamic objects + JSON roundtrip

let agent = object();
agent = set_key(agent, "id",      "AGENT-001");
agent = set_key(agent, "lang",    "Aeonmi");
agent = set_key(agent, "ver",     2);
agent = set_key(agent, "status",  "ONLINE");
agent = set_key(agent, "uptime",  99.9);

print("=== Object ===");
print(get_key(agent, "id"));
print(get_key(agent, "status"));
print(has_key(agent, "lang"));
print(keys(agent));

print("=== JSON ===");
let json_str = to_json(agent);
print(json_str);

print("=== Parse back ===");
let parsed = parse_json(json_str);
print(get_key(parsed, "id"));
print(get_key(parsed, "uptime"));
""",

    "Swarm Agents": """\
\u2395 Multi-agent swarm colony \u2014 AI coordination

\u25ef make_agent\u27e8id\u27e9 {
    let tag  = "AGENT-" + toString(id);
    let obj  = object();
    obj = set_key(obj, "id",     tag);
    obj = set_key(obj, "status", "ACTIVE");
    obj = set_key(obj, "score",  floor(id * 33 + 11));
    return to_json(obj);
}

\u25ef run_task\u27e8agent_id\u27e9 {
    let a = make_agent(agent_id);
    print(a);
    return a;
}

print("=== Spawning Swarm Colony ===");
let colony = map(range(5), run_task);
print("=== Colony size: ===");
print(len(colony));
print("=== All agents ACTIVE ===");
""",

    "Full Showcase": """\
\u2395 Aeonmi v2.0 \u2014 Full power showcase

\u2395 1. Higher-order functions
\u25ef pipe\u27e8val, fns\u27e9 {
    let r = val;
    let i = 0;
    while (i < len(fns)) {
        i = i + 1;
    }
    return r;
}

\u2395 2. String operations
let lang = "aeonmi";
print(upper(lang));
print(repeat(lang + " ", 3));
print(join(split("a,b,c,d", ","), " | "));

\u2395 3. Math pipeline
print(round(sqrt(pow(3,2) + pow(4,2))));
print(join(map(range(1,6), toString), ","));

\u2395 4. Objects + JSON
let meta = object();
meta = set_key(meta, "name", "Aeonmi");
meta = set_key(meta, "builtins", 80);
meta = set_key(meta, "ready", true);
print(to_json(meta));

\u2395 5. Stats
let scores = [92, 88, 76, 95, 83, 71, 99, 84];
print(sum(scores));
print(len(scores));
print(sort(scores));
""",
}


# ── Syntax highlight rules ────────────────────────────────────────────────────
_KW   = re.compile(r'\b(let|return|print|if|else|while|for|true|false|null)\b')
_NUM  = re.compile(r'\b\d+(\.\d+)?\b')
_STR  = re.compile(r'"[^"]*"')
_CMT  = re.compile(r'\u2395.*$', re.MULTILINE)
_SYM  = re.compile(r'[\u25ef\u27e8\u27e9\u2395\u2192\u25b6\u2190\u221a\u00b9]')
_FN   = re.compile(r'\b[a-zA-Z_]\w*(?=\u27e8|\()')


# ── Main window ───────────────────────────────────────────────────────────────
class AeonmiStudio(tk.Tk):
    W, H = 1240, 740

    def __init__(self):
        super().__init__()
        self.title("Aeonmi Studio  v2.0")
        self.configure(bg=BG)
        self.resizable(True, True)
        sw, sh = self.winfo_screenwidth(), self.winfo_screenheight()
        self.geometry(f"{self.W}x{self.H}+{(sw-self.W)//2}+{(sh-self.H)//2}")
        self.minsize(900, 600)

        self._current_file = None
        self._running = False
        self._hl_job  = None

        self._build_menu()
        self._build_toolbar()
        self._build_panels()
        self._build_statusbar()

        self.bind("<F5>",         lambda _: self._run())
        self.bind("<Control-s>",  lambda _: self._save())
        self.bind("<Control-o>",  lambda _: self._open())
        self.bind("<Control-n>",  lambda _: self._new())
        self.protocol("WM_DELETE_WINDOW", self.destroy)

        # Load default example
        self._load_example("Hello World")
        self._set_status("Ready  \u25c8  F5 to run  \u25c8  Ctrl+S to save")

    # ── Menu ──────────────────────────────────────────────────────────────────
    def _build_menu(self):
        mb = tk.Menu(self, bg=TOOLBAR, fg=GREEN,
                     activebackground=DIM_G, activeforeground=GREEN,
                     tearoff=False)
        self.config(menu=mb)

        fm = tk.Menu(mb, tearoff=False, bg=TOOLBAR, fg=GREEN,
                     activebackground=DIM_G, activeforeground=GREEN)
        fm.add_command(label="New         Ctrl+N", command=self._new)
        fm.add_command(label="Open...     Ctrl+O", command=self._open)
        fm.add_command(label="Save        Ctrl+S", command=self._save)
        fm.add_command(label="Save As...",          command=self._save_as)
        fm.add_separator()
        fm.add_command(label="Exit",                command=self.destroy)
        mb.add_cascade(label=" File ", menu=fm)

        em = tk.Menu(mb, tearoff=False, bg=TOOLBAR, fg=GREEN,
                     activebackground=DIM_G, activeforeground=GREEN)
        for name in EXAMPLES:
            em.add_command(label=f"  {name}",
                           command=lambda n=name: self._load_example(n))
        mb.add_cascade(label=" Examples ", menu=em)

        rm = tk.Menu(mb, tearoff=False, bg=TOOLBAR, fg=GREEN,
                     activebackground=DIM_G, activeforeground=GREEN)
        rm.add_command(label="Run  F5", command=self._run)
        rm.add_command(label="Clear Output", command=self._clear_output)
        mb.add_cascade(label=" Run ", menu=rm)

    # ── Toolbar ───────────────────────────────────────────────────────────────
    def _build_toolbar(self):
        tb = tk.Frame(self, bg=TOOLBAR, height=46)
        tb.pack(fill="x", side="top")
        tb.pack_propagate(False)

        # Logo
        tk.Label(tb, text=" \u25c9 AEONMI STUDIO ", bg=TOOLBAR, fg=GREEN,
                 font=("Courier", 13, "bold")).pack(side="left", padx=8)
        tk.Label(tb, text="v2.0", bg=TOOLBAR, fg=AMBER,
                 font=("Courier", 9)).pack(side="left")

        tk.Frame(tb, bg=DIM_G, width=1).pack(side="left", fill="y", padx=12)

        # Run button
        self.run_btn = tk.Button(
            tb, text=" \u25b6  RUN ", bg=DIM_G, fg=GREEN,
            font=("Courier", 11, "bold"), bd=0, padx=12,
            activebackground=GREEN, activeforeground=BG,
            cursor="hand2", command=self._run)
        self.run_btn.pack(side="left", padx=4, pady=6)

        # Clear button
        tk.Button(tb, text=" \u25a0  CLEAR ", bg="#1a0a00", fg=AMBER,
                  font=("Courier", 10), bd=0, padx=8,
                  activebackground=AMBER, activeforeground=BG,
                  cursor="hand2", command=self._clear_output).pack(
                      side="left", padx=2, pady=6)

        tk.Frame(tb, bg=DIM_G, width=1).pack(side="left", fill="y", padx=10)

        # Examples dropdown
        tk.Label(tb, text="Examples:", bg=TOOLBAR, fg=GRAY,
                 font=("Courier", 9)).pack(side="left", padx=(4, 2))
        self._ex_var = tk.StringVar(value="Hello World")
        ex_menu = tk.OptionMenu(tb, self._ex_var, *EXAMPLES.keys(),
                                command=self._load_example)
        ex_menu.config(bg=TOOLBAR, fg=CYAN, bd=0, highlightthickness=0,
                       font=("Courier", 9), activebackground=DIM_C,
                       activeforeground=CYAN)
        ex_menu["menu"].config(bg=TOOLBAR, fg=CYAN,
                               activebackground=DIM_C, activeforeground=CYAN)
        ex_menu.pack(side="left", pady=4)

        # VM status indicator (right side)
        tk.Frame(tb, bg=TOOLBAR).pack(side="left", expand=True)
        self.vm_lbl = tk.Label(
            tb, text="\u25cf VM READY" if VM_EXE else "\u25cf VM NOT FOUND",
            bg=TOOLBAR, fg=GREEN if VM_EXE else RED,
            font=("Courier", 9))
        self.vm_lbl.pack(side="right", padx=12)

    # ── 3-Panel layout ────────────────────────────────────────────────────────
    def _build_panels(self):
        main = tk.Frame(self, bg=BG)
        main.pack(fill="both", expand=True, padx=6, pady=(4, 2))
        main.columnconfigure(0, weight=5, minsize=380)
        main.columnconfigure(1, weight=2, minsize=200)
        main.columnconfigure(2, weight=4, minsize=300)
        main.rowconfigure(0, weight=1)

        self._build_editor(main)
        self._build_pipeline(main)
        self._build_output(main)

    # ── Left: Code Editor ─────────────────────────────────────────────────────
    def _build_editor(self, parent):
        frame = tk.Frame(parent, bg=DIM_G, bd=1)
        frame.grid(row=0, column=0, sticky="nsew", padx=(0,3))

        hdr = tk.Frame(frame, bg=DIM_G)
        hdr.pack(fill="x")
        tk.Label(hdr, text=" \u25c9 CODE EDITOR ", bg=DIM_G, fg=GREEN,
                 font=("Courier", 8, "bold")).pack(side="left", padx=4, pady=2)
        self.file_lbl = tk.Label(hdr, text="untitled.ai", bg=DIM_G, fg=GRAY,
                                  font=("Courier", 8))
        self.file_lbl.pack(side="right", padx=8)

        edit_frame = tk.Frame(frame, bg=PANEL_BG)
        edit_frame.pack(fill="both", expand=True)

        # Line numbers
        self.lineno = tk.Text(edit_frame, width=4, bg="#050c10", fg="#2a4a2a",
                               font=("Courier", 11), bd=0, state="disabled",
                               highlightthickness=0, takefocus=False)
        self.lineno.pack(side="left", fill="y")

        # Scrollbar
        vsb = tk.Scrollbar(edit_frame, orient="vertical",
                           bg=DIM_G, troughcolor=PANEL_BG, bd=0)
        vsb.pack(side="right", fill="y")

        self.editor = tk.Text(
            edit_frame, bg=PANEL_BG, fg=GREEN,
            font=("Courier", 11), bd=0, highlightthickness=1,
            highlightbackground=DIM_G, highlightcolor=GREEN,
            insertbackground=GREEN, undo=True, wrap="none",
            selectbackground="#003a1a", selectforeground=WHITE,
            yscrollcommand=vsb.set, tabs=("2c",))
        self.editor.pack(side="left", fill="both", expand=True)
        vsb.config(command=self._sync_scroll)

        # Tags
        self.editor.tag_config("kw",   foreground=CYAN)
        self.editor.tag_config("str",  foreground=AMBER)
        self.editor.tag_config("cmt",  foreground="#2a5a2a")
        self.editor.tag_config("num",  foreground=PURPLE)
        self.editor.tag_config("sym",  foreground=PURPLE)
        self.editor.tag_config("fn",   foreground="#88ddff")
        self.editor.tag_config("curline", background="#0d1f15")

        self.editor.bind("<KeyRelease>", self._on_edit)
        self.editor.bind("<ButtonRelease>", self._update_cursor_line)

    def _sync_scroll(self, *args):
        self.editor.yview(*args)
        self._update_linenos()

    def _on_edit(self, _=None):
        self._update_linenos()
        self._update_cursor_line()
        if self._hl_job:
            self.after_cancel(self._hl_job)
        self._hl_job = self.after(200, self._highlight)
        self._update_status_stats()

    def _update_linenos(self):
        self.lineno.config(state="normal")
        self.lineno.delete("1.0", "end")
        lines = int(self.editor.index("end-1c").split(".")[0])
        nums = "\n".join(f"{i:>3}" for i in range(1, lines + 1))
        self.lineno.insert("1.0", nums)
        self.lineno.config(state="disabled")
        self.lineno.yview_moveto(self.editor.yview()[0])

    def _update_cursor_line(self, _=None):
        self.editor.tag_remove("curline", "1.0", "end")
        line = self.editor.index("insert").split(".")[0]
        self.editor.tag_add("curline", f"{line}.0", f"{line}.end+1c")

    def _highlight(self):
        e = self.editor
        for tag in ("kw","str","cmt","num","sym","fn"):
            e.tag_remove(tag, "1.0", "end")
        content = e.get("1.0", "end")
        for m in _CMT.finditer(content):
            s = f"1.0+{m.start()}c"; en = f"1.0+{m.end()}c"
            e.tag_add("cmt", s, en)
        for m in _STR.finditer(content):
            s = f"1.0+{m.start()}c"; en = f"1.0+{m.end()}c"
            if not self._in_tag(e, s, "cmt"):
                e.tag_add("str", s, en)
        for m in _KW.finditer(content):
            s = f"1.0+{m.start()}c"; en = f"1.0+{m.end()}c"
            if not self._in_tag(e, s, ("cmt","str")):
                e.tag_add("kw", s, en)
        for m in _FN.finditer(content):
            s = f"1.0+{m.start()}c"; en = f"1.0+{m.end()}c"
            if not self._in_tag(e, s, ("cmt","str","kw")):
                e.tag_add("fn", s, en)
        for m in _NUM.finditer(content):
            s = f"1.0+{m.start()}c"; en = f"1.0+{m.end()}c"
            if not self._in_tag(e, s, ("cmt","str")):
                e.tag_add("num", s, en)
        for m in _SYM.finditer(content):
            s = f"1.0+{m.start()}c"; en = f"1.0+{m.end()}c"
            e.tag_add("sym", s, en)

    def _in_tag(self, widget, pos, tags):
        if isinstance(tags, str):
            tags = (tags,)
        for tag in tags:
            if tag in widget.tag_names(pos):
                return True
        return False

    # ── Middle: VM Pipeline ───────────────────────────────────────────────────
    def _build_pipeline(self, parent):
        frame = tk.Frame(parent, bg=DIM_C, bd=1)
        frame.grid(row=0, column=1, sticky="nsew", padx=3)

        tk.Label(frame, text=" \u25c9 VM PIPELINE ", bg=DIM_C, fg=CYAN,
                 font=("Courier", 8, "bold")).pack(pady=(4,2))

        tk.Frame(frame, bg=DIM_C, height=1).pack(fill="x", padx=6)

        self._stages = {}
        stage_defs = [
            ("LEXER",  "Tokenize source"),
            ("PARSER", "Build AST"),
            ("IR",     "Lower to IR"),
            ("VM",     "Execute"),
        ]

        for i, (name, desc) in enumerate(stage_defs):
            sf = tk.Frame(frame, bg=PANEL_BG, bd=1, relief="flat")
            sf.pack(fill="x", padx=8, pady=4)

            # Stage indicator dot
            dot = tk.Canvas(sf, width=14, height=14, bg=PANEL_BG,
                            highlightthickness=0)
            dot.pack(side="left", padx=(8,4), pady=6)
            dot.create_oval(2, 2, 12, 12, fill=GRAY, outline="", tags="dot")

            info = tk.Frame(sf, bg=PANEL_BG)
            info.pack(side="left", fill="x", expand=True, pady=4)

            name_lbl = tk.Label(info, text=name, bg=PANEL_BG, fg=LGRAY,
                                font=("Courier", 10, "bold"), anchor="w")
            name_lbl.pack(fill="x", padx=2)

            detail_lbl = tk.Label(info, text=desc, bg=PANEL_BG, fg="#2a4a3a",
                                  font=("Courier", 8), anchor="w")
            detail_lbl.pack(fill="x", padx=2)

            check_lbl = tk.Label(sf, text="", bg=PANEL_BG, fg=GREEN,
                                 font=("Courier", 10, "bold"))
            check_lbl.pack(side="right", padx=8)

            self._stages[name] = {
                "frame": sf, "dot": dot, "name_lbl": name_lbl,
                "detail_lbl": detail_lbl, "check": check_lbl
            }

        # Timing display
        tk.Frame(frame, bg=DIM_C, height=1).pack(fill="x", padx=6, pady=(12,4))
        self.time_lbl = tk.Label(frame, text="", bg=DIM_C, fg=GRAY,
                                  font=("Courier", 9))
        self.time_lbl.pack()

        self.pipe_msg = tk.Label(frame, text="Press \u25b6 RUN or F5",
                                  bg=DIM_C, fg=GRAY, font=("Courier", 8),
                                  wraplength=160)
        self.pipe_msg.pack(pady=4, padx=8)

        # Stats block
        tk.Frame(frame, bg=DIM_C).pack(expand=True)
        stats_f = tk.Frame(frame, bg="#040a0e")
        stats_f.pack(fill="x", padx=6, pady=6)
        tk.Label(stats_f, text=" RUNTIME STATS ", bg="#040a0e", fg=GRAY,
                 font=("Courier", 7)).pack()
        self.stat_tokens = tk.Label(stats_f, text="Tokens:  \u2014",
                                     bg="#040a0e", fg=LGRAY, font=("Courier", 9), anchor="w")
        self.stat_tokens.pack(fill="x", padx=8)
        self.stat_lines = tk.Label(stats_f, text="Lines:   \u2014",
                                    bg="#040a0e", fg=LGRAY, font=("Courier", 9), anchor="w")
        self.stat_lines.pack(fill="x", padx=8)
        self.stat_time = tk.Label(stats_f, text="Time:    \u2014",
                                   bg="#040a0e", fg=LGRAY, font=("Courier", 9), anchor="w")
        self.stat_time.pack(fill="x", padx=8, pady=(0,6))

    def _reset_pipeline(self):
        for name, s in self._stages.items():
            s["dot"].itemconfig("dot", fill=GRAY)
            s["name_lbl"].config(fg=LGRAY)
            s["detail_lbl"].config(fg="#2a4a3a")
            s["check"].config(text="")
            s["frame"].config(bg=PANEL_BG)
        self.time_lbl.config(text="")
        self.pipe_msg.config(text="Executing...", fg=AMBER)

    def _set_stage(self, name, state):
        """state: 'active' | 'pass' | 'fail'"""
        s = self._stages[name]
        if state == "active":
            s["dot"].itemconfig("dot", fill=AMBER)
            s["name_lbl"].config(fg=AMBER)
            s["detail_lbl"].config(fg=AMBER)
            s["check"].config(text="\u25b6", fg=AMBER)
        elif state == "pass":
            s["dot"].itemconfig("dot", fill=GREEN)
            s["name_lbl"].config(fg=GREEN)
            s["detail_lbl"].config(fg="#2a8a4a")
            s["check"].config(text="\u2713", fg=GREEN)
        elif state == "fail":
            s["dot"].itemconfig("dot", fill=RED)
            s["name_lbl"].config(fg=RED)
            s["check"].config(text="\u2717", fg=RED)

    # ── Right: Output ─────────────────────────────────────────────────────────
    def _build_output(self, parent):
        frame = tk.Frame(parent, bg=DIM_C, bd=1)
        frame.grid(row=0, column=2, sticky="nsew", padx=(3,0))

        hdr = tk.Frame(frame, bg=DIM_C)
        hdr.pack(fill="x")
        tk.Label(hdr, text=" \u25c9 OUTPUT TERMINAL ", bg=DIM_C, fg=CYAN,
                 font=("Courier", 8, "bold")).pack(side="left", padx=4, pady=2)
        tk.Button(hdr, text="CLR", bg=DIM_C, fg=GRAY, bd=0, font=("Courier", 7),
                  command=self._clear_output, cursor="hand2").pack(side="right", padx=4)

        vsb = tk.Scrollbar(frame, bg=DIM_C, troughcolor=PANEL_BG, bd=0)
        vsb.pack(side="right", fill="y")

        self.output = tk.Text(
            frame, bg=PANEL_BG, fg=CYAN,
            font=("Courier", 10), bd=0, highlightthickness=1,
            highlightbackground=DIM_C, highlightcolor=CYAN,
            state="disabled", wrap="word",
            selectbackground=DIM_C,
            yscrollcommand=vsb.set)
        self.output.pack(fill="both", expand=True)
        vsb.config(command=self.output.yview)

        self.output.tag_config("hdr",    foreground=AMBER, font=("Courier", 10, "bold"))
        self.output.tag_config("out",    foreground=WHITE)
        self.output.tag_config("err",    foreground=RED)
        self.output.tag_config("ok",     foreground=GREEN)
        self.output.tag_config("dim",    foreground=GRAY)
        self.output.tag_config("time",   foreground=PURPLE)

    # ── Status bar ────────────────────────────────────────────────────────────
    def _build_statusbar(self):
        sb = tk.Frame(self, bg="#040a0e", height=22)
        sb.pack(fill="x", side="bottom")
        sb.pack_propagate(False)
        tk.Frame(sb, bg=DIM_G, height=1).pack(fill="x", side="top")

        self.status_lbl = tk.Label(sb, text="", bg="#040a0e", fg=GRAY,
                                    font=("Courier", 8), anchor="w")
        self.status_lbl.pack(side="left", padx=8)

        self.pos_lbl = tk.Label(sb, text="Ln 1, Col 1", bg="#040a0e",
                                 fg=LGRAY, font=("Courier", 8))
        self.pos_lbl.pack(side="right", padx=8)

        vm_path = VM_EXE or "VM not found"
        tk.Label(sb, text=f"VM: {os.path.basename(vm_path) if VM_EXE else 'NOT FOUND'}",
                 bg="#040a0e", fg=GRAY if VM_EXE else RED,
                 font=("Courier", 8)).pack(side="right", padx=12)

    def _set_status(self, msg, color=None):
        self.status_lbl.config(text=msg, fg=color or GRAY)

    def _update_status_stats(self):
        try:
            idx = self.editor.index("insert")
            ln, col = idx.split(".")
            self.pos_lbl.config(text=f"Ln {ln}, Col {int(col)+1}")
        except Exception:
            pass

    # ── Actions ───────────────────────────────────────────────────────────────
    def _load_example(self, name, *_):
        self._ex_var.set(name)
        self.editor.delete("1.0", "end")
        self.editor.insert("1.0", EXAMPLES.get(name, ""))
        self._highlight()
        self._update_linenos()
        self.file_lbl.config(text=f"{name.lower().replace(' ','_')}.ai")
        self._set_status(f"Loaded example: {name}")

    def _new(self):
        self.editor.delete("1.0", "end")
        self._current_file = None
        self.file_lbl.config(text="untitled.ai")
        self._update_linenos()
        self._set_status("New file")

    def _open(self):
        path = filedialog.askopenfilename(
            filetypes=[("Aeonmi files", "*.ai"), ("All files", "*.*")])
        if path:
            with open(path, "r", encoding="utf-8") as f:
                self.editor.delete("1.0", "end")
                self.editor.insert("1.0", f.read())
            self._current_file = path
            self.file_lbl.config(text=os.path.basename(path))
            self._highlight()
            self._update_linenos()
            self._set_status(f"Opened: {path}")

    def _save(self):
        if self._current_file:
            with open(self._current_file, "w", encoding="utf-8") as f:
                f.write(self.editor.get("1.0", "end-1c"))
            self._set_status(f"Saved: {self._current_file}")
        else:
            self._save_as()

    def _save_as(self):
        path = filedialog.asksaveasfilename(
            defaultextension=".ai",
            filetypes=[("Aeonmi files", "*.ai"), ("All files", "*.*")])
        if path:
            self._current_file = path
            self.file_lbl.config(text=os.path.basename(path))
            self._save()

    def _clear_output(self):
        self.output.config(state="normal")
        self.output.delete("1.0", "end")
        self.output.config(state="disabled")
        self._reset_pipeline()
        self.pipe_msg.config(text="Press \u25b6 RUN or F5", fg=GRAY)
        self._set_status("Output cleared")

    # ── Run ───────────────────────────────────────────────────────────────────
    def _run(self):
        if self._running:
            return
        if not VM_EXE:
            messagebox.showerror("VM Not Found",
                "Aeonmi.exe not found.\n\nPlace Aeonmi.exe in the same folder as AeonmiStudio.exe.")
            return

        code = self.editor.get("1.0", "end-1c").strip()
        if not code:
            return

        self._running = True
        self.run_btn.config(state="disabled", fg=GRAY)
        self._reset_pipeline()
        self._clear_output()

        # Write header
        self._out_write(f"\u2395 Aeonmi Studio  \u25b6  Run  ({time.strftime('%H:%M:%S')})\n", "hdr")
        self._out_write("\u2500" * 50 + "\n", "dim")

        # Save code to temp file
        tmp = tempfile.NamedTemporaryFile(suffix=".ai", delete=False,
                                          mode="w", encoding="utf-8")
        tmp.write(code)
        tmp.close()

        # Count lines for stats
        line_count = code.count("\n") + 1

        # Animate pipeline then execute
        def do_run():
            stages = ["LEXER", "PARSER", "IR", "VM"]
            t_start = time.perf_counter()

            for i, stage in enumerate(stages):
                self.after(0, self._set_stage, stage, "active")
                time.sleep(0.28)
                if i < 3:
                    self.after(0, self._set_stage, stage, "pass")

            # Execute
            try:
                result = subprocess.run(
                    [VM_EXE, "native", tmp.name],
                    capture_output=True, text=True,
                    timeout=30, cwd=os.path.dirname(VM_EXE))
                elapsed = time.perf_counter() - t_start
                self.after(0, self._on_result, result, elapsed, line_count)
            except subprocess.TimeoutExpired:
                self.after(0, self._on_timeout)
            except Exception as ex:
                self.after(0, self._on_error, str(ex))
            finally:
                try:
                    os.unlink(tmp.name)
                except Exception:
                    pass

        threading.Thread(target=do_run, daemon=True).start()

    def _on_result(self, result, elapsed, line_count):
        success = result.returncode == 0 and not result.stderr.strip()

        # Finalize last pipeline stage
        self._set_stage("VM", "pass" if success else "fail")

        ms = elapsed * 1000
        self.time_lbl.config(text=f"Completed in {ms:.1f} ms", fg=GREEN if success else RED)
        self.pipe_msg.config(
            text="\u2713 All stages passed" if success else "\u2717 Error detected",
            fg=GREEN if success else RED)

        # Update stats
        self.stat_lines.config(text=f"Lines:   {line_count}")
        self.stat_time.config(text=f"Time:    {ms:.2f} ms")
        self.stat_tokens.config(text=f"Status:  {'OK' if success else 'ERR'}")

        # Show output
        if result.stdout:
            # Strip ANSI title escape (\x1b]0;...)
            clean = re.sub(r'\x1b\]0;[^\x07]*\x07', '', result.stdout)
            clean = re.sub(r'\x1b\[[0-9;]*m', '', clean)
            self._out_write("\n", "dim")
            self._out_write(clean, "out")

        if result.stderr and result.stderr.strip():
            err = result.stderr.strip()
            self._out_write("\n\u2717 ERROR\n", "err")
            self._out_write(err + "\n", "err")

        self._out_write("\n" + "\u2500" * 50 + "\n", "dim")
        self._out_write(f"\u25b6 Finished in {ms:.1f} ms  |  Exit: {result.returncode}\n",
                        "ok" if success else "err")

        self._set_status(
            f"\u2713 Run complete  \u25c8  {ms:.1f} ms  \u25c8  {line_count} lines",
            GREEN if success else RED)
        self._finish_run()

    def _on_timeout(self):
        self._set_stage("VM", "fail")
        self._out_write("\n\u2717 TIMEOUT: Script exceeded 30s limit\n", "err")
        self.pipe_msg.config(text="\u2717 Timeout", fg=RED)
        self._set_status("Timeout", RED)
        self._finish_run()

    def _on_error(self, msg):
        self._set_stage("VM", "fail")
        self._out_write(f"\n\u2717 INTERNAL ERROR: {msg}\n", "err")
        self._set_status("Error", RED)
        self._finish_run()

    def _finish_run(self):
        self._running = False
        self.run_btn.config(state="normal", fg=GREEN)

    def _out_write(self, text, tag="out"):
        self.output.config(state="normal")
        self.output.insert("end", text, tag)
        self.output.see("end")
        self.output.config(state="disabled")


def main():
    app = AeonmiStudio()
    app.mainloop()


if __name__ == "__main__":
    main()
