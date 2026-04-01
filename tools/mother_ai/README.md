# Mother AI — Live Stream Interface

The public face of Aeonmi. An AI that speaks, answers questions, and executes .ai code live on stream.

## What it is

Fullscreen terminal interface powered by Claude API + Edge TTS (Microsoft Aria Neural voice).
Warren operates as assistant in the background. Mother is the intelligence on screen.

## Features

- **Claude API** — real answers generated live, in character as Mother
- **Voice** — Microsoft Aria Neural (Edge TTS, free, no API key needed for voice)
- **Live .ai execution** — code blocks from responses run through Aeonmi.exe native
- **Stream-ready** — fullscreen dark terminal, quantum glyph aesthetic, pipeline animation
- **Conversation memory** — keeps last 12 turns of context

## Setup

1. Get an Anthropic API key from https://console.anthropic.com
2. Install dependencies: `pip install anthropic edge-tts pygame pyinstaller`
3. Run: `python mother_ai.py`
   - Or use the pre-built `MotherAI.exe` in AeonmiDist/
4. Enter your API key when prompted (saved to `mother_config.json`)

## Build exe

```
cd tools/mother_ai
pyinstaller --onefile --windowed --name MotherAI mother_ai.py
```

## Voice

Default: `en-US-AriaNeural` (Microsoft, warm female, free via edge-tts)

To change voice, edit `VOICE` constant in `mother_ai.py`.
Other good options: `en-US-JennyNeural`, `en-US-SaraNeural`

## Stream setup

- Open MotherAI.exe on your streaming PC
- Add as a Window Capture source in OBS
- Warren types questions from live chat into the input field
- Mother responds with voice + typewriter text
- Hit "RUN .AI" to execute any code Mother generates live

## Architecture

```
[Chat question from Warren]
         |
    Claude API (claude-opus-4-6)
         |
    Mother response (text)
         |
    ┌────┴────┐
    │         │
  Edge TTS  Typewriter
  (voice)   (display)
              │
         Code blocks?
              │
         Aeonmi.exe native
         (live execution)
```
