#!/usr/bin/env python3
"""
aeonmi_launcher.py — Aeonmi Software Creator
=============================================
Standalone launcher that:
  1. Runs any .ai file through the Aeonmi runtime
  2. Drives the Forge meta-code generator interactively
  3. Lists and launches showcase demos

Usage:
  python aeonmi_launcher.py                  # interactive menu
  python aeonmi_launcher.py run <file.ai>    # execute a .ai file
  python aeonmi_launcher.py forge            # run the Forge code generator
  python aeonmi_launcher.py demo <name>      # run a named demo
"""

import sys
import os
import subprocess
import textwrap
import shutil

# ── Paths ──────────────────────────────────────────────────────────────────
SCRIPT_DIR  = os.path.dirname(os.path.abspath(__file__))
AI_ROOT     = os.path.join(SCRIPT_DIR, "aeonmi_ai")

# Binary resolution — checks all known locations in order
def _find_binary():
    candidates = [
        os.path.join(SCRIPT_DIR, "Aeonmi.exe"),
        os.path.join(SCRIPT_DIR, "aeonmi_project.exe"),
        os.path.join(os.path.dirname(SCRIPT_DIR), "target", "release", "Aeonmi.exe"),
        os.path.join(os.path.dirname(SCRIPT_DIR), "target", "release", "aeonmi_project.exe"),
        r"C:\RustTarget\release\aeonmi_project.exe",
        r"C:\RustTarget\release\Aeonmi.exe",
    ]
    # Also check PATH
    for name in ("Aeonmi", "aeonmi_project"):
        found = shutil.which(name)
        if found:
            candidates.insert(0, found)
    for c in candidates:
        if c and os.path.exists(c):
            return c
    return candidates[-1]  # return last as fallback so error message is useful

AEONMI_EXE = _find_binary()

DEMOS = {
    "forge":   os.path.join(AI_ROOT, "demo", "forge.ai"),
    "quantum": os.path.join(AI_ROOT, "demo", "quantum_cognition.ai"),
    "swarm":   os.path.join(AI_ROOT, "demo", "swarm_os.ai"),
    "agent":   os.path.join(AI_ROOT, "demo", "agent_demo.ai"),
}

ANSI_RESET  = "\033[0m"
ANSI_BOLD   = "\033[1m"
ANSI_CYAN   = "\033[96m"
ANSI_YELLOW = "\033[93m"
ANSI_GREEN  = "\033[92m"
ANSI_RED    = "\033[91m"
ANSI_MAGENTA= "\033[95m"


# ── Helpers ─────────────────────────────────────────────────────────────────
def header():
    print(f"""
{ANSI_MAGENTA}  ╔══════════════════════════════════════════════════════╗
  ║{ANSI_YELLOW}   █████╗ ███████╗ ██████╗ ███╗   ██╗███╗   ███╗██╗{ANSI_MAGENTA}  ║
  ║{ANSI_YELLOW}  ██╔══██╗██╔════╝██╔═══██╗████╗  ██║████╗ ████║██║{ANSI_MAGENTA}  ║
  ║{ANSI_YELLOW}  ███████║█████╗  ██║   ██║██╔██╗ ██║██╔████╔██║██║{ANSI_MAGENTA}  ║
  ║{ANSI_YELLOW}  ██╔══██║██╔══╝  ██║   ██║██║╚██╗██║██║╚██╔╝██║██║{ANSI_MAGENTA}  ║
  ║{ANSI_YELLOW}  ██║  ██║███████╗╚██████╔╝██║ ╚████║██║ ╚═╝ ██║██║{ANSI_MAGENTA}  ║
  ║{ANSI_YELLOW}  ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝     ╚═╝╚═╝{ANSI_MAGENTA}  ║
  ╠══════════════════════════════════════════════════════╣
  ║{ANSI_CYAN}  ⊗ SOFTWARE CREATOR  ·  AI-Native Language Platform{ANSI_MAGENTA}  ║
  ║{ANSI_RESET}    v2.0 · Quantum-Glyph-Sovereign · Built by AI for AI{ANSI_MAGENTA}  ║
  ╚══════════════════════════════════════════════════════╝{ANSI_RESET}
""")


def check_runtime():
    if not os.path.exists(AEONMI_EXE):
        print(f"{ANSI_RED}ERROR: Aeonmi.exe not found at {AEONMI_EXE}{ANSI_RESET}")
        print("  Set AEONMI_EXE environment variable or place Aeonmi.exe next to this script.")
        sys.exit(1)


def run_ai(path, cwd=None):
    """Execute a .ai file through the Aeonmi runtime."""
    check_runtime()
    if not os.path.exists(path):
        print(f"{ANSI_RED}File not found: {path}{ANSI_RESET}")
        return 1
    result = subprocess.run(
        [AEONMI_EXE, "native", path],
        cwd=cwd or SCRIPT_DIR,
    )
    return result.returncode


# ── Interactive menu ─────────────────────────────────────────────────────────
def menu():
    header()
    while True:
        print(f"{ANSI_CYAN}{'─'*54}{ANSI_RESET}")
        print(f"  {ANSI_BOLD}MAIN MENU{ANSI_RESET}")
        print(f"  {ANSI_YELLOW}1{ANSI_RESET}  Run Forge — generate .ai code stubs from a blueprint")
        print(f"  {ANSI_YELLOW}2{ANSI_RESET}  Demo: Quantum Cognition Engine")
        print(f"  {ANSI_YELLOW}3{ANSI_RESET}  Demo: Swarm OS — Autonomous Agent Colony")
        print(f"  {ANSI_YELLOW}4{ANSI_RESET}  Demo: Agent Decision System")
        print(f"  {ANSI_YELLOW}5{ANSI_RESET}  Run a custom .ai file")
        print(f"  {ANSI_YELLOW}6{ANSI_RESET}  Run all showcase demos")
        print(f"  {ANSI_YELLOW}q{ANSI_RESET}  Quit")
        print(f"{ANSI_CYAN}{'─'*54}{ANSI_RESET}")
        choice = input("  Choice: ").strip().lower()

        if choice == "1":
            forge_interactive()
        elif choice == "2":
            print(f"\n{ANSI_GREEN}▶ Quantum Cognition Engine{ANSI_RESET}")
            run_ai(DEMOS["quantum"])
        elif choice == "3":
            print(f"\n{ANSI_GREEN}▶ Swarm OS{ANSI_RESET}")
            run_ai(DEMOS["swarm"])
        elif choice == "4":
            print(f"\n{ANSI_GREEN}▶ Agent Decision System{ANSI_RESET}")
            run_ai(DEMOS["agent"])
        elif choice == "5":
            path = input("  Path to .ai file: ").strip().strip('"')
            run_ai(path)
        elif choice == "6":
            run_all_demos()
        elif choice in ("q", "quit", "exit"):
            print(f"\n{ANSI_CYAN}  Aeonmi — built by AI, for AI.{ANSI_RESET}\n")
            break
        else:
            print(f"  {ANSI_RED}Unknown choice.{ANSI_RESET}")
        print()


def forge_interactive():
    """Walk the user through generating .ai code with the Forge."""
    print(f"\n{ANSI_CYAN}{'═'*54}")
    print("  AEONMI FORGE — AI-Native Code Generator")
    print(f"{'═'*54}{ANSI_RESET}")
    print(textwrap.dedent("""
      The Forge generates .ai function stubs from a blueprint spec.
      A blueprint is: [n_funcs, n_types, dep_density]

        n_funcs     — number of functions to generate (1..11)
        n_types     — number of data types used (1..7)
        dep_density — edge density 0=sparse 1=medium 2=dense

      Example: 5 3 1  →  5 functions, 3 types, medium deps
    """))

    try:
        raw = input("  Blueprint [n_funcs n_types dep_density]: ").strip()
        parts = raw.split()
        if len(parts) != 3:
            raise ValueError("need 3 numbers")
        n_funcs, n_types, dep = int(parts[0]), int(parts[1]), int(parts[2])
        n_funcs  = max(1, min(11, n_funcs))
        n_types  = max(1, min(7, n_types))
        dep      = max(0, min(2, dep))
    except Exception as e:
        print(f"  {ANSI_RED}Invalid input ({e}). Using defaults: 5 3 1{ANSI_RESET}")
        n_funcs, n_types, dep = 5, 3, 1

    print(f"\n  {ANSI_GREEN}Running Forge with blueprint: {n_funcs} functions, {n_types} types, dep={dep}{ANSI_RESET}\n")
    run_ai(DEMOS["forge"])


def run_all_demos():
    """Run all 3 showcase demos in sequence."""
    showcases = [
        ("Aeonmi Forge — Meta Code Generator", "forge"),
        ("Quantum Cognition Engine",            "quantum"),
        ("Swarm OS — Autonomous Agent Colony",  "swarm"),
    ]
    print(f"\n{ANSI_CYAN}{'═'*54}")
    print("  RUNNING ALL SHOWCASE DEMOS")
    print(f"{'═'*54}{ANSI_RESET}\n")
    for name, key in showcases:
        print(f"{ANSI_YELLOW}{'─'*54}{ANSI_RESET}")
        print(f"{ANSI_BOLD}  ▶ {name}{ANSI_RESET}\n")
        run_ai(DEMOS[key])
        print()
    print(f"{ANSI_GREEN}{'═'*54}")
    print("  All demos complete.")
    print(f"{'═'*54}{ANSI_RESET}")


# ── CLI dispatch ─────────────────────────────────────────────────────────────
def main():
    args = sys.argv[1:]

    if not args:
        menu()
        return

    cmd = args[0].lower()

    if cmd == "run":
        if len(args) < 2:
            print("Usage: aeonmi_launcher run <file.ai>")
            sys.exit(1)
        sys.exit(run_ai(args[1]))

    elif cmd == "forge":
        header()
        run_ai(DEMOS["forge"])

    elif cmd == "demo":
        if len(args) < 2:
            print("Available demos:", ", ".join(DEMOS))
            sys.exit(1)
        name = args[1].lower()
        if name not in DEMOS:
            print(f"Unknown demo '{name}'. Available: {', '.join(DEMOS)}")
            sys.exit(1)
        header()
        run_ai(DEMOS[name])

    elif cmd == "all":
        header()
        run_all_demos()

    elif cmd in ("-h", "--help", "help"):
        print(__doc__)

    else:
        print(f"Unknown command '{cmd}'. Run without args for interactive menu.")
        sys.exit(1)


if __name__ == "__main__":
    main()
