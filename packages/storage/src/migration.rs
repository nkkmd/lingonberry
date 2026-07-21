use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const STORAGE_MANIFEST_FILE: &str = "storage-format.manifest";
pub const MIGRATION_JOURNAL_FILE: &str = "storage-migration.journal";
pub const CURRENT_STORAGE_FORMAT_VERSION: u32 = 1;
pub const CURRENT_LAYOUT_ID: &str = "single-node-canonical-v1";
const MANIFEST_MAGIC: &str = "lingonberry-storage-manifest-v1";
const JOURNAL_MAGIC: &str = "lingonberry-storage-migration-journal-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageFormatManifest {
    pub format_version: u32,
    pub layout_id: String,
    pub created_by: String,
    pub source_format_version: Option<u32>,
}

impl StorageFormatManifest {
    pub fn current(created_by: impl Into<String>, source_format_version: Option<u32>) -> Self {
        Self {
            format_version: CURRENT_STORAGE_FORMAT_VERSION,
            layout_id: CURRENT_LAYOUT_ID.to_string(),
            created_by: created_by.into(),
            source_format_version,
        }
    }

    fn encode(&self) -> String {
        let mut output = String::new();
        output.push_str(MANIFEST_MAGIC);
        output.push('\n');
        output.push_str(&format!("format_version={}\n", self.format_version));
        output.push_str(&format!("layout_id={}\n", encode_value(&self.layout_id)));
        output.push_str(&format!("created_by={}\n", encode_value(&self.created_by)));
        if let Some(version) = self.source_format_version {
            output.push_str(&format!("source_format_version={version}\n"));
        }
        output
    }

    fn decode(input: &str) -> Result<Self, String> {
        let fields = parse_document(input, MANIFEST_MAGIC)?;
        reject_unknown_fields(
            &fields,
            &[
                "format_version",
                "layout_id",
                "created_by",
                "source_format_version",
            ],
        )?;
        let manifest = Self {
            format_version: required_u32(&fields, "format_version")?,
            layout_id: required_string(&fields, "layout_id")?,
            created_by: required_string(&fields, "created_by")?,
            source_format_version: optional_u32(&fields, "source_format_version")?,
        };
        if manifest.layout_id.is_empty() {
            return Err("storage manifest layout_id must not be empty".to_string());
        }
        if manifest.created_by.is_empty() {
            return Err("storage manifest created_by must not be empty".to_string());
        }
        Ok(manifest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageFormatState {
    Empty,
    LegacyUnversioned { inventory_digest: String },
    Supported(StorageFormatManifest),
    UnknownNewer { format_version: u32 },
    Corrupt { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageInspection {
    pub data_dir: PathBuf,
    pub state: StorageFormatState,
    pub inventory_digest: String,
    pub inventory: Vec<String>,
}

pub fn inspect_storage(data_dir: impl AsRef<Path>) -> Result<StorageInspection, String> {
    let data_dir = data_dir.as_ref().to_path_buf();
    let inventory = durable_inventory(&data_dir)?;
    let inventory_digest = digest_lines(&inventory);
    let manifest_path = data_dir.join(STORAGE_MANIFEST_FILE);
    let state = if manifest_path.exists() {
        classify_manifest(&manifest_path)?
    } else if inventory.is_empty() {
        StorageFormatState::Empty
    } else {
        StorageFormatState::LegacyUnversioned {
            inventory_digest: inventory_digest.clone(),
        }
    };
    Ok(StorageInspection {
        data_dir,
        state,
        inventory_digest,
        inventory,
    })
}

fn classify_manifest(path: &Path) -> Result<StorageFormatState, String> {
    let state = match read_utf8(path).and_then(|text| StorageFormatManifest::decode(&text)) {
        Ok(manifest) if manifest.format_version > CURRENT_STORAGE_FORMAT_VERSION => {
            StorageFormatState::UnknownNewer {
                format_version: manifest.format_version,
            }
        }
        Ok(manifest)
            if manifest.format_version == CURRENT_STORAGE_FORMAT_VERSION
                && manifest.layout_id == CURRENT_LAYOUT_ID =>
        {
            StorageFormatState::Supported(manifest)
        }
        Ok(manifest) => StorageFormatState::Corrupt {
            reason: format!(
                "unsupported storage manifest: version={}, layout_id={}",
                manifest.format_version, manifest.layout_id
            ),
        },
        Err(reason) => StorageFormatState::Corrupt { reason },
    };
    Ok(state)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    pub plan_id: String,
    pub source_inventory_digest: String,
    pub source_format_version: Option<u32>,
    pub target_format_version: u32,
    pub requires_verified_backup: bool,
    pub steps: Vec<MigrationStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStep {
    Inspect,
    VerifiedBackup,
    WriteManifest,
    Verify,
    Commit,
}

pub fn plan_migration(inspection: &StorageInspection) -> Result<MigrationPlan, String> {
    let (source_format_version, requires_verified_backup, steps) = match &inspection.state {
        StorageFormatState::Empty => (
            None,
            false,
            vec![
                MigrationStep::Inspect,
                MigrationStep::WriteManifest,
                MigrationStep::Verify,
                MigrationStep::Commit,
            ],
        ),
        StorageFormatState::LegacyUnversioned { .. } => (None, true, standard_migration_steps()),
        StorageFormatState::Supported(manifest) => {
            if manifest.format_version == CURRENT_STORAGE_FORMAT_VERSION {
                return Err("storage is already at the current format".to_string());
            }
            (
                Some(manifest.format_version),
                true,
                standard_migration_steps(),
            )
        }
        StorageFormatState::UnknownNewer { format_version } => {
            return Err(format!(
                "refusing migration from unknown newer storage format {format_version}"
            ));
        }
        StorageFormatState::Corrupt { reason } => {
            return Err(format!(
                "refusing migration from corrupt storage state: {reason}"
            ));
        }
    };
    let source_version = source_format_version
        .map(|value| value.to_string())
        .unwrap_or_else(|| "legacy".to_string());
    let seed = format!(
        "{}:{}:{}",
        inspection.inventory_digest, source_version, CURRENT_STORAGE_FORMAT_VERSION
    );
    Ok(MigrationPlan {
        plan_id: format!("migration-{}", digest_bytes(seed.as_bytes())),
        source_inventory_digest: inspection.inventory_digest.clone(),
        source_format_version,
        target_format_version: CURRENT_STORAGE_FORMAT_VERSION,
        requires_verified_backup,
        steps,
    })
}

fn standard_migration_steps() -> Vec<MigrationStep> {
    vec![
        MigrationStep::Inspect,
        MigrationStep::VerifiedBackup,
        MigrationStep::WriteManifest,
        MigrationStep::Verify,
        MigrationStep::Commit,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStage {
    Planned,
    BackupVerified,
    Migrating,
    Verified,
    Committed,
    RollingBack,
    RolledBack,
}

impl MigrationStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::BackupVerified => "backup_verified",
            Self::Migrating => "migrating",
            Self::Verified => "verified",
            Self::Committed => "committed",
            Self::RollingBack => "rolling_back",
            Self::RolledBack => "rolled_back",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "planned" => Ok(Self::Planned),
            "backup_verified" => Ok(Self::BackupVerified),
            "migrating" => Ok(Self::Migrating),
            "verified" => Ok(Self::Verified),
            "committed" => Ok(Self::Committed),
            "rolling_back" => Ok(Self::RollingBack),
            "rolled_back" => Ok(Self::RolledBack),
            _ => Err(format!("unknown migration stage: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationJournal {
    pub plan_id: String,
    pub source_inventory_digest: String,
    pub target_format_version: u32,
    pub stage: MigrationStage,
    pub backup_evidence_digest: Option<String>,
}

impl MigrationJournal {
    pub fn from_plan(plan: &MigrationPlan) -> Self {
        Self {
            plan_id: plan.plan_id.clone(),
            source_inventory_digest: plan.source_inventory_digest.clone(),
            target_format_version: plan.target_format_version,
            stage: MigrationStage::Planned,
            backup_evidence_digest: None,
        }
    }

    pub fn advance(
        &mut self,
        next: MigrationStage,
        backup_evidence_digest: Option<String>,
    ) -> Result<(), String> {
        if !valid_transition(self.stage, next) {
            return Err(format!(
                "invalid migration stage transition: {} -> {}",
                self.stage.as_str(),
                next.as_str()
            ));
        }
        if next == MigrationStage::BackupVerified {
            let evidence = backup_evidence_digest
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "backup verification requires an evidence digest".to_string())?;
            self.backup_evidence_digest = Some(evidence);
        } else if backup_evidence_digest.is_some() {
            return Err("backup evidence may only be set at backup_verified".to_string());
        }
        self.stage = next;
        Ok(())
    }

    fn encode(&self) -> String {
        let mut output = String::new();
        output.push_str(JOURNAL_MAGIC);
        output.push('\n');
        output.push_str(&format!("plan_id={}\n", encode_value(&self.plan_id)));
        output.push_str(&format!(
            "source_inventory_digest={}\n",
            encode_value(&self.source_inventory_digest)
        ));
        output.push_str(&format!(
            "target_format_version={}\n",
            self.target_format_version
        ));
        output.push_str(&format!("stage={}\n", self.stage.as_str()));
        if let Some(digest) = &self.backup_evidence_digest {
            output.push_str(&format!(
                "backup_evidence_digest={}\n",
                encode_value(digest)
            ));
        }
        output
    }

    fn decode(input: &str) -> Result<Self, String> {
        let fields = parse_document(input, JOURNAL_MAGIC)?;
        reject_unknown_fields(
            &fields,
            &[
                "plan_id",
                "source_inventory_digest",
                "target_format_version",
                "stage",
                "backup_evidence_digest",
            ],
        )?;
        let journal = Self {
            plan_id: required_string(&fields, "plan_id")?,
            source_inventory_digest: required_string(&fields, "source_inventory_digest")?,
            target_format_version: required_u32(&fields, "target_format_version")?,
            stage: MigrationStage::parse(&required_string(&fields, "stage")?)?,
            backup_evidence_digest: fields.get("backup_evidence_digest").cloned(),
        };
        if journal.plan_id.is_empty() || journal.source_inventory_digest.is_empty() {
            return Err("migration journal identifiers must not be empty".to_string());
        }
        if journal.stage == MigrationStage::BackupVerified
            && journal.backup_evidence_digest.is_none()
        {
            return Err("backup_verified journal is missing evidence".to_string());
        }
        Ok(journal)
    }
}

fn valid_transition(current: MigrationStage, next: MigrationStage) -> bool {
    matches!(
        (current, next),
        (MigrationStage::Planned, MigrationStage::BackupVerified)
            | (MigrationStage::Planned, MigrationStage::Migrating)
            | (MigrationStage::BackupVerified, MigrationStage::Migrating)
            | (MigrationStage::Migrating, MigrationStage::Verified)
            | (MigrationStage::Verified, MigrationStage::Committed)
            | (MigrationStage::Planned, MigrationStage::RollingBack)
            | (MigrationStage::BackupVerified, MigrationStage::RollingBack)
            | (MigrationStage::Migrating, MigrationStage::RollingBack)
            | (MigrationStage::Verified, MigrationStage::RollingBack)
            | (MigrationStage::RollingBack, MigrationStage::RolledBack)
    )
}

pub fn write_storage_manifest(
    data_dir: impl AsRef<Path>,
    manifest: &StorageFormatManifest,
) -> Result<(), String> {
    fs::create_dir_all(data_dir.as_ref()).map_err(|error| error.to_string())?;
    atomic_write(
        &data_dir.as_ref().join(STORAGE_MANIFEST_FILE),
        manifest.encode().as_bytes(),
    )
}

pub fn write_migration_journal(
    data_dir: impl AsRef<Path>,
    journal: &MigrationJournal,
) -> Result<(), String> {
    fs::create_dir_all(data_dir.as_ref()).map_err(|error| error.to_string())?;
    atomic_write(
        &data_dir.as_ref().join(MIGRATION_JOURNAL_FILE),
        journal.encode().as_bytes(),
    )
}

pub fn read_migration_journal(data_dir: impl AsRef<Path>) -> Result<MigrationJournal, String> {
    let text = read_utf8(&data_dir.as_ref().join(MIGRATION_JOURNAL_FILE))?;
    MigrationJournal::decode(&text)
}

pub fn verify_source_binding(
    data_dir: impl AsRef<Path>,
    journal: &MigrationJournal,
) -> Result<(), String> {
    let inspection = inspect_storage(data_dir)?;
    if inspection.inventory_digest != journal.source_inventory_digest {
        return Err(format!(
            "storage changed after planning: expected {}, found {}",
            journal.source_inventory_digest, inspection.inventory_digest
        ));
    }
    Ok(())
}

fn durable_inventory(data_dir: &Path) -> Result<Vec<String>, String> {
    if !data_dir.exists() {
        return Ok(Vec::new());
    }
    if !data_dir.is_dir() {
        return Err(format!(
            "data directory is not a directory: {}",
            data_dir.display()
        ));
    }
    let mut entries = Vec::new();
    collect_inventory(data_dir, data_dir, &mut entries)?;
    entries.sort();
    Ok(entries)
}

fn collect_inventory(root: &Path, current: &Path, entries: &mut Vec<String>) -> Result<(), String> {
    let mut children = fs::read_dir(current)
        .map_err(|error| format!("failed to read {}: {error}", current.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if relative == STORAGE_MANIFEST_FILE || relative == MIGRATION_JOURNAL_FILE {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlink is not allowed in durable inventory: {relative}"
            ));
        }
        if metadata.is_dir() {
            entries.push(format!("dir:{relative}"));
            collect_inventory(root, &path, entries)?;
        } else if metadata.is_file() {
            entries.push(format!(
                "file:{relative}:{}:{}",
                metadata.len(),
                digest_file(&path)?
            ));
        } else {
            return Err(format!("unsupported durable entry type: {relative}"));
        }
    }
    Ok(())
}

fn digest_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut state = 0xcbf29ce484222325u64;
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        state = fnv1a(state, &buffer[..read]);
    }
    Ok(format!("fnv1a64-{state:016x}"))
}

fn digest_lines(lines: &[String]) -> String {
    let mut state = 0xcbf29ce484222325u64;
    for line in lines {
        state = fnv1a(state, line.as_bytes());
        state = fnv1a(state, b"\n");
    }
    format!("fnv1a64-{state:016x}")
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a(0xcbf29ce484222325u64, bytes))
}

fn fnv1a(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temp = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| format!("failed to create {}: {error}", temp.display()))?;
    let result = (|| {
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&temp, path).map_err(|error| error.to_string())?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| error.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn read_utf8(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn parse_document(input: &str, expected_magic: &str) -> Result<BTreeMap<String, String>, String> {
    let mut lines = input.lines();
    if lines.next() != Some(expected_magic) {
        return Err(format!("invalid document magic; expected {expected_magic}"));
    }
    let mut fields = BTreeMap::new();
    for (index, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let (key, encoded) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid field at line {}", index + 2))?;
        if fields
            .insert(key.to_string(), decode_value(encoded)?)
            .is_some()
        {
            return Err(format!("duplicate field: {key}"));
        }
    }
    Ok(fields)
}

fn reject_unknown_fields(
    fields: &BTreeMap<String, String>,
    allowed: &[&str],
) -> Result<(), String> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    for key in fields.keys() {
        if !allowed.contains(key.as_str()) {
            return Err(format!("unknown field: {key}"));
        }
    }
    Ok(())
}

fn required_string(fields: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    fields
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing required field: {key}"))
}

fn required_u32(fields: &BTreeMap<String, String>, key: &str) -> Result<u32, String> {
    required_string(fields, key)?
        .parse::<u32>()
        .map_err(|_| format!("field {key} must be an unsigned integer"))
}

fn optional_u32(fields: &BTreeMap<String, String>, key: &str) -> Result<Option<u32>, String> {
    fields
        .get(key)
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| format!("field {key} must be an unsigned integer"))
        })
        .transpose()
}

fn encode_value(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        match byte {
            b'%' => output.push_str("%25"),
            b'\n' => output.push_str("%0A"),
            b'\r' => output.push_str("%0D"),
            b'=' => output.push_str("%3D"),
            byte if byte.is_ascii_control() => output.push_str(&format!("%{byte:02X}")),
            byte => output.push(char::from(byte)),
        }
    }
    output
}

fn decode_value(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err("truncated percent escape".to_string());
            }
            let high = decode_hex(bytes[index + 1])?;
            let low = decode_hex(bytes[index + 2])?;
            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).map_err(|_| "field is not valid UTF-8".to_string())
}

fn decode_hex(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("invalid percent escape".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "lingonberry-storage-migration-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn empty_directory_gets_non_destructive_plan() {
        let dir = temp_dir("empty");
        let inspection = inspect_storage(&dir).expect("inspect");
        assert_eq!(inspection.state, StorageFormatState::Empty);
        let plan = plan_migration(&inspection).expect("plan");
        assert!(!plan.requires_verified_backup);
        assert!(!plan.steps.contains(&MigrationStep::VerifiedBackup));
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn legacy_state_requires_verified_backup() {
        let dir = temp_dir("legacy");
        fs::write(dir.join("relay-wire-log.jsonl"), b"legacy\n").expect("write fixture");
        let inspection = inspect_storage(&dir).expect("inspect");
        assert!(matches!(
            inspection.state,
            StorageFormatState::LegacyUnversioned { .. }
        ));
        let plan = plan_migration(&inspection).expect("plan");
        assert!(plan.requires_verified_backup);
        assert!(plan.steps.contains(&MigrationStep::VerifiedBackup));
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn supported_manifest_round_trips() {
        let dir = temp_dir("manifest");
        let manifest = StorageFormatManifest::current("0.7.0", None);
        write_storage_manifest(&dir, &manifest).expect("write manifest");
        let inspection = inspect_storage(&dir).expect("inspect");
        assert_eq!(inspection.state, StorageFormatState::Supported(manifest));
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn unknown_newer_format_fails_closed() {
        let dir = temp_dir("newer");
        let manifest = StorageFormatManifest {
            format_version: CURRENT_STORAGE_FORMAT_VERSION + 1,
            layout_id: CURRENT_LAYOUT_ID.to_string(),
            created_by: "future".to_string(),
            source_format_version: Some(CURRENT_STORAGE_FORMAT_VERSION),
        };
        write_storage_manifest(&dir, &manifest).expect("write manifest");
        let inspection = inspect_storage(&dir).expect("inspect");
        assert!(matches!(
            inspection.state,
            StorageFormatState::UnknownNewer { .. }
        ));
        assert!(plan_migration(&inspection).is_err());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn journal_rejects_skipping_verification() {
        let dir = temp_dir("journal");
        fs::write(dir.join("canonical-catalog.sqlite3"), b"fixture").expect("write fixture");
        let inspection = inspect_storage(&dir).expect("inspect");
        let plan = plan_migration(&inspection).expect("plan");
        let mut journal = MigrationJournal::from_plan(&plan);
        assert!(journal.advance(MigrationStage::Committed, None).is_err());
        journal
            .advance(
                MigrationStage::BackupVerified,
                Some("backup-proof-123".to_string()),
            )
            .expect("backup verified");
        journal
            .advance(MigrationStage::Migrating, None)
            .expect("migrating");
        journal
            .advance(MigrationStage::Verified, None)
            .expect("verified");
        journal
            .advance(MigrationStage::Committed, None)
            .expect("committed");
        write_migration_journal(&dir, &journal).expect("write journal");
        assert_eq!(read_migration_journal(&dir).expect("read journal"), journal);
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn source_binding_detects_post_plan_mutation() {
        let dir = temp_dir("binding");
        fs::write(dir.join("relay-wire-log.jsonl"), b"before\n").expect("write fixture");
        let inspection = inspect_storage(&dir).expect("inspect");
        let plan = plan_migration(&inspection).expect("plan");
        let journal = MigrationJournal::from_plan(&plan);
        verify_source_binding(&dir, &journal).expect("unchanged source");
        fs::write(dir.join("relay-wire-log.jsonl"), b"after\n").expect("mutate fixture");
        assert!(verify_source_binding(&dir, &journal).is_err());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn inventory_rejects_symlinks() {
        use std::os::unix::fs::symlink;
        let dir = temp_dir("symlink");
        let outside = temp_dir("outside");
        fs::write(outside.join("data"), b"secret").expect("write outside");
        symlink(outside.join("data"), dir.join("linked")).expect("create symlink");
        assert!(inspect_storage(&dir).is_err());
        fs::remove_dir_all(dir).expect("cleanup");
        fs::remove_dir_all(outside).expect("cleanup outside");
    }
}
