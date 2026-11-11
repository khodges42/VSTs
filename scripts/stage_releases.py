"""Stage existing local Windows builds for installation; never build/install/publish."""
from pathlib import Path
import hashlib
import json
import shutil
import struct
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "releases/windows-x64"
RUST = ("VIXEN", "ECHOGIRL", "EIS", "SPASM")
CPP = (
    ("soundgyal99", "0.3", "SOUNDGYAL99", "SOUNDGYAL 99.vst3"),
    ("dubmist98", "0.2", "Dubmist98", "Dubmist 98.vst3"),
)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def x64(binary):
    data = binary.read_bytes()
    if data[:2] != b"MZ" or len(data) < 64:
        raise RuntimeError(f"Not a Windows plugin binary: {binary}")
    offset = struct.unpack_from("<I", data, 60)[0]
    if data[offset:offset + 4] != b"PE\0\0" or struct.unpack_from("<H", data, offset + 4)[0] != 0x8664:
        raise RuntimeError(f"Not Windows x64: {binary}")


def main():
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    plans = []
    # Preflight every input before staging; do not silently omit missing products.
    for name in RUST:
        slug = name.lower()
        package = tomllib.loads((ROOT / f"plugins/{slug}/Cargo.toml").read_text())["package"]
        version = package["version"]
        if isinstance(version, dict):
            version = workspace["workspace"]["package"]["version"]
        bundle = ROOT / f"target/bundled/{name}.vst3"
        x64(bundle / f"Contents/x86_64-win/{name}.vst3")
        plans.append((slug, version, bundle, None))
    for slug, version, prefix, bundle_name in CPP:
        folder = ROOT / f"target/releases/{slug}-{version}"
        hashes = json.loads((folder / "SHA256.json").read_text())
        for name, expected in hashes.items():
            archive = folder / name
            if archive.parent != folder or archive.suffix != ".zip" or digest(archive) != expected:
                raise RuntimeError(f"Invalid package/checksum: {archive}")
            with zipfile.ZipFile(archive) as z:
                if z.testzip() is not None:
                    raise RuntimeError(f"Corrupt package: {archive}")
                for entry in z.namelist():
                    if ".." in Path(entry.replace("\\", "/")).parts or entry.startswith(("/", "\\")) or ":" in entry:
                        raise RuntimeError(f"Unsafe archive path: {entry}")
        plans.append((slug, version, bundle_name, (folder, hashes, prefix)))

    records = []
    for slug, version, bundle, cpp in plans:
        for part in ("packages", "source", "examples", "VST3"):
            (OUT / part).mkdir(parents=True, exist_ok=True)
        if cpp is None:
            archive = OUT / f"packages/{bundle.stem}-{version}-Windows-x64.zip"
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as z:
                for f in sorted(bundle.rglob("*")):
                    if f.is_file():
                        z.write(f, bundle.name + "/" + f.relative_to(bundle).as_posix())
            target = OUT / "VST3" / bundle.name
            shutil.copytree(bundle, target, dirs_exist_ok=True)
            source = f"../../plugins/{slug}"
        else:
            folder, hashes, prefix = cpp
            archive = OUT / f"packages/{prefix}-{version}-Windows-x64.zip"
            for name in hashes:
                part = "source" if "complete-source" in name else "examples" if ("audio" in name or "audition" in name) else "packages"
                shutil.copy2(folder / name, OUT / part / name)
            with zipfile.ZipFile(archive) as z:
                for entry in z.namelist():
                    if entry.startswith(bundle + "/"):
                        z.extract(entry, OUT / "VST3")
            target = OUT / "VST3" / bundle
            source = f"source/{prefix}-{version}-complete-source.zip"
        binary = target / "Contents/x86_64-win" / target.name
        x64(binary)
        records.append({"plugin": slug, "version": version, "bundle": target.name,
                        "package": archive.relative_to(OUT).as_posix(),
                        "sha256": digest(archive), "binary_sha256": digest(binary), "source": source})

    (OUT / "current.json").write_text(json.dumps(records, indent=2) + "\n")
    hashes = {p.relative_to(OUT).as_posix(): digest(p) for folder in ("packages", "source", "examples") for p in sorted((OUT / folder).glob("*.zip"))}
    (OUT / "SHA256.json").write_text(json.dumps(hashes, indent=2) + "\n")
    rows = "\n".join(f"| {r['bundle'][:-5]} | {r['version']} | [Install ZIP]({r['package']}) | [Source]({r['source']}) |" for r in records)
    (OUT / "README.md").write_text(
        "# Windows x64 local install packages\n\n"
        "Copy complete folders from **VST3/** to a VST3 folder scanned by your DAW, or extract one install ZIP. "
        "Do not copy only the inner binary. Close the DAW before replacing a plugin.\n\n"
        "| Plugin | Version | Package | Source |\n| --- | --- | --- | --- |\n" + rows +
        "\n\nThese are unsigned local development builds, not published GitHub releases. "
        "Staging checks packaging integrity and x64 format; it does not qualify host compatibility. "
        "See each product's validation notes.\n\n"
        "**packages/**: install ZIPs; **VST3/**: ready-to-copy folders; **source/**: complete C++ source archives; "
        "**examples/**: listening examples; **SHA256.json**: archive checksums. "
        "Rust sources and locked dependency/build instructions remain in the repository.\n",
        encoding="utf-8")
    print(f"Staged {len(records)} Windows x64 plugins in {OUT}")


if __name__ == "__main__":
    main()
