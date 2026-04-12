import json
from pathlib import Path

genesis_path = Path("genesis.json")
data = json.loads(genesis_path.read_text())

# Increment cognitive metrics
data["cognitive"