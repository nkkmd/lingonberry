from pathlib import Path

files = sorted(Path("packages").glob("*/Cargo.toml")) + [Path("Cargo.lock")]
changed = []
for path in files:
    text = path.read_text()
    count = text.count('0.8.0')
    if count:
        path.write_text(text.replace('0.8.0', '0.9.0'))
        changed.append((str(path), count))

if len(changed) < 8:
    raise SystemExit(f"expected all seven crates and Cargo.lock to change, got {changed}")

print("updated version references:")
for path, count in changed:
    print(f"- {path}: {count}")
