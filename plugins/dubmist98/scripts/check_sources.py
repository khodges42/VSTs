"""Verify the audited upstream source inventory without modifying the tree."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
expected = json.loads((root / "vendor-sha256.json").read_text(encoding="utf-8"))
actual = {
    str(p.relative_to(root / "vendor")).replace("\\", "/"): hashlib.sha256(p.read_bytes()).hexdigest()
    for p in (root / "vendor").rglob("*") if p.is_file()
}
if actual != expected:
    raise SystemExit("Vendor inventory differs; review changes and notices before updating hashes")
print(f"PASS {len(actual)} upstream files match the reviewed SHA-256 inventory")
