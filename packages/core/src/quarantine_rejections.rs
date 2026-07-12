use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use super::QuarantineStore;
use crate::{store_error, StoreError};

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
    pub fn permanent_rejections_path(&self) -> PathBuf {
        self.path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("quarantine-rejections.jsonl")
    }

    pub fn permanently_reject(
        &self,
        quarantine_id: &str,
        operator: &str,
        reason_code: &str,
        note: &str,
    ) -> Result<QuarantinePermanentRejection, StoreError> {
        if self.get(quarantine_id)?.is_none() {
            return Err(store_error(
                "LB_QUARANTINE_NOT_FOUND",
                format!("quarantine record not found: {quarantine_id}"),
            ));
        }
        if self.get_resolution(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_ALREADY_PROMOTED",
                format!("promoted quarantine record cannot be permanently rejected: {quarantine_id}"),
            ));
        }
        if self.get_dismissal(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_ALREADY_DISMISSED",
                format!("dismissed quarantine record cannot be permanently rejected: {quarantine_id}"),
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
        append_line(&self.permanent_rejections_path(), &event)?;
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
        let path = self.permanent_rejections_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(path)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let mut events = Vec::new();
        let mut seen = BTreeSet::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }
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

fn append_line(path: &std::path::Path, event: &QuarantinePermanentRejection) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    writeln!(file, "{}", to_canonical_json(&quarantine_permanent_rejection_json(event)))
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

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-quarantine-rejections-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn appends_lists_filters_and_is_idempotent() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let first = store.append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[]).unwrap();
        let second = store.append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[]).unwrap();
        let event = store
            .permanently_reject(
                &first.id,
                "operator-a",
                OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                "known prohibited content",
            )
            .unwrap();
        let duplicate = store
            .permanently_reject(
                &first.id,
                "operator-b",
                OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                "ignored because already terminal",
            )
            .unwrap();
        assert_eq!(event, duplicate);
        assert_eq!(store.list_permanent_rejections(None).unwrap().len(), 1);
        assert!(store.list_permanent_rejections(Some(&second.id)).unwrap().is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_unknown_promoted_dismissed_and_invalid_fields() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        assert_eq!(
            store
                .permanently_reject(
                    "lb:q:missing",
                    "operator",
                    OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                    "note",
                )
                .unwrap_err()
                .code,
            "LB_QUARANTINE_NOT_FOUND"
        );
        let promoted = store.append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[]).unwrap();
        store.append_resolution(&promoted.id, "lb:obj:test", false).unwrap();
        assert_eq!(
            store
                .permanently_reject(
                    &promoted.id,
                    "operator",
                    OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                    "note",
                )
                .unwrap_err()
                .code,
            "LB_QUARANTINE_ALREADY_PROMOTED"
        );
        let dismissed = store.append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[]).unwrap();
        store.dismiss(&dismissed.id, "operator", crate::OPERATOR_DISMISSED_REASON_CODE, "note").unwrap();
        assert_eq!(
            store
                .permanently_reject(
                    &dismissed.id,
                    "operator",
                    OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                    "note",
                )
                .unwrap_err()
                .code,
            "LB_QUARANTINE_ALREADY_DISMISSED"
        );
        let pending = store.append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[]).unwrap();
        for (operator, reason, note) in [
            ("", OPERATOR_PERMANENTLY_REJECTED_REASON_CODE, "note"),
            ("operator", "LB_FREE_FORM", "note"),
            ("operator", OPERATOR_PERMANENTLY_REJECTED_REASON_CODE, ""),
        ] {
            assert_eq!(
                store.permanently_reject(&pending.id, operator, reason, note).unwrap_err().code,
                "LB_QUARANTINE_PERMANENT_REJECTION"
            );
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_ledger_is_reported() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.permanent_rejections_path(), "not-json\n").unwrap();
        assert_eq!(
            store.list_permanent_rejections(None).unwrap_err().code,
            "LB_QUARANTINE_CORRUPT"
        );
        let _ = fs::remove_dir_all(dir);
    }
}