# AeonmiStudio

3-panel IDE for the Aeonmi language. Wraps `Aeonmi.exe native` for code execution.

## Requirements
- Python 3.11+
- `Aeonmi.exe` in the same directory as `AeonmiStudio.exe` (or same dir as this script)
- PyInstaller (to build the .exe)

## Run from source
```bash
python aeonmi_studio.py
```

## Build .exe
```bash
pyinstaller --onefile --windowed --name AeonmiStudio aeonmi_studio.py
```
Then place `Aeonmi.exe` alongside the output `AeonmiStudio.exe`.

## Build installer (requires NSIS)
```bash
makensis AeonmiStudio.nsi
```

## Honest notes
- The VM pipeline panel (Lexer → Parser → IR → VM) animates on a fixed timer.
  It does NOT receive real stage-complete signals from the VM.
- Code execution is real: code is saved to a temp .ai file and run via `Aeonmi.exe native`.
- The syntax highlighting is regex-based and approximate.
