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

pub const OPERATOR_DISMISSED_REASON_CODE: &str = "LB_OPERATOR_DISMISSED";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineDismissal {
    pub id: String,
    pub quarantine_id: String,
    pub dismissed_at: String,
    pub operator: String,
    pub reason_code: String,
    pub note: String,
}

impl QuarantineStore {
    pub fn dismissals_path(&self) -> Result<PathBuf, StoreError> {
        resolve_quarantine_active_path(self.state_dir(), "quarantine-dismissals.jsonl")
    }

    pub fn dismiss(
        &self,
        quarantine_id: &str,
        operator: &str,
        reason_code: &str,
        note: &str,
    ) -> Result<QuarantineDismissal, StoreError> {
        let _lock = acquire_quarantine_lock(self.state_dir(), "quarantine-dismiss")?;
        if self.get(quarantine_id)?.is_none() {
            return Err(store_error(
                "LB_QUARANTINE_NOT_FOUND",
                format!("quarantine record not found: {quarantine_id}"),
            ));
        }
        if self.get_resolution(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_ALREADY_PROMOTED",
                format!("promoted quarantine record cannot be dismissed: {quarantine_id}"),
            ));
        }
        if self.get_permanent_rejection(quarantine_id)?.is_some() {
            return Err(store_error(
                "LB_QUARANTINE_PERMANENTLY_REJECTED",
                format!(
                    "permanently rejected quarantine record cannot be dismissed: {quarantine_id}"
                ),
            ));
        }
        if let Some(existing) = self.get_dismissal(quarantine_id)? {
            return Ok(existing);
        }

        let operator = operator.trim();
        let reason_code = reason_code.trim();
        let note = note.trim();
        if operator.is_empty() {
            return Err(store_error(
                "LB_QUARANTINE_DISMISSAL",
                "operator must not be empty",
            ));
        }
        if reason_code != OPERATOR_DISMISSED_REASON_CODE {
            return Err(store_error(
                "LB_QUARANTINE_DISMISSAL",
                format!("unsupported dismissal reason code: {reason_code}"),
            ));
        }
        if note.is_empty() {
            return Err(store_error(
                "LB_QUARANTINE_DISMISSAL",
                "note must not be empty",
            ));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let dismissal = QuarantineDismissal {
            id: format!("lb:qd:{}-{}", now.as_secs(), now.subsec_nanos()),
            quarantine_id: quarantine_id.to_string(),
            dismissed_at: format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()),
            operator: operator.to_string(),
            reason_code: reason_code.to_string(),
            note: note.to_string(),
        };
        append_dismissal_line(&self.dismissals_path()?, &dismissal)?;
        Ok(dismissal)
    }

    pub fn get_dismissal(
        &self,
        quarantine_id: &str,
    ) -> Result<Option<QuarantineDismissal>, StoreError> {
        Ok(self
            .list_dismissals(Some(quarantine_id))?
            .into_iter()
            .next())
    }

    pub fn list_dismissals(
        &self,
        quarantine_id: Option<&str>,
    ) -> Result<Vec<QuarantineDismissal>, StoreError> {
        let mut dismissals = Vec::new();
        let mut seen = BTreeSet::new();
        for line in read_managed_ledger_lines(self.state_dir(), "quarantine-dismissals.jsonl")? {
            let dismissal = parse_dismissal(&line)?;
            if !seen.insert(dismissal.quarantine_id.clone()) {
                return Err(store_error(
                    "LB_QUARANTINE_CORRUPT",
                    format!(
                        "duplicate dismissal event for quarantine record: {}",
                        dismissal.quarantine_id
                    ),
                ));
            }
            if quarantine_id
                .map(|id| dismissal.quarantine_id == id)
                .unwrap_or(true)
            {
                dismissals.push(dismissal);
            }
        }
        Ok(dismissals)
    }
}

pub fn quarantine_dismissal_json(dismissal: &QuarantineDismissal) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("id".to_string(), JsonValue::String(dismissal.id.clone())),
        (
            "quarantineId".to_string(),
            JsonValue::String(dismissal.quarantine_id.clone()),
        ),
        (
            "dismissedAt".to_string(),
            JsonValue::String(dismissal.dismissed_at.clone()),
        ),
        (
            "operator".to_string(),
            JsonValue::String(dismissal.operator.clone()),
        ),
        (
            "reasonCode".to_string(),
            JsonValue::String(dismissal.reason_code.clone()),
        ),
        (
            "note".to_string(),
            JsonValue::String(dismissal.note.clone()),
        ),
    ]))
}

fn append_dismissal_line(
    path: &std::path::Path,
    dismissal: &QuarantineDismissal,
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
        to_canonical_json(&quarantine_dismissal_json(dismissal))
    )
    .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

fn parse_dismissal(line: &str) -> Result<QuarantineDismissal, StoreError> {
    let map = match parse_json(line)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?
    {
        JsonValue::Object(map) => map,
        _ => {
            return Err(store_error(
                "LB_QUARANTINE_CORRUPT",
                "dismissal is not an object",
            ))
        }
    };
    Ok(QuarantineDismissal {
        id: required_string(&map, "id")?,
        quarantine_id: required_string(&map, "quarantineId")?,
        dismissed_at: required_string(&map, "dismissedAt")?,
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
            format!("dismissal missing {name}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_quarantine_ledger_index, rotate_quarantine_ledger};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-dismissals-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn reads_archived_dismissal_and_preserves_idempotency() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let record = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();
        let first = store
            .dismiss(
                &record.id,
                "operator",
                OPERATOR_DISMISSED_REASON_CODE,
                "duplicate",
            )
            .unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine-dismissals.jsonl").unwrap();
        let second = store
            .dismiss(
                &record.id,
                "other",
                OPERATOR_DISMISSED_REASON_CODE,
                "ignored",
            )
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(store.list_dismissals(None).unwrap().len(), 1);
        let _ = fs::remove_dir_all(dir);
    }
}
