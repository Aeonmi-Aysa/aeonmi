"""
screen_recorder.py — Mother AI screen recording.

Mother records herself during autonomous sessions.
Output: C:\\Users\\wlwil\\Videos\\Mother\\YYYY-MM-DD_HH-MM_<reason>.mp4

Two modes:
  - Full video: mss + cv2.VideoWriter (requires opencv-python + mss)
  - Snapshot:   PIL / mss (lighter, single frame)

CLI:
  python screen_recorder.py start [reason]
  python screen_recorder.py stop
  python screen_recorder.py snapshot [note]
  python screen_recorder.py status

Flask endpoints (imported by dashboard.py):
  POST /api/record/start   { "reason": "..." }
  POST /api/record/stop
  POST /api/record/snapshot { "note": "..." }
  GET  /api/record/status
"""

from __future__ import annotations

import os
import sys
import json
import time
import threading
import datetime
import pathlib
from typing import Optional

# ── Output directory ─────────────────────────────────────────────────────────

MOTHER_VIDEO_DIR = pathlib.Path(r"C:\Users\wlwil\Videos\Mother")

def ensure_video_dir() -> pathlib.Path:
    MOTHER_VIDEO_DIR.mkdir(parents=True, exist_ok=True)
    return MOTHER_VIDEO_DIR


# ── Optional heavy imports ────────────────────────────────────────────────────

def _try_import_cv2():
    try:
        import cv2
        return cv2
    except ImportError:
        return None

def _try_import_mss():
    try:
        import mss
        return mss
    except ImportError:
        return None

def _try_import_pil():
    try:
        from PIL import ImageGrab
        return ImageGrab
    except ImportError:
        return None

def _try_import_numpy():
    try:
        import numpy as np
        return np
    except ImportError:
        return None


# ── State ─────────────────────────────────────────────────────────────────────

_state: dict = {
    "recording":   False,
    "reason":      "",
    "start_ts":    "",
    "output_path": "",
    "snapshots":   [],
    "frames_captured": 0,
}

_stop_event: threading.Event = threading.Event()
_record_thread: Optional[threading.Thread] = None


# ── Helpers ───────────────────────────────────────────────────────────────────

def _ts_filename(reason: str = "session") -> str:
    now = datetime.datetime.now()
    safe_reason = "".join(c if c.isalnum() or c in "-_" else "_" for c in reason)[:40]
    return now.strftime("%Y-%m-%d_%H-%M") + f"_{safe_reason}"


def _grab_frame(mss_instance, monitor: dict):
    """Grab one frame via mss, return as numpy BGR array or None."""
    np = _try_import_numpy()
    if np is None:
        return None
    try:
        img = mss_instance.grab(monitor)
        frame = np.array(img)
        # mss returns BGRA — drop alpha, keep BGR for cv2
        return frame[:, :, :3]
    except Exception:
        return None


def _record_loop(output_path: str, fps: int = 10):
    """Background thread: capture frames and write video."""
    cv2 = _try_import_cv2()
    mss_mod = _try_import_mss()
    np = _try_import_numpy()

    if cv2 is None or mss_mod is None or np is None:
        # Fallback: take periodic PIL snapshots instead
        _pil_fallback_loop(output_path)
        return

    with mss_mod.mss() as sct:
        monitor = sct.monitors[0]  # full virtual screen
        width   = monitor["width"]
        height  = monitor["height"]

        fourcc = cv2.VideoWriter_fourcc(*"mp4v")
        writer = cv2.VideoWriter(output_path, fourcc, fps, (width, height))

        interval = 1.0 / fps
        while not _stop_event.is_set():
            t0 = time.time()
            frame = _grab_frame(sct, monitor)
            if frame is not None:
                writer.write(frame)
                _state["frames_captured"] += 1
            elapsed = time.time() - t0
            sleep_time = max(0.0, interval - elapsed)
            _stop_event.wait(timeout=sleep_time)

        writer.release()


def _pil_fallback_loop(output_path: str, interval_secs: float = 3.0):
    """Fallback: save periodic PNG snapshots when cv2/mss unavailable."""
    ImageGrab = _try_import_pil()
    if ImageGrab is None:
        return

    snap_dir = MOTHER_VIDEO_DIR / "snapshots"
    snap_dir.mkdir(parents=True, exist_ok=True)
    base = pathlib.Path(output_path).stem

    while not _stop_event.is_set():
        try:
            img  = ImageGrab.grab()
            ts   = datetime.datetime.now().strftime("%H-%M-%S")
            path = snap_dir / f"{base}_{ts}.png"
            img.save(str(path))
            _state["snapshots"].append(str(path))
            _state["frames_captured"] += 1
        except Exception:
            pass
        _stop_event.wait(timeout=interval_secs)


# ── Public API ────────────────────────────────────────────────────────────────

def start_recording(reason: str = "autonomous_session") -> dict:
    global _record_thread

    if _state["recording"]:
        return {"ok": False, "error": "Already recording", "path": _state["output_path"]}

    ensure_video_dir()
    filename = _ts_filename(reason) + ".mp4"
    out_path = str(MOTHER_VIDEO_DIR / filename)

    _stop_event.clear()
    _state.update({
        "recording":       True,
        "reason":          reason,
        "start_ts":        datetime.datetime.now().isoformat(),
        "output_path":     out_path,
        "frames_captured": 0,
        "snapshots":       [],
    })

    _record_thread = threading.Thread(
        target=_record_loop, args=(out_path,), daemon=True, name="mother-recorder"
    )
    _record_thread.start()

    return {"ok": True, "path": out_path, "reason": reason}


def stop_recording() -> dict:
    if not _state["recording"]:
        return {"ok": False, "error": "Not recording"}

    _stop_event.set()
    if _record_thread is not None:
        _record_thread.join(timeout=5)

    _state["recording"] = False
    path    = _state["output_path"]
    frames  = _state["frames_captured"]
    snaps   = _state["snapshots"]

    result = {
        "ok":              True,
        "path":            path,
        "frames_captured": frames,
        "snapshots":       snaps,
        "duration_est":    f"{frames / 10:.1f}s" if frames > 0 else "unknown",
    }
    return result


def take_snapshot(note: str = "") -> dict:
    ImageGrab = _try_import_pil()
    mss_mod   = _try_import_mss()
    np        = _try_import_numpy()
    cv2       = _try_import_cv2()

    ensure_video_dir()
    snap_dir = MOTHER_VIDEO_DIR / "snapshots"
    snap_dir.mkdir(parents=True, exist_ok=True)

    safe_note = "".join(c if c.isalnum() or c in "-_" else "_" for c in note)[:30] if note else "snap"
    ts   = datetime.datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    path = str(snap_dir / f"{ts}_{safe_note}.png")

    if mss_mod is not None and np is not None and cv2 is not None:
        try:
            with mss_mod.mss() as sct:
                monitor = sct.monitors[0]
                img = sct.grab(monitor)
                arr = np.array(img)[:, :, :3]
                cv2.imwrite(path, arr)
            _state["snapshots"].append(path)
            return {"ok": True, "path": path}
        except Exception as e:
            pass

    if ImageGrab is not None:
        try:
            img = ImageGrab.grab()
            img.save(path)
            _state["snapshots"].append(path)
            return {"ok": True, "path": path}
        except Exception as e:
            return {"ok": False, "error": str(e)}

    return {"ok": False, "error": "No screenshot library available. pip install mss opencv-python or Pillow"}


def get_status() -> dict:
    return {
        "recording":       _state["recording"],
        "reason":          _state["reason"],
        "start_ts":        _state["start_ts"],
        "output_path":     _state["output_path"],
        "frames_captured": _state["frames_captured"],
        "snapshots_count": len(_state["snapshots"]),
        "video_dir":       str(MOTHER_VIDEO_DIR),
        "backends": {
            "cv2_available":  _try_import_cv2() is not None,
            "mss_available":  _try_import_mss() is not None,
            "pil_available":  _try_import_pil() is not None,
        },
    }


# ── Flask integration (call from dashboard.py) ───────────────────────────────

def register_routes(app):
    """Register /api/record/* routes on a Flask app instance."""
    from flask import request, jsonify

    @app.route("/api/record/start", methods=["POST"])
    def api_record_start():
        data   = request.get_json(silent=True) or {}
        reason = data.get("reason", "dashboard_session")
        return jsonify(start_recording(reason))

    @app.route("/api/record/stop", methods=["POST"])
    def api_record_stop():
        return jsonify(stop_recording())

    @app.route("/api/record/snapshot", methods=["POST"])
    def api_record_snapshot():
        data = request.get_json(silent=True) or {}
        note = data.get("note", "")
        return jsonify(take_snapshot(note))

    @app.route("/api/record/status", methods=["GET"])
    def api_record_status():
        return jsonify(get_status())


# ── CLI ───────────────────────────────────────────────────────────────────────

def _cli():
    args = sys.argv[1:]
    if not args:
        print("Usage: screen_recorder.py <start [reason]|stop|snapshot [note]|status>")
        return

    cmd = args[0].lower()

    if cmd == "start":
        reason = " ".join(args[1:]) if len(args) > 1 else "cli_session"
        result = start_recording(reason)
        print(json.dumps(result, indent=2))
        if result["ok"]:
            print(f"\nRecording started → {result['path']}")
            print("Press Ctrl+C to stop.")
            try:
                while _state["recording"]:
                    time.sleep(1)
            except KeyboardInterrupt:
                print("\nStopping...")
                result = stop_recording()
                print(json.dumps(result, indent=2))

    elif cmd == "stop":
        result = stop_recording()
        print(json.dumps(result, indent=2))

    elif cmd == "snapshot":
        note = " ".join(args[1:]) if len(args) > 1 else ""
        result = take_snapshot(note)
        print(json.dumps(result, indent=2))

    elif cmd == "status":
        result = get_status()
        print(json.dumps(result, indent=2))

    else:
        print(f"Unknown command: {cmd}")
        print("Commands: start [reason] | stop | snapshot [note] | status")


if __name__ == "__main__":
    _cli()
