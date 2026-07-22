from pathlib import Path

path = Path("scripts/apply_v0_9_protocol_hardening.py")
text = path.read_text()
old = '            input.push_str("{\\"x\\":");'
new = '            input.push_str(r#"{"x":"#);'
if text.count(old) != 1:
    raise SystemExit("generated raw string target was not unique")
path.write_text(text.replace(old, new))
