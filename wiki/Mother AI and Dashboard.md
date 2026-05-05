# Mother AI and Dashboard

## Mother AI loop

`aeonmi mother` runs the Mother embryo loop:
- interactive conversation mode
- optional one-shot file mode (`--file`)
- action queue and action log support
- configurable creator identity (`--creator`)

## Dashboard

Launch:

```bash
python Aeonmi_Master/dashboard.py
```

Default URL: `http://localhost:7777`

Dashboard includes:
- Mother conversation panel
- file explorer/actions
- shard/build/run panels
- persistent genesis memory (`Aeonmi_Master/genesis.json`)

## API integrations

Dashboard supports external LLM keys via env vars (e.g. Anthropic/OpenRouter/OpenAI) and routes requests based on available configuration.

