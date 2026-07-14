use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use super::QuarantineStore;
use crate::{
    acquire_quarantine_lock, read_managed_ledger_lines, resolve_quarantine_active_path,
    store_error, StoreError,
};

pub const OPERATOR_PERMANENTLY_REJECTED_REASON_CODE: &str = "LB_OPERATOR_PERMANENTLY_REJECTED";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinePermanentRejection {
    pub id: String,
    pub quarantine_id: String,
    pub rejected_at: String,
    pub operator: String,
    pub reason_code: String,
    pub note: String,
}

impl QuarantineStore {
    pub fn permanent_rejections_path(&self) -> Result<PathBuf, StoreError> {
        resolve_quarantine_active_path(self.state_dir(), "quarantine-rejections.jsonl")
    }

    pub fn permanently_reject(
        &self,
        quarantine_id: &str,
        operator: &str,
        reason_code: &str,
        note: &str,
    ) -> Result<QuarantinePermanentRejection, StoreError> {
        let _lock = acquire_quarantine_lock(self.state_dir(), "quarantine-permanently-reject")?;
        if self.get(quarantine_id)?.is_none() {
            return Err(store_error(
                "LB_QUARANTINE_NOT_FOUND",
                format!("quarantine record not found: {quarantine_id}"),
            ));
        }
        if self.get_resolution(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_ALREADY_PROMOTED",
                format!(
                    "promoted quarantine record cannot be permanently rejected: {quarantine_id}"
                ),
            ));
        }
        if self.get_dismissal(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_ALREADY_DISMISSED",
                format!(
                    "dismissed quarantine record cannot be permanently rejected: {quarantine_id}"
                ),
            ));
        }
        if let Some(existing) = self.get_permanent_rejection(quarantine_id)? {
            return Ok(existing);
        }

        let operator = operator.trim();
        let reason_code = reason_code.trim();
        let note = note.trim();
        if operator.is_empty() {
            return Err(store_error(
                "LB_QUARANTINE_PERMANENT_REJECTION",
                "operator must not be empty",
            ));
        }
        if reason_code != OPERATOR_PERMANENTLY_REJECTED_REASON_CODE {
            return Err(store_error(
                "LB_QUARANTINE_PERMANENT_REJECTION",
                format!("unsupported permanent rejection reason code: {reason_code}"),
            ));
        }
        if note.is_empty() {
            return Err(store_error(
                "LB_QUARANTINE_PERMANENT_REJECTION",
                "note must not be empty",
            ));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let event = QuarantinePermanentRejection {
            id: format!("lb:qr:{}-{}", now.as_secs(), now.subsec_nanos()),
            quarantine_id: quarantine_id.to_string(),
            rejected_at: format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()),
            operator: operator.to_string(),
            reason_code: reason_code.to_string(),
            note: note.to_string(),
        };
        append_line(&self.permanent_rejections_path()?, &event)?;
        Ok(event)
    }

    pub fn get_permanent_rejection(
        &self,
        quarantine_id: &str,
    ) -> Result<Option<QuarantinePermanentRejection>, StoreError> {
        Ok(self
            .list_permanent_rejections(Some(quarantine_id))?
            .into_iter()
            .next())
    }

    pub fn list_permanent_rejections(
        &self,
        quarantine_id: Option<&str>,
    ) -> Result<Vec<QuarantinePermanentRejection>, StoreError> {
        let mut events = Vec::new();
        let mut seen = BTreeSet::new();
        for line in read_managed_ledger_lines(self.state_dir(), "quarantine-rejections.jsonl")? {
            let event = parse_event(&line)?;
            if !seen.insert(event.quarantine_id.clone()) {
                return Err(store_error(
                    "LB_QUARANTINE_CORRUPT",
                    format!(
                        "duplicate permanent rejection event for quarantine record: {}",
                        event.quarantine_id
                    ),
                ));
            }
            if quarantine_id
                .map(|id| event.quarantine_id == id)
                .unwrap_or(true)
            {
                events.push(event);
            }
        }
        Ok(events)
    }
}

pub fn quarantine_permanent_rejection_json(event: &QuarantinePermanentRejection) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("id".to_string(), JsonValue::String(event.id.clone())),
        (
            "quarantineId".to_string(),
            JsonValue::String(event.quarantine_id.clone()),
        ),
        (
            "rejectedAt".to_string(),
            JsonValue::String(event.rejected_at.clone()),
        ),
        (
            "operator".to_string(),
            JsonValue::String(event.operator.clone()),
        ),
        (
            "reasonCode".to_string(),
            JsonValue::String(event.reason_code.clone()),
        ),
        ("note".to_string(), JsonValue::String(event.note.clone())),
    ]))
}

fn append_line(
    path: &std::path::Path,
    event: &QuarantinePermanentRejection,
) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    writeln!(
        file,
        "{}",
        to_canonical_json(&quarantine_permanent_rejection_json(event))
    )
    .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

fn parse_event(line: &str) -> Result<QuarantinePermanentRejection, StoreError> {
    let map = match parse_json(line)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?
    {
        JsonValue::Object(map) => map,
        _ => {
            return Err(store_error(
                "LB_QUARANTINE_CORRUPT",
                "permanent rejection is not an object",
            ))
        }
    };
    Ok(QuarantinePermanentRejection {
        id: required_string(&map, "id")?,
        quarantine_id: required_string(&map, "quarantineId")?,
        rejected_at: required_string(&map, "rejectedAt")?,
        operator: required_string(&map, "operator")?,
        reason_code: required_string(&map, "reasonCode")?,
        note: required_string(&map, "note")?,
    })
}

fn required_string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("permanent rejection missing {name}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_quarantine_ledger_index, rotate_quarantine_ledger};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-rejections-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn reads_archived_rejection_and_preserves_idempotency() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let record = store
            .append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[])
            .unwrap();
        let first = store
            .permanently_reject(
                &record.id,
                "operator",
                OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                "prohibited",
            )
            .unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine-rejections.jsonl").unwrap();
        let second = store
            .permanently_reject(
                &record.id,
                "other",
                OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                "ignored",
            )
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(store.list_permanent_rejections(None).unwrap().len(), 1);
        let _ = fs::remove_dir_all(dir);
    }
}
