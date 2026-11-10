"""Verify the exact JUCE source included in an offline source archive."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
revision = "4f43011b96eb0636104cb3e433894cda98243626"
if (root / "SOURCE_REVISION").read_text().strip() != revision:
    raise SystemExit("Unexpected JUCE revision")
expected = json.loads((root / "SOURCE_SHA256.json").read_text())
for relative, digest in expected.items():
    path = root / relative
    if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
        raise SystemExit(f"JUCE source differs: {relative}")
print(f"PASS {len(expected)} bundled JUCE source files")
