use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{to_canonical_json, JsonValue};

use crate::{
    advance_quarantine_replacement_cleanup_transaction_journal,
    read_quarantine_replacement_cleanup_transaction_details,
    record_quarantine_replacement_cleanup_path_deleted, store_error,
    QuarantineReplacementCleanupProof, QuarantineReplacementCleanupTransactionState, StoreError,
};

pub const QUARANTINE_REPLACEMENT_CLEANUP_TOMB_DIR: &str = "tomb";
pub const QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_FILE: &str =
    "quarantine-replacement-cleanup-tomb-inventory.json";
pub const QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_DIGEST_FILE: &str =
    "quarantine-replacement-cleanup-tomb-inventory.digest";
pub const QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_VERSION: &str =
    "lingonberry-quarantine-replacement-cleanup-tomb-inventory/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupTombReport {
    pub tomb_dir: PathBuf,
    pub inventory_digest: String,
    pub managed_paths: Vec<String>,
}

pub fn move_quarantine_replacement_cleanup_to_tomb(
    state_dir: impl AsRef<Path>,
    cleanup_transaction_dir: impl AsRef<Path>,
    proof: &QuarantineReplacementCleanupProof,
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let transaction_dir = cleanup_transaction_dir.as_ref();
    let journal = read_quarantine_replacement_cleanup_transaction_details(transaction_dir)?;
    if journal.state != QuarantineReplacementCleanupTransactionState::Revalidated {
        return Err(tomb_error(
            "cleanup transaction must be revalidated before tomb rename",
        ));
    }

    let entries = proof_paths(proof)?;
    let tomb_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_DIR);
    if tomb_dir.exists() {
        return Err(tomb_error("cleanup tomb directory already exists"));
    }
    fs::create_dir(&tomb_dir).map_err(io_error)?;
    sync_directory(transaction_dir)?;
    advance_quarantine_replacement_cleanup_transaction_journal(
        transaction_dir,
        QuarantineReplacementCleanupTransactionState::RenamingToTomb,
        None,
    )?;

    for managed_path in &entries {
        let source = state_dir.join(managed_path);
        require_regular_file(&source)?;
        let destination = tomb_dir.join(managed_path);
        let parent = destination
            .parent()
            .ok_or_else(|| tomb_error("invalid tomb destination"))?;
        fs::create_dir_all(parent).map_err(io_error)?;
        fs::rename(&source, &destination).map_err(io_error)?;
        sync_directory(
            source
                .parent()
                .ok_or_else(|| tomb_error("invalid source parent"))?,
        )?;
        sync_directory(parent)?;
    }

    seal_tomb_inventory(transaction_dir, &entries)
}

pub fn resume_quarantine_replacement_cleanup_deletion(
    cleanup_transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    let transaction_dir = cleanup_transaction_dir.as_ref();
    let mut journal = read_quarantine_replacement_cleanup_transaction_details(transaction_dir)?;
    let report = verify_tomb_inventory(transaction_dir)?;
    if journal.state == QuarantineReplacementCleanupTransactionState::TombSealed {
        advance_quarantine_replacement_cleanup_transaction_journal(
            transaction_dir,
            QuarantineReplacementCleanupTransactionState::Deleting,
            None,
        )?;
        journal = read_quarantine_replacement_cleanup_transaction_details(transaction_dir)?;
    }
    if journal.state != QuarantineReplacementCleanupTransactionState::Deleting {
        return Err(tomb_error(
            "cleanup deletion may only resume in deleting state",
        ));
    }

    let deleted = journal.deleted_paths.into_iter().collect::<BTreeSet<_>>();
    for managed_path in &report.managed_paths {
        if deleted.contains(managed_path) {
            continue;
        }
        let path = report.tomb_dir.join(managed_path);
        require_regular_file(&path)?;
        fs::remove_file(&path).map_err(io_error)?;
        sync_directory(
            path.parent()
                .ok_or_else(|| tomb_error("invalid tomb entry parent"))?,
        )?;
        record_quarantine_replacement_cleanup_path_deleted(transaction_dir, managed_path)?;
    }
    remove_empty_directories(&report.tomb_dir, &report.tomb_dir)?;
    advance_quarantine_replacement_cleanup_transaction_journal(
        transaction_dir,
        QuarantineReplacementCleanupTransactionState::Committed,
        None,
    )?;
    Ok(report)
}

pub fn rollback_quarantine_replacement_cleanup_tomb(
    state_dir: impl AsRef<Path>,
    cleanup_transaction_dir: impl AsRef<Path>,
) -> Result<(), StoreError> {
    let state_dir = state_dir.as_ref();
    let transaction_dir = cleanup_transaction_dir.as_ref();
    let journal = read_quarantine_replacement_cleanup_transaction_details(transaction_dir)?;
    if journal.state.deletion_has_started() || !journal.deleted_paths.is_empty() {
        return Err(tomb_error(
            "cleanup rollback is forbidden after deletion begins",
        ));
    }
    let report = verify_tomb_inventory(transaction_dir)?;
    for managed_path in report.managed_paths.iter().rev() {
        let source = report.tomb_dir.join(managed_path);
        require_regular_file(&source)?;
        let destination = state_dir.join(managed_path);
        if destination.exists() {
            return Err(tomb_error("rollback destination already exists"));
        }
        let parent = destination
            .parent()
            .ok_or_else(|| tomb_error("invalid rollback destination"))?;
        fs::create_dir_all(parent).map_err(io_error)?;
        fs::rename(&source, &destination).map_err(io_error)?;
        sync_directory(parent)?;
    }
    advance_quarantine_replacement_cleanup_transaction_journal(
        transaction_dir,
        QuarantineReplacementCleanupTransactionState::RecoveryRequired,
        None,
    )?;
    advance_quarantine_replacement_cleanup_transaction_journal(
        transaction_dir,
        QuarantineReplacementCleanupTransactionState::RolledBack,
        None,
    )?;
    Ok(())
}

fn seal_tomb_inventory(
    transaction_dir: &Path,
    managed_paths: &[String],
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    let tomb_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_DIR);
    verify_exact_files(&tomb_dir, managed_paths)?;
    let inventory = inventory_json(managed_paths);
    let text = to_canonical_json(&inventory);
    let digest = integrity_digest(text.as_bytes());
    write_new_synced(
        &transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_FILE),
        text.as_bytes(),
    )?;
    write_new_synced(
        &transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_DIGEST_FILE),
        format!("{digest}\n").as_bytes(),
    )?;
    sync_directory(transaction_dir)?;
    advance_quarantine_replacement_cleanup_transaction_journal(
        transaction_dir,
        QuarantineReplacementCleanupTransactionState::TombSealed,
        Some(digest.clone()),
    )?;
    Ok(QuarantineReplacementCleanupTombReport {
        tomb_dir,
        inventory_digest: digest,
        managed_paths: managed_paths.to_vec(),
    })
}

fn verify_tomb_inventory(
    transaction_dir: &Path,
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    let inventory_path = transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_FILE);
    let digest_path =
        transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_DIGEST_FILE);
    let text = fs::read_to_string(&inventory_path).map_err(io_error)?;
    let digest = fs::read_to_string(&digest_path).map_err(io_error)?;
    let digest = digest.trim().to_string();
    if integrity_digest(text.as_bytes()) != digest {
        return Err(tomb_error("cleanup tomb inventory digest mismatch"));
    }
    let value = lingonberry_protocol::parse_json(&text)
        .map_err(|error| tomb_error(format!("invalid cleanup tomb inventory JSON: {error}")))?;
    let managed_paths = parse_inventory(&value)?;
    let journal = read_quarantine_replacement_cleanup_transaction_details(transaction_dir)?;
    if journal.tomb_inventory_digest.as_deref() != Some(digest.as_str()) {
        return Err(tomb_error("cleanup journal tomb inventory digest mismatch"));
    }
    let tomb_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_DIR);
    let remaining = managed_paths
        .iter()
        .filter(|path| !journal.deleted_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    verify_exact_files(&tomb_dir, &remaining)?;
    Ok(QuarantineReplacementCleanupTombReport {
        tomb_dir,
        inventory_digest: digest,
        managed_paths,
    })
}

fn proof_paths(proof: &QuarantineReplacementCleanupProof) -> Result<Vec<String>, StoreError> {
    let mut paths = Vec::new();
    for subject in &proof.plan.subjects {
        for path in &subject.managed_paths {
            validate_relative(path)?;
            paths.push(format!("{}/{}", subject.generation_id, path));
        }
    }
    paths.sort();
    if paths.is_empty() || paths.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(tomb_error(
            "cleanup proof contains empty or duplicate managed paths",
        ));
    }
    Ok(paths)
}

fn inventory_json(paths: &[String]) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "managedPaths".to_string(),
            JsonValue::Array(paths.iter().cloned().map(JsonValue::String).collect()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_VERSION.to_string()),
        ),
    ]))
}

fn parse_inventory(value: &JsonValue) -> Result<Vec<String>, StoreError> {
    let JsonValue::Object(map) = value else {
        return Err(tomb_error("cleanup tomb inventory must be an object"));
    };
    match map.get("version") {
        Some(JsonValue::String(version))
            if version == QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_VERSION => {}
        _ => return Err(tomb_error("unsupported cleanup tomb inventory version")),
    }
    let Some(JsonValue::Array(values)) = map.get("managedPaths") else {
        return Err(tomb_error("cleanup tomb inventory paths are missing"));
    };
    let paths = values
        .iter()
        .map(|value| match value {
            JsonValue::String(path) => {
                validate_relative(path)?;
                Ok(path.clone())
            }
            _ => Err(tomb_error("invalid cleanup tomb inventory path")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() || !paths.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(tomb_error(
            "cleanup tomb inventory must be non-empty and strictly sorted",
        ));
    }
    Ok(paths)
}

fn verify_exact_files(root: &Path, expected: &[String]) -> Result<(), StoreError> {
    let mut actual = Vec::new();
    collect_files(root, root, &mut actual)?;
    actual.sort();
    if actual != expected {
        return Err(tomb_error("cleanup tomb inventory changed"));
    }
    Ok(())
}

fn collect_files(root: &Path, current: &Path, paths: &mut Vec<String>) -> Result<(), StoreError> {
    for entry in fs::read_dir(current).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(tomb_error("cleanup tomb contains a symlink"));
        }
        if metadata.is_dir() {
            collect_files(root, &path, paths)?;
        } else if metadata.is_file() {
            paths.push(
                path.strip_prefix(root)
                    .map_err(|_| tomb_error("cleanup tomb path escaped root"))?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        } else {
            return Err(tomb_error("cleanup tomb contains unsupported file type"));
        }
    }
    Ok(())
}

fn remove_empty_directories(root: &Path, current: &Path) -> Result<(), StoreError> {
    let entries = fs::read_dir(current)
        .map_err(io_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(io_error)?;
    for entry in entries {
        if entry.file_type().map_err(io_error)?.is_dir() {
            remove_empty_directories(root, &entry.path())?;
        }
    }
    if current != root && fs::read_dir(current).map_err(io_error)?.next().is_none() {
        fs::remove_dir(current).map_err(io_error)?;
    }
    Ok(())
}

fn require_regular_file(path: &Path) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(tomb_error("cleanup managed path must be a regular file"));
    }
    Ok(())
}

fn validate_relative(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(tomb_error("invalid cleanup managed path"));
    }
    Ok(())
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}

fn sync_directory(path: &Path) -> Result<(), StoreError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(io_error)
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn io_error(error: std::io::Error) -> StoreError {
    tomb_error(error.to_string())
}

fn tomb_error(message: impl Into<String>) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_CLEANUP_TOMB", message)
}
