use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use super::{
    preview_quarantine_record, promote_quarantine_record, QuarantineBatchReport,
    QuarantinePromotionOutcome, QuarantineStore, StorageBackend,
};
use crate::{store_error, StoreError};

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
    pub fn dismissals_path(&self) -> PathBuf {
        self.path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("quarantine-dismissals.jsonl")
    }

    pub fn dismiss(
        &self,
        quarantine_id: &str,
        operator: &str,
        reason_code: &str,
        note: &str,
    ) -> Result<QuarantineDismissal, StoreError> {
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
        append_dismissal_line(&self.dismissals_path(), &dismissal)?;
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
        let path = self.dismissals_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(path)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let mut dismissals = Vec::new();
        let mut seen = BTreeSet::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }
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

pub fn promote_quarantine_batch_excluding_dismissed(
    limit: usize,
    dry_run: bool,
    backend: &impl StorageBackend,
) -> Result<QuarantineBatchReport, StoreError> {
    if limit == 0 {
        return Err(store_error(
            "LB_QUARANTINE_BATCH",
            "limit must be greater than zero",
        ));
    }
    let store = QuarantineStore::new(crate::runtime_state_dir());
    let resolved = store
        .list_resolutions()?
        .into_iter()
        .map(|resolution| resolution.quarantine_id)
        .collect::<BTreeSet<_>>();
    let dismissed = store
        .list_dismissals(None)?
        .into_iter()
        .map(|dismissal| dismissal.quarantine_id)
        .collect::<BTreeSet<_>>();
    let ids = store
        .list()?
        .into_iter()
        .filter(|record| !resolved.contains(&record.id) && !dismissed.contains(&record.id))
        .map(|record| record.id)
        .take(limit)
        .collect::<Vec<_>>();

    let mut report = QuarantineBatchReport {
        dry_run,
        limit,
        scanned: ids.len(),
        promoted: 0,
        already_promoted: 0,
        deferred: 0,
        rejected: 0,
        outcomes: Vec::new(),
    };
    for id in ids {
        let outcome = if dry_run {
            preview_quarantine_record(&id)?
        } else {
            promote_quarantine_record(&id, backend)?
        };
        match &outcome {
            QuarantinePromotionOutcome::Promoted { .. } => report.promoted += 1,
            QuarantinePromotionOutcome::AlreadyPromoted { .. } => report.already_promoted += 1,
            QuarantinePromotionOutcome::StillDeferred { .. } => report.deferred += 1,
            QuarantinePromotionOutcome::Rejected { .. } => report.rejected += 1,
        }
        report.outcomes.push(outcome);
    }
    Ok(report)
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
        ("note".to_string(), JsonValue::String(dismissal.note.clone())),
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

fn required_string(
    map: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<String, StoreError> {
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

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-quarantine-dismissals-{}",
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
        let first = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();
        let second = store
            .append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[])
            .unwrap();

        let dismissal = store
            .dismiss(
                &first.id,
                "operator-a",
                OPERATOR_DISMISSED_REASON_CODE,
                "duplicate external submission",
            )
            .unwrap();
        let duplicate = store
            .dismiss(
                &first.id,
                "operator-b",
                OPERATOR_DISMISSED_REASON_CODE,
                "ignored because already dismissed",
            )
            .unwrap();
        assert_eq!(dismissal, duplicate);
        assert_eq!(store.list_dismissals(None).unwrap().len(), 1);
        assert_eq!(store.list_dismissals(Some(&first.id)).unwrap().len(), 1);
        assert!(store.list_dismissals(Some(&second.id)).unwrap().is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_unknown_promoted_and_invalid_fields() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        assert_eq!(
            store
                .dismiss(
                    "lb:q:missing",
                    "operator",
                    OPERATOR_DISMISSED_REASON_CODE,
                    "note",
                )
                .unwrap_err()
                .code,
            "LB_QUARANTINE_NOT_FOUND"
        );
        let record = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();
        store
            .append_resolution(&record.id, "lb:obj:promoted", false)
            .unwrap();
        assert_eq!(
            store
                .dismiss(
                    &record.id,
                    "operator",
                    OPERATOR_DISMISSED_REASON_CODE,
                    "note",
                )
                .unwrap_err()
                .code,
            "LB_QUARANTINE_ALREADY_PROMOTED"
        );

        let pending = store
            .append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[])
            .unwrap();
        for (operator, reason_code, note) in [
            ("", OPERATOR_DISMISSED_REASON_CODE, "note"),
            ("operator", "LB_FREE_FORM", "note"),
            ("operator", OPERATOR_DISMISSED_REASON_CODE, "  "),
        ] {
            assert_eq!(
                store
                    .dismiss(&pending.id, operator, reason_code, note)
                    .unwrap_err()
                    .code,
                "LB_QUARANTINE_DISMISSAL"
            );
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_dismissal_ledger_is_reported() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.dismissals_path(), "not-json\n").unwrap();
        assert_eq!(
            store.list_dismissals(None).unwrap_err().code,
            "LB_QUARANTINE_CORRUPT"
        );
        let _ = fs::remove_dir_all(dir);
    }
}