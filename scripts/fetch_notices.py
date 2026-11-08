"""Reproduce checked-in supplements omitted from registry source archives."""
import json
from pathlib import Path
import urllib.request

root = Path(__file__).resolve().parents[1]
for item in json.loads((root / "notices/sources.json").read_text()):
    for name in item["files"]:
        url = f"https://raw.githubusercontent.com/{item['repository']}/{item['revision']}/{name}"
        with urllib.request.urlopen(url, timeout=30) as response:
            data = response.read()
        path = root / "notices" / item["group"] / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        print(f"Retained {item['group']}/{name}")
