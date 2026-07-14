use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    acquire_quarantine_lock, read_managed_ledger_lines, resolve_quarantine_active_path,
    store_error, StoreError,
};

#[derive(Debug, Clone)]
pub struct QuarantineRecord {
    pub id: String,
    pub received_at: String,
    pub reason_code: String,
    pub reasons: Vec<String>,
    pub request_json: String,
}

#[derive(Debug, Clone)]
pub struct QuarantineResolution {
    pub quarantine_id: String,
    pub resolved_at: String,
    pub status: String,
    pub canonical_id: String,
    pub duplicate: bool,
}

#[derive(Debug, Clone)]
pub struct QuarantineStore {
    state_dir: PathBuf,
}

impl QuarantineStore {
    pub fn new(state_dir: impl AsRef<Path>) -> Self {
        Self {
            state_dir: state_dir.as_ref().to_path_buf(),
        }
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn path(&self) -> Result<PathBuf, StoreError> {
        resolve_quarantine_active_path(&self.state_dir, "quarantine.jsonl")
    }

    pub fn resolutions_path(&self) -> Result<PathBuf, StoreError> {
        resolve_quarantine_active_path(&self.state_dir, "quarantine-resolutions.jsonl")
    }

    pub fn append(
        &self,
        request_json: &str,
        reason_code: &str,
        reasons: &[String],
    ) -> Result<QuarantineRecord, StoreError> {
        let _lock = acquire_quarantine_lock(self.state_dir(), "quarantine-append")?;
        let (id_suffix, received_at) = timestamp()?;
        let record = QuarantineRecord {
            id: format!("lb:q:{id_suffix}"),
            received_at,
            reason_code: reason_code.to_string(),
            reasons: reasons.to_vec(),
            request_json: request_json.to_string(),
        };
        append_json_line(&self.path()?, &record_json(&record))?;
        Ok(record)
    }

    pub fn list_all(&self) -> Result<Vec<QuarantineRecord>, StoreError> {
        read_managed_ledger_lines(self.state_dir(), "quarantine.jsonl")?
            .into_iter()
            .map(|line| parse_record(&line))
            .collect()
    }

    pub fn list(&self) -> Result<Vec<QuarantineRecord>, StoreError> {
        let dismissed = self
            .list_dismissals(None)?
            .into_iter()
            .map(|event| event.quarantine_id)
            .collect::<BTreeSet<_>>();
        let permanently_rejected = self
            .list_permanent_rejections(None)?
            .into_iter()
            .map(|event| event.quarantine_id)
            .collect::<BTreeSet<_>>();
        Ok(self
            .list_all()?
            .into_iter()
            .filter(|record| {
                !dismissed.contains(&record.id) && !permanently_rejected.contains(&record.id)
            })
            .collect())
    }

    pub fn get(&self, id: &str) -> Result<Option<QuarantineRecord>, StoreError> {
        Ok(self
            .list_all()?
            .into_iter()
            .find(|record| record.id == id))
    }

    pub fn append_resolution(
        &self,
        quarantine_id: &str,
        canonical_id: &str,
        duplicate: bool,
    ) -> Result<QuarantineResolution, StoreError> {
        let _lock = acquire_quarantine_lock(self.state_dir(), "quarantine-promote")?;
        if let Some(existing) = self.get_resolution(quarantine_id)? {
            return Ok(existing);
        }
        if self.get_dismissal(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_ALREADY_DISMISSED",
                format!("dismissed quarantine record cannot be promoted: {quarantine_id}"),
            ));
        }
        if self.get_permanent_rejection(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_PERMANENTLY_REJECTED",
                format!(
                    "permanently rejected quarantine record cannot be promoted: {quarantine_id}"
                ),
            ));
        }
        let (_, resolved_at) = timestamp()?;
        let resolution = QuarantineResolution {
            quarantine_id: quarantine_id.to_string(),
            resolved_at,
            status: "promoted".to_string(),
            canonical_id: canonical_id.to_string(),
            duplicate,
        };
        append_json_line(&self.resolutions_path()?, &resolution_json(&resolution))?;
        Ok(resolution)
    }

    pub fn list_resolutions(&self) -> Result<Vec<QuarantineResolution>, StoreError> {
        read_managed_ledger_lines(self.state_dir(), "quarantine-resolutions.jsonl")?
            .into_iter()
            .map(|line| parse_resolution(&line))
            .collect()
    }

    pub fn get_resolution(
        &self,
        quarantine_id: &str,
    ) -> Result<Option<QuarantineResolution>, StoreError> {
        Ok(self
            .list_resolutions()?
            .into_iter()
            .find(|resolution| resolution.quarantine_id == quarantine_id))
    }
}

fn timestamp() -> Result<(String, String), StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    Ok((
        format!("{}-{}", now.as_secs(), now.subsec_nanos()),
        format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()),
    ))
}

fn append_json_line(path: &Path, value: &JsonValue) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    writeln!(file, "{}", to_canonical_json(value))
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

fn record_json(record: &QuarantineRecord) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("id".to_string(), JsonValue::String(record.id.clone())),
        (
            "receivedAt".to_string(),
            JsonValue::String(record.received_at.clone()),
        ),
        (
            "reasonCode".to_string(),
            JsonValue::String(record.reason_code.clone()),
        ),
        (
            "reasons".to_string(),
            JsonValue::Array(
                record
                    .reasons
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
        ),
        (
            "requestJson".to_string(),
            JsonValue::String(record.request_json.clone()),
        ),
    ]))
}

fn resolution_json(resolution: &QuarantineResolution) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "quarantineId".to_string(),
            JsonValue::String(resolution.quarantine_id.clone()),
        ),
        (
            "resolvedAt".to_string(),
            JsonValue::String(resolution.resolved_at.clone()),
        ),
        (
            "status".to_string(),
            JsonValue::String(resolution.status.clone()),
        ),
        (
            "canonicalId".to_string(),
            JsonValue::String(resolution.canonical_id.clone()),
        ),
        (
            "duplicate".to_string(),
            JsonValue::Bool(resolution.duplicate),
        ),
    ]))
}

fn parse_record(line: &str) -> Result<QuarantineRecord, StoreError> {
    let map = parse_object(line)?;
    Ok(QuarantineRecord {
        id: required_string(&map, "id")?,
        received_at: required_string(&map, "receivedAt")?,
        reason_code: required_string(&map, "reasonCode")?,
        reasons: required_strings(&map, "reasons")?,
        request_json: required_string(&map, "requestJson")?,
    })
}

fn parse_resolution(line: &str) -> Result<QuarantineResolution, StoreError> {
    let map = parse_object(line)?;
    let duplicate = match map.get("duplicate") {
        Some(JsonValue::Bool(value)) => *value,
        _ => {
            return Err(store_error(
                "LB_QUARANTINE_CORRUPT",
                "resolution missing duplicate",
            ))
        }
    };
    Ok(QuarantineResolution {
        quarantine_id: required_string(&map, "quarantineId")?,
        resolved_at: required_string(&map, "resolvedAt")?,
        status: required_string(&map, "status")?,
        canonical_id: required_string(&map, "canonicalId")?,
        duplicate,
    })
}

fn parse_object(line: &str) -> Result<BTreeMap<String, JsonValue>, StoreError> {
    match parse_json(line)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?
    {
        JsonValue::Object(map) => Ok(map),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            "record is not an object",
        )),
    }
}

fn required_string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("record missing {name}"),
        )),
    }
}

fn required_strings(
    map: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Vec<String>, StoreError> {
    match map.get(name) {
        Some(JsonValue::Array(items)) => items
            .iter()
            .map(|item| match item {
                JsonValue::String(value) => Ok(value.clone()),
                _ => Err(store_error(
                    "LB_QUARANTINE_CORRUPT",
                    format!("{name} item is not a string"),
                )),
            })
            .collect(),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("record missing {name}"),
        )),
    }
}

pub fn quarantine_record_json(record: &QuarantineRecord) -> JsonValue {
    record_json(record)
}

pub fn quarantine_resolution_json(resolution: &QuarantineResolution) -> JsonValue {
    resolution_json(resolution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_quarantine_ledger_index, rotate_quarantine_ledger};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-quarantine-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn reads_archived_and_active_records_and_resolutions() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let first = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine.jsonl").unwrap();
        let second = store
            .append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[])
            .unwrap();
        assert_eq!(store.list_all().unwrap().len(), 2);

        store
            .append_resolution(&first.id, "lb:obj:first", false)
            .unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine-resolutions.jsonl").unwrap();
        store
            .append_resolution(&second.id, "lb:obj:second", false)
            .unwrap();
        assert_eq!(store.list_resolutions().unwrap().len(), 2);
        let _ = fs::remove_dir_all(dir);
    }
}