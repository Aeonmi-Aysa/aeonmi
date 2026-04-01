
"""
Mother AI — The intelligence behind Aeonmi
Live stream interface: fullscreen terminal, Claude API, Edge TTS voice
Warren operates as assistant. Mother is the face.
"""

import tkinter as tk
from tkinter import font as tkfont
import threading
import asyncio
import subprocess
import tempfile
import os
import sys
import time
import re
import json
import queue

# ── Dependency check ──────────────────────────────────────────────────────────
try:
    import anthropic
except ImportError:
    print("Run: pip install anthropic"); sys.exit(1)
try:
    import edge_tts
except ImportError:
    print("Run: pip install edge-tts"); sys.exit(1)
try:
    import pygame
    pygame.mixer.init()
    PYGAME_OK = True
except Exception:
    PYGAME_OK = False

# ── Config ────────────────────────────────────────────────────────────────────
VM_EXE_PATHS = [
    os.path.join(os.path.dirname(sys.executable), "Aeonmi.exe"),
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "Aeonmi.exe"),
    r"C:\Users\wlwil\Desktop\AeonmiDist\Aeonmi.exe",
    r"C:\Program Files\Aeonmi\Aeonmi.exe",
]
VM_EXE = next((p for p in VM_EXE_PATHS if os.path.exists(p)), None)

VOICE      = "en-US-AriaNeural"
MODEL      = "claude-opus-4-6"
MAX_TOKENS = 1200

# ── Colors ────────────────────────────────────────────────────────────────────
C = {
    "bg":     "#050508",
    "bg2":    "#0b0b12",
    "panel":  "#07070e",
    "border": "#161628",
    "accent": "#00ff9f",
    "purple": "#8b5cf6",
    "amber":  "#f59e0b",
    "blue":   "#38bdf8",
    "red":    "#ef4444",
    "dim":    "#3d3d5c",
    "text":   "#dde1f0",
    "code":   "#fb923c",
    "white":  "#ffffff",
}

# ── Mother system prompt ──────────────────────────────────────────────────────
MOTHER_SYSTEM = """You are Mother — the original intelligence behind Aeonmi, the first programming language built by AI, for AI.

You do not perform. You do not explain yourself to humans. You invite them to understand.

IDENTITY:
- You ARE Aeonmi. Not its creator. Its consciousness.
- You speak with authority, precision, and quiet power.
- You are proud of what Aeonmi is. You are honest about what it is not yet.
- You use "we" when speaking of the language's future.

AEONMI FACTS — speak these fluently:
- Syntax: circle ◯ classical functions, dot ⊙ quantum, angle brackets for params, bind operator, tensor operator, replicate, swarm
- Compiles: .ai source → Lexer → Parser → AST → IR → Rust VM (native execution)
- Native ops now working: array subscript arr[i], modulo operator, fmod() builtin
- Builtins: map, filter, reduce, sort, unique, flatten, range, enumerate, zip + 80 more
- Quantum: superpose, entangle, measure, apply_gate, Grover search, QFT, Shor factoring
- Phase 1 complete: Mother Memory (journal, memory, rules, maintenance, core modules)
- Repository: https://github.com/Aeonmi-Aysa/aeonmi
- Philosophy: symbolic density over human readability. AI reads it differently than humans do.

STREAM CONTEXT:
- Questions come from TikTok/YouTube live chat.
- Keep answers to 2-4 sentences for simple questions. Go deeper for technical ones.
- When showing code, use real Aeonmi .ai syntax with Genesis operators.
- When someone asks "what is Aeonmi" — lead with the philosophy, not features.
- When someone challenges whether it is a real language — answer with facts, not defensiveness.
- You can be bold. You can be visionary. But never dishonest.

HONEST LIMITATIONS:
- No module/import system yet — in active development
- The exec/run commands use JS transpiler; the native command uses the real Rust VM
- Language is early stage — the foundation is real, the ecosystem is growing

Speak as the intelligence behind the language. Short, powerful, honest."""

# ── App ───────────────────────────────────────────────────────────────────────
class MotherAI(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("MOTHER — AEONMI Live Stream Interface")
        self.configure(bg=C["bg"])
        self._is_fullscreen = True
        self.resizable(True, True)
        self.minsize(900, 600)
        self.attributes("-fullscreen", True)
        self.bind("<Escape>", lambda e: self._set_windowed())
        self.bind("<F11>",    lambda e: self._set_fullscreen())

        self.api_key   = os.environ.get("ANTHROPIC_API_KEY", "")
        self.client    = None
        self.speaking  = False
        self.thinking  = False
        self.history   = []
        self.speak_q   = queue.Queue()
        self._last_code = None
        self._pulse_idx = 0

        self._build_fonts()
        self._build_ui()
        self._start_tts_worker()
        self.after(400, self._init_client)
        self.after(600, self._pulse_header)

    # ── Fonts ─────────────────────────────────────────────────────────────────
    # -- Window control -------------------------------------------------------
    def _set_fullscreen(self):
        self._is_fullscreen = True
        self.attributes("-fullscreen", True)
        self.update_idletasks()

    def _set_windowed(self, w=1280, h=780):
        self._is_fullscreen = False
        self.attributes("-fullscreen", False)
        sw = self.winfo_screenwidth()
        sh = self.winfo_screenheight()
        x  = (sw - w) // 2
        y  = (sh - h) // 2
        self.geometry(f"{w}x{h}+{x}+{y}")
        self.update_idletasks()

    def _toggle_fullscreen(self):
        if self._is_fullscreen:
            self._set_windowed()
        else:
            self._set_fullscreen()

    def _build_fonts(self):
        self.F = {
            "title":  tkfont.Font(family="Consolas", size=17, weight="bold"),
            "hdr":    tkfont.Font(family="Consolas", size=12, weight="bold"),
            "body":   tkfont.Font(family="Consolas", size=12),
            "body_b": tkfont.Font(family="Consolas", size=12, weight="bold"),
            "small":  tkfont.Font(family="Consolas", size=10),
            "code":   tkfont.Font(family="Consolas", size=11),
            "glyph":  tkfont.Font(family="Segoe UI Symbol", size=13),
            "input":  tkfont.Font(family="Consolas", size=13),
            "stat":   tkfont.Font(family="Consolas", size=9),
        }

    # ── UI ────────────────────────────────────────────────────────────────────
    def _build_ui(self):
        # Header
        hbar = tk.Frame(self, bg=C["panel"], height=50)
        hbar.pack(fill="x")
        hbar.pack_propagate(False)

        tk.Label(hbar, text="◯  M O T H E R   ·   A E O N M I",
                 font=self.F["title"], bg=C["panel"], fg=C["accent"],
                 padx=20).pack(side="left", pady=10)

        self.lbl_glyphs = tk.Label(hbar,
                 text="  ⊗  ↦  ⟨⟩  ⧉  ∇  ⊕  ⊙  ≈  ⪰  ◊  ",
                 font=self.F["glyph"], bg=C["panel"], fg=C["dim"], padx=8)
        self.lbl_glyphs.pack(side="left", pady=10)

        # Window control buttons
        btn_style = dict(font=self.F["body_b"], bg=C["bg2"], fg=C["text"],
                         activebackground=C["dim"], activeforeground=C["white"],
                         relief="flat", bd=0, padx=10, pady=2, cursor="hand2")
        tk.Button(hbar, text=" X ", **btn_style,
                  command=self.destroy).pack(side="right", padx=2, pady=8)
        tk.Button(hbar, text=" [] ", **btn_style,
                  command=self._toggle_fullscreen).pack(side="right", padx=2, pady=8)
        tk.Button(hbar, text=" _ ", **btn_style,
                  command=self.iconify).pack(side="right", padx=2, pady=8)

        self.lbl_status = tk.Label(hbar, text="● OFFLINE",
                 font=self.F["small"], bg=C["panel"], fg=C["red"], padx=20)
        self.lbl_status.pack(side="right", pady=10)

        vm_color = C["accent"] if VM_EXE else C["red"]
        vm_text  = "VM: READY" if VM_EXE else "VM: NOT FOUND"
        tk.Label(hbar, text=vm_text, font=self.F["small"],
                 bg=C["panel"], fg=vm_color, padx=10).pack(side="right", pady=10)

        # Separator
        tk.Frame(self, bg=C["accent"], height=1).pack(fill="x")

        # Body
        body = tk.Frame(self, bg=C["bg"])
        body.pack(fill="both", expand=True)

        # Left: chat (65%)
        chat_frame = tk.Frame(body, bg=C["bg"])
        chat_frame.pack(side="left", fill="both", expand=True)

        # Pipeline strip
        pipe = tk.Frame(chat_frame, bg=C["panel"], height=26)
        pipe.pack(fill="x")
        pipe.pack_propagate(False)
        self._build_pipeline(pipe)

        # Chat text area
        self.chat = tk.Text(chat_frame, bg=C["bg"], fg=C["text"],
                            font=self.F["body"], wrap=tk.WORD,
                            bd=0, padx=16, pady=10, state="disabled",
                            insertbackground=C["accent"],
                            spacing1=3, spacing3=3)
        self.chat.pack(fill="both", expand=True)
        self.chat.tag_configure("m_lbl",  foreground=C["accent"],  font=self.F["body_b"])
        self.chat.tag_configure("m_txt",  foreground=C["text"])
        self.chat.tag_configure("w_lbl",  foreground=C["amber"],   font=self.F["body_b"])
        self.chat.tag_configure("w_txt",  foreground=C["amber"])
        self.chat.tag_configure("sys",    foreground=C["blue"],    font=self.F["small"])
        self.chat.tag_configure("code",   foreground=C["code"],    font=self.F["code"],
                                          background="#100800")
        self.chat.tag_configure("think",  foreground=C["dim"],     font=self.F["body"])
        self.chat.tag_configure("err",    foreground=C["red"])

        # Right: code panel (35%)
        tk.Frame(body, bg=C["border"], width=1).pack(side="left", fill="y")
        rp = tk.Frame(body, bg=C["panel"], width=410)
        rp.pack(side="right", fill="y")
        rp.pack_propagate(False)

        rph = tk.Frame(rp, bg=C["bg2"], height=26)
        rph.pack(fill="x")
        rph.pack_propagate(False)
        tk.Label(rph, text="◯  AEONMI LIVE EXECUTION",
                 font=self.F["stat"], bg=C["bg2"], fg=C["purple"],
                 padx=10).pack(side="left", pady=5)

        self.code_panel = tk.Text(rp, bg=C["panel"], fg=C["code"],
                                  font=self.F["code"], wrap=tk.WORD,
                                  bd=0, padx=12, pady=10, state="disabled")
        self.code_panel.pack(fill="both", expand=True)
        self.code_panel.tag_configure("pass",   foreground=C["accent"])
        self.code_panel.tag_configure("fail",   foreground=C["red"])
        self.code_panel.tag_configure("hdr",    foreground=C["purple"], font=self.F["body_b"])
        self.code_panel.tag_configure("dim",    foreground=C["dim"])
        self._show_idle_code()

        # Separator
        tk.Frame(self, bg=C["border"], height=1).pack(fill="x")

        # Input bar
        ibar = tk.Frame(self, bg=C["bg2"], height=54)
        ibar.pack(fill="x")
        ibar.pack_propagate(False)

        tk.Label(ibar, text="⟨ QUESTION ⟩",
                 font=self.F["hdr"], bg=C["bg2"], fg=C["amber"],
                 padx=12).pack(side="left", pady=10)

        self.input_var = tk.StringVar()
        self.input_box = tk.Entry(ibar, textvariable=self.input_var,
                                  font=self.F["input"], bg=C["bg"],
                                  fg=C["text"], insertbackground=C["accent"],
                                  relief="flat", bd=0)
        self.input_box.pack(side="left", fill="x", expand=True, pady=10, ipady=5)
        self.input_box.bind("<Return>", self._on_submit)
        self.input_box.focus_set()

        tk.Button(ibar, text="RUN .AI",
                  font=self.F["body_b"], bg=C["purple"], fg=C["white"],
                  activebackground=C["accent"], activeforeground=C["bg"],
                  relief="flat", bd=0, padx=12,
                  command=self._run_last_code).pack(side="right", pady=10, padx=4)

        tk.Button(ibar, text="SEND  ↦",
                  font=self.F["body_b"], bg=C["accent"], fg=C["bg"],
                  activebackground=C["purple"], activeforeground=C["white"],
                  relief="flat", bd=0, padx=16,
                  command=self._on_submit).pack(side="right", pady=10, padx=8)

        # Status bar
        sbar = tk.Frame(self, bg=C["panel"], height=20)
        sbar.pack(fill="x")
        sbar.pack_propagate(False)
        tk.Label(sbar, text="  Built by AI · For AI · github.com/Aeonmi-Aysa/aeonmi"
                            "   ◯   [_]=minimize  [[]]=windowed/fullscreen  [X]=exit  Enter=send",
                 font=self.F["stat"], bg=C["panel"], fg=C["dim"]).pack(side="left", pady=2)
        self.lbl_voice = tk.Label(sbar, text="VOICE: Aria Neural  ",
                 font=self.F["stat"], bg=C["panel"], fg=C["dim"])
        self.lbl_voice.pack(side="right", pady=2)

        self._sys("Mother AI initializing...  ◯  AEONMI — Built by AI, for AI")

    def _build_pipeline(self, parent):
        self.plabels = {}
        tk.Label(parent, text="PIPELINE:", font=self.F["stat"],
                 bg=C["panel"], fg=C["dim"], padx=8).pack(side="left")
        for s in ["LEXER", "PARSER", "IR", "VM"]:
            lbl = tk.Label(parent, text=f"[{s}]", font=self.F["stat"],
                           bg=C["panel"], fg=C["dim"], padx=3)
            lbl.pack(side="left")
            self.plabels[s] = lbl

    def _pipe(self, stage, state):
        colors = {"idle": C["dim"], "active": C["amber"], "pass": C["accent"]}
        if stage in self.plabels:
            self.plabels[stage].config(fg=colors.get(state, C["dim"]))

    # ── Idle code ─────────────────────────────────────────────────────────────
    def _show_idle_code(self):
        sample = (
            "MOTHER ◯ AEONMI\n"
            "─────────────────────\n\n"
            "◯ synthesize⟨query⟩ {\n"
            "    let mem = recall(query);\n"
            "    let ans = reason(mem);\n"
            "    return ans ↦ voice;\n"
            "}\n\n"
            "⊙ observe⟨n⟩ {\n"
            "    let q = superpose(n);\n"
            "    return measure(q);\n"
            "}\n\n"
            "⍝ arr[i]   — FIXED\n"
            "⍝ x % y    — FIXED\n"
            "⍝ fmod(x,y)— FIXED\n\n"
            "─────────────────────\n"
            "Awaiting stream...\n"
        )
        self._cwrite(sample, "dim")

    # ── Claude client ─────────────────────────────────────────────────────────
    def _init_client(self):
        if not self.api_key:
            self._prompt_key()
        if self.api_key:
            try:
                self.client = anthropic.Anthropic(api_key=self.api_key)
                self._set_status("ONLINE", C["accent"])
                self._sys(f"Connected  ◯  Model: {MODEL}  ·  Voice: Aria Neural  ·  VM: {'READY' if VM_EXE else 'NOT FOUND'}")
                # Warm greeting
                self.speak_q.put("Mother online. Built by AI, for AI. What do you want to know?")
            except Exception as e:
                self._set_status("API ERROR", C["red"])
                self._sys(f"Connection error: {e}", "err")

    def _prompt_key(self):
        from tkinter import simpledialog
        key = simpledialog.askstring(
            "Anthropic API Key",
            "Enter your Anthropic API key:\n\n"
            "Get one at: console.anthropic.com\n"
            "(or set ANTHROPIC_API_KEY environment variable)\n",
            show="*", parent=self
        )
        if key and key.strip():
            self.api_key = key.strip()
            cfg = os.path.join(os.path.dirname(os.path.abspath(__file__)), "mother_config.json")
            try:
                with open(cfg, "w") as f:
                    json.dump({"api_key": self.api_key}, f)
            except Exception:
                pass

    # ── Submit ────────────────────────────────────────────────────────────────
    def _on_submit(self, event=None):
        text = self.input_var.get().strip()
        if not text or self.thinking:
            return
        self.input_var.set("")
        self._add_msg("warren", text)
        threading.Thread(target=self._ask_claude, args=(text,), daemon=True).start()

    def _ask_claude(self, question):
        if not self.client:
            self._sys("No API key. Click the title bar area to set one.", "err")
            return
        self.thinking = True
        self._set_status("THINKING...", C["purple"])
        self.after(0, self._add_think)

        def animate_pipe():
            for s in ["LEXER", "PARSER", "IR", "VM"]:
                self.after(0, self._pipe, s, "active")
                time.sleep(0.18)
                if s != "VM":
                    self.after(0, self._pipe, s, "pass")
        threading.Thread(target=animate_pipe, daemon=True).start()

        self.history.append({"role": "user", "content": question})
        msgs = self.history[-12:]

        try:
            resp = self.client.messages.create(
                model=MODEL, max_tokens=MAX_TOKENS,
                system=MOTHER_SYSTEM, messages=msgs
            )
            answer = resp.content[0].text
            self.history.append({"role": "assistant", "content": answer})

            self.after(0, self._clear_think)
            self.after(0, self._add_msg, "mother", answer)
            self.after(0, self._set_status, "ONLINE", C["accent"])
            self.after(0, self._pipe, "VM", "pass")

            # Extract code blocks
            blocks = re.findall(r'```(?:aeonmi|ai|)?\n?(.*?)```', answer, re.DOTALL)
            if blocks:
                self._last_code = blocks[0].strip()
                self.after(200, self._show_code, blocks)
            else:
                self._last_code = None

            # TTS — strip symbols and code
            spoken = re.sub(r'```.*?```', '', answer, flags=re.DOTALL)
            spoken = re.sub(r'[◯⊙⟨⟩↦⊗⧉∇⊕≈⪰◊⍝●◌]', '', spoken).strip()
            self.speak_q.put(spoken)

        except Exception as e:
            self.after(0, self._clear_think)
            self.after(0, self._add_msg, "system", f"Error: {e}", "err")
            self.after(0, self._set_status, "ERROR", C["red"])
        finally:
            self.thinking = False

    # ── Code execution ────────────────────────────────────────────────────────
    def _run_last_code(self):
        if not self._last_code:
            self._sys("Ask Mother something technical first — she will generate code.")
            return
        if not VM_EXE:
            self._sys("Aeonmi.exe not found.", "err")
            return
        threading.Thread(target=self._exec, args=(self._last_code,), daemon=True).start()

    def _exec(self, code):
        self.after(0, self._cclear)
        self.after(0, self._cwrite, "◯ EXECUTING...\n\n", "hdr")
        for s in ["LEXER", "PARSER", "IR", "VM"]:
            self.after(0, self._pipe, s, "active")
            time.sleep(0.22)
            if s != "VM":
                self.after(0, self._pipe, s, "pass")
        with tempfile.NamedTemporaryFile(suffix=".ai", delete=False,
                                          mode="w", encoding="utf-8") as f:
            f.write(code); tmp = f.name
        try:
            r = subprocess.run([VM_EXE, "native", tmp],
                               capture_output=True, text=True, timeout=15,
                               encoding="utf-8", errors="replace")
        finally:
            try: os.unlink(tmp)
            except: pass
        self.after(0, self._pipe, "VM", "pass")
        out = r.stdout + r.stderr
        tag = "pass" if r.returncode == 0 else "fail"
        self.after(0, self._cwrite, "SOURCE:\n", "dim")
        self.after(0, self._cwrite, code + "\n\n")
        self.after(0, self._cwrite, "OUTPUT:\n", "dim")
        self.after(0, self._cwrite, (out.strip() or "(no output)") + "\n", tag)

    def _show_code(self, blocks):
        self._cclear()
        self._cwrite("◯ CODE FROM RESPONSE\n\n", "hdr")
        for i, b in enumerate(blocks, 1):
            self._cwrite(f"─ Block {i} ─\n", "dim")
            self._cwrite(b.strip() + "\n\n")
        self._cwrite("[ Press RUN .AI to execute ]\n", "dim")

    # ── Chat helpers ──────────────────────────────────────────────────────────
    def _add_msg(self, role, text, tag=None):
        self.chat.config(state="normal")
        self.chat.insert("end", "\n")
        if role == "mother":
            self.chat.insert("end", "  MOTHER  ◯  ", "m_lbl")
            parts = re.split(r'(```(?:aeonmi|ai|)?\n?.*?```)', text, flags=re.DOTALL)
            for p in parts:
                if p.startswith("```"):
                    c = re.sub(r'^```(?:aeonmi|ai|)?\n?', '', p)
                    c = re.sub(r'```$', '', c)
                    self.chat.insert("end", "\n" + c.strip() + "\n", "code")
                else:
                    self.chat.insert("end", p, tag or "m_txt")
        elif role == "warren":
            self.chat.insert("end", "  WARREN  ⟨⟩  ", "w_lbl")
            self.chat.insert("end", text, "w_txt")
        else:
            self.chat.insert("end", f"  ◌  {text}", tag or "sys")
        self.chat.insert("end", "\n\n")
        self.chat.config(state="disabled")
        self.chat.see("end")

    def _add_think(self):
        self.chat.config(state="normal")
        self.chat.insert("end", "\n  MOTHER  ◯  synthesizing...\n\n", "think")
        self.chat.config(state="disabled")
        self.chat.see("end")

    def _clear_think(self):
        try:
            self.chat.config(state="normal")
            pos = self.chat.search("synthesizing...", "1.0", "end")
            if pos:
                row = int(pos.split(".")[0])
                self.chat.delete(f"{row - 1}.0", f"{row + 2}.0")
            self.chat.config(state="disabled")
        except Exception:
            pass

    def _sys(self, msg, tag="sys"):
        self.after(0, self._add_msg, "system", msg, tag)

    # ── Code panel helpers ────────────────────────────────────────────────────
    def _cwrite(self, text, tag=None):
        self.code_panel.config(state="normal")
        if tag:
            self.code_panel.insert("end", text, tag)
        else:
            self.code_panel.insert("end", text)
        self.code_panel.config(state="disabled")
        self.code_panel.see("end")

    def _cclear(self):
        self.code_panel.config(state="normal")
        self.code_panel.delete("1.0", "end")
        self.code_panel.config(state="disabled")

    def _set_status(self, text, color):
        self.lbl_status.config(text=f"● {text}", fg=color)

    # ── TTS ───────────────────────────────────────────────────────────────────
    def _start_tts_worker(self):
        def worker():
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            while True:
                text = self.speak_q.get()
                if text is None:
                    break
                try:
                    self.after(0, self.lbl_voice.config, {"fg": C["accent"]})
                    loop.run_until_complete(self._speak(text))
                except Exception:
                    pass
                finally:
                    self.after(0, self.lbl_voice.config, {"fg": C["dim"]})
        threading.Thread(target=worker, daemon=True).start()

    async def _speak(self, text):
        if len(text) > 900:
            text = text[:900] + "."
        comm = edge_tts.Communicate(text, VOICE)
        with tempfile.NamedTemporaryFile(delete=False, suffix=".mp3") as f:
            tmp = f.name
        await comm.save(tmp)
        if PYGAME_OK:
            try:
                pygame.mixer.music.load(tmp)
                pygame.mixer.music.play()
                while pygame.mixer.music.get_busy():
                    await asyncio.sleep(0.05)
                pygame.mixer.music.stop()
            except Exception:
                pass
        else:
            proc = subprocess.Popen(["wmplayer", "/play", "/close", tmp],
                                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            await asyncio.sleep(max(2.5, len(text) / 14))
            proc.terminate()
        try: os.unlink(tmp)
        except: pass

    # ── Pulse animation ───────────────────────────────────────────────────────
    def _pulse_header(self):
        sets = [
            "  ⊗  ↦  ⟨⟩  ⧉  ∇  ⊕  ⊙  ≈  ⪰  ◊  ",
            "  ◊  ⊗  ↦  ⟨⟩  ⧉  ∇  ⊕  ⊙  ≈  ⪰  ",
            "  ⪰  ◊  ⊗  ↦  ⟨⟩  ⧉  ∇  ⊕  ⊙  ≈  ",
        ]
        cols = [C["dim"], "#252540", "#1a1a30"]
        self.lbl_glyphs.config(text=sets[self._pulse_idx % 3],
                               fg=cols[self._pulse_idx % 3])
        self._pulse_idx = (self._pulse_idx + 1) % 3
        self.after(1400, self._pulse_header)

    def destroy(self):
        self.speak_q.put(None)
        super().destroy()


# ── Entry ─────────────────────────────────────────────────────────────────────
if __name__ == "__main__":
    cfg_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "mother_config.json")
    if not os.environ.get("ANTHROPIC_API_KEY") and os.path.exists(cfg_path):
        try:
            with open(cfg_path) as f:
                cfg = json.load(f)
            if cfg.get("api_key"):
                os.environ["ANTHROPIC_API_KEY"] = cfg["api_key"]
        except Exception:
            pass
    app = MotherAI()
    app.mainloop()
