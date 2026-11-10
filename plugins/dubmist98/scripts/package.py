"""Package a local Dubmist preview and complete corresponding source; never publish."""
from pathlib import Path
import hashlib
import json
import shutil
import zipfile

project = Path(__file__).resolve().parents[1]
repo = project.parents[1]
build = repo / 'target/dubmist-cmake'
release = repo / 'target/releases/dubmist98-0.2'
release.mkdir(parents=True, exist_ok=True)
bundle = build / 'Dubmist98_artefacts/Release/VST3/Dubmist 98.vst3'
dependency = repo / 'target/dubmist-deps/JUCE'
if not bundle.is_dir() or not dependency.is_dir():
    raise SystemExit('Build and validate before packaging')
for f in (build / 'validation').glob('*.dm98.xml'):
    shutil.copy2(f, project / 'factory' / f.name)
for name in ('dubmist-vst3.png', 'dubmist-editor.png', 'dubmist-rhythm.png'):
    shutil.copy2(build / 'validation' / name, project / 'docs/images' / name)
files = [f for f in project.rglob('*') if f.is_file() and not {'build', '__pycache__'}.intersection(f.relative_to(project).parts)]
with zipfile.ZipFile(release / 'Dubmist98-0.2-Windows-x64.zip', 'w', zipfile.ZIP_DEFLATED) as z:
    for f in bundle.rglob('*'):
        if f.is_file():
            z.write(f, 'Dubmist 98.vst3/' + f.relative_to(bundle).as_posix())
    for f in files:
        rel = f.relative_to(project)
        if rel.parts[0] in {'docs', 'factory', 'notices'} or rel.name in {'README.md', 'LICENSE', 'NOTICE', 'UPSTREAM.md'}:
            z.write(f, rel.as_posix())
source = [f for f in dependency.rglob('*') if f.is_file() and '.git' not in f.relative_to(dependency).parts and f.name not in {'SOURCE_SHA256.json', 'SOURCE_REVISION'}]
manifest = {f.relative_to(dependency).as_posix(): hashlib.sha256(f.read_bytes()).hexdigest() for f in source}
with zipfile.ZipFile(release / 'Dubmist98-0.2-complete-source.zip', 'w', zipfile.ZIP_DEFLATED) as z:
    for f in files:
        z.write(f, 'plugins/dubmist98/' + f.relative_to(project).as_posix())
    prefix = 'target/dubmist-deps/JUCE/'
    for f in source:
        z.write(f, prefix + f.relative_to(dependency).as_posix())
    z.writestr(prefix + 'SOURCE_REVISION', '4f43011b96eb0636104cb3e433894cda98243626\n')
    z.writestr(prefix + 'SOURCE_SHA256.json', json.dumps(manifest, indent=2) + '\n')
with zipfile.ZipFile(release / 'Dubmist98-0.2-audio-examples.zip', 'w', zipfile.ZIP_DEFLATED) as z:
    for f in (build / 'validation').glob('*.wav'):
        z.write(f, f.name)
    z.write(project / 'docs/validation.md', 'Read-first-audio-notes.md')
    z.write(project / 'tests/fixtures/CC0.txt', 'CC0-recorded-loop-license.txt')
    z.write(project / 'tests/fixtures/SOURCE.md', 'Recorded-loop-source.md')
(release / 'SHA256.json').write_text(json.dumps({f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in release.glob('*.zip')}, indent=2) + '\n')
print(release)
