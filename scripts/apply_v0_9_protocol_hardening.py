from pathlib import Path

path = Path("packages/protocol/src/lib.rs")
text = path.read_text()
replacements: list[tuple[str, str]] = []

replacements.append((
    "use std::fs;\nuse std::path::Path;\nuse std::process::Command;",
    "use std::fs::{self, OpenOptions};\nuse std::io::Write;\nuse std::path::{Path, PathBuf};\nuse std::process::Command;\nuse std::sync::atomic::{AtomicU64, Ordering};",
))

replacements.append((
    'pub const CARRIER_KIND_RELAY: &str = "relay";',
    'pub const CARRIER_KIND_RELAY: &str = "relay";\npub const MAX_JSON_INPUT_BYTES: usize = 1024 * 1024;\npub const MAX_JSON_NESTING_DEPTH: usize = 128;',
))

replacements.append((
    """pub fn parse_json(input: &str) -> Result<JsonValue, JsonError> {
    let mut parser = Parser {
        input: input.as_bytes(),
        position: 0,
    };""",
    """pub fn parse_json(input: &str) -> Result<JsonValue, JsonError> {
    if input.len() > MAX_JSON_INPUT_BYTES {
        return Err(JsonError {
            message: format!("JSON input exceeds {MAX_JSON_INPUT_BYTES} bytes"),
            position: MAX_JSON_INPUT_BYTES,
        });
    }
    let mut parser = Parser {
        input: input.as_bytes(),
        position: 0,
        depth: 0,
    };""",
))

replacements.append((
    """struct Parser<'a> {
    input: &'a [u8],
    position: usize,
}""",
    """struct Parser<'a> {
    input: &'a [u8],
    position: usize,
    depth: usize,
}""",
))

replacements.append((
    """    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.consume(b'{')?;
        self.skip_whitespace();
        let mut map = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(b':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                }
                Some(b'}') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.error("expected , or } in object")),
            }
        }
        Ok(JsonValue::Object(map))
    }""",
    """    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = (|| {
            self.consume(b'{')?;
            self.skip_whitespace();
            let mut map = BTreeMap::new();
            if self.peek() == Some(b'}') {
                self.position += 1;
                return Ok(JsonValue::Object(map));
            }
            loop {
                self.skip_whitespace();
                let key = self.parse_string()?;
                self.skip_whitespace();
                self.consume(b':')?;
                let value = self.parse_value()?;
                map.insert(key, value);
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                    }
                    Some(b'}') => {
                        self.position += 1;
                        break;
                    }
                    _ => return Err(self.error("expected , or } in object")),
                }
            }
            Ok(JsonValue::Object(map))
        })();
        self.depth -= 1;
        result
    }""",
))

replacements.append((
    """    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.consume(b'[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                }
                Some(b']') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.error("expected , or ] in array")),
            }
        }
        Ok(JsonValue::Array(items))
    }""",
    """    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = (|| {
            self.consume(b'[')?;
            self.skip_whitespace();
            let mut items = Vec::new();
            if self.peek() == Some(b']') {
                self.position += 1;
                return Ok(JsonValue::Array(items));
            }
            loop {
                let value = self.parse_value()?;
                items.push(value);
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.position += 1;
                    }
                    Some(b']') => {
                        self.position += 1;
                        break;
                    }
                    _ => return Err(self.error("expected , or ] in array")),
                }
            }
            Ok(JsonValue::Array(items))
        })();
        self.depth -= 1;
        result
    }""",
))

replacements.append((
    "    fn skip_whitespace(&mut self) {",
    """    fn enter_nesting(&mut self) -> Result<(), JsonError> {
        if self.depth >= MAX_JSON_NESTING_DEPTH {
            return Err(self.error(format!(
                "JSON nesting exceeds {MAX_JSON_NESTING_DEPTH}"
            )));
        }
        self.depth += 1;
        Ok(())
    }

    fn skip_whitespace(&mut self) {""",
))

old_signature = """fn verify_publish_request_signature_with_openssl(
    message: &[u8],
    public_key: &[u8],
    signature: &[u8],
) -> Result<(), String> {
    let temp_root = std::env::temp_dir().join(format!(
        "lingonberry-signature-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create temp dir: {}", error))?;

    let key_path = temp_root.join("public-key.der");
    let sig_path = temp_root.join("signature.bin");
    let msg_path = temp_root.join("message.bin");

    let mut der = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    der.extend_from_slice(public_key);
    fs::write(&key_path, der).map_err(|error| format!("failed to write public key: {}", error))?;
    fs::write(&sig_path, signature)
        .map_err(|error| format!("failed to write signature: {}", error))?;
    fs::write(&msg_path, message).map_err(|error| format!("failed to write message: {}", error))?;

    let output = Command::new("openssl")
        .args([
            "pkeyutl",
            "-verify",
            "-pubin",
            "-inkey",
            key_path
                .to_str()
                .ok_or_else(|| "temp key path is not valid UTF-8".to_string())?,
            "-keyform",
            "DER",
            "-rawin",
            "-in",
            msg_path
                .to_str()
                .ok_or_else(|| "temp message path is not valid UTF-8".to_string())?,
            "-sigfile",
            sig_path
                .to_str()
                .ok_or_else(|| "temp signature path is not valid UTF-8".to_string())?,
        ])
        .output()
        .map_err(|error| format!("failed to run openssl: {}", error))?;

    if output.status.success() {
        return Ok(());
    }

    Err("publisher.signature does not verify the canonical request payload".to_string())
}"""

new_signature = """static SIGNATURE_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct SignatureWorkspace {
    path: PathBuf,
}

impl SignatureWorkspace {
    fn create() -> Result<Self, String> {
        for _ in 0..32 {
            let counter = SIGNATURE_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "lingonberry-signature-{}-{timestamp}-{counter}",
                std::process::id()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err("failed to create signature verification workspace".to_string()),
            }
        }
        Err("failed to create unique signature verification workspace".to_string())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SignatureWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "failed to create signature verification artifact".to_string())?;
    file.write_all(bytes)
        .map_err(|_| "failed to write signature verification artifact".to_string())
}

fn verify_publish_request_signature_with_openssl(
    message: &[u8],
    public_key: &[u8],
    signature: &[u8],
) -> Result<(), String> {
    let workspace = SignatureWorkspace::create()?;
    let key_path = workspace.path().join("public-key.der");
    let sig_path = workspace.path().join("signature.bin");
    let msg_path = workspace.path().join("message.bin");

    let mut der = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    der.extend_from_slice(public_key);
    write_new_file(&key_path, &der)?;
    write_new_file(&sig_path, signature)?;
    write_new_file(&msg_path, message)?;

    let output = Command::new("openssl")
        .arg("pkeyutl")
        .arg("-verify")
        .arg("-pubin")
        .arg("-inkey")
        .arg(&key_path)
        .arg("-keyform")
        .arg("DER")
        .arg("-rawin")
        .arg("-in")
        .arg(&msg_path)
        .arg("-sigfile")
        .arg(&sig_path)
        .output()
        .map_err(|_| "failed to run signature verification command".to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err("publisher.signature does not verify the canonical request payload".to_string())
    }
}"""
replacements.append((old_signature, new_signature))

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"exact-match replacement count was {count}, expected 1")
    text = text.replace(old, new)

path.write_text(text)

Path("packages/protocol/tests/parser_limits.rs").write_text("""use lingonberry_protocol::{parse_json, MAX_JSON_INPUT_BYTES, MAX_JSON_NESTING_DEPTH};

#[test]
fn rejects_input_larger_than_limit() {
    let input = " ".repeat(MAX_JSON_INPUT_BYTES + 1);
    let error = parse_json(&input).unwrap_err();
    assert!(error.message.contains("exceeds"));
}

#[test]
fn accepts_input_at_limit_when_valid() {
    let padding = " ".repeat(MAX_JSON_INPUT_BYTES - 4);
    let input = format!("null{padding}");
    assert_eq!(input.len(), MAX_JSON_INPUT_BYTES);
    assert!(parse_json(&input).is_ok());
}

#[test]
fn accepts_maximum_nesting_depth() {
    let input = format!(
        "{}null{}",
        "[".repeat(MAX_JSON_NESTING_DEPTH),
        "]".repeat(MAX_JSON_NESTING_DEPTH)
    );
    assert!(parse_json(&input).is_ok());
}

#[test]
fn rejects_nesting_above_limit_without_panicking() {
    let input = format!(
        "{}null{}",
        "[".repeat(MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(MAX_JSON_NESTING_DEPTH + 1)
    );
    let error = parse_json(&input).unwrap_err();
    assert!(error.message.contains("nesting exceeds"));
}

#[test]
fn rejects_mixed_nesting_above_limit() {
    let mut input = String::new();
    for index in 0..=MAX_JSON_NESTING_DEPTH {
        if index % 2 == 0 {
            input.push_str("{\"x\":");
        } else {
            input.push('[');
        }
    }
    input.push_str("null");
    for index in (0..=MAX_JSON_NESTING_DEPTH).rev() {
        if index % 2 == 0 {
            input.push('}');
        } else {
            input.push(']');
        }
    }
    assert!(parse_json(&input).is_err());
}
""")
