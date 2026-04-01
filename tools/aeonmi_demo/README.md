# AeonmiDemo

Animated showcase application cycling through 5 Aeonmi language demos.
Uses matrix rain background, typewriter code animation, and VM output panels.

## Honest notes
- This is a display-only application. No real VM execution happens.
- Code is typed character-by-character by a timer, not actually run.
- Use for advertising/streaming, not as a functional demo.

## Run from source
```bash
python aeonmi_demo.py
```

## Build .exe
```bash
pyinstaller --onefile --windowed --name AeonmiDemo aeonmi_demo.py
```

## Controls
- `ESC` — exit
- `←` / `→` — navigate scenes
