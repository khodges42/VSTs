"""Inventory the locked Windows graph and retain available upstream license texts.

Metadata screening is not a legal certification or vulnerability audit.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "docs/dependencies-windows.json"
SELECTIONS = {
    "Apache-2.0": "Apache-2.0",
    "MIT": "MIT",
    "ISC": "ISC",
    "BSL-1.0": "BSL-1.0",
    "GPLv3": "GPL-3.0-only (upstream declares GPLv3)",
    "GPL-3.0-or-later": "GPL-3.0-or-later",
    "Apache-2.0 AND MIT": "Apache-2.0 AND MIT",
    "(MIT OR Apache-2.0) AND Unicode-3.0": "Apache-2.0 AND Unicode-3.0",
    "(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0": "Apache-2.0 AND OFL-1.1 AND Ubuntu-font-1.0",
}


def selection(expression):
    if expression in SELECTIONS:
        return SELECTIONS[expression]
    if expression in {
        "MIT OR Apache-2.0", "MIT/Apache-2.0", "MIT / Apache-2.0",
        "Apache-2.0 OR MIT", "BlueOak-1.0.0 OR MIT OR Apache-2.0",
        "Apache-2.0 OR MIT OR Zlib", "Zlib OR Apache-2.0 OR MIT",
        "MIT OR Apache-2.0 OR Zlib", "BSD-2-Clause OR Apache-2.0 OR MIT",
    }:
        return "Apache-2.0"
    if expression == "Unlicense OR MIT":
        return "MIT"
    raise SystemExit(f"Unreviewed license expression: {expression!r}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--collect", type=Path)
    args = parser.parse_args()
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--offline", "--format-version", "1",
        "--filter-platform", "x86_64-pc-windows-msvc",
    ], cwd=ROOT))
    records = []
    for package in metadata["packages"]:
        if package["source"] is None:
            continue
        source = Path(package["manifest_path"]).parent
        licenses = [(p,p.relative_to(source).as_posix()) for p in sorted(source.rglob("*")) if p.is_file() and (
            p.name.lower().startswith(("license", "copying", "notice"))
            or p.name.lower() in {"ofl.txt", "ufl.txt", "hack-regular.txt", "emoji-icon-font-mit-license.txt"}
        )]
        if not licenses and package["source"].startswith("git+"):
            # Workspace subcrates inherit the nearest parent notice. Never copy
            # ../ paths into the output tree.
            for parent in source.parents:
                candidates = sorted(p for p in parent.iterdir() if p.is_file() and p.name.lower().startswith(("license", "copying", "notice")))
                if candidates:
                    licenses = [(p,"inherited/"+p.name) for p in candidates]
                    break
                if (parent / ".git").exists():
                    break
        supplement = "egui-0.31.1" if package["name"] in {"ecolor","egui","egui_glow","emath","epaint"} else f"{package['name']}-{package['version']}"
        supplement_root = ROOT / "notices" / supplement
        if supplement_root.is_dir():
            licenses.extend((p,"supplement/"+p.name) for p in sorted(supplement_root.iterdir()) if p.is_file())
        if not licenses:
            raise SystemExit(f"Missing license text for {package['name']} {package['version']}")
        # Keep relative paths: a fresh checkout must not depend on this machine.
        record = {"name": package["name"], "version": package["version"],
                  "source": package["source"], "declared": package["license"],
                  "selected": selection(package["license"]),
                  "license_files": [name for _,name in licenses]}
        records.append(record)
        if args.collect:
            destination = args.collect / f"{package['name']}-{package['version']}"
            for path,relative in licenses:
                target = destination / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(path, target)
    records.sort(key=lambda r: (r["name"], r["version"], r["source"]))
    text = json.dumps(records, indent=2) + "\n"
    if args.write:
        INVENTORY.parent.mkdir(parents=True, exist_ok=True)
        INVENTORY.write_text(text, encoding="utf-8")
    elif not INVENTORY.exists() or json.loads(INVENTORY.read_text(encoding="utf-8")) != records:
        raise SystemExit("Dependency inventory differs; review licenses, then use --write")
    if args.collect:
        shutil.copyfile(INVENTORY, args.collect / "inventory.json")
    print(f"Reviewed metadata matches {len(records)} locked external packages (includes development dependencies)")


if __name__ == "__main__":
    main()
